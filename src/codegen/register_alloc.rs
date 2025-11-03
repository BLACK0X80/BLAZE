use std::collections::{HashMap, HashSet};
use crate::ir::{Value, Function, Instruction};
use crate::analysis::control_flow::{ControlFlowGraph, BlockId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP,
    R8, R9, R10, R11, R12, R13, R14, R15,
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
}

pub struct RegisterAllocator {
    allocation: HashMap<String, Register>,
    free_registers: HashSet<Register>,
    spilled_variables: HashSet<String>,
    stack_offset: usize,
    interference_graph: InterferenceGraph,
}

struct InterferenceGraph {
    nodes: HashSet<String>,
    edges: HashSet<(String, String)>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        let mut free_registers = HashSet::new();
        free_registers.insert(Register::RAX);
        free_registers.insert(Register::RBX);
        free_registers.insert(Register::RCX);
        free_registers.insert(Register::RDX);
        free_registers.insert(Register::RSI);
        free_registers.insert(Register::RDI);
        free_registers.insert(Register::R8);
        free_registers.insert(Register::R9);
        free_registers.insert(Register::R10);
        free_registers.insert(Register::R11);
        
        Self {
            allocation: HashMap::new(),
            free_registers,
            spilled_variables: HashSet::new(),
            stack_offset: 0,
            interference_graph: InterferenceGraph::new(),
        }
    }
    
    pub fn allocate(&mut self, function: &Function, cfg: &ControlFlowGraph) -> AllocationResult {
        self.build_interference_graph(function, cfg);
        
        let coloring = self.graph_coloring();
        
        for (var, color) in coloring {
            if let Some(reg) = self.color_to_register(color) {
                self.allocation.insert(var, reg);
            } else {
                self.spill_variable(var);
            }
        }
        
        AllocationResult {
            allocation: self.allocation.clone(),
            spilled: self.spilled_variables.clone(),
            stack_size: self.stack_offset,
        }
    }
    
    fn build_interference_graph(&mut self, function: &Function, cfg: &ControlFlowGraph) {
        let live_ranges = self.compute_live_ranges(function, cfg);
        
        for (var1, range1) in &live_ranges {
            self.interference_graph.add_node(var1.clone());
            
            for (var2, range2) in &live_ranges {
                if var1 != var2 && self.ranges_overlap(range1, range2) {
                    self.interference_graph.add_edge(var1.clone(), var2.clone());
                }
            }
        }
    }
    
    fn compute_live_ranges(&self, function: &Function, cfg: &ControlFlowGraph) -> HashMap<String, LiveRange> {
        let mut live_ranges = HashMap::new();
        
        for (block_idx, block) in function.blocks.iter().enumerate() {
            for (instr_idx, instr) in block.instructions.iter().enumerate() {
                let position = block_idx * 1000 + instr_idx;
                
                match instr {
                    Instruction::Store { dest, .. } |
                    Instruction::Binary { result: dest, .. } |
                    Instruction::Unary { result: dest, .. } => {
                        live_ranges
                            .entry(dest.clone())
                            .or_insert_with(|| LiveRange::new(position))
                            .extend(position);
                    }
                    _ => {}
                }
                
                for var in self.get_used_variables(instr) {
                    live_ranges
                        .entry(var.clone())
                        .or_insert_with(|| LiveRange::new(position))
                        .extend(position);
                }
            }
        }
        
        live_ranges
    }
    
    fn get_used_variables(&self, instr: &Instruction) -> Vec<String> {
        let mut vars = Vec::new();
        
        match instr {
            Instruction::Binary { left, right, .. } => {
                if let Value::Variable(v) = left {
                    vars.push(v.clone());
                }
                if let Value::Variable(v) = right {
                    vars.push(v.clone());
                }
            }
            Instruction::Unary { operand, .. } => {
                if let Value::Variable(v) = operand {
                    vars.push(v.clone());
                }
            }
            Instruction::Call { args, .. } => {
                for arg in args {
                    if let Value::Variable(v) = arg {
                        vars.push(v.clone());
                    }
                }
            }
            Instruction::Return { value: Some(Value::Variable(v)) } => {
                vars.push(v.clone());
            }
            _ => {}
        }
        
        vars
    }
    
    fn ranges_overlap(&self, range1: &LiveRange, range2: &LiveRange) -> bool {
        !(range1.end < range2.start || range2.end < range1.start)
    }
    
    fn graph_coloring(&self) -> HashMap<String, usize> {
        let mut coloring = HashMap::new();
        let mut stack = Vec::new();
        let mut remaining_nodes: HashSet<_> = self.interference_graph.nodes.iter().cloned().collect();
        
        while !remaining_nodes.is_empty() {
            if let Some(node) = self.find_low_degree_node(&remaining_nodes) {
                stack.push(node.clone());
                remaining_nodes.remove(&node);
            } else if let Some(node) = self.select_spill_candidate(&remaining_nodes) {
                stack.push(node.clone());
                remaining_nodes.remove(&node);
            } else {
                break;
            }
        }
        
        while let Some(node) = stack.pop() {
            let used_colors = self.get_neighbor_colors(&node, &coloring);
            let color = self.find_available_color(&used_colors);
            coloring.insert(node, color);
        }
        
        coloring
    }
    
    fn find_low_degree_node(&self, nodes: &HashSet<String>) -> Option<String> {
        const MAX_REGISTERS: usize = 10;
        
        for node in nodes {
            let degree = self.interference_graph.get_degree(node, nodes);
            if degree < MAX_REGISTERS {
                return Some(node.clone());
            }
        }
        
        None
    }
    
    fn select_spill_candidate(&self, nodes: &HashSet<String>) -> Option<String> {
        nodes.iter().next().cloned()
    }
    
    fn get_neighbor_colors(&self, node: &str, coloring: &HashMap<String, usize>) -> HashSet<usize> {
        self.interference_graph
            .get_neighbors(node)
            .iter()
            .filter_map(|n| coloring.get(n).copied())
            .collect()
    }
    
    fn find_available_color(&self, used_colors: &HashSet<usize>) -> usize {
        (0..=15)
            .find(|c| !used_colors.contains(c))
            .unwrap_or(16)
    }
    
    fn color_to_register(&self, color: usize) -> Option<Register> {
        match color {
            0 => Some(Register::RAX),
            1 => Some(Register::RBX),
            2 => Some(Register::RCX),
            3 => Some(Register::RDX),
            4 => Some(Register::RSI),
            5 => Some(Register::RDI),
            6 => Some(Register::R8),
            7 => Some(Register::R9),
            8 => Some(Register::R10),
            9 => Some(Register::R11),
            10 => Some(Register::R12),
            11 => Some(Register::R13),
            12 => Some(Register::R14),
            13 => Some(Register::R15),
            _ => None,
        }
    }
    
    fn spill_variable(&mut self, var: String) {
        self.spilled_variables.insert(var);
        self.stack_offset += 8;
    }
    
    pub fn get_register(&self, var: &str) -> Option<Register> {
        self.allocation.get(var).copied()
    }
    
    pub fn get_stack_offset(&self, var: &str) -> Option<usize> {
        if self.spilled_variables.contains(var) {
            Some(self.spilled_variables.iter().position(|v| v == var).unwrap() * 8)
        } else {
            None
        }
    }
}

