use crate::parser::{Program, Item, Function, Statement, Expression, Type};
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct ScopeResolver {
    scopes: Vec<Scope>,
    function_signatures: HashMap<String, FunctionSignature>,
    type_definitions: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
struct Scope {
    bindings: HashMap<String, Binding>,
}

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    ty: Type,
    mutable: bool,
    resolved: bool,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    name: String,
    params: Vec<Type>,
    return_type: Option<Type>,
}

impl ScopeResolver {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope { bindings: HashMap::new() }],
            function_signatures: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, program: &Program) -> Result<()> {
        self.collect_declarations(program)?;
        
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.resolve_function(func)?;
                }
                Item::Struct(_) => {}
            }
        }
        
        Ok(())
    }

    fn collect_declarations(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let signature = FunctionSignature {
                        name: func.name.clone(),
                        params: func.params.iter().map(|p| p.ty.clone()).collect(),
                        return_type: func.return_type.clone(),
                    };
                    self.function_signatures.insert(func.name.clone(), signature);
                }
                Item::Struct(s) => {
                    self.type_definitions.insert(s.name.clone(), Type::Custom(s.name.clone()));
                }
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, func: &Function) -> Result<()> {
        self.enter_scope();
        
        for param in &func.params {
            self.add_binding(param.name.clone(), param.ty.clone(), false)?;
        }
        
        for stmt in &func.body {
            self.resolve_statement(stmt)?;
        }
        
        self.exit_scope();
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, ty, value, mutable } => {
                if let Some(expr) = value {
                    self.resolve_expression(expr)?;
                }
                
                if let Some(ty) = ty {
                    self.resolve_type(ty)?;
                    self.add_binding(name.clone(), ty.clone(), *mutable)?;
                }
            }
            Statement::If { condition, then_body, else_body } => {
                self.resolve_expression(condition)?;
                
                self.enter_scope();
                for stmt in then_body {
                    self.resolve_statement(stmt)?;
                }
                self.exit_scope();
                
                if let Some(else_stmts) = else_body {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.resolve_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }
            Statement::While { condition, body } => {
                self.resolve_expression(condition)?;
                
                self.enter_scope();
                for stmt in body {
                    self.resolve_statement(stmt)?;
                }
                self.exit_scope();
            }
            Statement::Return(Some(expr)) => {
                self.resolve_expression(expr)?;
            }
            Statement::Expression(expr) => {
                self.resolve_expression(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::Ident(name) => {
                if !self.lookup(name) {
                    bail!("Undefined variable '{}'", name);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.resolve_expression(operand)?;
            }
            Expression::Call { func, args } => {
                if let Expression::Ident(func_name) = &**func {
                    if !self.function_signatures.contains_key(func_name) {
                        bail!("Undefined function '{}'", func_name);
                    }
                }
                
                for arg in args {
                    self.resolve_expression(arg)?;
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.resolve_expression(object)?;
            }
            Expression::Index { array, index } => {
                self.resolve_expression(array)?;
                self.resolve_expression(index)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_type(&self, ty: &Type) -> Result<()> {
        match ty {
            Type::Custom(name) => {
                if !self.type_definitions.contains_key(name) {
                    bail!("Undefined type '{}'", name);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope { bindings: HashMap::new() });
    }

    fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn add_binding(&mut self, name: String, ty: Type, mutable: bool) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.bindings.contains_key(&name) {
                bail!("Variable '{}' already defined in current scope", name);
            }
            scope.bindings.insert(name.clone(), Binding {
                name,
                ty,
                mutable,
                resolved: true,
            });
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.bindings.contains_key(name) {
                return true;
            }
        }
        self.function_signatures.contains_key(name)
    }
}
