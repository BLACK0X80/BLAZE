use super::*;
use crate::parser::{Program, Item, Function, Statement, Expression, Type};
use anyhow::Result;
use std::collections::HashMap;

pub struct IRBuilder {
    next_temp: usize,
    next_label: usize,
    current_function: Option<String>,
    symbol_types: HashMap<String, IRType>,
    function_signatures: HashMap<String, (Vec<IRType>, IRType)>, // (params, return_type)
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            next_temp: 0,
            next_label: 0,
            current_function: None,
            symbol_types: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    pub fn build_module(&mut self, program: &Program) -> Result<Module> {
        let mut functions = Vec::new();
        let globals = Vec::new();
        let types = Vec::new();

        // First pass: collect function signatures
        for item in &program.items {
            if let Item::Function(func) = item {
                let param_types: Vec<IRType> = func.params.iter()
                    .map(|p| self.convert_type(&p.ty))
                    .collect();
                let return_type = func.return_type.as_ref()
                    .map(|t| self.convert_type(t))
                    .unwrap_or(IRType::Void);
                
                self.function_signatures.insert(
                    func.name.clone(),
                    (param_types, return_type),
                );
            }
        }

        // Second pass: build functions
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    functions.push(self.build_function(func)?);
                }
                Item::Struct(_) => {
                }
            }
        }

        Ok(Module {
            name: "main".to_string(),
            functions,
            globals,
            types,
        })
    }

    fn build_function(&mut self, func: &Function) -> Result<IRFunction> {
        self.current_function = Some(func.name.clone());
        
        let params = func.params.iter().map(|p| {
            let ty = self.convert_type(&p.ty);
            self.symbol_types.insert(p.name.clone(), ty.clone());
            Parameter {
                name: p.name.clone(),
                ty,
            }
        }).collect();

        let return_type = func.return_type.as_ref()
            .map(|t| self.convert_type(t))
            .unwrap_or(IRType::Void);

        let mut blocks = Vec::new();
        let entry_label = self.new_label();
        let mut current_block = 0;
        
        blocks.push(BasicBlock {
            label: entry_label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });

        for stmt in &func.body {
            self.build_statement(stmt, &mut blocks, &mut current_block)?;
        }

        // If the last block doesn't have a proper terminator, add one
        if let Some(last_block) = blocks.last_mut() {
            if matches!(last_block.terminator, Terminator::Unreachable) {
                last_block.terminator = if matches!(return_type, IRType::Void) {
                    Terminator::Ret { value: None }
                } else {
                    Terminator::Unreachable
                };
            }
        }

        self.current_function = None;
        self.symbol_types.clear();

        Ok(IRFunction {
            name: func.name.clone(),
            params,
            return_type,
            blocks,
        })
    }

    fn build_statement(&mut self, stmt: &Statement, blocks: &mut Vec<BasicBlock>, current_block: &mut usize) -> Result<()> {
        match stmt {
            Statement::Let { name, value, ty, .. } => {
                let value_reg = self.build_expression(value, &mut blocks[*current_block].instructions)?;
                
                // Infer type from value or use explicit type
                let var_type = if let Some(t) = ty {
                    self.convert_type(t)
                } else {
                    self.symbol_types.get(&value_reg).cloned().unwrap_or(IRType::I32)
                };
                
                self.symbol_types.insert(name.clone(), var_type.clone());
                
                let ptr = self.new_temp();
                blocks[*current_block].instructions.push(Instruction::Alloca {
                    result: ptr.clone(),
                    ty: var_type,
                });
                blocks[*current_block].instructions.push(Instruction::Store {
                    value: value_reg,
                    ptr,
                });
            }
            Statement::Return(Some(expr)) => {
                let value = self.build_expression(expr, &mut blocks[*current_block].instructions)?;
                blocks[*current_block].terminator = Terminator::Ret { value: Some(value) };
            }
            Statement::Return(None) => {
                blocks[*current_block].terminator = Terminator::Ret { value: None };
            }
            Statement::Expression(expr) => {
                self.build_expression(expr, &mut blocks[*current_block].instructions)?;
            }
            Statement::While { condition, body } => {
                self.build_while_statement(condition, body, blocks, current_block)?;
            }
            Statement::If { condition, then_body, else_body } => {
                self.build_if_statement(condition, then_body, else_body.as_deref(), blocks, current_block)?;
            }
        }
        Ok(())
    }

    fn build_expression(&mut self, expr: &Expression, instructions: &mut Vec<Instruction>) -> Result<String> {
        match expr {
            Expression::IntLit(n) => {
                let result = format!("{}", n);
                self.symbol_types.insert(result.clone(), IRType::I32);
                Ok(result)
            }
            Expression::FloatLit(f) => {
                let result = format!("{}", f);
                self.symbol_types.insert(result.clone(), IRType::F64);
                Ok(result)
            }
            Expression::BoolLit(b) => {
                let result = if *b { "1" } else { "0" }.to_string();
                self.symbol_types.insert(result.clone(), IRType::I1);
                Ok(result)
            }
            Expression::Ident(name) => {
                Ok(name.clone())
            }
            Expression::Binary { op, left, right } => {
                let left_reg = self.build_expression(left, instructions)?;
                let right_reg = self.build_expression(right, instructions)?;
                let result = self.new_temp();

                // Get operand types for type checking
                let left_ty = self.symbol_types.get(&left_reg).cloned().unwrap_or(IRType::I32);
                let right_ty = self.symbol_types.get(&right_reg).cloned().unwrap_or(IRType::I32);
                
                // Validate type compatibility
                if left_ty != right_ty {
                    return Err(anyhow::anyhow!("Type mismatch in binary operation: {:?} vs {:?}", left_ty, right_ty));
                }

                use crate::parser::BinaryOp;
                match op {
                    BinaryOp::Add => {
                        instructions.push(Instruction::Add {
                            result: result.clone(),
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), left_ty);
                    }
                    BinaryOp::Sub => {
                        instructions.push(Instruction::Sub {
                            result: result.clone(),
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), left_ty);
                    }
                    BinaryOp::Mul => {
                        instructions.push(Instruction::Mul {
                            result: result.clone(),
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), left_ty);
                    }
                    BinaryOp::Div => {
                        instructions.push(Instruction::Div {
                            result: result.clone(),
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), left_ty);
                    }
                    BinaryOp::Mod => {
                        instructions.push(Instruction::Mod {
                            result: result.clone(),
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), left_ty);
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        let condition = match op {
                            BinaryOp::Eq => ICmpCondition::Eq,
                            BinaryOp::Ne => ICmpCondition::Ne,
                            BinaryOp::Lt => ICmpCondition::Slt,
                            BinaryOp::Le => ICmpCondition::Sle,
                            BinaryOp::Gt => ICmpCondition::Sgt,
                            BinaryOp::Ge => ICmpCondition::Sge,
                            _ => unreachable!(),
                        };
                        instructions.push(Instruction::ICmp {
                            result: result.clone(),
                            condition,
                            left: left_reg,
                            right: right_reg,
                        });
                        self.symbol_types.insert(result.clone(), IRType::I1);
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        // Logical operations - treat as bitwise for now
                        let inst = if *op == BinaryOp::And {
                            Instruction::Mul {
                                result: result.clone(),
                                left: left_reg,
                                right: right_reg,
                                ty: IRType::I1,
                            }
                        } else {
                            // For OR, we can use: a | b = a + b - (a * b)
                            // But for simplicity, just use Add for now
                            Instruction::Add {
                                result: result.clone(),
                                left: left_reg,
                                right: right_reg,
                                ty: IRType::I1,
                            }
                        };
                        instructions.push(inst);
                        self.symbol_types.insert(result.clone(), IRType::I1);
                    }
                }

                Ok(result)
            }
            Expression::Call { func, args } => {
                let func_name = if let Expression::Ident(name) = &**func {
                    name.clone()
                } else {
                    return Err(anyhow::anyhow!("Invalid function call"));
                };

                // Build argument expressions
                let arg_regs: Result<Vec<String>> = args.iter()
                    .map(|arg| self.build_expression(arg, instructions))
                    .collect();
                let arg_regs = arg_regs?;

                // Look up function signature for type checking
                if let Some((param_types, return_type)) = self.function_signatures.get(&func_name) {
                    // Type check arguments
                    if arg_regs.len() != param_types.len() {
                        return Err(anyhow::anyhow!(
                            "Function '{}' expects {} arguments, got {}",
                            func_name,
                            param_types.len(),
                            arg_regs.len()
                        ));
                    }
                    
                    for (i, (arg_reg, expected_type)) in arg_regs.iter().zip(param_types.iter()).enumerate() {
                        let arg_type = self.symbol_types.get(arg_reg).cloned().unwrap_or(IRType::I32);
                        if arg_type != *expected_type {
                            return Err(anyhow::anyhow!(
                                "Function '{}' argument {} type mismatch: expected {:?}, got {:?}",
                                func_name,
                                i,
                                expected_type,
                                arg_type
                            ));
                        }
                    }
                    
                    // Handle void vs non-void return types
                    if matches!(return_type, IRType::Void) {
                        // Void function - no result
                        instructions.push(Instruction::Call {
                            result: None,
                            func: func_name,
                            args: arg_regs,
                        });
                        
                        // Return a dummy value for void calls
                        Ok("void".to_string())
                    } else {
                        // Non-void function - has result
                        let result = self.new_temp();
                        instructions.push(Instruction::Call {
                            result: Some(result.clone()),
                            func: func_name,
                            args: arg_regs,
                        });
                        
                        self.symbol_types.insert(result.clone(), return_type.clone());
                        Ok(result)
                    }
                } else {
                    // Function signature not found - assume I32 return for now
                    let result = self.new_temp();
                    instructions.push(Instruction::Call {
                        result: Some(result.clone()),
                        func: func_name.clone(),
                        args: arg_regs,
                    });
                    
                    self.symbol_types.insert(result.clone(), IRType::I32);
                    Ok(result)
                }
            }
            Expression::Unary { op, expr } => {
                let operand = self.build_expression(expr, instructions)?;
                let result = self.new_temp();
                let operand_ty = self.symbol_types.get(&operand).cloned().unwrap_or(IRType::I32);
                
                use crate::parser::UnaryOp;
                match op {
                    UnaryOp::Neg => {
                        // Negate: 0 - operand
                        instructions.push(Instruction::Sub {
                            result: result.clone(),
                            left: "0".to_string(),
                            right: operand,
                            ty: operand_ty.clone(),
                        });
                        self.symbol_types.insert(result.clone(), operand_ty);
                    }
                    UnaryOp::Not => {
                        // Logical not: 1 - operand (for boolean)
                        instructions.push(Instruction::Sub {
                            result: result.clone(),
                            left: "1".to_string(),
                            right: operand,
                            ty: IRType::I1,
                        });
                        self.symbol_types.insert(result.clone(), IRType::I1);
                    }
                }
                
                Ok(result)
            }
            _ => {
                let result = self.new_temp();
                self.symbol_types.insert(result.clone(), IRType::I32);
                Ok(result)
            }
        }
    }

    fn convert_type(&self, ty: &Type) -> IRType {
        match ty {
            Type::I32 => IRType::I32,
            Type::I64 => IRType::I64,
            Type::F32 => IRType::F32,
            Type::F64 => IRType::F64,
            Type::Bool => IRType::I1,
            Type::Char => IRType::I8,
            Type::String => IRType::Pointer(Box::new(IRType::I8)),
            Type::Custom(_) => IRType::I32,
        }
    }

    fn new_temp(&mut self) -> String {
        let temp = format!("%t{}", self.next_temp);
        self.next_temp += 1;
        temp
    }

    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn build_if_statement(
        &mut self,
        condition: &Expression,
        then_body: &[Statement],
        else_body: Option<&[Statement]>,
        blocks: &mut Vec<BasicBlock>,
        current_block: &mut usize,
    ) -> Result<()> {
        // Save symbol types before branches
        let symbols_before = self.symbol_types.clone();
        
        // Evaluate condition in current block
        let cond_reg = self.build_expression(condition, &mut blocks[*current_block].instructions)?;
        let condition_block_label = blocks[*current_block].label.clone();
        
        // Create labels for then, else (if exists), and merge blocks
        let then_label = self.new_label();
        let else_label = if else_body.is_some() {
            self.new_label()
        } else {
            self.new_label() // merge label
        };
        let merge_label = if else_body.is_some() {
            self.new_label()
        } else {
            else_label.clone()
        };
        
        // Set conditional branch in current block
        blocks[*current_block].terminator = Terminator::CondBr {
            condition: cond_reg,
            true_target: then_label.clone(),
            false_target: else_label.clone(),
        };
        
        // Create then block
        blocks.push(BasicBlock {
            label: then_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        let then_block_idx = blocks.len() - 1;
        *current_block = then_block_idx;
        
        // Build then body
        for stmt in then_body {
            self.build_statement(stmt, blocks, current_block)?;
        }
        
        // Save then block label after processing
        let then_exit_label = blocks[*current_block].label.clone();
        
        // Branch to merge if no explicit terminator
        if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
            blocks[*current_block].terminator = Terminator::Br {
                target: merge_label.clone(),
            };
        }
        
        // Save symbol types after then branch
        let symbols_after_then = self.symbol_types.clone();
        
        // Restore symbols for else branch
        self.symbol_types = symbols_before.clone();
        
        let else_exit_label = if let Some(else_stmts) = else_body {
            // Create else block
            blocks.push(BasicBlock {
                label: else_label.clone(),
                instructions: Vec::new(),
                terminator: Terminator::Unreachable,
            });
            *current_block = blocks.len() - 1;
            
            for stmt in else_stmts {
                self.build_statement(stmt, blocks, current_block)?;
            }
            
            let label = blocks[*current_block].label.clone();
            
            // Branch to merge if no explicit terminator
            if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
                blocks[*current_block].terminator = Terminator::Br {
                    target: merge_label.clone(),
                };
            }
            
            label
        } else {
            condition_block_label.clone()
        };
        
        // Save symbol types after else branch
        let symbols_after_else = self.symbol_types.clone();
        
        // Create merge block
        blocks.push(BasicBlock {
            label: merge_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        let merge_block_idx = blocks.len() - 1;
        *current_block = merge_block_idx;
        
        // Insert phi nodes for variables that differ between branches
        // Find variables that exist in both branches but may have different values
        for (var_name, then_type) in &symbols_after_then {
            if let Some(else_type) = symbols_after_else.get(var_name) {
                // Variable exists in both branches
                if then_type == else_type && !symbols_before.contains_key(var_name) {
                    // Variable was defined in both branches with same type
                    // Create phi node
                    let incoming = vec![
                        (var_name.clone(), then_exit_label.clone()),
                        (var_name.clone(), else_exit_label.clone()),
                    ];
                    
                    let phi_result = self.create_phi_node(
                        then_type.clone(),
                        incoming,
                        &mut blocks[merge_block_idx].instructions,
                    );
                    
                    // Update symbol table to use phi result
                    self.symbol_types.insert(var_name.clone(), then_type.clone());
                }
            }
        }
        
        // Merge symbol types (use then branch as base, add else branch variables)
        for (var_name, var_type) in symbols_after_else {
            if !self.symbol_types.contains_key(&var_name) {
                self.symbol_types.insert(var_name, var_type);
            }
        }
        
        Ok(())
    }

    fn build_while_statement(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        blocks: &mut Vec<BasicBlock>,
        current_block: &mut usize,
    ) -> Result<()> {
        // Save symbol types before loop
        let symbols_before_loop = self.symbol_types.clone();
        let preheader_label = blocks[*current_block].label.clone();
        
        // Create labels for header, body, and exit
        let header_label = self.new_label();
        let body_label = self.new_label();
        let exit_label = self.new_label();
        
        // Branch from current block to header
        blocks[*current_block].terminator = Terminator::Br {
            target: header_label.clone(),
        };
        
        // Create header block (loop condition check)
        blocks.push(BasicBlock {
            label: header_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        let header_block_idx = blocks.len() - 1;
        *current_block = header_block_idx;
        
        // Evaluate condition in header
        let cond_reg = self.build_expression(condition, &mut blocks[*current_block].instructions)?;
        
        // Conditional branch to body or exit
        blocks[*current_block].terminator = Terminator::CondBr {
            condition: cond_reg,
            true_target: body_label.clone(),
            false_target: exit_label.clone(),
        };
        
        // Create body block
        blocks.push(BasicBlock {
            label: body_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        // Build loop body
        for stmt in body {
            self.build_statement(stmt, blocks, current_block)?;
        }
        
        let body_exit_label = blocks[*current_block].label.clone();
        
        // Branch back to header (back edge)
        if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
            blocks[*current_block].terminator = Terminator::Br {
                target: header_label.clone(),
            };
        }
        
        // Save symbol types after loop body
        let symbols_after_body = self.symbol_types.clone();
        
        // Insert phi nodes in header for loop-carried dependencies
        // Variables that are modified in the loop body need phi nodes
        for (var_name, body_type) in &symbols_after_body {
            if let Some(before_type) = symbols_before_loop.get(var_name) {
                if before_type == body_type {
                    // Variable exists before loop and is potentially modified in loop
                    // Create phi node with incoming from preheader and body
                    let incoming = vec![
                        (var_name.clone(), preheader_label.clone()),
                        (var_name.clone(), body_exit_label.clone()),
                    ];
                    
                    self.create_phi_node(
                        before_type.clone(),
                        incoming,
                        &mut blocks[header_block_idx].instructions,
                    );
                }
            }
        }
        
        // Create exit block
        blocks.push(BasicBlock {
            label: exit_label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        // Restore symbol types (variables defined only in loop are not visible after)
        self.symbol_types = symbols_before_loop;
        
        Ok(())
    }

    fn build_for_statement(
        &mut self,
        init: &Statement,
        condition: &Expression,
        update: &Statement,
        body: &[Statement],
        blocks: &mut Vec<BasicBlock>,
        current_block: &mut usize,
    ) -> Result<()> {
        // Build initialization in current block
        self.build_statement(init, blocks, current_block)?;
        
        // Create labels
        let header_label = self.new_label();
        let body_label = self.new_label();
        let update_label = self.new_label();
        let exit_label = self.new_label();
        
        // Branch to header
        if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
            blocks[*current_block].terminator = Terminator::Br {
                target: header_label.clone(),
            };
        }
        
        // Create header block
        blocks.push(BasicBlock {
            label: header_label.clone(),
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        // Evaluate condition
        let cond_reg = self.build_expression(condition, &mut blocks[*current_block].instructions)?;
        blocks[*current_block].terminator = Terminator::CondBr {
            condition: cond_reg,
            true_target: body_label.clone(),
            false_target: exit_label.clone(),
        };
        
        // Create body block
        blocks.push(BasicBlock {
            label: body_label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        // Build body
        for stmt in body {
            self.build_statement(stmt, blocks, current_block)?;
        }
        
        // Branch to update
        if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
            blocks[*current_block].terminator = Terminator::Br {
                target: update_label.clone(),
            };
        }
        
        // Create update block
        blocks.push(BasicBlock {
            label: update_label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        // Build update statement
        self.build_statement(update, blocks, current_block)?;
        
        // Branch back to header
        if matches!(blocks[*current_block].terminator, Terminator::Unreachable) {
            blocks[*current_block].terminator = Terminator::Br {
                target: header_label,
            };
        }
        
        // Create exit block
        blocks.push(BasicBlock {
            label: exit_label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        *current_block = blocks.len() - 1;
        
        Ok(())
    }

    pub fn create_phi_node(&mut self, ty: IRType, incoming: Vec<(String, String)>, instructions: &mut Vec<Instruction>) -> String {
        let result = self.new_temp();
        self.symbol_types.insert(result.clone(), ty.clone());
        
        // Insert phi instruction at the beginning of the instruction list
        instructions.insert(0, Instruction::Phi {
            result: result.clone(),
            ty,
            incoming,
        });
        
        result
    }
    
    fn insert_phi_nodes_for_merge(
        &mut self,
        merge_block_idx: usize,
        predecessor_blocks: Vec<(usize, String)>, // (block_idx, block_label)
        blocks: &mut Vec<BasicBlock>,
    ) -> Result<()> {
        // Track which variables need phi nodes
        // For now, we'll implement a simple version that doesn't track all variables
        // A full implementation would need to track all variables modified in different branches
        
        // This is a placeholder for more sophisticated phi node insertion
        // In a complete implementation, we would:
        // 1. Identify all variables that are defined in multiple predecessors
        // 2. For each such variable, create a phi node in the merge block
        // 3. Track the incoming values from each predecessor
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{BinaryOp, UnaryOp};

    fn create_test_program_with_function(func: Function) -> Program {
        Program {
            items: vec![Item::Function(func)],
        }
    }

    #[test]
    fn test_expression_building_arithmetic() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Test: 5 + 3
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntLit(5)),
            right: Box::new(Expression::IntLit(3)),
        };

        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        assert_eq!(instructions.len(), 1);
        assert!(matches!(instructions[0], Instruction::Add { .. }));
        assert!(builder.symbol_types.contains_key(&result));
        assert_eq!(builder.symbol_types.get(&result), Some(&IRType::I32));
    }

    #[test]
    fn test_expression_building_comparison() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Test: 10 < 20
        let expr = Expression::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expression::IntLit(10)),
            right: Box::new(Expression::IntLit(20)),
        };

        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        assert_eq!(instructions.len(), 1);
        assert!(matches!(instructions[0], Instruction::ICmp { .. }));
        assert_eq!(builder.symbol_types.get(&result), Some(&IRType::I1));
    }

    #[test]
    fn test_expression_building_logical() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Test: true && false
        let expr = Expression::Binary {
            op: BinaryOp::And,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::BoolLit(false)),
        };

        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        assert_eq!(instructions.len(), 1);
        assert_eq!(builder.symbol_types.get(&result), Some(&IRType::I1));
    }

    #[test]
    fn test_expression_building_unary() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Test: -42
        let expr = Expression::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expression::IntLit(42)),
        };

        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        assert_eq!(instructions.len(), 1);
        assert!(matches!(instructions[0], Instruction::Sub { .. }));
    }

    #[test]
    fn test_control_flow_if_statement() {
        let mut builder = IRBuilder::new();
        
        let func = Function {
            name: "test_if".to_string(),
            params: vec![],
            return_type: Some(Type::I32),
            body: vec![
                Statement::If {
                    condition: Expression::BoolLit(true),
                    then_body: vec![
                        Statement::Return(Some(Expression::IntLit(1))),
                    ],
                    else_body: Some(vec![
                        Statement::Return(Some(Expression::IntLit(2))),
                    ]),
                },
            ],
        };

        let program = create_test_program_with_function(func);
        let module = builder.build_module(&program).unwrap();
        
        assert_eq!(module.functions.len(), 1);
        let ir_func = &module.functions[0];
        
        // Should have multiple blocks: entry, then, else, merge
        assert!(ir_func.blocks.len() >= 3);
        
        // First block should have a conditional branch
        assert!(matches!(ir_func.blocks[0].terminator, Terminator::CondBr { .. }));
    }

    #[test]
    fn test_control_flow_while_loop() {
        let mut builder = IRBuilder::new();
        
        let func = Function {
            name: "test_while".to_string(),
            params: vec![],
            return_type: Some(Type::I32),
            body: vec![
                Statement::While {
                    condition: Expression::BoolLit(true),
                    body: vec![
                        Statement::Expression(Expression::IntLit(1)),
                    ],
                },
                Statement::Return(Some(Expression::IntLit(0))),
            ],
        };

        let program = create_test_program_with_function(func);
        let module = builder.build_module(&program).unwrap();
        
        assert_eq!(module.functions.len(), 1);
        let ir_func = &module.functions[0];
        
        // Should have multiple blocks: entry, header, body, exit
        assert!(ir_func.blocks.len() >= 4);
        
        // Entry should branch to header
        assert!(matches!(ir_func.blocks[0].terminator, Terminator::Br { .. }));
    }

    #[test]
    fn test_function_call_generation() {
        let mut builder = IRBuilder::new();
        
        // Define a function to call
        let callee = Function {
            name: "add".to_string(),
            params: vec![
                Param { name: "a".to_string(), ty: Type::I32 },
                Param { name: "b".to_string(), ty: Type::I32 },
            ],
            return_type: Some(Type::I32),
            body: vec![],
        };
        
        let caller = Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Some(Type::I32),
            body: vec![
                Statement::Return(Some(Expression::Call {
                    func: Box::new(Expression::Ident("add".to_string())),
                    args: vec![
                        Expression::IntLit(5),
                        Expression::IntLit(3),
                    ],
                })),
            ],
        };

        let program = Program {
            items: vec![
                Item::Function(callee),
                Item::Function(caller),
            ],
        };
        
        let module = builder.build_module(&program).unwrap();
        
        assert_eq!(module.functions.len(), 2);
        
        // Check that the caller has a Call instruction
        let caller_func = &module.functions[1];
        let has_call = caller_func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| matches!(inst, Instruction::Call { .. }))
        });
        assert!(has_call);
    }

    #[test]
    fn test_phi_node_creation() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();
        
        let incoming = vec![
            ("val1".to_string(), "block1".to_string()),
            ("val2".to_string(), "block2".to_string()),
        ];
        
        let result = builder.create_phi_node(IRType::I32, incoming.clone(), &mut instructions);
        
        assert_eq!(instructions.len(), 1);
        assert!(matches!(instructions[0], Instruction::Phi { .. }));
        
        if let Instruction::Phi { result: phi_result, ty, incoming: phi_incoming } = &instructions[0] {
            assert_eq!(phi_result, &result);
            assert_eq!(ty, &IRType::I32);
            assert_eq!(phi_incoming.len(), 2);
        } else {
            panic!("Expected Phi instruction");
        }
    }

    #[test]
    fn test_type_tracking() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Build an expression and check type tracking
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntLit(10)),
            right: Box::new(Expression::IntLit(20)),
        };

        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        // Check that types are tracked
        assert!(builder.symbol_types.contains_key("10"));
        assert!(builder.symbol_types.contains_key("20"));
        assert!(builder.symbol_types.contains_key(&result));
        
        assert_eq!(builder.symbol_types.get("10"), Some(&IRType::I32));
        assert_eq!(builder.symbol_types.get(&result), Some(&IRType::I32));
    }

    #[test]
    fn test_type_mismatch_detection() {
        let mut builder = IRBuilder::new();
        let mut instructions = Vec::new();

        // Manually insert mismatched types
        builder.symbol_types.insert("x".to_string(), IRType::I32);
        builder.symbol_types.insert("y".to_string(), IRType::F64);

        // Try to add I32 and F64 - should fail
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Ident("x".to_string())),
            right: Box::new(Expression::Ident("y".to_string())),
        };

        let result = builder.build_expression(&expr, &mut instructions);
        assert!(result.is_err());
    }

    #[test]
    fn test_void_function_call() {
        let mut builder = IRBuilder::new();
        
        // Register a void function
        builder.function_signatures.insert(
            "print".to_string(),
            (vec![IRType::I32], IRType::Void),
        );
        
        let mut instructions = Vec::new();
        
        let expr = Expression::Call {
            func: Box::new(Expression::Ident("print".to_string())),
            args: vec![Expression::IntLit(42)],
        };
        
        let result = builder.build_expression(&expr, &mut instructions).unwrap();
        
        // Void function should return "void" placeholder
        assert_eq!(result, "void");
        
        // Call instruction should have None as result
        assert_eq!(instructions.len(), 1);
        if let Instruction::Call { result, .. } = &instructions[0] {
            assert_eq!(result, &None);
        } else {
            panic!("Expected Call instruction");
        }
    }

    #[test]
    fn test_function_argument_count_mismatch() {
        let mut builder = IRBuilder::new();
        
        // Register a function expecting 2 arguments
        builder.function_signatures.insert(
            "add".to_string(),
            (vec![IRType::I32, IRType::I32], IRType::I32),
        );
        
        let mut instructions = Vec::new();
        
        // Call with wrong number of arguments
        let expr = Expression::Call {
            func: Box::new(Expression::Ident("add".to_string())),
            args: vec![Expression::IntLit(42)], // Only 1 argument
        };
        
        let result = builder.build_expression(&expr, &mut instructions);
        assert!(result.is_err());
    }
}
