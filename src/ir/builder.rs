use super::{Module, IRFunction, BasicBlock, Instruction, Terminator, IRType, Parameter, Constant, GlobalVariable, TypeDefinition};
use crate::parser::{Program, Item, Function, Statement, Expression, Type, BinaryOperator, UnaryOperator, Literal};
use std::collections::HashMap;
use anyhow::Result;

pub struct IRBuilder {
    current_function: Option<String>,
    current_block: Option<String>,
    next_temp: usize,
    next_block: usize,
    variables: HashMap<String, String>,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
            next_temp: 0,
            next_block: 0,
            variables: HashMap::new(),
        }
    }

    pub fn build_module(&mut self, program: &Program) -> Result<Module> {
        let mut functions = Vec::new();
        let mut globals = Vec::new();
        let mut types = Vec::new();

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    functions.push(self.build_function(func)?);
                }
                Item::Struct(struct_def) => {
                    let fields: Vec<IRType> = struct_def.fields.iter()
                        .map(|field| self.convert_type(&field.ty))
                        .collect();
                    
                    types.push(TypeDefinition {
                        name: struct_def.name.clone(),
                        ty: IRType::Struct { fields },
                    });
                }
                Item::Const(const_def) => {
                    let initializer = self.build_constant(&const_def.value)?;
                    globals.push(GlobalVariable {
                        name: const_def.name.clone(),
                        ty: self.convert_type(&const_def.ty),
                        initializer: Some(initializer),
                        is_constant: true,
                    });
                }
                Item::Static(static_def) => {
                    let initializer = self.build_constant(&static_def.value)?;
                    globals.push(GlobalVariable {
                        name: static_def.name.clone(),
                        ty: self.convert_type(&static_def.ty),
                        initializer: Some(initializer),
                        is_constant: false,
                    });
                }
                _ => {}
            }
        }

        Ok(Module {
            functions,
            globals,
            types,
        })
    }

    fn build_function(&mut self, func: &Function) -> Result<IRFunction> {
        self.current_function = Some(func.name.clone());
        self.next_temp = 0;
        self.next_block = 0;
        self.variables.clear();

        let params: Vec<Parameter> = func.params.iter()
            .map(|param| Parameter {
                name: param.name.clone(),
                ty: self.convert_type(&param.ty),
            })
            .collect();

        let return_type = func.return_type.as_ref()
            .map(|ty| self.convert_type(ty))
            .unwrap_or(IRType::Void);

        let entry_block = self.new_block("entry");
        self.current_block = Some(entry_block.label.clone());

        for param in &params {
            let alloca_name = self.new_temp();
            let ptr_name = self.new_temp();
            self.variables.insert(param.name.clone(), alloca_name.clone());
        }

        let mut blocks = vec![entry_block];
        let mut current_instructions = Vec::new();

        for stmt in &func.body {
            let (mut new_instructions, new_blocks) = self.build_statement(stmt)?;
            current_instructions.append(&mut new_instructions);
            blocks.extend(new_blocks);
        }

        if let Some(current_block_label) = &self.current_block {
            if let Some(block) = blocks.iter_mut().find(|b| b.label == *current_block_label) {
                block.instructions.extend(current_instructions);
                if matches!(block.terminator, Terminator::Unreachable) {
                    block.terminator = Terminator::Return { value: None };
                }
            }
        }

        Ok(IRFunction {
            name: func.name.clone(),
            params,
            return_type,
            blocks,
        })
    }

    fn build_statement(&mut self, stmt: &Statement) -> Result<(Vec<Instruction>, Vec<BasicBlock>)> {
        let mut instructions = Vec::new();
        let mut blocks = Vec::new();

        match stmt {
            Statement::Let { name, ty, value, mutable: _ } => {
                let alloca_name = self.new_temp();
                let ir_type = ty.as_ref()
                    .map(|t| self.convert_type(t))
                    .unwrap_or(IRType::I32);

                instructions.push(Instruction::Alloca {
                    result: alloca_name.clone(),
                    ty: ir_type.clone(),
                });

                if let Some(value_expr) = value {
                    let (mut value_instructions, value_blocks, value_result) = 
                        self.build_expression(value_expr)?;
                    instructions.append(&mut value_instructions);
                    blocks.extend(value_blocks);

                    instructions.push(Instruction::Store {
                        ty: ir_type,
                        value: value_result,
                        ptr: alloca_name.clone(),
                    });
                }

                self.variables.insert(name.clone(), alloca_name);
            }
            Statement::Expression(expr) => {
                let (mut expr_instructions, expr_blocks, _) = self.build_expression(expr)?;
                instructions.append(&mut expr_instructions);
                blocks.extend(expr_blocks);
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let (mut expr_instructions, expr_blocks, result) = self.build_expression(expr)?;
                    instructions.append(&mut expr_instructions);
                    blocks.extend(expr_blocks);
                    
                    if let Some(current_block_label) = &self.current_block {
                        if let Some(block) = blocks.iter_mut().find(|b| b.label == *current_block_label) {
                            block.terminator = Terminator::Return {
                                value: Some((IRType::I32, result)),
                            };
                        }
                    }
                } else {
                    if let Some(current_block_label) = &self.current_block {
                        if let Some(block) = blocks.iter_mut().find(|b| b.label == *current_block_label) {
                            block.terminator = Terminator::Return { value: None };
                        }
                    }
                }
            }
            Statement::While { condition, body } => {
                let header_block = self.new_block("while.header");
                let body_block = self.new_block("while.body");
                let exit_block = self.new_block("while.exit");

                instructions.push(Instruction::GetElementPtr {
                    result: self.new_temp(),
                    ty: IRType::I1,
                    ptr: "dummy".to_string(),
                    indices: vec![],
                });

                let (mut cond_instructions, cond_blocks, cond_result) = 
                    self.build_expression(condition)?;
                header_block.instructions.extend(cond_instructions);
                blocks.extend(cond_blocks);

                header_block.terminator = Terminator::ConditionalBranch {
                    condition: cond_result,
                    true_dest: body_block.label.clone(),
                    false_dest: exit_block.label.clone(),
                };

                for stmt in body {
                    let (mut stmt_instructions, stmt_blocks) = self.build_statement(stmt)?;
                    body_block.instructions.extend(stmt_instructions);
                    blocks.extend(stmt_blocks);
                }

                body_block.terminator = Terminator::Branch {
                    dest: header_block.label.clone(),
                };

                exit_block.terminator = Terminator::Unreachable;

                blocks.extend([header_block, body_block, exit_block]);
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    let (mut stmt_instructions, stmt_blocks) = self.build_statement(stmt)?;
                    instructions.append(&mut stmt_instructions);
                    blocks.extend(stmt_blocks);
                }
            }
            _ => {}
        }

        Ok((instructions, blocks))
    }

    fn build_expression(&mut self, expr: &Expression) -> Result<(Vec<Instruction>, Vec<BasicBlock>, String)> {
        let mut instructions = Vec::new();
        let mut blocks = Vec::new();

        let result = match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Integer(value) => {
                        let temp = self.new_temp();
                        format!("{}", value)
                    }
                    Literal::Float(value) => {
                        let temp = self.new_temp();
                        format!("{}", value)
                    }
                    Literal::Boolean(value) => {
                        format!("{}", if *value { 1 } else { 0 })
                    }
                    Literal::String(value) => {
                        format!("\"{}\"", value)
                    }
                    _ => self.new_temp(),
                }
            }
            Expression::Identifier(name) => {
                if let Some(ptr) = self.variables.get(name) {
                    let result = self.new_temp();
                    instructions.push(Instruction::Load {
                        result: result.clone(),
                        ty: IRType::I32,
                        ptr: ptr.clone(),
                    });
                    result
                } else {
                    name.clone()
                }
            }
            Expression::Binary { left, operator, right } => {
                let (mut left_instructions, left_blocks, left_result) = 
                    self.build_expression(left)?;
                let (mut right_instructions, right_blocks, right_result) = 
                    self.build_expression(right)?;
                
                instructions.append(&mut left_instructions);
                instructions.append(&mut right_instructions);
                blocks.extend(left_blocks);
                blocks.extend(right_blocks);

                let result = self.new_temp();
                let instruction = match operator {
                    BinaryOperator::Add => Instruction::Add {
                        result: result.clone(),
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Subtract => Instruction::Sub {
                        result: result.clone(),
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Multiply => Instruction::Mul {
                        result: result.clone(),
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Divide => Instruction::Div {
                        result: result.clone(),
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Equal => Instruction::ICmp {
                        result: result.clone(),
                        condition: super::instruction::ICmpCondition::Eq,
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::NotEqual => Instruction::ICmp {
                        result: result.clone(),
                        condition: super::instruction::ICmpCondition::Ne,
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Less => Instruction::ICmp {
                        result: result.clone(),
                        condition: super::instruction::ICmpCondition::Slt,
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    BinaryOperator::Greater => Instruction::ICmp {
                        result: result.clone(),
                        condition: super::instruction::ICmpCondition::Sgt,
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                    _ => Instruction::Add {
                        result: result.clone(),
                        ty: IRType::I32,
                        left: left_result,
                        right: right_result,
                    },
                };
                
                instructions.push(instruction);
                result
            }
            Expression::Call { callee, args } => {
                let function_name = if let Expression::Identifier(name) = &**callee {
                    name.clone()
                } else {
                    return Err(anyhow::anyhow!("Complex function calls not supported"));
                };

                let mut arg_results = Vec::new();
                for arg in args {
                    let (mut arg_instructions, arg_blocks, arg_result) = 
                        self.build_expression(arg)?;
                    instructions.append(&mut arg_instructions);
                    blocks.extend(arg_blocks);
                    arg_results.push((IRType::I32, arg_result));
                }

                let result = self.new_temp();
                instructions.push(Instruction::Call {
                    result: Some(result.clone()),
                    function: function_name,
                    args: arg_results,
                });
                result
            }
            Expression::If { condition, then_branch, else_branch } => {
                let (mut cond_instructions, cond_blocks, cond_result) = 
                    self.build_expression(condition)?;
                instructions.append(&mut cond_instructions);
                blocks.extend(cond_blocks);

                let then_block = self.new_block("if.then");
                let else_block = self.new_block("if.else");
                let merge_block = self.new_block("if.merge");

                let (mut then_instructions, then_blocks, then_result) = 
                    self.build_expression(then_branch)?;
                then_block.instructions.extend(then_instructions);
                blocks.extend(then_blocks);

                let else_result = if let Some(else_expr) = else_branch {
                    let (mut else_instructions, else_blocks, else_result) = 
                        self.build_expression(else_expr)?;
                    else_block.instructions.extend(else_instructions);
                    blocks.extend(else_blocks);
                    else_result
                } else {
                    "0".to_string()
                };

                then_block.terminator = Terminator::Branch {
                    dest: merge_block.label.clone(),
                };
                else_block.terminator = Terminator::Branch {
                    dest: merge_block.label.clone(),
                };

                let result = self.new_temp();
                merge_block.instructions.push(Instruction::Phi {
                    result: result.clone(),
                    ty: IRType::I32,
                    values: vec![
                        (then_result, then_block.label.clone()),
                        (else_result, else_block.label.clone()),
                    ],
                });

                merge_block.terminator = Terminator::Unreachable;

                blocks.extend([then_block, else_block, merge_block]);
                result
            }
            _ => self.new_temp(),
        };

        Ok((instructions, blocks, result))
    }

    fn build_constant(&self, expr: &Expression) -> Result<Constant> {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Integer(value) => Ok(Constant::Integer {
                        value: *value,
                        ty: IRType::I32,
                    }),
                    Literal::Float(value) => Ok(Constant::Float {
                        value: *value,
                        ty: IRType::F64,
                    }),
                    Literal::Boolean(value) => Ok(Constant::Boolean(*value)),
                    Literal::String(value) => Ok(Constant::String(value.clone())),
                    _ => Ok(Constant::Null),
                }
            }
            _ => Err(anyhow::anyhow!("Non-constant expression in constant context")),
        }
    }

    fn convert_type(&self, ty: &Type) -> IRType {
        match ty {
            Type::Primitive(prim) => {
                match prim {
                    crate::parser::PrimitiveType::I8 => IRType::I8,
                    crate::parser::PrimitiveType::I16 => IRType::I16,
                    crate::parser::PrimitiveType::I32 => IRType::I32,
                    crate::parser::PrimitiveType::I64 => IRType::I64,
                    crate::parser::PrimitiveType::I128 => IRType::I128,
                    crate::parser::PrimitiveType::U8 => IRType::I8,
                    crate::parser::PrimitiveType::U16 => IRType::I16,
                    crate::parser::PrimitiveType::U32 => IRType::I32,
                    crate::parser::PrimitiveType::U64 => IRType::I64,
                    crate::parser::PrimitiveType::U128 => IRType::I128,
                    crate::parser::PrimitiveType::F32 => IRType::F32,
                    crate::parser::PrimitiveType::F64 => IRType::F64,
                    crate::parser::PrimitiveType::Bool => IRType::I1,
                    crate::parser::PrimitiveType::Char => IRType::I32,
                    crate::parser::PrimitiveType::Str => IRType::Pointer(Box::new(IRType::I8)),
                }
            }
            Type::Reference { mutable: _, inner } => {
                IRType::Pointer(Box::new(self.convert_type(inner)))
            }
            Type::Pointer { mutable: _, inner } => {
                IRType::Pointer(Box::new(self.convert_type(inner)))
            }
            Type::Array { element_type, size } => {
                let size_val = if let Some(Expression::Literal(Literal::Integer(n))) = size {
                    *n as u64
                } else {
                    0
                };
                IRType::Array {
                    element_type: Box::new(self.convert_type(element_type)),
                    size: size_val,
                }
            }
            Type::Slice(element_type) => {
                IRType::Pointer(Box::new(self.convert_type(element_type)))
            }
            Type::Unit => IRType::Void,
            _ => IRType::I32,
        }
    }

    fn new_temp(&mut self) -> String {
        let temp = format!("%{}", self.next_temp);
        self.next_temp += 1;
        temp
    }

    fn new_block(&mut self, prefix: &str) -> BasicBlock {
        let label = format!("{}.{}", prefix, self.next_block);
        self.next_block += 1;
        BasicBlock {
            label,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
        }
    }
}

