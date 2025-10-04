use super::{OptimizationPass, Module};
use crate::ir::{Instruction, IRType};
use anyhow::Result;

pub struct PeepholeOptimizer;

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self
    }

    fn optimize_instruction_pair(&self, inst1: &Instruction, inst2: &Instruction) -> Option<Vec<Instruction>> {
        match (inst1, inst2) {
            (
                Instruction::Add { result: r1, ty, left, right },
                Instruction::Add { result: r2, ty: ty2, left: left2, right: right2 }
            ) if r1 == left2 && right == "0" => {
                Some(vec![Instruction::Add {
                    result: r2.clone(),
                    ty: ty2.clone(),
                    left: left.clone(),
                    right: right2.clone(),
                }])
            }
            (
                Instruction::Mul { result: r1, ty, left, right },
                Instruction::Mul { result: r2, ty: ty2, left: left2, right: right2 }
            ) if r1 == left2 && right == "1" => {
                Some(vec![Instruction::BitCast {
                    result: r2.clone(),
                    value: left.clone(),
                    from_ty: ty.clone(),
                    to_ty: ty2.clone(),
                }])
            }
            (
                Instruction::Load { result: r1, ty, ptr },
                Instruction::Store { ty: ty2, value, ptr: ptr2 }
            ) if r1 == value && ptr == ptr2 => {
                Some(vec![])
            }
            (
                Instruction::Store { ty, value, ptr },
                Instruction::Load { result, ty: ty2, ptr: ptr2 }
            ) if ptr == ptr2 => {
                Some(vec![
                    inst1.clone(),
                    Instruction::BitCast {
                        result: result.clone(),
                        value: value.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty2.clone(),
                    }
                ])
            }
            _ => None,
        }
    }

    fn optimize_single_instruction(&self, instruction: &Instruction) -> Option<Instruction> {
        match instruction {
            Instruction::Add { result, ty, left, right } => {
                if right == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: left.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else if left == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: right.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::Sub { result, ty, left, right } => {
                if right == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: left.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::Mul { result, ty, left, right } => {
                if right == "1" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: left.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else if left == "1" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: right.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else if right == "0" || left == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: "0".to_string(),
                        from_ty: IRType::I32,
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::Div { result, ty, left, right } => {
                if right == "1" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: left.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::Or { result, ty, left, right } => {
                if right == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: left.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else if left == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: right.clone(),
                        from_ty: ty.clone(),
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::And { result, ty, left, right } => {
                if right == "0" || left == "0" {
                    Some(Instruction::BitCast {
                        result: result.clone(),
                        value: "0".to_string(),
                        from_ty: IRType::I32,
                        to_ty: ty.clone(),
                    })
                } else {
                    None
                }
            }
            Instruction::BitCast { result, value, from_ty, to_ty } => {
                if from_ty == to_ty {
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn strength_reduction(&self, instruction: &Instruction) -> Option<Instruction> {
        match instruction {
            Instruction::Mul { result, ty, left, right } => {
                if let Ok(val) = right.parse::<i64>() {
                    if val.is_power_of_two() {
                        let shift_amount = val.trailing_zeros().to_string();
                        return Some(Instruction::Shl {
                            result: result.clone(),
                            ty: ty.clone(),
                            left: left.clone(),
                            right: shift_amount,
                        });
                    }
                }
                if let Ok(val) = left.parse::<i64>() {
                    if val.is_power_of_two() {
                        let shift_amount = val.trailing_zeros().to_string();
                        return Some(Instruction::Shl {
                            result: result.clone(),
                            ty: ty.clone(),
                            left: right.clone(),
                            right: shift_amount,
                        });
                    }
                }
                None
            }
            Instruction::Div { result, ty, left, right } => {
                if let Ok(val) = right.parse::<i64>() {
                    if val.is_power_of_two() {
                        let shift_amount = val.trailing_zeros().to_string();
                        return Some(Instruction::Shr {
                            result: result.clone(),
                            ty: ty.clone(),
                            left: left.clone(),
                            right: shift_amount,
                        });
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl OptimizationPass for PeepholeOptimizer {
    fn optimize(&mut self, module: &Module) -> Result<Module> {
        let mut optimized_module = module.clone();

        for function in &mut optimized_module.functions {
            for block in &mut function.blocks {
                let mut new_instructions = Vec::new();
                let mut i = 0;

                while i < block.instructions.len() {
                    let current = &block.instructions[i];
                    
                    if let Some(optimized) = self.optimize_single_instruction(current) {
                        new_instructions.push(optimized);
                        i += 1;
                        continue;
                    }
                    
                    if let Some(strength_reduced) = self.strength_reduction(current) {
                        new_instructions.push(strength_reduced);
                        i += 1;
                        continue;
                    }
                    
                    if i + 1 < block.instructions.len() {
                        let next = &block.instructions[i + 1];
                        if let Some(mut optimized_pair) = self.optimize_instruction_pair(current, next) {
                            new_instructions.append(&mut optimized_pair);
                            i += 2;
                            continue;
                        }
                    }
                    
                    new_instructions.push(current.clone());
                    i += 1;
                }

                block.instructions = new_instructions;
            }
        }

        Ok(optimized_module)
    }
}

