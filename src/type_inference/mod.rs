use std::collections::HashMap;
use crate::parser::{Type, Expression, Statement, Function};

pub struct TypeInferenceEngine {
    type_map: HashMap<String, Type>,
    constraints: Vec<TypeConstraint>,
    next_type_var: usize,
}

#[derive(Debug, Clone)]
struct TypeConstraint {
    left: InferredType,
    right: InferredType,
}

#[derive(Debug, Clone)]
enum InferredType {
    Known(Type),
    Variable(usize),
    Function(Vec<InferredType>, Box<InferredType>),
}

impl TypeInferenceEngine {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
            constraints: Vec::new(),
            next_type_var: 0,
        }
    }
    
    pub fn infer_function(&mut self, function: &Function) -> Result<Type, String> {
        for param in &function.params {
            self.type_map.insert(param.name.clone(), param.ty.clone());
        }
        
        let mut last_type = Type::Custom("()".to_string());
        
        for stmt in &function.body {
            last_type = self.infer_statement(stmt)?;
        }
        
        if let Some(ret_type) = &function.return_type {
            if !self.types_match(ret_type, &last_type) {
                return Err(format!(
                    "Return type mismatch: expected {:?}, found {:?}",
                    ret_type, last_type
                ));
            }
            Ok(ret_type.clone())
        } else {
            Ok(last_type)
        }
    }
    
    fn infer_statement(&mut self, stmt: &Statement) -> Result<Type, String> {
        match stmt {
            Statement::Let { name, ty, value, .. } => {
                let inferred_type = if let Some(expr) = value {
                    self.infer_expression(expr)?
                } else {
                    ty.clone().unwrap_or(Type::Custom("()".to_string()))
                };
                
                if let Some(declared_type) = ty {
                    if !self.types_match(declared_type, &inferred_type) {
                        return Err(format!(
                            "Type mismatch in let binding '{}': expected {:?}, found {:?}",
                            name, declared_type, inferred_type
                        ));
                    }
                }
                
                self.type_map.insert(name.clone(), inferred_type.clone());
                Ok(inferred_type)
            }
            
            Statement::Return(Some(expr)) => self.infer_expression(expr),
            
            Statement::Expression(expr) => self.infer_expression(expr),
            
            Statement::If { condition, then_body, else_body } => {
                let cond_type = self.infer_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("Condition must be boolean, found {:?}", cond_type));
                }
                
                let mut then_type = Type::Custom("()".to_string());
                for stmt in then_body {
                    then_type = self.infer_statement(stmt)?;
                }
                
                if let Some(else_stmts) = else_body {
                    let mut else_type = Type::Custom("()".to_string());
                    for stmt in else_stmts {
                        else_type = self.infer_statement(stmt)?;
                    }
                    
                    if !self.types_match(&then_type, &else_type) {
                        return Err(format!(
                            "If-else branches have different types: {:?} vs {:?}",
                            then_type, else_type
                        ));
                    }
                }
                
                Ok(then_type)
            }
            
            Statement::While { condition, body } => {
                let cond_type = self.infer_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("Condition must be boolean, found {:?}", cond_type));
                }
                
                for stmt in body {
                    self.infer_statement(stmt)?;
                }
                
                Ok(Type::Custom("()".to_string()))
            }
            
            _ => Ok(Type::Custom("()".to_string())),
        }
    }
    
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::IntLit(_) => Ok(Type::I64),
            Expression::FloatLit(_) => Ok(Type::F64),
            Expression::BoolLit(_) => Ok(Type::Bool),
            Expression::StringLit(_) => Ok(Type::String),
            Expression::CharLit(_) => Ok(Type::Char),
            
            Expression::Ident(name) | Expression::Identifier(name) => {
                self.type_map
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable '{}'", name))
            }
            
            Expression::Binary { op, left, right } => {
                let left_type = self.infer_expression(left)?;
                let right_type = self.infer_expression(right)?;
                
                if !self.types_match(&left_type, &right_type) {
                    return Err(format!(
                        "Binary operation type mismatch: {:?} vs {:?}",
                        left_type, right_type
                    ));
                }
                
                use crate::parser::BinaryOp::*;
                match op {
                    Add | Sub | Mul | Div | Mod => Ok(left_type),
                    Eq | Ne | Lt | Le | Gt | Ge => Ok(Type::Bool),
                    And | Or => {
                        if left_type != Type::Bool {
                            return Err(format!("Logical operation requires boolean operands"));
                        }
                        Ok(Type::Bool)
                    }
                    BitwiseAnd | BitwiseOr | BitwiseXor | LeftShift | RightShift => {
                        if !matches!(left_type, Type::I32 | Type::I64 | Type::U32 | Type::U64) {
                            return Err(format!("Bitwise operation requires integer operands"));
                        }
                        Ok(left_type)
                    }
                }
            }
            
            Expression::Unary { op, expr } => {
                let expr_type = self.infer_expression(expr)?;
                
                use crate::parser::UnaryOp::*;
                match op {
                    Neg => {
                        if !matches!(expr_type, Type::I32 | Type::I64 | Type::F32 | Type::F64) {
                            return Err(format!("Negation requires numeric type"));
                        }
                        Ok(expr_type)
                    }
                    Not => {
                        if expr_type != Type::Bool {
                            return Err(format!("Logical not requires boolean type"));
                        }
                        Ok(Type::Bool)
                    }
                    _ => Ok(expr_type),
                }
            }
            
            Expression::Call { func, args } | Expression::CallAlt { callee: func, args } => {
                for arg in args {
                    self.infer_expression(arg)?;
                }
                
                Ok(Type::Custom("unknown".to_string()))
            }
            
            Expression::If { condition, then_branch, else_branch } => {
                let cond_type = self.infer_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("Condition must be boolean"));
                }
                
                let then_type = self.infer_expression(then_branch)?;
                
                if let Some(else_expr) = else_branch {
                    let else_type = self.infer_expression(else_expr)?;
                    
                    if !self.types_match(&then_type, &else_type) {
                        return Err(format!(
                            "If-else expression branches have different types: {:?} vs {:?}",
                            then_type, else_type
                        ));
                    }
                }
                
                Ok(then_type)
            }
            
            _ => Ok(Type::Custom("unknown".to_string())),
        }
    }
    
    fn types_match(&self, a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::I32, Type::I32) |
            (Type::I64, Type::I64) |
            (Type::F32, Type::F32) |
            (Type::F64, Type::F64) |
            (Type::Bool, Type::Bool) |
            (Type::Char, Type::Char) |
            (Type::String, Type::String) => true,
            
            (Type::Custom(a), Type::Custom(b)) => a == b,
            
            _ => false,
        }
    }
    
    fn new_type_var(&mut self) -> InferredType {
        let var = InferredType::Variable(self.next_type_var);
        self.next_type_var += 1;
        var
    }
    
    fn add_constraint(&mut self, left: InferredType, right: InferredType) {
        self.constraints.push(TypeConstraint { left, right });
    }
    
    pub fn solve_constraints(&mut self) -> Result<(), String> {
        let mut substitutions: HashMap<usize, InferredType> = HashMap::new();
        
        for constraint in &self.constraints {
            self.unify(&constraint.left, &constraint.right, &mut substitutions)?;
        }
        
        Ok(())
    }
    
    fn unify(
        &self,
        a: &InferredType,
        b: &InferredType,
        substitutions: &mut HashMap<usize, InferredType>,
    ) -> Result<(), String> {
        match (a, b) {
            (InferredType::Known(t1), InferredType::Known(t2)) => {
                if !self.types_match(t1, t2) {
                    return Err(format!("Cannot unify types {:?} and {:?}", t1, t2));
                }
                Ok(())
            }
            
            (InferredType::Variable(v), t) | (t, InferredType::Variable(v)) => {
                if let Some(existing) = substitutions.get(v) {
                    self.unify(existing, t, substitutions)
                } else {
                    substitutions.insert(*v, t.clone());
                    Ok(())
                }
            }
            
            (InferredType::Function(params1, ret1), InferredType::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err("Function parameter count mismatch".to_string());
                }
                
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2, substitutions)?;
                }
                
                self.unify(ret1, ret2, substitutions)
            }
            
            _ => Err("Type unification failed".to_string()),
        }
    }
}

impl Default for TypeInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}
