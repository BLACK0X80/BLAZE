use super::{Module, IRFunction, BasicBlock, Instruction, Terminator, IRType};
use std::collections::{HashMap, HashSet};
use anyhow::Result;

pub struct IRValidator {
    errors: Vec<String>,
}

impl IRValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn validate(&mut self, module: &Module) -> Result<()> {
        self.errors.clear();

        for function in &module.functions {
            self.validate_function(function);
        }

        for global in &module.globals {
            self.validate_global_variable(global);
        }

        if !self.errors.is_empty() {
            return Err(anyhow::anyhow!("IR validation failed:\n{}", self.errors.join("\n")));
        }

        Ok(())
    }

    fn validate_function(&mut self, function: &IRFunction) {
        if function.blocks.is_empty() {
            self.errors.push(format!("Function '{}' has no basic blocks", function.name));
            return;
        }

        let mut defined_values = HashSet::new();
        let mut block_labels = HashSet::new();

        for param in &function.params {
            defined_values.insert(param.name.clone());
        }

        for block in &function.blocks {
            if !block_labels.insert(block.label.clone()) {
                self.errors.push(format!(
                    "Duplicate block label '{}' in function '{}'", 
                    block.label, 
                    function.name
                ));
            }
        }

        for block in &function.blocks {
            self.validate_basic_block(block, function, &mut defined_values, &block_labels);
        }

        self.validate_control_flow(function);
    }

    fn validate_basic_block(
        &mut self,
        block: &BasicBlock,
        function: &IRFunction,
        defined_values: &mut HashSet<String>,
        block_labels: &HashSet<String>,
    ) {
        for instruction in &block.instructions {
            self.validate_instruction(instruction, function, defined_values);
            
            if let Some(result) = instruction.get_result() {
                if !defined_values.insert(result.to_string()) {
                    self.errors.push(format!(
                        "Value '{}' redefined in block '{}' of function '{}'",
                        result,
                        block.label,
                        function.name
                    ));
                }
            }
        }

        self.validate_terminator(&block.terminator, function, defined_values, block_labels);
    }

    fn validate_instruction(
        &mut self,
        instruction: &Instruction,
        function: &IRFunction,
        defined_values: &HashSet<String>,
    ) {
        for operand in instruction.get_operands() {
            if !self.is_constant(operand) && !defined_values.contains(operand) {
                self.errors.push(format!(
                    "Use of undefined value '{}' in function '{}'",
                    operand,
                    function.name
                ));
            }
        }

        match instruction {
            Instruction::Load { ty, ptr, .. } => {
                if !self.is_pointer_type(ty) {
                    self.errors.push(format!(
                        "Load instruction expects pointer type, got {:?} in function '{}'",
                        ty,
                        function.name
                    ));
                }
            }
            Instruction::Store { ty, value: _, ptr } => {
                if !defined_values.contains(ptr) && !self.is_constant(ptr) {
                    self.errors.push(format!(
                        "Store to undefined pointer '{}' in function '{}'",
                        ptr,
                        function.name
                    ));
                }
            }
            Instruction::Call { function: callee, args, .. } => {
                if callee.is_empty() {
                    self.errors.push(format!(
                        "Call instruction with empty function name in function '{}'",
                        function.name
                    ));
                }
                
                for (ty, arg) in args {
                    if !defined_values.contains(arg) && !self.is_constant(arg) {
                        self.errors.push(format!(
                            "Call with undefined argument '{}' in function '{}'",
                            arg,
                            function.name
                        ));
                    }
                }
            }
            Instruction::Phi { values, .. } => {
                for (value, block_label) in values {
                    if !defined_values.contains(value) && !self.is_constant(value) {
                        self.errors.push(format!(
                            "Phi node references undefined value '{}' from block '{}' in function '{}'",
                            value,
                            block_label,
                            function.name
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn validate_terminator(
        &mut self,
        terminator: &Terminator,
        function: &IRFunction,
        defined_values: &HashSet<String>,
        block_labels: &HashSet<String>,
    ) {
        match terminator {
            Terminator::Return { value } => {
                if let Some((ty, val)) = value {
                    if !defined_values.contains(val) && !self.is_constant(val) {
                        self.errors.push(format!(
                            "Return of undefined value '{}' in function '{}'",
                            val,
                            function.name
                        ));
                    }
                    
                    if !self.types_compatible(ty, &function.return_type) {
                        self.errors.push(format!(
                            "Return type mismatch in function '{}': expected {:?}, got {:?}",
                            function.name,
                            function.return_type,
                            ty
                        ));
                    }
                } else if !matches!(function.return_type, IRType::Void) {
                    self.errors.push(format!(
                        "Function '{}' expects return value but returns void",
                        function.name
                    ));
                }
            }
            Terminator::Branch { dest } => {
                if !block_labels.contains(dest) {
                    self.errors.push(format!(
                        "Branch to undefined block '{}' in function '{}'",
                        dest,
                        function.name
                    ));
                }
            }
            Terminator::ConditionalBranch { condition, true_dest, false_dest } => {
                if !defined_values.contains(condition) && !self.is_constant(condition) {
                    self.errors.push(format!(
                        "Conditional branch with undefined condition '{}' in function '{}'",
                        condition,
                        function.name
                    ));
                }
                
                if !block_labels.contains(true_dest) {
                    self.errors.push(format!(
                        "Conditional branch to undefined block '{}' in function '{}'",
                        true_dest,
                        function.name
                    ));
                }
                
                if !block_labels.contains(false_dest) {
                    self.errors.push(format!(
                        "Conditional branch to undefined block '{}' in function '{}'",
                        false_dest,
                        function.name
                    ));
                }
            }
            Terminator::Switch { value, default_dest, cases } => {
                if !defined_values.contains(value) && !self.is_constant(value) {
                    self.errors.push(format!(
                        "Switch with undefined value '{}' in function '{}'",
                        value,
                        function.name
                    ));
                }
                
                if !block_labels.contains(default_dest) {
                    self.errors.push(format!(
                        "Switch to undefined default block '{}' in function '{}'",
                        default_dest,
                        function.name
                    ));
                }
                
                for (_, case_dest) in cases {
                    if !block_labels.contains(case_dest) {
                        self.errors.push(format!(
                            "Switch to undefined case block '{}' in function '{}'",
                            case_dest,
                            function.name
                        ));
                    }
                }
            }
            Terminator::Unreachable => {}
        }
    }

    fn validate_control_flow(&mut self, function: &IRFunction) {
        let mut reachable = HashSet::new();
        let mut worklist = vec![];

        if let Some(entry_block) = function.blocks.first() {
            reachable.insert(entry_block.label.clone());
            worklist.push(entry_block.label.clone());
        }

        while let Some(block_label) = worklist.pop() {
            if let Some(block) = function.blocks.iter().find(|b| b.label == block_label) {
                for successor in block.terminator.get_successors() {
                    if reachable.insert(successor.to_string()) {
                        worklist.push(successor.to_string());
                    }
                }
            }
        }

        for block in &function.blocks {
            if !reachable.contains(&block.label) {
                self.errors.push(format!(
                    "Unreachable block '{}' in function '{}'",
                    block.label,
                    function.name
                ));
            }
        }
    }

    fn validate_global_variable(&mut self, global: &super::GlobalVariable) {
        if global.name.is_empty() {
            self.errors.push("Global variable with empty name".to_string());
        }

        if let Some(initializer) = &global.initializer {
            if !self.constant_type_matches(initializer, &global.ty) {
                self.errors.push(format!(
                    "Global variable '{}' initializer type mismatch",
                    global.name
                ));
            }
        }
    }

    fn is_constant(&self, value: &str) -> bool {
        value.parse::<i64>().is_ok() || 
        value.parse::<f64>().is_ok() ||
        value == "true" || 
        value == "false" ||
        value.starts_with('"')
    }

    fn is_pointer_type(&self, ty: &IRType) -> bool {
        matches!(ty, IRType::Pointer(_))
    }

    fn types_compatible(&self, ty1: &IRType, ty2: &IRType) -> bool {
        match (ty1, ty2) {
            (IRType::I32, IRType::I32) => true,
            (IRType::I64, IRType::I64) => true,
            (IRType::F32, IRType::F32) => true,
            (IRType::F64, IRType::F64) => true,
            (IRType::I1, IRType::I1) => true,
            (IRType::Void, IRType::Void) => true,
            (IRType::Pointer(inner1), IRType::Pointer(inner2)) => {
                self.types_compatible(inner1, inner2)
            }
            _ => false,
        }
    }

    fn constant_type_matches(&self, constant: &super::Constant, ty: &IRType) -> bool {
        match (constant, ty) {
            (super::Constant::Integer { ty: const_ty, .. }, ty) => {
                self.types_compatible(const_ty, ty)
            }
            (super::Constant::Float { ty: const_ty, .. }, ty) => {
                self.types_compatible(const_ty, ty)
            }
            (super::Constant::Boolean(_), IRType::I1) => true,
            (super::Constant::String(_), IRType::Pointer(inner)) => {
                matches!(**inner, IRType::I8)
            }
            _ => false,
        }
    }

    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
}

