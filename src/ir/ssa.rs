use std::collections::{HashMap, HashSet, VecDeque};
use super::{Module, IRFunction, BasicBlock, Instruction, Terminator, IRType, Parameter};

pub struct SSABuilder {
    variable_stacks: HashMap<String, Vec<String>>,
    phi_nodes: HashMap<(String, String), Vec<(String, String)>>,
    dominance_frontier: HashMap<String, HashSet<String>>,
    immediate_dominators: HashMap<String, String>,
    rename_counter: HashMap<String, usize>,
}

impl SSABuilder {
    pub fn new() -> Self {
        Self {
            variable_stacks: HashMap::new(),
            phi_nodes: HashMap::new(),
            dominance_frontier: HashMap::new(),
            immediate_dominators: HashMap::new(),
            rename_counter: HashMap::new(),
        }
    }

    pub fn transform_to_ssa(&mut self, module: &mut Module) -> anyhow::Result<()> {
        for function in &mut module.functions {
            self.transform_function(function)?;
        }
        Ok(())
    }

    fn transform_function(&mut self, function: &mut IRFunction) -> anyhow::Result<()> {
        self.clear_state();
        self.compute_dominance(function);
        self.compute_dominance_frontier(function);
        self.insert_phi_nodes(function);
        self.rename_variables(function);
        Ok(())
    }

    fn clear_state(&mut self) {
        self.variable_stacks.clear();
        self.phi_nodes.clear();
        self.dominance_frontier.clear();
        self.immediate_dominators.clear();
        self.rename_counter.clear();
    }

    fn compute_dominance(&mut self, function: &IRFunction) {
        let blocks = &function.blocks;
        if blocks.is_empty() {
            return;
        }

        let entry_label = &blocks[0].label;
        let mut changed = true;

        let mut dominators: HashMap<String, HashSet<String>> = HashMap::new();
        for block in blocks {
            dominators.insert(block.label.clone(), HashSet::new());
        }

        let all_blocks: HashSet<String> = blocks.iter().map(|b| b.label.clone()).collect();
        dominators.get_mut(entry_label).unwrap().insert(entry_label.clone());

        for block in &blocks[1..] {
            dominators.get_mut(&block.label).unwrap().extend(all_blocks.iter().cloned());
        }

        while changed {
            changed = false;
            for block in &blocks[1..] {
                let predecessors = self.get_predecessors(&block.label, blocks);
                if predecessors.is_empty() {
                    continue;
                }

                let mut new_doms = all_blocks.clone();
                for pred in predecessors {
                    if let Some(pred_doms) = dominators.get(&pred) {
                        new_doms = new_doms.intersection(pred_doms).cloned().collect();
                    }
                }
                new_doms.insert(block.label.clone());

                if let Some(current_doms) = dominators.get_mut(&block.label) {
                    if current_doms != &new_doms {
                        *current_doms = new_doms;
                        changed = true;
                    }
                }
            }
        }

        self.compute_immediate_dominators(&dominators, entry_label);
    }

    fn compute_immediate_dominators(&mut self, dominators: &HashMap<String, HashSet<String>>, entry: &str) {
        for (block, doms) in dominators {
            if block == entry {
                continue;
            }

            let mut strict_doms: Vec<String> = doms.iter()
                .filter(|d| *d != block)
                .cloned()
                .collect();

            if strict_doms.is_empty() {
                continue;
            }

            strict_doms.sort_by(|a, b| {
                let a_doms = dominators.get(a).map(|s| s.len()).unwrap_or(0);
                let b_doms = dominators.get(b).map(|s| s.len()).unwrap_or(0);
                b_doms.cmp(&a_doms)
            });

            if let Some(idom) = strict_doms.first() {
                self.immediate_dominators.insert(block.clone(), idom.clone());
            }
        }
    }

    fn compute_dominance_frontier(&mut self, function: &IRFunction) {
        let blocks = &function.blocks;
        
        for block in blocks {
            let predecessors = self.get_predecessors(&block.label, blocks);
            if predecessors.len() >= 2 {
                for pred in predecessors {
                    let mut runner = pred.clone();
                    while let Some(idom) = self.immediate_dominators.get(&runner) {
                        if idom == &block.label {
                            break;
                        }
                        self.dominance_frontier.entry(runner.clone())
                            .or_insert_with(HashSet::new)
                            .insert(block.label.clone());
                        runner = idom.clone();
                    }
                }
            }
        }
    }

