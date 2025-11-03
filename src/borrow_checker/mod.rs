use std::collections::{HashMap, HashSet};
use crate::parser::{Statement, Expression, Function};

pub struct BorrowChecker {
    borrows: HashMap<String, BorrowState>,
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone)]
enum BorrowState {
    Owned,
    Borrowed(BorrowKind),
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BorrowKind {
    Immutable(usize),
    Mutable(usize),
}

struct Scope {
    variables: HashSet<String>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            borrows: HashMap::new(),
            scopes: vec![Scope { variables: HashSet::new() }],
        }
    }
    
    pub fn check_function(&mut self, function: &Function) -> Result<(), String> {
        for param in &function.params {
            self.declare_variable(param.name.clone());
        }
        
        for stmt in &function.body {
            self.check_statement(stmt)?;
        }
        
        Ok(())
    }
    
    fn check_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let { name, value, .. } => {
                if let Some(expr) = value {
                    self.check_expression(expr)?;
                }
                self.declare_variable(name.clone());
            }
            
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
            }
            
            Statement::Return(Some(expr)) => {
                self.check_expression(expr)?;
            }
            
            Statement::While { condition, body } => {
                self.check_expression(condition)?;
                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            
            Statement::For { variable, iterable, body } => {
                self.check_expression(iterable)?;
                self.enter_scope();
                self.declare_variable(variable.clone());
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            
            Statement::If { condition, then_body, else_body } => {
                self.check_expression(condition)?;
                
                self.enter_scope();
                for stmt in then_body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
                
                if let Some(else_stmts) = else_body {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }
            
            Statement::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn check_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Ident(name) | Expression::Identifier(name) => {
                self.check_variable_use(name)?;
            }
            
            Expression::Binary { left, right, .. } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
            }
            
            Expression::Unary { expr, .. } => {
                self.check_expression(expr)?;
            }
            
            Expression::Call { func, args } | Expression::CallAlt { callee: func, args } => {
                self.check_expression(func)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
            }
            
            Expression::Assignment { target, value } => {
                self.check_mutable_access(target)?;
                self.check_expression(value)?;
            }
            
            Expression::FieldAccess { object, .. } => {
                self.check_expression(object)?;
            }
            
            Expression::MethodCall { object, args, .. } => {
                self.check_expression(object)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
            }
            
            Expression::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn declare_variable(&mut self, name: String) {
        self.borrows.insert(name.clone(), BorrowState::Owned);
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(name);
        }
    }
    
    fn check_variable_use(&self, name: &str) -> Result<(), String> {
        match self.borrows.get(name) {
            Some(BorrowState::Owned) | Some(BorrowState::Borrowed(_)) => Ok(()),
            Some(BorrowState::Moved) => {
                Err(format!("Use of moved value '{}'", name))
            }
            None => {
                Err(format!("Use of undeclared variable '{}'", name))
            }
        }
    }
    
    fn check_mutable_access(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Ident(name) | Expression::Identifier(name) => {
                match self.borrows.get(name) {
                    Some(BorrowState::Owned) => Ok(()),
                    Some(BorrowState::Borrowed(BorrowKind::Mutable(_))) => Ok(()),
                    Some(BorrowState::Borrowed(BorrowKind::Immutable(_))) => {
                        Err(format!("Cannot mutate immutably borrowed value '{}'", name))
                    }
                    Some(BorrowState::Moved) => {
                        Err(format!("Use of moved value '{}'", name))
                    }
                    None => {
                        Err(format!("Use of undeclared variable '{}'", name))
                    }
                }
            }
            _ => Ok(()),
        }
    }
    
    fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashSet::new(),
        });
    }
    
    fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for var in &scope.variables {
                self.borrows.remove(var);
            }
        }
    }
    
    pub fn borrow_immutable(&mut self, name: &str) -> Result<(), String> {
        match self.borrows.get(name) {
            Some(BorrowState::Owned) => {
                let count = 1;
                self.borrows.insert(name.to_string(), BorrowState::Borrowed(BorrowKind::Immutable(count)));
                Ok(())
            }
            Some(BorrowState::Borrowed(BorrowKind::Immutable(count))) => {
                self.borrows.insert(name.to_string(), BorrowState::Borrowed(BorrowKind::Immutable(count + 1)));
                Ok(())
            }
            Some(BorrowState::Borrowed(BorrowKind::Mutable(_))) => {
                Err(format!("Cannot borrow '{}' as immutable because it is already borrowed as mutable", name))
            }
            Some(BorrowState::Moved) => {
                Err(format!("Use of moved value '{}'", name))
            }
            None => {
                Err(format!("Use of undeclared variable '{}'", name))
            }
        }
    }
    
    pub fn borrow_mutable(&mut self, name: &str) -> Result<(), String> {
        match self.borrows.get(name) {
            Some(BorrowState::Owned) => {
                self.borrows.insert(name.to_string(), BorrowState::Borrowed(BorrowKind::Mutable(1)));
                Ok(())
            }
            Some(BorrowState::Borrowed(_)) => {
                Err(format!("Cannot borrow '{}' as mutable because it is already borrowed", name))
            }
            Some(BorrowState::Moved) => {
                Err(format!("Use of moved value '{}'", name))
            }
            None => {
                Err(format!("Use of undeclared variable '{}'", name))
            }
        }
    }
    
    pub fn move_value(&mut self, name: &str) -> Result<(), String> {
        match self.borrows.get(name) {
            Some(BorrowState::Owned) => {
                self.borrows.insert(name.to_string(), BorrowState::Moved);
                Ok(())
            }
            Some(BorrowState::Borrowed(_)) => {
                Err(format!("Cannot move '{}' because it is borrowed", name))
            }
            Some(BorrowState::Moved) => {
                Err(format!("Use of moved value '{}'", name))
            }
            None => {
                Err(format!("Use of undeclared variable '{}'", name))
            }
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}
