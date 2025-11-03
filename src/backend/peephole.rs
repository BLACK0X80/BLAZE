use crate::codegen::instruction_selection::X86Instruction;
use std::collections::HashMap;

pub struct PeepholeOptimizer {
    patterns: Vec<OptimizationPattern>,
}

struct OptimizationPattern {
    matcher: Box<dyn Fn(&[X86Instruction]) -> bool>,
    replacer: Box<dyn Fn(&[X86Instruction]) -> Vec<X86Instruction>>,
    pattern_size: usize,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        let mut optimizer = Self {
            patterns: Vec::new(),
        };
        
        optimizer.add_default_patterns();
        optimizer
    }
    
    fn add_default_patterns(&mut self) {
        self.add_pattern(
            2,
            Box::new(|instrs| {
                matches!(
                    (&instrs[0], &instrs[1]),
                    (X86Instruction::Mov { dest: d1, .. }, X86Instruction::Mov { dest: d2, src: s2 })
                    if format!("{:?}", d1) == format!("{:?}", s2)
                )
            }),
            Box::new(|instrs| vec![instrs[0].clone()]),
        );
        
        self.add_pattern(
            2,
            Box::new(|instrs| {
                matches!(
                    (&instrs[0], &instrs[1]),
                    (X86Instruction::Push { .. }, X86Instruction::Pop { .. })
                )
            }),
            Box::new(|_| Vec::new()),
        );
        
        self.add_pattern(
            1,
            Box::new(|instrs| {
                matches!(
                    &instrs[0],
                    X86Instruction::Add { src, .. } | X86Instruction::Sub { src, .. }
                    if matches!(src, crate::codegen::instruction_selection::Operand::Immediate(0))
                )
            }),
            Box::new(|_| Vec::new()),
        );
        
        self.add_pattern(
            1,
            Box::new(|instrs| {
                matches!(
                    &instrs[0],
                    X86Instruction::Imul { src, .. }
                    if matches!(src, crate::codegen::instruction_selection::Operand::Immediate(1))
                )
            }),
            Box::new(|_| Vec::new()),
        );
        
        self.add_pattern(
            2,
            Box::new(|instrs| {
                matches!(
                    (&instrs[0], &instrs[1]),
                    (X86Instruction::Xor { dest: d1, src: s1 }, _)
                    if format!("{:?}", d1) == format!("{:?}", s1)
                )
            }),
            Box::new(|instrs| {
                vec![
                    X86Instruction::Mov {
                        dest: if let X86Instruction::Xor { dest, .. } = &instrs[0] {
                            dest.clone()
                        } else {
                            unreachable!()
                        },
                        src: crate::codegen::instruction_selection::Operand::Immediate(0),
                    },
                    instrs[1].clone(),
                ]
            }),
        );
    }
    
    fn add_pattern(
        &mut self,
        size: usize,
        matcher: Box<dyn Fn(&[X86Instruction]) -> bool>,
        replacer: Box<dyn Fn(&[X86Instruction]) -> Vec<X86Instruction>>,
    ) {
        self.patterns.push(OptimizationPattern {
            matcher,
            replacer,
            pattern_size: size,
        });
    }
    
    pub fn optimize(&self, instructions: Vec<X86Instruction>) -> Vec<X86Instruction> {
        let mut optimized = instructions;
        let mut changed = true;
        
        while changed {
            changed = false;
            let new_instrs = self.optimize_pass(&optimized);
            if new_instrs.len() != optimized.len() {
                changed = true;
            }
            optimized = new_instrs;
        }
        
        optimized
    }
    
    fn optimize_pass(&self, instructions: &[X86Instruction]) -> Vec<X86Instruction> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < instructions.len() {
            let mut matched = false;
            
            for pattern in &self.patterns {
                if i + pattern.pattern_size <= instructions.len() {
                    let window = &instructions[i..i + pattern.pattern_size];
                    
                    if (pattern.matcher)(window) {
                        let replacement = (pattern.replacer)(window);
                        result.extend(replacement);
                        i += pattern.pattern_size;
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                result.push(instructions[i].clone());
                i += 1;
            }
        }
        
        result
    }
    
    pub fn eliminate_dead_code(&self, instructions: Vec<X86Instruction>) -> Vec<X86Instruction> {
        let live_vars = self.compute_live_variables(&instructions);
        
        instructions
            .into_iter()
            .enumerate()
            .filter(|(idx, instr)| {
                if let Some(def) = self.get_defined_variable(instr) {
                    live_vars.get(idx).map_or(true, |vars| vars.contains(&def))
                } else {
                    true
                }
            })
            .map(|(_, instr)| instr)
            .collect()
    }
    
    fn compute_live_variables(&self, instructions: &[X86Instruction]) -> HashMap<usize, std::collections::HashSet<String>> {
        let mut live_vars: HashMap<usize, std::collections::HashSet<String>> = HashMap::new();
        
        for i in (0..instructions.len()).rev() {
            let mut live = if i + 1 < instructions.len() {
                live_vars.get(&(i + 1)).cloned().unwrap_or_default()
            } else {
                std::collections::HashSet::new()
            };
            
            if let Some(def) = self.get_defined_variable(&instructions[i]) {
                live.remove(&def);
            }
            
            for var in self.get_used_variables(&instructions[i]) {
                live.insert(var);
            }
            
            live_vars.insert(i, live);
        }
        
        live_vars
    }
    
    fn get_defined_variable(&self, instr: &X86Instruction) -> Option<String> {
        match instr {
            X86Instruction::Mov { dest, .. }
            | X86Instruction::Add { dest, .. }
            | X86Instruction::Sub { dest, .. }
            | X86Instruction::Imul { dest, .. } => {
                if let crate::codegen::instruction_selection::Operand::Label(name) = dest {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    fn get_used_variables(&self, instr: &X86Instruction) -> Vec<String> {
        let mut vars = Vec::new();
        
        match instr {
            X86Instruction::Mov { src, .. }
            | X86Instruction::Add { src, .. }
            | X86Instruction::Sub { src, .. }
            | X86Instruction::Imul { src, .. } => {
                if let crate::codegen::instruction_selection::Operand::Label(name) = src {
                    vars.push(name.clone());
                }
            }
            X86Instruction::Cmp { left, right } => {
                if let crate::codegen::instruction_selection::Operand::Label(name) = left {
                    vars.push(name.clone());
                }
                if let crate::codegen::instruction_selection::Operand::Label(name) = right {
                    vars.push(name.clone());
                }
            }
            _ => {}
        }
        
        vars
    }
}

impl Default for PeepholeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
