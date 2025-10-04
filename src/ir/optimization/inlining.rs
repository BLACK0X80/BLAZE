use super::{OptimizationPass, Module};
use crate::ir::{IRFunction, Instruction, BasicBlock};
use std::collections::HashMap;
use anyhow::Result;

pub struct Inliner {
    inline_threshold: usize,
}

impl Inliner {
    pub fn new() -> Self {
        Self {
            inline_threshold: 50,
        }
    }

    fn should_inline(&self, function: &IRFunction) -> bool {
        let instruction_count: usize = function.blocks.iter()
            .map(|block| block.instructions.len())
            .sum();
        
        instruction_count < self.inline_threshold && 
        !self.is_recursive(function) &&
        !self.has_complex_control_flow(function)
    }

    fn is_recursive(&self, function: &IRFunction) -> bool {
        for block in &function.blocks {
            for instruction in &block.instructions {
                if let Instruction::Call { function: callee, .. } = instruction {
                    if callee == &function.name {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_complex_control_flow(&self, function: &IRFunction) -> bool {
        function.blocks.len() > 5
    }

    fn count_call_sites(&self, function_name: &str, module: &Module) -> usize {
        let mut count = 0;
        
        for function in &module.functions {
            for block in &function.blocks {
                for instruction in &block.instructions {
                    if let Instruction::Call { function: callee, .. } = instruction {
                        if callee == function_name {
                            count += 1;
                        }
                    }
                }
            }
        }
        
        count
    }

    fn inline_function_call(
        &self,
        caller: &mut IRFunction,
        callee: &IRFunction,
        call_instruction: &Instruction,
        block_index: usize,
        instruction_index: usize,
    ) -> Result<()> {
        let call_block = &mut caller.blocks[block_index];
        
        let mut new_instructions = Vec::new();
        new_instructions.extend_from_slice(&call_block.instructions[..instruction_index]);
        
        let mut variable_mapping = HashMap::new();
        let mut next_temp = self.find_next_temp_number(caller);
        
        if let Instruction::Call { result, args, .. } = call_instruction {
            for (i, param) in callee.params.iter().enumerate() {
                if let Some((_, arg_value)) = args.get(i) {
                    variable_mapping.insert(param.name.clone(), arg_value.clone());
                }
            }
            
            for block in &callee.blocks {
                for instruction in &block.instructions {
                    let inlined_instruction = self.remap_instruction(
                        instruction, 
                        &variable_mapping, 
                        &mut next_temp
                    );
                    new_instructions.push(inlined_instruction);
                }
            }
            
            if let Some(result_var) = result {
                if let Some(return_block) = callee.blocks.last() {
                    if let crate::ir::Terminator::Return { value: Some((_, return_value)) } = &return_block.terminator {
                        let mapped_return = variable_mapping.get(return_value)
                            .unwrap_or(return_value)
                            .clone();
                        
                        new_instructions.push(Instruction::BitCast {
                            result: result_var.clone(),
                            value: mapped_return,
                            from_ty: crate::ir::IRType::I32,
                            to_ty: crate::ir::IRType::I32,
                        });
                    }
                }
            }
        }
        
        new_instructions.extend_from_slice(&call_block.instructions[instruction_index + 1..]);
        call_block.instructions = new_instructions;
        
        Ok(())
    }

    fn remap_instruction(
        &self,
        instruction: &Instruction,
        variable_mapping: &HashMap<String, String>,
        next_temp: &mut usize,
    ) -> Instruction {
        let remap_operand = |operand: &str| -> String {
            variable_mapping.get(operand).unwrap_or(&operand.to_string()).clone()
        };

        match instruction {
            Instruction::Add { result, ty, left, right } => {
                let new_result = format!("%inline.{}", *next_temp);
                *next_temp += 1;
                Instruction::Add {
                    result: new_result,
                    ty: ty.clone(),
                    left: remap_operand(left),
                    right: remap_operand(right),
                }
            }
            Instruction::Sub { result, ty, left, right } => {
                let new_result = format!("%inline.{}", *next_temp);
                *next_temp += 1;
                Instruction::Sub {
                    result: new_result,
                    ty: ty.clone(),
                    left: remap_operand(left),
                    right: remap_operand(right),
                }
            }
            Instruction::Mul { result, ty, left, right } => {
                let new_result = format!("%inline.{}", *next_temp);
                *next_temp += 1;
                Instruction::Mul {
                    result: new_result,
                    ty: ty.clone(),
                    left: remap_operand(left),
                    right: remap_operand(right),
                }
            }
            Instruction::Load { result, ty, ptr } => {
                let new_result = format!("%inline.{}", *next_temp);
                *next_temp += 1;
                Instruction::Load {
                    result: new_result,
                    ty: ty.clone(),
                    ptr: remap_operand(ptr),
                }
            }
            Instruction::Store { ty, value, ptr } => {
                Instruction::Store {
                    ty: ty.clone(),
                    value: remap_operand(value),
                    ptr: remap_operand(ptr),
                }
            }
            _ => instruction.clone(),
        }
    }

    fn find_next_temp_number(&self, function: &IRFunction) -> usize {
        let mut max_temp = 0;
        
        for block in &function.blocks {
            for instruction in &block.instructions {
                if let Some(result) = instruction.get_result() {
                    if result.starts_with('%') {
                        if let Ok(num) = result[1..].parse::<usize>() {
                            max_temp = max_temp.max(num);
                        }
                    }
                }
            }
        }
        
        max_temp + 1
    }
}

impl OptimizationPass for Inliner {
    fn optimize(&mut self, module: &Module) -> Result<Module> {
        let mut optimized_module = module.clone();
        
        let inlinable_functions: Vec<String> = optimized_module.functions.iter()
            .filter(|func| self.should_inline(func))
            .filter(|func| self.count_call_sites(&func.name, &optimized_module) <= 3)
            .map(|func| func.name.clone())
            .collect();

        for function_name in &inlinable_functions {
            let callee = optimized_module.functions.iter()
                .find(|f| f.name == *function_name)
                .unwrap()
                .clone();

            for caller in &mut optimized_module.functions {
                if caller.name == *function_name {
                    continue;
                }

                let mut block_index = 0;
                while block_index < caller.blocks.len() {
                    let mut instruction_index = 0;
                    while instruction_index < caller.blocks[block_index].instructions.len() {
                        let instruction = &caller.blocks[block_index].instructions[instruction_index];
                        
                        if let Instruction::Call { function: call_target, .. } = instruction {
                            if call_target == function_name {
                                self.inline_function_call(
                                    caller,
                                    &callee,
                                    instruction,
                                    block_index,
                                    instruction_index,
                                )?;
                                continue;
                            }
                        }
                        
                        instruction_index += 1;
                    }
                    block_index += 1;
                }
            }
        }

        optimized_module.functions.retain(|func| !inlinable_functions.contains(&func.name));

        Ok(optimized_module)
    }
}

