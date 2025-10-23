use crate::ir::IRFunction;
use anyhow::{Result, bail};
use std::collections::HashMap;

const MAX_PHYSICAL_REGISTERS: usize = 16;

pub struct RegisterAllocator {
    register_assignments: HashMap<String, usize>,
    spilled_variables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LiveInterval {
    pub variable: String,
    pub start: usize,
    pub end: usize,
    pub uses: Vec<usize>,
    pub register: Option<usize>,
    pub spilled: bool,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            register_assignments: HashMap::new(),
            spilled_variables: Vec::new(),
        }
    }

    pub fn allocate(&mut self, function: &IRFunction) -> Result<()> {
        let mut intervals = self.compute_live_intervals(function)?;
        self.linear_scan_allocation(&mut intervals)?;
        
        for interval in &intervals {
            if let Some(reg) = interval.register {
                self.register_assignments.insert(interval.variable.clone(), reg);
            } else if interval.spilled {
                self.spilled_variables.push(interval.variable.clone());
            }
        }
        
        Ok(())
    }

    fn compute_live_intervals(&self, function: &IRFunction) -> Result<Vec<LiveInterval>> {
        let mut intervals = Vec::new();
        let mut variable_positions: HashMap<String, usize> = HashMap::new();
        let mut position: usize = 0;

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
                                register: None,
                                spilled: false,
                            });
                        }
                    }
                }

                position = position.checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("Position overflow in register allocation"))?;
            }
        }

        intervals.sort_by_key(|interval| interval.start);
        Ok(intervals)
    }

    fn linear_scan_allocation(&mut self, intervals: &mut [LiveInterval]) -> Result<()> {
        let mut active: Vec<usize> = Vec::new();
        let mut free_registers: Vec<usize> = (0..MAX_PHYSICAL_REGISTERS).collect();

        for i in 0..intervals.len() {
            self.expire_old_intervals(i, intervals, &mut active, &mut free_registers);

            if free_registers.is_empty() {
                self.spill_at_interval(i, intervals, &mut active)?;
            } else {
                let reg = free_registers.pop().unwrap();
                intervals[i].register = Some(reg);
                active.push(i);
                active.sort_by_key(|&idx| intervals[idx].end);
            }
        }

        Ok(())
    }

    fn expire_old_intervals(
        &self,
        current: usize,
        intervals: &mut [LiveInterval],
        active: &mut Vec<usize>,
        free_registers: &mut Vec<usize>,
    ) {
        let current_start = intervals[current].start;
        
        active.retain(|&idx| {
            if intervals[idx].end <= current_start {
                if let Some(reg) = intervals[idx].register {
                    free_registers.push(reg);
                    free_registers.sort();
                }
                false
            } else {
                true
            }
        });
    }

    fn spill_at_interval(
        &mut self,
        current: usize,
        intervals: &mut [LiveInterval],
        active: &mut Vec<usize>,
    ) -> Result<()> {
        if let Some(&last_active) = active.last() {
            if intervals[last_active].end > intervals[current].end {
                intervals[current].register = intervals[last_active].register;
                intervals[last_active].register = None;
                intervals[last_active].spilled = true;
                
                active.pop();
                active.push(current);
                active.sort_by_key(|&idx| intervals[idx].end);
            } else {
                intervals[current].spilled = true;
            }
        } else {
            intervals[current].spilled = true;
        }
        
        Ok(())
    }

    pub fn get_register(&self, variable: &str) -> Option<usize> {
        self.register_assignments.get(variable).copied()
    }

    pub fn is_spilled(&self, variable: &str) -> bool {
        self.spilled_variables.contains(&variable.to_string())
    }

    pub fn get_spilled_variables(&self) -> &[String] {
        &self.spilled_variables
    }
}