impl InterferenceGraph {
    fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }
    
    fn add_node(&mut self, node: String) {
        self.nodes.insert(node);
    }
    
    fn add_edge(&mut self, a: String, b: String) {
        if a != b {
            self.edges.insert((a.clone(), b.clone()));
            self.edges.insert((b, a));
        }
    }
    
    fn get_degree(&self, node: &str, active_nodes: &HashSet<String>) -> usize {
        self.edges
            .iter()
            .filter(|(a, b)| a == node && active_nodes.contains(b))
            .count()
    }
    
    fn get_neighbors(&self, node: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(a, _)| a == node)
            .map(|(_, b)| b.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
struct LiveRange {
    start: usize,
    end: usize,
}

impl LiveRange {
    fn new(position: usize) -> Self {
        Self {
            start: position,
            end: position,
        }
    }
    
    fn extend(&mut self, position: usize) {
        self.start = self.start.min(position);
        self.end = self.end.max(position);
    }
}

#[derive(Debug, Clone)]
pub struct AllocationResult {
    pub allocation: HashMap<String, Register>,
    pub spilled: HashSet<String>,
    pub stack_size: usize,
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Register {
    pub fn to_string(&self) -> &'static str {
        match self {
            Register::RAX => "rax",
            Register::RBX => "rbx",
            Register::RCX => "rcx",
            Register::RDX => "rdx",
            Register::RSI => "rsi",
            Register::RDI => "rdi",
            Register::RBP => "rbp",
            Register::RSP => "rsp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::XMM0 => "xmm0",
            Register::XMM1 => "xmm1",
            Register::XMM2 => "xmm2",
            Register::XMM3 => "xmm3",
            Register::XMM4 => "xmm4",
            Register::XMM5 => "xmm5",
            Register::XMM6 => "xmm6",
            Register::XMM7 => "xmm7",
        }
    }
    
    pub fn is_caller_saved(&self) -> bool {
        matches!(
            self,
            Register::RAX
                | Register::RCX
                | Register::RDX
                | Register::RSI
                | Register::RDI
                | Register::R8
                | Register::R9
                | Register::R10
                | Register::R11
        )
    }
    
    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            Register::RBX | Register::R12 | Register::R13 | Register::R14 | Register::R15
        )
    }
}