    fn insert_phi_nodes(&mut self, function: &IRFunction) {
        let variables = self.collect_variables(function);
        
        for var in variables {
            let mut has_phi: HashSet<String> = HashSet::new();
            let mut work_list: VecDeque<String> = VecDeque::new();

            for block in &function.blocks {
                if self.block_defines_variable(block, &var) {
                    work_list.push_back(block.label.clone());
                }
            }

            while let Some(block_label) = work_list.pop_front() {
                if let Some(df) = self.dominance_frontier.get(&block_label) {
                    for frontier_block in df {
                        if !has_phi.contains(frontier_block) {
                            self.phi_nodes.entry((frontier_block.clone(), var.clone()))
                                .or_insert_with(Vec::new);
                            has_phi.insert(frontier_block.clone());
                            work_list.push_back(frontier_block.clone());
                        }
                    }
                }
            }
        }
    }

    fn rename_variables(&mut self, function: &mut IRFunction) {
        for param in &function.params {
            self.variable_stacks.entry(param.name.clone())
                .or_insert_with(Vec::new)
                .push(format!("{}.0", param.name));
            self.rename_counter.insert(param.name.clone(), 1);
        }

        if !function.blocks.is_empty() {
            let entry_label = function.blocks[0].label.clone();
            self.rename_block(&entry_label, function);
        }
    }

    fn rename_block(&mut self, block_label: &str, function: &mut IRFunction) {
        let block_idx = function.blocks.iter().position(|b| b.label == block_label);
        if block_idx.is_none() {
            return;
        }
        let block_idx = block_idx.unwrap();

        let saved_stacks: HashMap<String, Vec<String>> = self.variable_stacks.clone();

        for (var, _) in self.phi_nodes.iter().filter(|((bl, _), _)| bl == block_label) {
            let new_name = self.fresh_name(&var.1);
            self.variable_stacks.entry(var.1.clone())
                .or_insert_with(Vec::new)
                .push(new_name);
        }

        let successors = self.get_successors_from_terminator(&function.blocks[block_idx].terminator);
        for succ in successors {
            for ((phi_block, phi_var), phi_args) in self.phi_nodes.iter_mut() {
                if phi_block == &succ {
                    if let Some(stack) = self.variable_stacks.get(phi_var) {
                        if let Some(current_name) = stack.last() {
                            phi_args.push((block_label.to_string(), current_name.clone()));
                        }
                    }
                }
            }
        }

        let dominated_blocks = self.get_dominated_blocks(block_label);
        for dominated in dominated_blocks {
            if dominated != block_label {
                self.rename_block(&dominated, function);
            }
        }

        self.variable_stacks = saved_stacks;
    }

    fn fresh_name(&mut self, var: &str) -> String {
        let counter = self.rename_counter.entry(var.to_string()).or_insert(0);
        let name = format!("{}.{}", var, counter);
        *counter += 1;
        name
    }

    fn get_predecessors(&self, block_label: &str, blocks: &[BasicBlock]) -> Vec<String> {
        let mut predecessors = Vec::new();
        for block in blocks {
            match &block.terminator {
                Terminator::Branch { target } if target == block_label => {
                    predecessors.push(block.label.clone());
                }
                Terminator::ConditionalBranch { true_target, false_target, .. } => {
                    if true_target == block_label || false_target == block_label {
                        predecessors.push(block.label.clone());
                    }
                }
                _ => {}
            }
        }
        predecessors
    }

    fn get_successors_from_terminator(&self, terminator: &Terminator) -> Vec<String> {
        match terminator {
            Terminator::Branch { target } => vec![target.clone()],
            Terminator::ConditionalBranch { true_target, false_target, .. } => {
                vec![true_target.clone(), false_target.clone()]
            }
            Terminator::Return(_) | Terminator::Unreachable => vec![],
        }
    }

