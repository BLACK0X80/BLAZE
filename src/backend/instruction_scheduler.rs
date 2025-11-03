use std::collections::{HashMap, HashSet, VecDeque};

pub struct InstructionScheduler {
    dag: DependencyDAG,
    ready_list: VecDeque<usize>,
    scheduled: Vec<usize>,
    latencies: HashMap<InstructionType, usize>,
}

#[derive(Debug, Clone)]
pub struct DependencyDAG {
    nodes: Vec<DAGNode>,
    edges: Vec<(usize, usize, DependencyType)>,
}

#[derive(Debug, Clone)]
pub struct DAGNode {
    pub id: usize,
    pub instruction: String,
    pub instr_type: InstructionType,
    pub cycle: usize,
    pub predecessors: Vec<usize>,
    pub successors: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstructionType {
    Load,
    Store,
    IntALU,
    IntMul,
    IntDiv,
    FloatALU,
    FloatMul,
    FloatDiv,
    Branch,
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DependencyType {
    RAW,
    WAR,
    WAW,
    Control,
}

impl InstructionScheduler {
    pub fn new() -> Self {
        let mut scheduler = Self {
            dag: DependencyDAG::new(),
            ready_list: VecDeque::new(),
            scheduled: Vec::new(),
            latencies: HashMap::new(),
        };
        
        scheduler.initialize_latencies();
        scheduler
    }
    
    fn initialize_latencies(&mut self) {
        self.latencies.insert(InstructionType::Load, 3);
        self.latencies.insert(InstructionType::Store, 1);
        self.latencies.insert(InstructionType::IntALU, 1);
        self.latencies.insert(InstructionType::IntMul, 3);
        self.latencies.insert(InstructionType::IntDiv, 20);
        self.latencies.insert(InstructionType::FloatALU, 2);
        self.latencies.insert(InstructionType::FloatMul, 4);
        self.latencies.insert(InstructionType::FloatDiv, 12);
        self.latencies.insert(InstructionType::Branch, 1);
        self.latencies.insert(InstructionType::Call, 2);
    }
    
    pub fn schedule(&mut self, instructions: Vec<String>) -> Vec<String> {
        self.build_dag(&instructions);
        self.find_critical_path();
        self.list_scheduling();
        
        self.scheduled
            .iter()
            .map(|&id| self.dag.nodes[id].instruction.clone())
            .collect()
    }
    
    fn build_dag(&mut self, instructions: &[String]) {
        for (id, instr) in instructions.iter().enumerate() {
            let node = DAGNode {
                id,
                instruction: instr.clone(),
                instr_type: self.classify_instruction(instr),
                cycle: 0,
                predecessors: Vec::new(),
                successors: Vec::new(),
            };
            self.dag.add_node(node);
        }
        
        self.analyze_dependencies();
    }
    
    fn classify_instruction(&self, instr: &str) -> InstructionType {
        if instr.contains("load") || instr.contains("mov") && instr.contains("(") {
            InstructionType::Load
        } else if instr.contains("store") || instr.contains("mov") && instr.contains(")") {
            InstructionType::Store
        } else if instr.contains("mul") {
            InstructionType::IntMul
        } else if instr.contains("div") {
            InstructionType::IntDiv
        } else if instr.contains("add") || instr.contains("sub") {
            InstructionType::IntALU
        } else if instr.contains("jmp") || instr.contains("br") {
            InstructionType::Branch
        } else if instr.contains("call") {
            InstructionType::Call
        } else {
            InstructionType::IntALU
        }
    }
    
    fn analyze_dependencies(&mut self) {
        let mut defs: HashMap<String, usize> = HashMap::new();
        let mut uses: HashMap<String, Vec<usize>> = HashMap::new();
        
        for i in 0..self.dag.nodes.len() {
            let defined = self.get_defined_register(&self.dag.nodes[i].instruction);
            let used = self.get_used_registers(&self.dag.nodes[i].instruction);
            
            for reg in &used {
                if let Some(&def_id) = defs.get(reg) {
                    self.dag.add_edge(def_id, i, DependencyType::RAW);
                }
                uses.entry(reg.clone()).or_insert_with(Vec::new).push(i);
            }
            
            if let Some(ref reg) = defined {
                if let Some(def_id) = defs.get(reg) {
                    self.dag.add_edge(*def_id, i, DependencyType::WAW);
                }
                
                if let Some(use_ids) = uses.get(reg) {
                    for &use_id in use_ids {
                        if use_id < i {
                            self.dag.add_edge(use_id, i, DependencyType::WAR);
                        }
                    }
                }
                
                defs.insert(reg.clone(), i);
            }
        }
    }
    
    fn get_defined_register(&self, instr: &str) -> Option<String> {
        let parts: Vec<&str> = instr.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(parts[1].trim_end_matches(',').to_string())
        } else {
            None
        }
    }
    
    fn get_used_registers(&self, instr: &str) -> Vec<String> {
        let parts: Vec<&str> = instr.split_whitespace().collect();
        parts.iter().skip(2).map(|s| s.trim_end_matches(',').to_string()).collect()
    }
    
    fn find_critical_path(&mut self) {
        let mut max_cycle = vec![0; self.dag.nodes.len()];
        
        for i in 0..self.dag.nodes.len() {
            for &pred in &self.dag.nodes[i].predecessors {
                let latency = self.latencies.get(&self.dag.nodes[pred].instr_type).unwrap_or(&1);
                max_cycle[i] = max_cycle[i].max(max_cycle[pred] + latency);
            }
            self.dag.nodes[i].cycle = max_cycle[i];
        }
    }
    
    fn list_scheduling(&mut self) {
        self.ready_list.clear();
        self.scheduled.clear();
        
        let mut remaining_deps = vec![0; self.dag.nodes.len()];
        for i in 0..self.dag.nodes.len() {
            remaining_deps[i] = self.dag.nodes[i].predecessors.len();
            if remaining_deps[i] == 0 {
                self.ready_list.push_back(i);
            }
        }
        
        while let Some(node_id) = self.select_next_instruction() {
            self.scheduled.push(node_id);
            
            for &succ in &self.dag.nodes[node_id].successors {
                remaining_deps[succ] -= 1;
                if remaining_deps[succ] == 0 {
                    self.ready_list.push_back(succ);
                }
            }
        }
    }
    
    fn select_next_instruction(&mut self) -> Option<usize> {
        if self.ready_list.is_empty() {
            return None;
        }
        
        let mut best_idx = 0;
        let mut best_priority = 0;
        
        for (i, &node_id) in self.ready_list.iter().enumerate() {
            let priority = self.compute_priority(node_id);
            if priority > best_priority {
                best_priority = priority;
                best_idx = i;
            }
        }
        
        self.ready_list.remove(best_idx)
    }
    
    fn compute_priority(&self, node_id: usize) -> usize {
        self.dag.nodes[node_id].cycle
    }
}

impl DependencyDAG {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    fn add_node(&mut self, node: DAGNode) {
        self.nodes.push(node);
    }
    
    fn add_edge(&mut self, from: usize, to: usize, dep_type: DependencyType) {
        self.edges.push((from, to, dep_type));
        self.nodes[from].successors.push(to);
        self.nodes[to].predecessors.push(from);
    }
}

impl Default for InstructionScheduler {
    fn default() -> Self {
        Self::new()
    }
}
