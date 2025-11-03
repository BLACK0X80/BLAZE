use std::collections::{HashMap, HashSet, VecDeque};

pub struct SSAOptimizer {
    ssa_form: SSAProgram,
    value_numbers: HashMap<String, usize>,
    dominance_frontiers: HashMap<usize, HashSet<usize>>,
}

#[derive(Debug, Clone)]
pub struct SSAProgram {
    pub blocks: Vec<SSABlock>,
    pub entry_block: usize,
}

#[derive(Debug, Clone)]
pub struct SSABlock {
    pub id: usize,
    pub instructions: Vec<SSAInstruction>,
    pub phi_nodes: Vec<PhiNode>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum SSAInstruction {
    Assign { dest: String, value: SSAValue },
    BinaryOp { dest: String, op: String, left: SSAValue, right: SSAValue },
    UnaryOp { dest: String, op: String, operand: SSAValue },
    Call { dest: Option<String>, function: String, args: Vec<SSAValue> },
    Return { value: Option<SSAValue> },
    Branch { condition: SSAValue, true_block: usize, false_block: usize },
    Jump { target: usize },
}

#[derive(Debug, Clone)]
pub struct PhiNode {
    pub dest: String,
    pub sources: Vec<(SSAValue, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SSAValue {
    Variable(String),
    Constant(i64),
    Undef,
}

impl SSAOptimizer {
    pub fn new() -> Self {
        Self {
            ssa_form: SSAProgram {
                blocks: Vec::new(),
                entry_block: 0,
            },
            value_numbers: HashMap::new(),
            dominance_frontiers: HashMap::new(),
        }
    }
    
    pub fn optimize(&mut self) {
        self.global_value_numbering();
        self.sparse_conditional_constant_propagation();
        self.dead_code_elimination_ssa();
        self.copy_propagation();
        self.phi_node_elimination();
    }
    
    fn global_value_numbering(&mut self) {
        let mut next_value = 0;
        
        for block in &self.ssa_form.blocks {
            for instr in &block.instructions {
                match instr {
                    SSAInstruction::Assign { dest, .. } |
                    SSAInstruction::BinaryOp { dest, .. } |
                    SSAInstruction::UnaryOp { dest, .. } => {
                        if !self.value_numbers.contains_key(dest) {
                            self.value_numbers.insert(dest.clone(), next_value);
                            next_value += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn sparse_conditional_constant_propagation(&mut self) {
        let mut lattice: HashMap<String, LatticeValue> = HashMap::new();
        let mut ssa_worklist = VecDeque::new();
        let mut cfg_worklist = VecDeque::new();
        
        cfg_worklist.push_back(self.ssa_form.entry_block);
        
        while !ssa_worklist.is_empty() || !cfg_worklist.is_empty() {
            if let Some(block_id) = cfg_worklist.pop_front() {
                let block = &self.ssa_form.blocks[block_id];
                
                for phi in &block.phi_nodes {
                    let value = self.evaluate_phi(phi, &lattice);
                    if self.update_lattice(&phi.dest, value, &mut lattice) {
                        ssa_worklist.push_back(phi.dest.clone());
                    }
                }
                
                for instr in &block.instructions {
                    match instr {
                        SSAInstruction::BinaryOp { dest, op, left, right } => {
                            let left_val = self.get_lattice_value(left, &lattice);
                            let right_val = self.get_lattice_value(right, &lattice);
                            
                            let result = self.evaluate_binary(op, left_val, right_val);
                            if self.update_lattice(dest, result, &mut lattice) {
                                ssa_worklist.push_back(dest.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    fn evaluate_phi(&self, phi: &PhiNode, lattice: &HashMap<String, LatticeValue>) -> LatticeValue {
        let mut result = LatticeValue::Bottom;
        
        for (value, _) in &phi.sources {
            let val = self.get_lattice_value(value, lattice);
            result = self.meet(result, val);
        }
        
        result
    }
    
    fn get_lattice_value(&self, value: &SSAValue, lattice: &HashMap<String, LatticeValue>) -> LatticeValue {
        match value {
            SSAValue::Variable(var) => lattice.get(var).cloned().unwrap_or(LatticeValue::Top),
            SSAValue::Constant(c) => LatticeValue::Constant(*c),
            SSAValue::Undef => LatticeValue::Bottom,
        }
    }
    
    fn update_lattice(&self, var: &str, new_val: LatticeValue, lattice: &mut HashMap<String, LatticeValue>) -> bool {
        let old_val = lattice.get(var).cloned().unwrap_or(LatticeValue::Top);
        let merged = self.meet(old_val, new_val);
        
        if merged != old_val {
            lattice.insert(var.to_string(), merged);
            true
        } else {
            false
        }
    }
    
    fn meet(&self, a: LatticeValue, b: LatticeValue) -> LatticeValue {
        match (a, b) {
            (LatticeValue::Bottom, _) | (_, LatticeValue::Bottom) => LatticeValue::Bottom,
            (LatticeValue::Top, x) | (x, LatticeValue::Top) => x,
            (LatticeValue::Constant(c1), LatticeValue::Constant(c2)) => {
                if c1 == c2 {
                    LatticeValue::Constant(c1)
                } else {
                    LatticeValue::Bottom
                }
            }
        }
    }
    
    fn evaluate_binary(&self, op: &str, left: LatticeValue, right: LatticeValue) -> LatticeValue {
        match (left, right) {
            (LatticeValue::Constant(l), LatticeValue::Constant(r)) => {
                let result = match op {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" if r != 0 => l / r,
                    _ => return LatticeValue::Bottom,
                };
                LatticeValue::Constant(result)
            }
            (LatticeValue::Bottom, _) | (_, LatticeValue::Bottom) => LatticeValue::Bottom,
            _ => LatticeValue::Top,
        }
    }
    
    fn dead_code_elimination_ssa(&mut self) {
        let mut live = HashSet::new();
        let mut worklist = VecDeque::new();
        
        for block in &self.ssa_form.blocks {
            for instr in &block.instructions {
                match instr {
                    SSAInstruction::Return { .. } | SSAInstruction::Call { .. } => {
                        worklist.push_back(instr.clone());
                    }
                    _ => {}
                }
            }
        }
        
        while let Some(instr) = worklist.pop_front() {
            for used in self.get_used_vars(&instr) {
                if live.insert(used.clone()) {
                    for block in &self.ssa_form.blocks {
                        for instr in &block.instructions {
                            if self.defines_var(instr, &used) {
                                worklist.push_back(instr.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn copy_propagation(&mut self) {
        let mut copies: HashMap<String, SSAValue> = HashMap::new();
        
        for block in &mut self.ssa_form.blocks {
            for instr in &mut block.instructions {
                match instr {
                    SSAInstruction::Assign { dest, value } => {
                        if let SSAValue::Variable(_) = value {
                            copies.insert(dest.clone(), value.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn phi_node_elimination(&mut self) {
        for block in &mut self.ssa_form.blocks {
            block.phi_nodes.retain(|phi| {
                let all_same = phi.sources.iter().all(|(v, _)| v == &phi.sources[0].0);
                !all_same
            });
        }
    }
    
    fn get_used_vars(&self, instr: &SSAInstruction) -> Vec<String> {
        let mut vars = Vec::new();
        
        match instr {
            SSAInstruction::BinaryOp { left, right, .. } => {
                if let SSAValue::Variable(v) = left {
                    vars.push(v.clone());
                }
                if let SSAValue::Variable(v) = right {
                    vars.push(v.clone());
                }
            }
            SSAInstruction::UnaryOp { operand, .. } => {
                if let SSAValue::Variable(v) = operand {
                    vars.push(v.clone());
                }
            }
            _ => {}
        }
        
        vars
    }
    
    fn defines_var(&self, instr: &SSAInstruction, var: &str) -> bool {
        match instr {
            SSAInstruction::Assign { dest, .. } |
            SSAInstruction::BinaryOp { dest, .. } |
            SSAInstruction::UnaryOp { dest, .. } => dest == var,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LatticeValue {
    Top,
    Constant(i64),
    Bottom,
}

impl Default for SSAOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
