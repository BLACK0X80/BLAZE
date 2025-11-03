use crate::ir::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub struct Optimizer {
    options: OptimizationOptions,
}

#[derive(Debug, Clone)]
pub struct OptimizationOptions {
    pub level: u8,
    pub inline_threshold: usize,
    pub unroll_loops: bool,
    pub dead_code_elimination: bool,
    pub constant_folding: bool,
    pub constant_propagation: bool,
    pub common_subexpression_elimination: bool,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            level: 0,
            inline_threshold: 50,
            unroll_loops: false,
            dead_code_elimination: true,
            constant_folding: true,
            constant_propagation: true,
            common_subexpression_elimination: false,
        }
    }
}

impl OptimizationOptions {
    pub fn level_0() -> Self {
        Self::default()
    }

    pub fn level_1() -> Self {
        Self {
            level: 1,
            constant_folding: true,
            dead_code_elimination: true,
            ..Self::default()
        }
    }

    pub fn level_2() -> Self {
        Self {
            level: 2,
            inline_threshold: 75,
            constant_folding: true,
            constant_propagation: true,
            dead_code_elimination: true,
            common_subexpression_elimination: true,
            ..Self::default()
        }
    }

    pub fn level_3() -> Self {
        Self {
            level: 3,
            inline_threshold: 100,
            unroll_loops: true,
            constant_folding: true,
            constant_propagation: true,
            dead_code_elimination: true,
            common_subexpression_elimination: true,
        }
    }
}

impl Optimizer {
    pub fn new(options: OptimizationOptions) -> Self {
        Self { options }
    }

    pub fn with_level(level: u8) -> Self {
        let options = match level {
            0 => OptimizationOptions::level_0(),
            1 => OptimizationOptions::level_1(),
            2 => OptimizationOptions::level_2(),
            _ => OptimizationOptions::level_3(),
        };
        Self::new(options)
    }

    pub fn optimize(&mut self, mut module: Module) -> Result<Module> {
        match self.options.level {
            0 => {
                self.validate(&module)?;
            }
            1 => {
                if self.options.constant_folding {
                    module = self.constant_folding(module)?;
                }
                if self.options.dead_code_elimination {
                    module = self.dead_code_elimination(module)?;
                }
            }
            2 => {
                module = self.constant_folding(module)?;
                module = self.constant_propagation(module)?;
                module = self.dead_code_elimination(module)?;
                module = self.inline_functions(module)?;
                if self.options.common_subexpression_elimination {
                    module = self.common_subexpression_elimination(module)?;
                }
            }
            _ => {
                module = self.run_optimization_pipeline(module)?;
            }
        }

        Ok(module)
    }

    fn run_optimization_pipeline(&mut self, mut module: Module) -> Result<Module> {
        const MAX_ITERATIONS: usize = 10;
        let mut changed = true;
        let mut iterations = 0;

        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            let old_size = self.estimate_code_size(&module);

            module = self.constant_folding(module)?;
            module = self.constant_propagation(module)?;
            module = self.dead_code_elimination(module)?;
            module = self.inline_functions(module)?;
            module = self.common_subexpression_elimination(module)?;

            let new_size = self.estimate_code_size(&module);
            changed = new_size < old_size;
            iterations += 1;
        }

