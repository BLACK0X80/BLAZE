use std::collections::{HashMap, HashSet, BTreeSet};
use crate::ir::{IRFunction, BasicBlock, Instruction};

#[derive(Debug, Clone)]
pub struct RegisterAllocator {
    physical_registers: Vec<String>,
    register_map: HashMap<String, String>,
    spilled_variables: HashSet<String>,
}

#[derive(Debug, Clone)]
struct LiveInterval {
    variable: String,
    start: usize,
    end: usize,
    uses: Vec<usize>,
}

#[derive(Debug, Clone)]
struct InterferenceGraph {
    nodes: HashSet<String>,
    edges: HashSet<(String, String)>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        let physical_registers = vec![
            "rax".to_string(),
            "rbx".to_string(),
            "rcx".to_string(),
            "rdx".to_string(),
            "rsi".to_string(),
            "rdi".to_string(),
            "r8".to_string(),
            "r9".to_string(),
            "r10".to_string(),
            "r11".to_string(),
            "r12".to_string(),
            "r13".to_string(),
            "r14".to_string(),
            "r15".to_string(),
        ];

        Self {
            physical_registers,
            register_map: HashMap::new(),
            spilled_variables: HashSet::new(),
        }
    }

    pub fn allocate(&mut self, function: &mut IRFunction) -> Result<(), String> {
        let live_intervals = self.compute_live_intervals(function);
        let interference_graph = self.build_interference_graph(&live_intervals);
        
        self.color_graph(&interference_graph)?;
        self.apply_allocation(function);
        
        Ok(())
    }

    fn compute_live_intervals(&self, function: &IRFunction) -> Vec<LiveInterval> {
        let mut intervals = Vec::new();
        let mut variable_positions = HashMap::new();
        let mut position = 0;

        for block in &function.blocks {
            for instruction in &block.instructions {
                if let Some(result) = instruction.get_result() {
                    variable_positions.insert(result.to_string(), position);
                }
                
                for operand in instruction.get_operands() {
                    if let Some(&start_pos) = variable_positions.get(operand) {
                        if let Some(interval) = intervals.iter_mut().find(|i| i.variable == operand) {
                            interval.end = position;
                            interval.uses.push(position);
                        } else {
                            intervals.push(LiveInterval {
                                variable: operand.to_string(),
                                start: start_pos,
                                end: position,
                                uses: vec![position],
                            });
                        }
                    }
                }
                
                position += 1;
            }
        }

        intervals.sort_by_key(|interval| interval.start);
        intervals
    }

    fn build_interference_graph(&self, intervals: &[LiveInterval]) -> InterferenceGraph {
        let mut graph = InterferenceGraph {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        };

        for interval in intervals {
            graph.nodes.insert(interval.variable.clone());
        }

        for i in 0..intervals.len() {
            for j in (i + 1)..intervals.len() {
                let interval1 = &intervals[i];
                let interval2 = &intervals[j];
                
                if self.intervals_interfere(interval1, interval2) {
                    let edge1 = (interval1.variable.clone(), interval2.variable.clone());
                    let edge2 = (interval2.variable.clone(), interval1.variable.clone());
                    graph.edges.insert(edge1);
                    graph.edges.insert(edge2);
                }
            }
        }

        graph
    }

    fn intervals_interfere(&self, interval1: &LiveInterval, interval2: &LiveInterval) -> bool {
        !(interval1.end < interval2.start || interval2.end < interval1.start)
    }

    fn color_graph(&mut self, graph: &InterferenceGraph) -> Result<(), String> {
        let mut coloring = HashMap::new();
        let mut nodes: Vec<_> = graph.nodes.iter().collect();
        
        nodes.sort_by_key(|node| {
            graph.edges.iter()
                .filter(|(a, _)| a == *node)
                .count()
        });

        for node in nodes {
            let mut available_colors: BTreeSet<usize> = (0..self.physical_registers.len()).collect();
            
            for (neighbor, _) in graph.edges.iter().filter(|(a, _)| a == node) {
                if let Some(&color) = coloring.get(neighbor) {
                    available_colors.remove(&color);
                }
            }
            
            if let Some(&color) = available_colors.iter().next() {
                coloring.insert(node.clone(), color);
                self.register_map.insert(
                    node.clone(), 
                    self.physical_registers[color].clone()
                );
            } else {
                self.spilled_variables.insert(node.clone());
                return Err(format!("Failed to allocate register for variable: {}", node));
            }
        }

        Ok(())
    }

    fn apply_allocation(&self, function: &mut IRFunction) {
        for block in &mut function.blocks {
            for instruction in &mut block.instructions {
                self.replace_operands_in_instruction(instruction);
            }
        }
    }

    fn replace_operands_in_instruction(&self, instruction: &mut Instruction) {
        match instruction {
            Instruction::Add { result, left, right, .. } => {
                if let Some(reg) = self.register_map.get(result) {
                    *result = reg.clone();
                }
                if let Some(reg) = self.register_map.get(left) {
                    *left = reg.clone();
                }
                if let Some(reg) = self.register_map.get(right) {
                    *right = reg.clone();
                }
            }
            Instruction::Sub { result, left, right, .. } => {
                if let Some(reg) = self.register_map.get(result) {
                    *result = reg.clone();
                }
                if let Some(reg) = self.register_map.get(left) {
                    *left = reg.clone();
                }
                if let Some(reg) = self.register_map.get(right) {
                    *right = reg.clone();
                }
            }
            Instruction::Mul { result, left, right, .. } => {
                if let Some(reg) = self.register_map.get(result) {
                    *result = reg.clone();
                }
                if let Some(reg) = self.register_map.get(left) {
                    *left = reg.clone();
                }
                if let Some(reg) = self.register_map.get(right) {
                    *right = reg.clone();
                }
            }
            Instruction::Load { result, ptr, .. } => {
                if let Some(reg) = self.register_map.get(result) {
                    *result = reg.clone();
                }
                if let Some(reg) = self.register_map.get(ptr) {
                    *ptr = reg.clone();
                }
            }
            Instruction::Store { value, ptr, .. } => {
                if let Some(reg) = self.register_map.get(value) {
                    *value = reg.clone();
                }
                if let Some(reg) = self.register_map.get(ptr) {
                    *ptr = reg.clone();
                }
            }
            _ => {}
        }
    }

    pub fn get_register_assignment(&self, variable: &str) -> Option<&str> {
        self.register_map.get(variable).map(|s| s.as_str())
    }

    pub fn is_spilled(&self, variable: &str) -> bool {
        self.spilled_variables.contains(variable)
    }

    pub fn get_spilled_variables(&self) -> &HashSet<String> {
        &self.spilled_variables
    }

    fn linear_scan_allocation(&mut self, intervals: &[LiveInterval]) -> Result<(), String> {
        let mut active = Vec::new();
        let mut sorted_intervals = intervals.to_vec();
        sorted_intervals.sort_by_key(|interval| interval.start);

        for current in sorted_intervals {
            self.expire_old_intervals(&mut active, current.start);
            
            if active.len() >= self.physical_registers.len() {
                self.spill_at_interval(&mut active, &current)?;
            } else {
                let register_index = active.len();
                self.register_map.insert(
                    current.variable.clone(),
                    self.physical_registers[register_index].clone(),
                );
                active.push(current);
                active.sort_by_key(|interval| interval.end);
            }
        }

        Ok(())
    }

    fn expire_old_intervals(&self, active: &mut Vec<LiveInterval>, current_start: usize) {
        active.retain(|interval| interval.end >= current_start);
    }

    fn spill_at_interval(&mut self, active: &mut Vec<LiveInterval>, current: &LiveInterval) -> Result<(), String> {
        let spill_candidate = active.last().unwrap();
        
        if spill_candidate.end > current.end {
            let register = self.register_map.remove(&spill_candidate.variable).unwrap();
            self.register_map.insert(current.variable.clone(), register);
            self.spilled_variables.insert(spill_candidate.variable.clone());
            
            active.pop();
            active.push(current.clone());
            active.sort_by_key(|interval| interval.end);
        } else {
            self.spilled_variables.insert(current.variable.clone());
        }

        Ok(())
    }
}

