use super::{Module, OptimizationPass};
use crate::ir::{Constant, IRType, Instruction};
use anyhow::Result;

pub struct ConstantFolder;

impl ConstantFolder {
    pub fn new() -> Self {
        Self
    }

    fn fold_instruction(&self, instruction: &Instruction) -> Option<Instruction> {
        match instruction {
            Instruction::Add {
                result,
                ty,
                left,
                right,
            } => {
                if let (Ok(left_val), Ok(right_val)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    Some(Instruction::Add {
                        result: result.clone(),
                        ty: ty.clone(),
                        left: (left_val + right_val).to_string(),
                        right: "0".to_string(),
                    })
                } else {
                    None
                }
            }
            Instruction::Sub {
                result,
                ty,
                left,
                right,
            } => {
                if let (Ok(left_val), Ok(right_val)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    Some(Instruction::Add {
                        result: result.clone(),
                        ty: ty.clone(),
                        left: (left_val - right_val).to_string(),
                        right: "0".to_string(),
                    })
                } else {
                    None
                }
            }
            Instruction::Mul {
                result,
                ty,
                left,
                right,
            } => {
                if let (Ok(left_val), Ok(right_val)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    Some(Instruction::Add {
                        result: result.clone(),
                        ty: ty.clone(),
                        left: (left_val * right_val).to_string(),
                        right: "0".to_string(),
                    })
                } else {
                    None
                }
            }
            Instruction::Div {
                result,
                ty,
                left,
                right,
            } => {
                if let (Ok(left_val), Ok(right_val)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    if right_val != 0 {
                        Some(Instruction::Add {
                            result: result.clone(),
                            ty: ty.clone(),
                            left: (left_val / right_val).to_string(),
                            right: "0".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_constant(&self, operand: &str) -> bool {
        operand.parse::<i64>().is_ok() || operand.parse::<f64>().is_ok()
    }

    fn evaluate_binary_op(&self, op: &str, left: i64, right: i64) -> i64 {
        match op {
            "add" => left + right,
            "sub" => left - right,
            "mul" => left * right,
            "div" => {
                if right != 0 {
                    left / right
                } else {
                    0
                }
            }
            "rem" => {
                if right != 0 {
                    left % right
                } else {
                    0
                }
            }
            "and" => left & right,
            "or" => left | right,
            "xor" => left ^ right,
            "shl" => left << right,
            "shr" => left >> right,
            _ => 0,
        }
    }

    fn evaluate_comparison(&self, op: &str, left: i64, right: i64) -> bool {
        match op {
            "eq" => left == right,
            "ne" => left != right,
            "slt" => left < right,
            "sle" => left <= right,
            "sgt" => left > right,
            "sge" => left >= right,
            "ult" => (left as u64) < (right as u64),
            "ule" => (left as u64) <= (right as u64),
            "ugt" => (left as u64) > (right as u64),
            "uge" => (left as u64) >= (right as u64),
            _ => false,
        }
    }
}

impl OptimizationPass for ConstantFolder {
    fn optimize(&mut self, module: &Module) -> Result<Module> {
        let mut optimized_module = module.clone();

        for function in &mut optimized_module.functions {
            for block in &mut function.blocks {
                let mut new_instructions = Vec::new();

                for instruction in &block.instructions {
                    if let Some(folded) = self.fold_instruction(instruction) {
                        new_instructions.push(folded);
                    } else {
                        new_instructions.push(instruction.clone());
                    }
                }

                block.instructions = new_instructions;
            }
        }

        Ok(optimized_module)
    }
}
