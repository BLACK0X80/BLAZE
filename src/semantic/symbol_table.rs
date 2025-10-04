use std::collections::HashMap;
use crate::parser::{Program, Item, Function, Type};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub scope_level: usize,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            current_scope: 0,
            next_scope_id: 0,
        };
        table.enter_scope();
        table
    }

    pub fn analyze(&mut self, program: &Program) -> anyhow::Result<()> {
        for item in &program.items {
            self.analyze_item(item)?;
        }
        Ok(())
    }

    fn analyze_item(&mut self, item: &Item) -> anyhow::Result<()> {
        match item {
            Item::Function(func) => {
                self.analyze_function(func)?;
            }
            Item::Const(const_def) => {
                self.add_symbol(const_def.name.clone(), const_def.ty.clone(), false)?;
            }
            Item::Static(static_def) => {
                self.add_symbol(static_def.name.clone(), static_def.ty.clone(), false)?;
            }
            Item::Struct(struct_def) => {
                self.add_symbol(struct_def.name.clone(), Type::Identifier(struct_def.name.clone()), false)?;
            }
            Item::Enum(enum_def) => {
                self.add_symbol(enum_def.name.clone(), Type::Identifier(enum_def.name.clone()), false)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_function(&mut self, func: &Function) -> anyhow::Result<()> {
        self.enter_scope();
        
        for param in &func.params {
            self.add_symbol(param.name.clone(), param.ty.clone(), false)?;
        }
        
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }
        
        self.exit_scope();
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &crate::parser::Statement) -> anyhow::Result<()> {
        match stmt {
            crate::parser::Statement::Let { name, ty, mutable, .. } => {
                if let Some(ty) = ty {
                    self.add_symbol(name.clone(), ty.clone(), *mutable)?;
                }
            }
            crate::parser::Statement::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.analyze_statement(stmt)?;
                }
                self.exit_scope();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn enter_scope(&mut self) {
        let new_scope = Scope {
            level: self.scopes.len(),
            symbols: HashMap::new(),
            parent: Some(self.current_scope),
        };
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    pub fn add_symbol(&mut self, name: String, ty: Type, mutable: bool) -> anyhow::Result<()> {
        let symbol = Symbol {
            name: name.clone(),
            ty,
            mutable,
            scope_level: self.current_scope,
        };
        
        if self.scopes[self.current_scope].symbols.contains_key(&name) {
            return Err(anyhow::anyhow!("Symbol '{}' already defined in this scope", name));
        }
        
        self.scopes[self.current_scope].symbols.insert(name, symbol);
        Ok(())
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&Symbol> {
        let mut current = self.current_scope;
        
        loop {
            if let Some(symbol) = self.scopes[current].symbols.get(name) {
                return Some(symbol);
            }
            
            if let Some(parent) = self.scopes[current].parent {
                current = parent;
            } else {
                break;
            }
        }
        
        None
    }

    pub fn lookup_function(&self, name: &str) -> Option<&Symbol> {
        for scope in &self.scopes {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }
}