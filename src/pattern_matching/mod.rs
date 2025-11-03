use std::collections::HashMap;
use crate::parser::{Expression, Pattern};

pub struct PatternMatcher {
    bindings: HashMap<String, MatchedValue>,
}

#[derive(Debug, Clone)]
pub enum MatchedValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Tuple(Vec<MatchedValue>),
    Struct(HashMap<String, MatchedValue>),
    Enum(String, Box<MatchedValue>),
}

pub struct ExhaustivenessChecker {
    patterns: Vec<Pattern>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    
    pub fn match_pattern(&mut self, pattern: &Pattern, value: &MatchedValue) -> bool {
        match (pattern, value) {
            (Pattern::Wildcard, _) => true,
            
            (Pattern::Identifier(name), val) => {
                self.bindings.insert(name.clone(), val.clone());
                true
            }
            
            (Pattern::Literal(lit), val) => self.match_literal(lit, val),
            
            (Pattern::Tuple(patterns), MatchedValue::Tuple(values)) => {
                if patterns.len() != values.len() {
                    return false;
                }
                
                patterns.iter()
                    .zip(values.iter())
                    .all(|(p, v)| self.match_pattern(p, v))
            }
            
            (Pattern::Struct { name: pat_name, fields: pat_fields }, 
             MatchedValue::Struct(struct_val)) => {
                pat_fields.iter().all(|(field_name, field_pattern)| {
                    struct_val.get(field_name)
                        .map(|val| self.match_pattern(field_pattern, val))
                        .unwrap_or(false)
                })
            }
            
            _ => false,
        }
    }
    
    fn match_literal(&self, lit: &crate::parser::Literal, value: &MatchedValue) -> bool {
        match (lit, value) {
            (crate::parser::Literal::Integer(l), MatchedValue::Integer(v)) => l == v,
            (crate::parser::Literal::Float(l), MatchedValue::Float(v)) => (l - v).abs() < f64::EPSILON,
            (crate::parser::Literal::String(l), MatchedValue::String(v)) => l == v,
            (crate::parser::Literal::Boolean(l), MatchedValue::Boolean(v)) => l == v,
            (crate::parser::Literal::Unit, _) => true,
            _ => false,
        }
    }
    
    pub fn get_binding(&self, name: &str) -> Option<&MatchedValue> {
        self.bindings.get(name)
    }
    
    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
    }
}

impl ExhaustivenessChecker {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    
    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }
    
    pub fn check_exhaustive(&self) -> Result<(), Vec<String>> {
        let missing = self.find_missing_patterns();
        
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
    
    fn find_missing_patterns(&self) -> Vec<String> {
        let mut missing = Vec::new();
        
        let has_wildcard = self.patterns.iter().any(|p| matches!(p, Pattern::Wildcard));
        
        if !has_wildcard {
            missing.push("_ (wildcard)".to_string());
        }
        
        missing
    }
    
    pub fn is_reachable(&self, pattern: &Pattern, index: usize) -> bool {
        for i in 0..index {
            if self.pattern_subsumes(&self.patterns[i], pattern) {
                return false;
            }
        }
        true
    }
    
    fn pattern_subsumes(&self, general: &Pattern, specific: &Pattern) -> bool {
        match (general, specific) {
            (Pattern::Wildcard, _) => true,
            (Pattern::Identifier(_), _) => true,
            (Pattern::Literal(l1), Pattern::Literal(l2)) => l1 == l2,
            _ => false,
        }
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExhaustivenessChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GuardEvaluator {
    context: HashMap<String, MatchedValue>,
}

impl GuardEvaluator {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }
    
    pub fn evaluate_guard(&self, guard: &Expression) -> Result<bool, String> {
        match guard {
            Expression::BoolLit(b) => Ok(*b),
            Expression::Ident(name) | Expression::Identifier(name) => {
                match self.context.get(name) {
                    Some(MatchedValue::Boolean(b)) => Ok(*b),
                    Some(_) => Err("Guard must be boolean".to_string()),
                    None => Err(format!("Undefined variable '{}'", name)),
                }
            }
            Expression::Binary { op, left, right } => {
                self.evaluate_binary_guard(op, left, right)
            }
            _ => Err("Unsupported guard expression".to_string()),
        }
    }
    
    fn evaluate_binary_guard(
        &self,
        op: &crate::parser::BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<bool, String> {
        use crate::parser::BinaryOp::*;
        
        match op {
            And => {
                let l = self.evaluate_guard(left)?;
                let r = self.evaluate_guard(right)?;
                Ok(l && r)
            }
            Or => {
                let l = self.evaluate_guard(left)?;
                let r = self.evaluate_guard(right)?;
                Ok(l || r)
            }
            _ => Err("Unsupported guard operator".to_string()),
        }
    }
    
    pub fn set_context(&mut self, name: String, value: MatchedValue) {
        self.context.insert(name, value);
    }
}

impl Default for GuardEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
