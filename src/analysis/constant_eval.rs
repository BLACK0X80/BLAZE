use crate::parser::{Expression, BinaryOp, UnaryOp, Statement, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Unit,
}

pub type EvalResult = Result<ConstantValue, String>;

pub struct ConstantEvaluator {
    constants: HashMap<String, ConstantValue>,
}

impl ConstantEvaluator {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
        }
    }
    
    pub fn eval_expression(&mut self, expr: &Expression) -> EvalResult {
        match expr {
            Expression::IntLit(val) => Ok(ConstantValue::Int(*val)),
            Expression::FloatLit(val) => Ok(ConstantValue::Float(*val)),
            Expression::BoolLit(val) => Ok(ConstantValue::Bool(*val)),
            Expression::StringLit(val) => Ok(ConstantValue::String(val.clone())),
            Expression::CharLit(val) => Ok(ConstantValue::Char(*val)),
            
            Expression::Ident(name) | Expression::Identifier(name) => {
                self.constants
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable '{}' not found", name))
            }
            
            Expression::Binary { op, left, right } => {
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                self.eval_binary_op(*op, left_val, right_val)
            }
            
            Expression::Unary { op, expr } => {
                let val = self.eval_expression(expr)?;
                self.eval_unary_op(*op, val)
            }
            
            Expression::If { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    ConstantValue::Bool(true) => self.eval_expression(then_branch),
                    ConstantValue::Bool(false) => {
                        if let Some(else_expr) = else_branch {
                            self.eval_expression(else_expr)
                        } else {
                            Ok(ConstantValue::Unit)
                        }
                    }
                    _ => Err("Condition must be boolean".to_string()),
                }
            }
            
            Expression::Block(stmts) => {
                let mut last_val = ConstantValue::Unit;
                for stmt in stmts {
                    last_val = self.eval_statement(stmt)?;
                }
                Ok(last_val)
            }
            
            _ => Err(format!("Cannot evaluate expression: {:?}", expr)),
        }
    }
    
    fn eval_statement(&mut self, stmt: &Statement) -> EvalResult {
        match stmt {
            Statement::Let { name, value, .. } => {
                if let Some(expr) = value {
                    let val = self.eval_expression(expr)?;
                    self.constants.insert(name.clone(), val);
                }
                Ok(ConstantValue::Unit)
            }
            
            Statement::Expression(expr) => self.eval_expression(expr),
            
            Statement::Return(Some(expr)) => self.eval_expression(expr),
            
            _ => Ok(ConstantValue::Unit),
        }
    }
    
    fn eval_binary_op(&self, op: BinaryOp, left: ConstantValue, right: ConstantValue) -> EvalResult {
        match (left, right) {
            (ConstantValue::Int(l), ConstantValue::Int(r)) => {
                self.eval_int_binary_op(op, l, r)
            }
            
            (ConstantValue::Float(l), ConstantValue::Float(r)) => {
                self.eval_float_binary_op(op, l, r)
            }
            
            (ConstantValue::Bool(l), ConstantValue::Bool(r)) => {
                self.eval_bool_binary_op(op, l, r)
            }
            
            (ConstantValue::String(l), ConstantValue::String(r)) => {
                match op {
                    BinaryOp::Add => Ok(ConstantValue::String(format!("{}{}", l, r))),
                    BinaryOp::Eq => Ok(ConstantValue::Bool(l == r)),
                    BinaryOp::Ne => Ok(ConstantValue::Bool(l != r)),
                    _ => Err(format!("Invalid operation {:?} for strings", op)),
                }
            }
            
            (ConstantValue::Int(l), ConstantValue::Float(r)) => {
                self.eval_float_binary_op(op, l as f64, r)
            }
            
            (ConstantValue::Float(l), ConstantValue::Int(r)) => {
                self.eval_float_binary_op(op, l, r as f64)
            }
            
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }
    
    fn eval_int_binary_op(&self, op: BinaryOp, l: i64, r: i64) -> EvalResult {
        match op {
            BinaryOp::Add => l.checked_add(r)
                .map(ConstantValue::Int)
                .ok_or_else(|| "Integer overflow".to_string()),
            
            BinaryOp::Sub => l.checked_sub(r)
                .map(ConstantValue::Int)
                .ok_or_else(|| "Integer overflow".to_string()),
            
            BinaryOp::Mul => l.checked_mul(r)
                .map(ConstantValue::Int)
                .ok_or_else(|| "Integer overflow".to_string()),
            
            BinaryOp::Div => {
                if r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    l.checked_div(r)
                        .map(ConstantValue::Int)
                        .ok_or_else(|| "Integer overflow".to_string())
                }
            }
            
            BinaryOp::Mod => {
                if r == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    l.checked_rem(r)
                        .map(ConstantValue::Int)
                        .ok_or_else(|| "Integer overflow".to_string())
                }
            }
            
            BinaryOp::Eq => Ok(ConstantValue::Bool(l == r)),
            BinaryOp::Ne => Ok(ConstantValue::Bool(l != r)),
            BinaryOp::Lt => Ok(ConstantValue::Bool(l < r)),
            BinaryOp::Le => Ok(ConstantValue::Bool(l <= r)),
            BinaryOp::Gt => Ok(ConstantValue::Bool(l > r)),
            BinaryOp::Ge => Ok(ConstantValue::Bool(l >= r)),
            
            BinaryOp::BitwiseAnd => Ok(ConstantValue::Int(l & r)),
            BinaryOp::BitwiseOr => Ok(ConstantValue::Int(l | r)),
            BinaryOp::BitwiseXor => Ok(ConstantValue::Int(l ^ r)),
            BinaryOp::LeftShift => Ok(ConstantValue::Int(l << r)),
            BinaryOp::RightShift => Ok(ConstantValue::Int(l >> r)),
            
            _ => Err(format!("Invalid operation {:?} for integers", op)),
        }
    }
    
    fn eval_float_binary_op(&self, op: BinaryOp, l: f64, r: f64) -> EvalResult {
        match op {
            BinaryOp::Add => Ok(ConstantValue::Float(l + r)),
            BinaryOp::Sub => Ok(ConstantValue::Float(l - r)),
            BinaryOp::Mul => Ok(ConstantValue::Float(l * r)),
            BinaryOp::Div => {
                if r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(ConstantValue::Float(l / r))
                }
            }
            BinaryOp::Mod => Ok(ConstantValue::Float(l % r)),
            
            BinaryOp::Eq => Ok(ConstantValue::Bool((l - r).abs() < f64::EPSILON)),
            BinaryOp::Ne => Ok(ConstantValue::Bool((l - r).abs() >= f64::EPSILON)),
            BinaryOp::Lt => Ok(ConstantValue::Bool(l < r)),
            BinaryOp::Le => Ok(ConstantValue::Bool(l <= r)),
            BinaryOp::Gt => Ok(ConstantValue::Bool(l > r)),
            BinaryOp::Ge => Ok(ConstantValue::Bool(l >= r)),
            
            _ => Err(format!("Invalid operation {:?} for floats", op)),
        }
    }
    
    fn eval_bool_binary_op(&self, op: BinaryOp, l: bool, r: bool) -> EvalResult {
        match op {
            BinaryOp::And => Ok(ConstantValue::Bool(l && r)),
            BinaryOp::Or => Ok(ConstantValue::Bool(l || r)),
            BinaryOp::Eq => Ok(ConstantValue::Bool(l == r)),
            BinaryOp::Ne => Ok(ConstantValue::Bool(l != r)),
            _ => Err(format!("Invalid operation {:?} for booleans", op)),
        }
    }
    
    fn eval_unary_op(&self, op: UnaryOp, val: ConstantValue) -> EvalResult {
        match (op, val) {
            (UnaryOp::Neg, ConstantValue::Int(v)) => {
                v.checked_neg()
                    .map(ConstantValue::Int)
                    .ok_or_else(|| "Integer overflow".to_string())
            }
            
            (UnaryOp::Neg, ConstantValue::Float(v)) => Ok(ConstantValue::Float(-v)),
            
            (UnaryOp::Not, ConstantValue::Bool(v)) => Ok(ConstantValue::Bool(!v)),
            
            _ => Err(format!("Invalid unary operation {:?}", op)),
        }
    }
    
    pub fn fold_constants(&mut self, expr: &Expression) -> Expression {
        match self.eval_expression(expr) {
            Ok(ConstantValue::Int(v)) => Expression::IntLit(v),
            Ok(ConstantValue::Float(v)) => Expression::FloatLit(v),
            Ok(ConstantValue::Bool(v)) => Expression::BoolLit(v),
            Ok(ConstantValue::String(v)) => Expression::StringLit(v),
            Ok(ConstantValue::Char(v)) => Expression::CharLit(v),
            _ => expr.clone(),
        }
    }
    
    pub fn is_constant(&self, expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::IntLit(_)
                | Expression::FloatLit(_)
                | Expression::BoolLit(_)
                | Expression::StringLit(_)
                | Expression::CharLit(_)
        )
    }
    
    pub fn set_constant(&mut self, name: String, value: ConstantValue) {
        self.constants.insert(name, value);
    }
    
    pub fn get_constant(&self, name: &str) -> Option<&ConstantValue> {
        self.constants.get(name)
    }
    
    pub fn clear(&mut self) {
        self.constants.clear();
    }
}

impl Default for ConstantEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstantValue {
    pub fn to_type(&self) -> Type {
        match self {
            ConstantValue::Int(_) => Type::I64,
            ConstantValue::Float(_) => Type::F64,
            ConstantValue::Bool(_) => Type::Bool,
            ConstantValue::String(_) => Type::String,
            ConstantValue::Char(_) => Type::Char,
            ConstantValue::Unit => Type::Custom("()".to_string()),
        }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            ConstantValue::Bool(b) => *b,
            ConstantValue::Int(i) => *i != 0,
            ConstantValue::Float(f) => *f != 0.0,
            ConstantValue::String(s) => !s.is_empty(),
            _ => false,
        }
    }
}
