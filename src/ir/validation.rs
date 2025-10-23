use super::*;
use anyhow::{Result, bail};
use std::collections::HashSet;

pub fn validate_module(module: &Module) -> Result<()> {
    for function in &module.functions {
        validate_function(function)?;
    }
    Ok(())
}

fn validate_function(function: &IRFunction) -> Result<()> {
    let mut defined_values = HashSet::new();
    
    for param in &function.params {
        defined_values.insert(param.name.clone());
    }
    
    for block in &function.blocks {
        validate_block(block, &mut defined_values)?;
    }
    
    Ok(())
}

fn validate_block(block: &BasicBlock, defined_values: &mut HashSet<String>) -> Result<()> {
    for instruction in &block.instructions {
        for operand in instruction.get_operands() {
            if !operand.starts_with('%') && !operand.chars().all(|c| c.is_numeric() || c == '-' || c == '.') {
                if !defined_values.contains(operand) {
                    bail!("Undefined variable '{}' in block '{}'", operand, block.label);
                }
            }
        }
        
        if let Some(result) = instruction.get_result() {
            defined_values.insert(result.to_string());
        }
    }
    
    match &block.terminator {
        Terminator::Ret { value: Some(val) } => {
            if !val.starts_with('%') && !val.chars().all(|c| c.is_numeric() || c == '-' || c == '.') {
                if !defined_values.contains(val) {
                    bail!("Undefined return value '{}' in block '{}'", val, block.label);
                }
            }
        }
        Terminator::CondBr { condition, .. } => {
            if !condition.starts_with('%') && !defined_values.contains(condition) {
                bail!("Undefined condition '{}' in block '{}'", condition, block.label);
            }
        }
        _ => {}
    }
    
    Ok(())
}
