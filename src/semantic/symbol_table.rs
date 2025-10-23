use crate::parser::{Program, Type, Item, Function, Statement};
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub scope_level: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.analyze_function(func)?;
                }
                Item::Struct(_) => {
                }
            }
        }
        Ok(())
    }

    fn analyze_function(&mut self, func: &Function) -> Result<()> {
        self.enter_scope();
        
        for param in &func.params {
            self.insert(param.name.clone(), param.ty.clone(), false)?;
        }
        
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }
        
        self.exit_scope();
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, ty, mutable, .. } => {
                if let Some(ty) = ty {
                    self.insert(name.clone(), ty.clone(), *mutable)?;
                }
            }
            Statement::If { then_block, else_block, .. } => {
                self.enter_scope();
                for stmt in then_block {
                    self.analyze_statement(stmt)?;
                }
                self.exit_scope();
                
                if let Some(else_stmts) = else_block {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.analyze_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }
            Statement::While { body, .. } => {
                self.enter_scope();
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                self.exit_scope();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn insert(&mut self, name: String, ty: Type, mutable: bool) -> Result<()> {
        let scope_level = self.scopes.len() - 1;
        
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                bail!("Variable '{}' already defined in current scope", name);
            }
            scope.insert(name.clone(), Symbol { 
                name, 
                ty, 
                mutable,
                scope_level,
            });
        }
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| scope.get(name))
    }

    pub fn current_scope_level(&self) -> usize {
        self.scopes.len() - 1
    }
}
