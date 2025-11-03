use std::collections::HashMap;
use crate::parser::{Program, Function, Statement, Expression};

pub struct Linter {
    rules: HashMap<String, Box<dyn LintRule>>,
    diagnostics: Vec<LintDiagnostic>,
    severity_threshold: Severity,
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub message: String,
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub rule_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

pub trait LintRule {
    fn name(&self) -> &str;
    fn check_program(&self, program: &Program) -> Vec<LintDiagnostic>;
    fn check_function(&self, function: &Function) -> Vec<LintDiagnostic>;
    fn check_statement(&self, stmt: &Statement) -> Vec<LintDiagnostic>;
}

impl Linter {
    pub fn new() -> Self {
        let mut linter = Self {
            rules: HashMap::new(),
            diagnostics: Vec::new(),
            severity_threshold: Severity::Hint,
        };
        
        linter.register_default_rules();
        linter
    }
    
    fn register_default_rules(&mut self) {
    }
    
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.insert(rule.name().to_string(), rule);
    }
    
    pub fn lint(&mut self, program: &Program) -> Vec<LintDiagnostic> {
        self.diagnostics.clear();
        
        for rule in self.rules.values() {
            let mut diags = rule.check_program(program);
            self.diagnostics.append(&mut diags);
        }
        
        self.diagnostics.clone()
    }
    
    pub fn set_severity_threshold(&mut self, threshold: Severity) {
        self.severity_threshold = threshold;
    }
    
    pub fn get_diagnostics(&self) -> &[LintDiagnostic] {
        &self.diagnostics
    }
    
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct UnusedVariableRule;

impl LintRule for UnusedVariableRule {
    fn name(&self) -> &str {
        "unused_variable"
    }
    
    fn check_program(&self, _program: &Program) -> Vec<LintDiagnostic> {
        Vec::new()
    }
    
    fn check_function(&self, _function: &Function) -> Vec<LintDiagnostic> {
        Vec::new()
    }
    
    fn check_statement(&self, _stmt: &Statement) -> Vec<LintDiagnostic> {
        Vec::new()
    }
}

pub struct DeadCodeRule;

impl LintRule for DeadCodeRule {
    fn name(&self) -> &str {
        "dead_code"
    }
    
    fn check_program(&self, _program: &Program) -> Vec<LintDiagnostic> {
        Vec::new()
    }
    
    fn check_function(&self, _function: &Function) -> Vec<LintDiagnostic> {
        Vec::new()
    }
    
    fn check_statement(&self, _stmt: &Statement) -> Vec<LintDiagnostic> {
        Vec::new()
    }
}
