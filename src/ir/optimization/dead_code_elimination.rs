use super::{OptimizationPass, Module};
use crate::ir::{Instruction, BasicBlock};
use std::collections::{HashSet, VecDeque};
use anyhow::Result;

pub struct DeadCodeEliminator;

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self
    }

    fn find_reachable_blocks(&self, blocks: &[BasicBlock]) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut worklist = VecDeque::new();

        if let Some(entry_block) = blocks.first() {
            worklist.push_back(entry_block.label.clone());
            reachable.insert(entry_block.label.clone());
        }

        while let Some(block_label) = worklist.pop_front() {
            if let Some(block) = blocks.iter().find(|b| b.label == block_label) {
                for successor in block.terminator.get_successors() {
                    if reachable.insert(successor.to_string()) {
                        worklist.push_back(successor.to_string());
                    }
                }
            }
        }

        reachable
    }

    fn find_used_values(&self, blocks: &[BasicBlock]) -> HashSet<String> {
        let mut used = HashSet::new();

        for block in blocks {
            for instruction in &block.instructions {
                for operand in instruction.get_operands() {
                    used.insert(operand.to_string());
                }
            }

            for operand in block.terminator.get_operands() {
                used.insert(operand.to_string());
            }
        }

        used
    }

    fn is_side_effect_free(&self, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Store { .. } |
            Instruction::Call { .. } => false,
            _ => true,
        }
    }

    fn remove_dead_instructions(&self, blocks: &mut [BasicBlock]) {
        let used_values = self.find_used_values(blocks);

        for block in blocks {
            block.instructions.retain(|instruction| {
                if !self.is_side_effect_free(instruction) {
                    return true;
                }

                if let Some(result) = instruction.get_result() {
                    used_values.contains(result)
                } else {
                    true
                }
            });
        }
    }

    fn remove_unreachable_blocks(&self, blocks: &mut Vec<BasicBlock>) {
        let reachable = self.find_reachable_blocks(blocks);
        blocks.retain(|block| reachable.contains(&block.label));
    }

    fn simplify_control_flow(&self, blocks: &mut [BasicBlock]) {
        for block in blocks {
            match &block.terminator {
                crate::ir::Terminator::ConditionalBranch { condition, true_dest, false_dest } => {
                    if condition == "1" || condition == "true" {
                        block.terminator = crate::ir::Terminator::Branch {
                            dest: true_dest.clone(),
                        };
                    } else if condition == "0" || condition == "false" {
                        block.terminator = crate::ir::Terminator::Branch {
                            dest: false_dest.clone(),
                        };
                    }
                }
                _ => {}
            }
        }
    }
}

impl OptimizationPass for DeadCodeEliminator {
    fn optimize(&mut self, module: &Module) -> Result<Module> {
        let mut optimized_module = module.clone();

        for function in &mut optimized_module.functions {
            self.remove_unreachable_blocks(&mut function.blocks);
            
            let mut changed = true;
            while changed {
                let old_block_count = function.blocks.len();
                let old_instruction_count: usize = function.blocks.iter()
                    .map(|b| b.instructions.len())
                    .sum();

                self.remove_dead_instructions(&mut function.blocks);
                self.simplify_control_flow(&mut function.blocks);
                self.remove_unreachable_blocks(&mut function.blocks);

                let new_block_count = function.blocks.len();
                let new_instruction_count: usize = function.blocks.iter()
                    .map(|b| b.instructions.len())
                    .sum();

                changed = old_block_count != new_block_count || 
                         old_instruction_count != new_instruction_count;
            }
        }

        Ok(optimized_module)
    }
}