        Ok(module)
    }

    fn constant_folding(&self, mut module: Module) -> Result<Module> {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                let mut i = 0;
                while i < block.instructions.len() {
                    if let Some(folded) = self.try_fold_constant(&block.instructions[i]) {
                        block.instructions[i] = folded;
                    }
                    i += 1;
                }
            }
        }
        Ok(module)
    }

    fn try_fold_constant(&self, instr: &Instruction) -> Option<Instruction> {
        match instr {
            Instruction::Binary { op, left, right, result } => {
                if let (Value::Const(l), Value::Const(r)) = (left, right) {
                    let folded_value = self.evaluate_binary_op(*op, l, r)?;
                    return Some(Instruction::Store {
                        dest: result.clone(),
                        value: Value::Const(folded_value),
                    });
                }
            }
            Instruction::Unary { op, operand, result } => {
                if let Value::Const(val) = operand {
                    let folded_value = self.evaluate_unary_op(*op, val)?;
                    return Some(Instruction::Store {
                        dest: result.clone(),
                        value: Value::Const(folded_value),
                    });
                }
            }
            _ => {}
        }
        None
    }

    fn evaluate_binary_op(&self, op: BinaryOp, left: &Constant, right: &Constant) -> Option<Constant> {
        match (left, right) {
            (Constant::Int(l), Constant::Int(r)) => {
                let result = match op {
                    BinaryOp::Add => l.checked_add(*r)?,
                    BinaryOp::Sub => l.checked_sub(*r)?,
                    BinaryOp::Mul => l.checked_mul(*r)?,
                    BinaryOp::Div => l.checked_div(*r)?,
                    BinaryOp::Mod => l.checked_rem(*r)?,
                    BinaryOp::Eq => return Some(Constant::Bool(l == r)),
                    BinaryOp::Ne => return Some(Constant::Bool(l != r)),
                    BinaryOp::Lt => return Some(Constant::Bool(l < r)),
                    BinaryOp::Le => return Some(Constant::Bool(l <= r)),
                    BinaryOp::Gt => return Some(Constant::Bool(l > r)),
                    BinaryOp::Ge => return Some(Constant::Bool(l >= r)),
                    _ => return None,
                };
                Some(Constant::Int(result))
            }
            (Constant::Float(l), Constant::Float(r)) => {
                let result = match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => l / r,
                    BinaryOp::Eq => return Some(Constant::Bool(l == r)),
                    BinaryOp::Ne => return Some(Constant::Bool(l != r)),
                    BinaryOp::Lt => return Some(Constant::Bool(l < r)),
                    BinaryOp::Le => return Some(Constant::Bool(l <= r)),
                    BinaryOp::Gt => return Some(Constant::Bool(l > r)),
                    BinaryOp::Ge => return Some(Constant::Bool(l >= r)),
                    _ => return None,
                };
                Some(Constant::Float(result))
            }
            (Constant::Bool(l), Constant::Bool(r)) => {
                let result = match op {
                    BinaryOp::And => *l && *r,
                    BinaryOp::Or => *l || *r,
                    BinaryOp::Eq => l == r,
                    BinaryOp::Ne => l != r,
                    _ => return None,
                };
                Some(Constant::Bool(result))
            }
            _ => None,
        }
    }

    fn evaluate_unary_op(&self, op: UnaryOp, operand: &Constant) -> Option<Constant> {
        match (op, operand) {
            (UnaryOp::Neg, Constant::Int(val)) => Some(Constant::Int(-val)),
            (UnaryOp::Neg, Constant::Float(val)) => Some(Constant::Float(-val)),
            (UnaryOp::Not, Constant::Bool(val)) => Some(Constant::Bool(!val)),
            _ => None,
        }
    }

    fn constant_propagation(&self, mut module: Module) -> Result<Module> {
        for function in &mut module.functions {
            let mut constants: HashMap<String, Constant> = HashMap::new();

            for block in &mut function.blocks {
                for instr in &mut block.instructions {
                    match instr {
                        Instruction::Store { dest, value: Value::Const(c) } => {
                            constants.insert(dest.clone(), c.clone());
                        }
                        Instruction::Binary { left, right, .. } => {
                            if let Value::Variable(var) = left {
                                if let Some(c) = constants.get(var) {
                                    *left = Value::Const(c.clone());
                                }
                            }
                            if let Value::Variable(var) = right {
                                if let Some(c) = constants.get(var) {
                                    *right = Value::Const(c.clone());
                                }
                            }
                        }
                        Instruction::Store { dest, .. } => {
                            constants.remove(dest);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(module)
    }

    fn dead_code_elimination(&self, mut module: Module) -> Result<Module> {
        for function in &mut module.functions {
            let mut used_vars: HashSet<String> = HashSet::new();

            for block in &function.blocks {
                for instr in &block.instructions {
                    self.collect_used_vars(instr, &mut used_vars);
                }
            }

            for block in &mut function.blocks {
                block.instructions.retain(|instr| {
                    !self.is_dead_instruction(instr, &used_vars)
                });
            }

            function.blocks.retain(|block| !self.is_unreachable_block(block));
        }
        Ok(module)
    }

    fn collect_used_vars(&self, instr: &Instruction, used: &mut HashSet<String>) {
        match instr {
            Instruction::Binary { left, right, .. } => {
                if let Value::Variable(var) = left {
                    used.insert(var.clone());
                }
                if let Value::Variable(var) = right {
                    used.insert(var.clone());
                }
            }
            Instruction::Unary { operand, .. } => {
                if let Value::Variable(var) = operand {
                    used.insert(var.clone());
                }
            }
            Instruction::Call { args, .. } => {
                for arg in args {
                    if let Value::Variable(var) = arg {
                        used.insert(var.clone());
                    }
                }
            }
            Instruction::Return { value } => {
                if let Some(Value::Variable(var)) = value {
                    used.insert(var.clone());
                }
            }
            _ => {}
        }
    }

    fn is_dead_instruction(&self, instr: &Instruction, used_vars: &HashSet<String>) -> bool {
        match instr {
            Instruction::Store { dest, .. } => !used_vars.contains(dest),
            Instruction::Binary { result, .. } => !used_vars.contains(result),
            Instruction::Unary { result, .. } => !used_vars.contains(result),
            _ => false,
        }
    }

    fn is_unreachable_block(&self, block: &BasicBlock) -> bool {
        block.predecessors.is_empty() && !block.is_entry
    }

    fn inline_functions(&self, mut module: Module) -> Result<Module> {
        let mut inlined_any = true;

        while inlined_any {
            inlined_any = false;

            for i in 0..module.functions.len() {
                let function = &module.functions[i];

                let size = self.estimate_function_size(function);
                if size > self.options.inline_threshold {
                    continue;
                }

                let call_sites = self.find_call_sites(&module, &function.name);

                if !call_sites.is_empty() {
                    for (caller_idx, block_idx, instr_idx) in call_sites {
                        self.inline_call_site(
                            &mut module,
                            caller_idx,
                            block_idx,
                            instr_idx,
                            &function.clone(),
                        )?;
                        inlined_any = true;
                    }
                }
            }
        }

        Ok(module)
    }

    fn estimate_function_size(&self, function: &Function) -> usize {
        function.blocks.iter().map(|b| b.instructions.len()).sum()
    }

    fn estimate_code_size(&self, module: &Module) -> usize {
        module.functions.iter().map(|f| self.estimate_function_size(f)).sum()
    }

    fn find_call_sites(&self, module: &Module, function_name: &str) -> Vec<(usize, usize, usize)> {
        let mut sites = Vec::new();

        for (func_idx, function) in module.functions.iter().enumerate() {
            for (block_idx, block) in function.blocks.iter().enumerate() {
                for (instr_idx, instr) in block.instructions.iter().enumerate() {
                    if let Instruction::Call { function: callee, .. } = instr {
                        if callee == function_name {
                            sites.push((func_idx, block_idx, instr_idx));
                        }
                    }
                }
            }
        }

        sites
    }

    fn inline_call_site(
        &self,
        module: &mut Module,
        caller_idx: usize,
        block_idx: usize,
        instr_idx: usize,
        callee: &Function,
    ) -> Result<()> {
        let mut inlined_instrs = Vec::new();

        for block in &callee.blocks {
            for instr in &block.instructions {
                let renamed = self.rename_instruction_vars(instr, &format!("_inlined_{}_", callee.name));
                inlined_instrs.push(renamed);
            }
        }

        let caller = &mut module.functions[caller_idx];
        let block = &mut caller.blocks[block_idx];

        block.instructions.splice(instr_idx..=instr_idx, inlined_instrs);

        Ok(())
    }

    fn rename_instruction_vars(&self, instr: &Instruction, prefix: &str) -> Instruction {
        match instr {
            Instruction::Binary { op, left, right, result } => {
                Instruction::Binary {
                    op: *op,
                    left: self.rename_value(left, prefix),
                    right: self.rename_value(right, prefix),
                    result: format!("{}{}", prefix, result),
                }
            }
            Instruction::Store { dest, value } => {
                Instruction::Store {
                    dest: format!("{}{}", prefix, dest),
                    value: self.rename_value(value, prefix),
                }
            }
            _ => instr.clone(),
        }
    }

    fn rename_value(&self, value: &Value, prefix: &str) -> Value {
        match value {
            Value::Variable(var) => Value::Variable(format!("{}{}", prefix, var)),
            _ => value.clone(),
        }
    }

    fn common_subexpression_elimination(&self, mut module: Module) -> Result<Module> {
        for function in &mut module.functions {
            let mut expr_map: HashMap<String, String> = HashMap::new();

            for block in &mut function.blocks {
                for instr in &mut block.instructions {
                    if let Instruction::Binary { op, left, right, result } = instr {
                        let expr_key = format!("{:?}_{:?}_{:?}", op, left, right);

                        if let Some(existing_result) = expr_map.get(&expr_key) {
                            *instr = Instruction::Store {
                                dest: result.clone(),
                                value: Value::Variable(existing_result.clone()),
                            };
                        } else {
                            expr_map.insert(expr_key, result.clone());
                        }
                    }
                }
            }
        }
        Ok(module)
    }

    fn validate(&self, module: &Module) -> Result<()> {
        for function in &module.functions {
            if function.blocks.is_empty() {
                return Err(anyhow::anyhow!(
                    "Function '{}' has no basic blocks",
                    function.name
                ));
            }

            for block in &function.blocks {
                let mut found_return = false;
                for instr in &block.instructions {
                    if found_return {
                        return Err(anyhow::anyhow!(
                            "Unreachable code after return in function '{}'",
                            function.name
                        ));
                    }
                    if matches!(instr, Instruction::Return { .. }) {
                        found_return = true;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_options_levels() {
        let opt0 = OptimizationOptions::level_0();
        assert_eq!(opt0.level, 0);

        let opt1 = OptimizationOptions::level_1();
        assert_eq!(opt1.level, 1);
        assert!(opt1.constant_folding);

        let opt2 = OptimizationOptions::level_2();
        assert_eq!(opt2.level, 2);
        assert!(opt2.constant_propagation);

        let opt3 = OptimizationOptions::level_3();
        assert_eq!(opt3.level, 3);
        assert!(opt3.unroll_loops);
    }

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::with_level(2);
        assert_eq!(optimizer.options.level, 2);
    }
}