    fn get_dominated_blocks(&self, dominator: &str) -> Vec<String> {
        self.immediate_dominators.iter()
            .filter(|(_, idom)| idom.as_str() == dominator)
            .map(|(block, _)| block.clone())
            .collect()
    }

    fn collect_variables(&self, function: &IRFunction) -> HashSet<String> {
        let mut variables = HashSet::new();
        for block in &function.blocks {
            for instruction in &block.instructions {
                match instruction {
                    Instruction::Alloca { result, .. } |
                    Instruction::Load { result, .. } |
                    Instruction::Add { result, .. } |
                    Instruction::Sub { result, .. } |
                    Instruction::Mul { result, .. } |
                    Instruction::Div { result, .. } |
                    Instruction::And { result, .. } |
                    Instruction::Or { result, .. } |
                    Instruction::Xor { result, .. } |
                    Instruction::Shl { result, .. } |
                    Instruction::Shr { result, .. } => {
                        variables.insert(result.clone());
                    }
                    _ => {}
                }
            }
        }
        variables
    }

    fn block_defines_variable(&self, block: &BasicBlock, var: &str) -> bool {
        for instruction in &block.instructions {
            match instruction {
                Instruction::Alloca { result, .. } |
                Instruction::Load { result, .. } |
                Instruction::Add { result, .. } |
                Instruction::Sub { result, .. } |
                Instruction::Mul { result, .. } |
                Instruction::Div { result, .. } => {
                    if result == var {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

pub fn optimize_ssa(function: &mut IRFunction) -> anyhow::Result<()> {
    remove_dead_phi_nodes(function);
    coalesce_phi_nodes(function);
    Ok(())
}

fn remove_dead_phi_nodes(function: &mut IRFunction) {
    for block in &mut function.blocks {
        block.instructions.retain(|inst| {
            if let Instruction::Phi { result, .. } = inst {
                is_variable_used(result, function)
            } else {
                true
            }
        });
    }
}

fn coalesce_phi_nodes(function: &mut IRFunction) {
    for block in &mut function.blocks {
        let mut phi_map: HashMap<String, String> = HashMap::new();
        
        for inst in &block.instructions {
            if let Instruction::Phi { result, incoming, .. } = inst {
                if incoming.len() == 1 {
                    if let Some((_, value)) = incoming.first() {
                        phi_map.insert(result.clone(), value.clone());
                    }
                }
            }
        }

        for inst in &mut block.instructions {
            replace_phi_uses(inst, &phi_map);
        }
    }
}

fn is_variable_used(var: &str, function: &IRFunction) -> bool {
    for block in &function.blocks {
        for inst in &block.instructions {
            if instruction_uses_variable(inst, var) {
                return true;
            }
        }
        if terminator_uses_variable(&block.terminator, var) {
            return true;
        }
    }
    false
}

fn instruction_uses_variable(inst: &Instruction, var: &str) -> bool {
    match inst {
        Instruction::Store { value, .. } if value == var => true,
        Instruction::Add { lhs, rhs, .. } |
        Instruction::Sub { lhs, rhs, .. } |
        Instruction::Mul { lhs, rhs, .. } |
        Instruction::Div { lhs, rhs, .. } => lhs == var || rhs == var,
        _ => false,
    }
}

fn terminator_uses_variable(term: &Terminator, var: &str) -> bool {
    match term {
        Terminator::ConditionalBranch { condition, .. } if condition == var => true,
        Terminator::Return(Some(val)) if val == var => true,
        _ => false,
    }
}

fn replace_phi_uses(inst: &mut Instruction, phi_map: &HashMap<String, String>) {
    match inst {
        Instruction::Store { value, .. } => {
            if let Some(replacement) = phi_map.get(value) {
                *value = replacement.clone();
            }
        }
        Instruction::Add { lhs, rhs, .. } |
        Instruction::Sub { lhs, rhs, .. } |
        Instruction::Mul { lhs, rhs, .. } |
        Instruction::Div { lhs, rhs, .. } => {
            if let Some(replacement) = phi_map.get(lhs) {
                *lhs = replacement.clone();
            }
            if let Some(replacement) = phi_map.get(rhs) {
                *rhs = replacement.clone();
            }
        }
        _ => {}
    }
}
