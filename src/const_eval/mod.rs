use std::collections::HashMap;
use crate::parser::{Expression, Statement};
use crate::analysis::constant_eval::{ConstantValue, ConstantEvaluator};

pub struct ConstExprEngine {
    evaluator: ConstantEvaluator,
    const_values: HashMap<String, ConstantValue>,
}

impl ConstExprEngine {
    pub fn new() -> Self {
        Self {
            evaluator: ConstantEvaluator::new(),
            const_values: HashMap::new(),
        }
    }
    
    pub fn eval_const_expr(&mut self, expr: &Expression) -> Result<ConstantValue, String> {
        self.evaluator.eval_expression(expr)
    }
    
    pub fn register_const(&mut self, name: String, value: ConstantValue) {
        self.const_values.insert(name, value);
    }
    
    pub fn get_const(&self, name: &str) -> Option<&ConstantValue> {
        self.const_values.get(name)
    }
    
    pub fn compile_time_if(&mut self, condition: &Expression, then_expr: &Expression, else_expr: Option<&Expression>) -> Result<ConstantValue, String> {
        let cond_val = self.eval_const_expr(condition)?;
        
        match cond_val {
            ConstantValue::Bool(true) => self.eval_const_expr(then_expr),
            ConstantValue::Bool(false) => {
                if let Some(else_e) = else_expr {
                    self.eval_const_expr(else_e)
                } else {
                    Ok(ConstantValue::Unit)
                }
            }
            _ => Err("Condition must be boolean".to_string()),
        }
    }
}

impl Default for ConstExprEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompileTimeComputation {
    functions: HashMap<String, ConstFunction>,
}

#[derive(Debug, Clone)]
pub struct ConstFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

impl CompileTimeComputation {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
    
    pub fn register_const_fn(&mut self, func: ConstFunction) {
        self.functions.insert(func.name.clone(), func);
    }
    
    pub fn evaluate_const_fn(&self, name: &str, args: Vec<ConstantValue>) -> Result<ConstantValue, String> {
        let func = self.functions.get(name)
            .ok_or_else(|| format!("Const function '{}' not found", name))?;
        
        if args.len() != func.params.len() {
            return Err(format!("Wrong number of arguments for '{}'", name));
        }
        
        Ok(ConstantValue::Int(0))
    }
}

impl Default for CompileTimeComputation {
    fn default() -> Self {
        Self::new()
    }
}
