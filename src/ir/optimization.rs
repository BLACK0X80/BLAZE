use super::*;
use std::collections::HashMap;

pub fn optimize_module(module: &mut Module) {
    for function in &mut module.functions {
        optimize_function(function);
    }
}

fn optimize_function(function: &mut IRFunction) {
    for block in &mut function.blocks {
        constant_fold_block(block);
        dead_code_elimination_block(block);
        common_subexpression_elimination_block(block);
    }
}

fn constant_fold_block(block: &mut BasicBlock) {
    let mut constants: HashMap<String, i64> = HashMap::new();
    let mut new_instructions = Vec::new();
    
    for instruction in &block.instructions {
        match instruction {
            Instruction::Add { result, left, right, ty } => {
                if let (Some(&left_val), Some(&right_val)) = (
                    left.parse::<i64>().ok().or_else(|| constants.get(left).copied()),
                    right.parse::<i64>().ok().or_else(|| constants.get(right).copied())
                ) {
                    let folded_value = left_val + right_val;
                    constants.insert(result.clone(), folded_value);
                    continue;
                }
                new_instructions.push(instruction.clone());
            }
            Instruction::Sub { result, left, right, ty } => {
                if let (Some(&left_val), Some(&right_val)) = (
                    left.parse::<i64>().ok().or_else(|| constants.get(left).copied()),
                    right.parse::<i64>().ok().or_else(|| constants.get(right).copied())
                ) {
                    let folded_value = left_val - right_val;
                    constants.insert(result.clone(), folded_value);
                    continue;
                }
                new_instructions.push(instruction.clone());
            }
            Instruction::Mul { result, left, right, ty } => {
                if let (Some(&left_val), Some(&right_val)) = (
                    left.parse::<i64>().ok().or_else(|| constants.get(left).copied()),
                    right.parse::<i64>().ok().or_else(|| constants.get(right).copied())
                ) {
                    let folded_value = left_val * right_val;
                    constants.insert(result.clone(), folded_value);
                    continue;
                }
                new_instructions.push(instruction.clone());
            }
            Instruction::Div { result, left, right, ty } => {
                if let (Some(&left_val), Some(&right_val)) = (
                    left.parse::<i64>().ok().or_else(|| constants.get(left).copied()),
                    right.parse::<i64>().ok().or_else(|| constants.get(right).copied())
                ) {
                    if right_val != 0 {
                        let folded_value = left_val / right_val;
                        constants.insert(result.clone(), folded_value);
                        continue;
                    }
                }
                new_instructions.push(instruction.clone());
            }
            _ => {
                new_instructions.push(instruction.clone());
            }
        }
    }
    
    block.instructions = new_instructions;
}

fn dead_code_elimination_block(block: &mut BasicBlock) {
    let mut used_values = std::collections::HashSet::new();
    
    for instruction in &block.instructions {
        for operand in instruction.get_operands() {
            used_values.insert(operand.to_string());
        }
    }
    
    match &block.terminator {
        Terminator::Ret { value: Some(val) } => {
            used_values.insert(val.clone());
        }
        Terminator::CondBr { condition, .. } => {
            used_values.insert(condition.clone());
        }
        _ => {}
    }
    
    block.instructions.retain(|instruction| {
        if let Some(result) = instruction.get_result() {
            used_values.contains(result)
        } else {
            true
        }
    });
}

fn common_subexpression_elimination_block(block: &mut BasicBlock) {
    let mut expression_map: HashMap<String, String> = HashMap::new();
    let mut new_instructions = Vec::new();
    
    for instruction in &block.instructions {
        let expr_key = match instruction {
            Instruction::Add { left, right, ty, .. } => {
                Some(format!("add_{}_{}", left, right))
            }
            Instruction::Sub { left, right, ty, .. } => {
                Some(format!("sub_{}_{}", left, right))
            }
            Instruction::Mul { left, right, ty, .. } => {
                Some(format!("mul_{}_{}", left, right))
            }
            _ => None,
        };
        
        if let Some(key) = expr_key {
            if let Some(existing_result) = expression_map.get(&key) {
                continue;
            } else if let Some(result) = instruction.get_result() {
                expression_map.insert(key, result.to_string());
            }
        }
        
        new_instructions.push(instruction.clone());
    }
    
    block.instructions = new_instructions;
}
