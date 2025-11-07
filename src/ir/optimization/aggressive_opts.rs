use crate::ir::{Module, IRFunction, BasicBlock, Instruction, Terminator, IRType};
use std::collections::{HashMap, HashSet};

pub struct AggressiveOptimizer {
    inline_threshold: usize,
    loop_unroll_threshold: usize,
    use_counts: HashMap<String, usize>,
}

impl AggressiveOptimizer {
    pub fn new() -> Self {
        Self {
            inline_threshold: 50,
            loop_unroll_threshold: 4,
            use_counts: HashMap::new(),
        }
    }

    pub fn optimize(&mut self, module: &mut Module) -> anyhow::Result<()> {
        self.global_value_numbering(module)?;
        self.loop_invariant_code_motion(module)?;
        self.strength_reduction(module)?;
        self.tail_call_optimization(module)?;
        self.aggressive_inlining(module)?;
        Ok(())
    }

    fn global_value_numbering(&mut self, module: &mut Module) -> anyhow::Result<()> {
        for function in &mut module.functions {
            let mut value_map: HashMap<String, String> = HashMap::new();
            
            for block in &mut function.blocks {
                for inst in &mut block.instructions {
                    let expr = instruction_to_expression(inst);
                    if let Some(expr_str) = expr {
                        if let Some(existing) = value_map.get(&expr_str) {
                            if let Some(result) = get_instruction_result(inst) {
                                value_map.insert(result.clone(), existing.clone());
                            }
                        } else if let Some(result) = get_instruction_result(inst) {
                            value_map.insert(expr_str, result.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn loop_invariant_code_motion(&mut self, module: &mut Module) -> anyhow::Result<()> {
        for function in &mut module.functions {
            let loops = identify_loops(function);
            
            for loop_blocks in loops {
                let mut invariant_instructions = Vec::new();
                
                for block_label in &loop_blocks {
                    if let Some(block) = function.blocks.iter().find(|b| &b.label == block_label) {
                        for inst in &block.instructions {
                            if is_loop_invariant(inst, &loop_blocks, function) {
                                invariant_instructions.push(inst.clone());
                            }
                        }
                    }
                }

                if !invariant_instructions.is_empty() && !loop_blocks.is_empty() {
                    let preheader = find_loop_preheader(&loop_blocks[0], function);
                    if let Some(preheader_label) = preheader {
                        if let Some(preheader_block) = function.blocks.iter_mut()
                            .find(|b| b.label == preheader_label) {
                            preheader_block.instructions.extend(invariant_instructions.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn strength_reduction(&mut self, module: &mut Module) -> anyhow::Result<()> {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                for inst in &mut block.instructions {
                    match inst {
                        Instruction::Mul { result, lhs, rhs, ty } => {
                            if let Some(power) = is_power_of_two(rhs) {
                                *inst = Instruction::Shl {
                                    result: result.clone(),
                                    lhs: lhs.clone(),
                                    rhs: power.to_string(),
                                    ty: ty.clone(),
                                };
                            }
                        }
                        Instruction::Div { result, lhs, rhs, ty } => {
                            if let Some(power) = is_power_of_two(rhs) {
                                *inst = Instruction::Shr {
                                    result: result.clone(),
                                    lhs: lhs.clone(),
                                    rhs: power.to_string(),
                                    ty: ty.clone(),
                                };
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn tail_call_optimization(&mut self, module: &mut Module) -> anyhow::Result<()> {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                if let Terminator::Return(Some(ret_val)) = &block.terminator {
                    if let Some(last_inst) = block.instructions.last() {
                        if let Instruction::Call { callee, args, .. } = last_inst {
                            if is_tail_recursive(callee, &function.name) {
                                convert_to_loop(block, args);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn aggressive_inlining(&mut self, module: &mut Module) -> anyhow::Result<()> {
        let mut inline_candidates = Vec::new();
        
        for function in &module.functions {
            let cost = estimate_function_cost(function);
            if cost <= self.inline_threshold {
                inline_candidates.push(function.name.clone());
            }
        }

        for function in &mut module.functions {
            let mut inline_sites = Vec::new();
            
            for (block_idx, block) in function.blocks.iter().enumerate() {
                for (inst_idx, inst) in block.instructions.iter().enumerate() {
                    if let Instruction::Call { callee, args, .. } = inst {
                        if inline_candidates.contains(callee) {
                            inline_sites.push((block_idx, inst_idx, callee.clone(), args.clone()));
                        }
                    }
                }
            }

            for (block_idx, inst_idx, callee, args) in inline_sites.iter().rev() {
                if let Some(callee_func) = module.functions.iter()
                    .find(|f| &f.name == callee) {
                    inline_function_at(function, *block_idx, *inst_idx, callee_func, args);
                }
            }
        }
        
        Ok(())
    }
}

fn instruction_to_expression(inst: &Instruction) -> Option<String> {
    match inst {
        Instruction::Add { lhs, rhs, .. } => Some(format!("add({}, {})", lhs, rhs)),
        Instruction::Sub { lhs, rhs, .. } => Some(format!("sub({}, {})", lhs, rhs)),
        Instruction::Mul { lhs, rhs, .. } => Some(format!("mul({}, {})", lhs, rhs)),
        Instruction::Div { lhs, rhs, .. } => Some(format!("div({}, {})", lhs, rhs)),
        _ => None,
    }
}

fn get_instruction_result(inst: &Instruction) -> Option<String> {
    match inst {
        Instruction::Add { result, .. } |
        Instruction::Sub { result, .. } |
        Instruction::Mul { result, .. } |
        Instruction::Div { result, .. } |
        Instruction::Load { result, .. } |
        Instruction::Alloca { result, .. } => Some(result.clone()),
        _ => None,
    }
}

fn identify_loops(function: &IRFunction) -> Vec<Vec<String>> {
    let mut loops = Vec::new();
    let cfg = build_cfg(function);
    let back_edges = find_back_edges(&cfg, function);
    
    for (tail, head) in back_edges {
        let loop_blocks = find_loop_blocks(&tail, &head, &cfg);
        loops.push(loop_blocks);
    }
    
    loops
}

fn build_cfg(function: &IRFunction) -> HashMap<String, HashSet<String>> {
    let mut cfg: HashMap<String, HashSet<String>> = HashMap::new();
    
    for block in &function.blocks {
        let succs = match &block.terminator {
            Terminator::Branch { target } => vec![target.clone()],
            Terminator::ConditionalBranch { true_target, false_target, .. } => {
                vec![true_target.clone(), false_target.clone()]
            }
            _ => vec![],
        };
        cfg.insert(block.label.clone(), succs.into_iter().collect());
    }
    
    cfg
}

fn find_back_edges(cfg: &HashMap<String, HashSet<String>>, function: &IRFunction) -> Vec<(String, String)> {
    let mut back_edges = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    
    if !function.blocks.is_empty() {
        dfs_back_edges(&function.blocks[0].label, cfg, &mut visited, &mut stack, &mut back_edges);
    }
    
    back_edges
}

fn dfs_back_edges(
    node: &str,
    cfg: &HashMap<String, HashSet<String>>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
    back_edges: &mut Vec<(String, String)>,
) {
    visited.insert(node.to_string());
    stack.insert(node.to_string());
    
    if let Some(successors) = cfg.get(node) {
        for succ in successors {
            if stack.contains(succ) {
                back_edges.push((node.to_string(), succ.clone()));
            } else if !visited.contains(succ) {
                dfs_back_edges(succ, cfg, visited, stack, back_edges);
            }
        }
    }
    
    stack.remove(node);
}

fn find_loop_blocks(tail: &str, head: &str, cfg: &HashMap<String, HashSet<String>>) -> Vec<String> {
    let mut loop_blocks = vec![head.to_string()];
    let mut worklist = vec![tail.to_string()];
    let mut visited = HashSet::new();
    visited.insert(head.to_string());
    
    while let Some(block) = worklist.pop() {
        if visited.contains(&block) {
            continue;
        }
        visited.insert(block.clone());
        loop_blocks.push(block.clone());
        
        for (pred, succs) in cfg {
            if succs.contains(&block) && !visited.contains(pred) {
                worklist.push(pred.clone());
            }
        }
    }
    
    loop_blocks
}

fn is_loop_invariant(inst: &Instruction, loop_blocks: &[String], function: &IRFunction) -> bool {
    let operands = get_instruction_operands(inst);
    
    for operand in operands {
        if is_defined_in_loop(&operand, loop_blocks, function) {
            return false;
        }
    }
    
    true
}

fn get_instruction_operands(inst: &Instruction) -> Vec<String> {
    match inst {
        Instruction::Add { lhs, rhs, .. } |
        Instruction::Sub { lhs, rhs, .. } |
        Instruction::Mul { lhs, rhs, .. } |
        Instruction::Div { lhs, rhs, .. } => vec![lhs.clone(), rhs.clone()],
        Instruction::Load { pointer, .. } => vec![pointer.clone()],
        Instruction::Store { pointer, value, .. } => vec![pointer.clone(), value.clone()],
        _ => vec![],
    }
}

fn is_defined_in_loop(var: &str, loop_blocks: &[String], function: &IRFunction) -> bool {
    for block_label in loop_blocks {
        if let Some(block) = function.blocks.iter().find(|b| &b.label == block_label) {
            for inst in &block.instructions {
                if let Some(result) = get_instruction_result(inst) {
                    if result == var {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn find_loop_preheader(loop_head: &str, function: &IRFunction) -> Option<String> {
    for block in &function.blocks {
        match &block.terminator {
            Terminator::Branch { target } if target == loop_head => {
                return Some(block.label.clone());
            }
            Terminator::ConditionalBranch { true_target, false_target, .. } => {
                if true_target == loop_head || false_target == loop_head {
                    return Some(block.label.clone());
                }
            }
            _ => {}
        }
    }
    None
}

fn is_power_of_two(val: &str) -> Option<u32> {
    if let Ok(n) = val.parse::<u64>() {
        if n > 0 && (n & (n - 1)) == 0 {
            return Some(n.trailing_zeros());
        }
    }
    None
}

fn is_tail_recursive(callee: &str, function_name: &str) -> bool {
    callee == function_name
}

fn convert_to_loop(block: &mut BasicBlock, args: &[String]) {
    block.terminator = Terminator::Branch {
        target: block.label.clone(),
    };
}

fn estimate_function_cost(function: &IRFunction) -> usize {
    let mut cost = 0;
    for block in &function.blocks {
        cost += block.instructions.len();
        cost += match &block.terminator {
            Terminator::ConditionalBranch { .. } => 2,
            _ => 1,
        };
    }
    cost
}

fn inline_function_at(
    caller: &mut IRFunction,
    block_idx: usize,
    inst_idx: usize,
    callee: &IRFunction,
    args: &[String],
) {
    let mut inlined_blocks = callee.blocks.clone();
    
    for (i, param) in callee.params.iter().enumerate() {
        if let Some(arg) = args.get(i) {
            rename_variable_in_blocks(&mut inlined_blocks, &param.name, arg);
        }
    }

    if block_idx < caller.blocks.len() {
        caller.blocks.splice(block_idx + 1..block_idx + 1, inlined_blocks);
        caller.blocks[block_idx].instructions.remove(inst_idx);
    }
}

fn rename_variable_in_blocks(blocks: &mut [BasicBlock], old_name: &str, new_name: &str) {
    for block in blocks {
        for inst in &mut block.instructions {
            rename_variable_in_instruction(inst, old_name, new_name);
        }
    }
}

fn rename_variable_in_instruction(inst: &mut Instruction, old_name: &str, new_name: &str) {
    match inst {
        Instruction::Add { lhs, rhs, .. } |
        Instruction::Sub { lhs, rhs, .. } |
        Instruction::Mul { lhs, rhs, .. } |
        Instruction::Div { lhs, rhs, .. } => {
            if lhs == old_name {
                *lhs = new_name.to_string();
            }
            if rhs == old_name {
                *rhs = new_name.to_string();
            }
        }
        Instruction::Load { pointer, .. } => {
            if pointer == old_name {
                *pointer = new_name.to_string();
            }
        }
        Instruction::Store { pointer, value, .. } => {
            if pointer == old_name {
                *pointer = new_name.to_string();
            }
            if value == old_name {
                *value = new_name.to_string();
            }
        }
        _ => {}
    }
}
