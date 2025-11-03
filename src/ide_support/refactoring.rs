use std::collections::HashMap;
use crate::parser::{Program, Function, Statement, Expression};

pub struct RefactoringEngine {
    rename_operations: Vec<RenameOperation>,
    extract_operations: Vec<ExtractOperation>,
}

#[derive(Debug, Clone)]
pub struct RenameOperation {
    pub old_name: String,
    pub new_name: String,
    pub scope: RefactoringScope,
}

#[derive(Debug, Clone)]
pub struct ExtractOperation {
    pub target: ExtractionTarget,
    pub new_name: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefactoringScope {
    Local,
    Module,
    Global,
}

#[derive(Debug, Clone)]
pub enum ExtractionTarget {
    Function(Vec<Statement>),
    Variable(Expression),
    Constant(Expression),
}

impl RefactoringEngine {
    pub fn new() -> Self {
        Self {
            rename_operations: Vec::new(),
            extract_operations: Vec::new(),
        }
    }
    
    pub fn rename_symbol(&mut self, old_name: String, new_name: String, scope: RefactoringScope) -> Result<(), String> {
        if !self.is_valid_identifier(&new_name) {
            return Err("Invalid identifier name".to_string());
        }
        
        self.rename_operations.push(RenameOperation {
            old_name,
            new_name,
            scope,
        });
        
        Ok(())
    }
    
    pub fn extract_function(&mut self, statements: Vec<Statement>, name: String) -> Result<Function, String> {
        let params = self.analyze_dependencies(&statements);
        
        self.extract_operations.push(ExtractOperation {
            target: ExtractionTarget::Function(statements.clone()),
            new_name: name.clone(),
            parameters: params.clone(),
        });
        
        Ok(Function {
            name,
            params: params.iter().map(|p| crate::parser::Param {
                name: p.clone(),
                ty: crate::parser::Type::I32,
            }).collect(),
            return_type: None,
            body: statements,
        })
    }
    
    pub fn extract_variable(&mut self, expr: Expression, name: String) -> Result<Statement, String> {
        self.extract_operations.push(ExtractOperation {
            target: ExtractionTarget::Variable(expr.clone()),
            new_name: name.clone(),
            parameters: vec![],
        });
        
        Ok(Statement::Let {
            name,
            mutable: false,
            ty: None,
            value: Some(expr),
        })
    }
    
    pub fn inline_variable(&self, var_name: &str, program: &Program) -> Result<Program, String> {
        Ok(program.clone())
    }
    
    pub fn inline_function(&self, func_name: &str, program: &Program) -> Result<Program, String> {
        Ok(program.clone())
    }
    
    pub fn move_to_module(&self, symbol: &str, target_module: &str) -> Result<(), String> {
        Ok(())
    }
    
    pub fn change_signature(&self, func: &Function, new_params: Vec<crate::parser::Param>) -> Result<Function, String> {
        Ok(Function {
            name: func.name.clone(),
            params: new_params,
            return_type: func.return_type.clone(),
            body: func.body.clone(),
        })
    }
    
    fn analyze_dependencies(&self, statements: &[Statement]) -> Vec<String> {
        let mut deps = Vec::new();
        
        for stmt in statements {
            deps.extend(self.extract_variables_from_statement(stmt));
        }
        
        deps.sort();
        deps.dedup();
        deps
    }
    
    fn extract_variables_from_statement(&self, stmt: &Statement) -> Vec<String> {
        match stmt {
            Statement::Expression(expr) => self.extract_variables_from_expression(expr),
            Statement::Return(Some(expr)) => self.extract_variables_from_expression(expr),
            _ => vec![],
        }
    }
    
    fn extract_variables_from_expression(&self, expr: &Expression) -> Vec<String> {
        match expr {
            Expression::Ident(name) | Expression::Identifier(name) => vec![name.clone()],
            Expression::Binary { left, right, .. } => {
                let mut vars = self.extract_variables_from_expression(left);
                vars.extend(self.extract_variables_from_expression(right));
                vars
            }
            _ => vec![],
        }
    }
    
    fn is_valid_identifier(&self, name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
    
    pub fn apply_refactorings(&self, program: &mut Program) -> Result<(), String> {
        for rename_op in &self.rename_operations {
            self.apply_rename(program, rename_op)?;
        }
        
        Ok(())
    }
    
    fn apply_rename(&self, _program: &mut Program, _op: &RenameOperation) -> Result<(), String> {
        Ok(())
    }
}

impl Default for RefactoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CodeActionsProvider {
    actions: Vec<CodeAction>,
}

#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeActionKind {
    QuickFix,
    Refactor,
    RefactorExtract,
    RefactorInline,
    RefactorRewrite,
    Source,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: (usize, usize),
    pub new_text: String,
}

impl CodeActionsProvider {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
    
    pub fn get_available_actions(&self, _line: usize, _column: usize) -> Vec<CodeAction> {
        vec![
            CodeAction {
                title: "Extract function".to_string(),
                kind: CodeActionKind::RefactorExtract,
                edits: vec![],
            },
            CodeAction {
                title: "Rename symbol".to_string(),
                kind: CodeActionKind::Refactor,
                edits: vec![],
            },
        ]
    }
}

impl Default for CodeActionsProvider {
    fn default() -> Self {
        Self::new()
    }
}
