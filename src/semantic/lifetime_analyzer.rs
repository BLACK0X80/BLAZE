use std::collections::{HashMap, HashSet};
use crate::parser::{Program, Statement, Expression, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub id: usize,
    pub name: String,
    pub scope: Scope,
    pub outlives: HashSet<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub start: usize,
    pub end: usize,
    pub variables: HashSet<String>,
}

pub struct LifetimeAnalyzer {
    lifetimes: HashMap<usize, Lifetime>,
    next_lifetime_id: usize,
    variable_lifetimes: HashMap<String, usize>,
}

impl LifetimeAnalyzer {
    pub fn new() -> Self {
        Self {
            lifetimes: HashMap::new(),
            next_lifetime_id: 0,
            variable_lifetimes: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> anyhow::Result<()> {
        for item in &program.items {
            if let crate::parser::Item::Function(func) = item {
                self.analyze_function(func)?;
            }
        }
        Ok(())
    }

    fn analyze_function(&mut self, func: &crate::parser::Function) -> anyhow::Result<()> {
        let mut scope_stack = Vec::new();
        scope_stack.push(Scope {
            start: 0,
            end: func.body.len(),
            variables: func.params.iter().map(|p| p.name.clone()).collect(),
        });

        for (i, stmt) in func.body.iter().enumerate() {
            self.analyze_statement(stmt, i, &mut scope_stack)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement, position: usize, scope_stack: &mut Vec<Scope>) -> anyhow::Result<()> {
        match stmt {
            Statement::Let { name, ty, .. } => {
                let lifetime = self.create_lifetime(name.clone(), position, scope_stack.len());
                self.variable_lifetimes.insert(name.clone(), lifetime.id);
                
                if let Some(ty) = ty {
                    self.analyze_type_lifetimes(ty, &lifetime)?;
                }
            }
            Statement::Block(stmts) => {
                scope_stack.push(Scope {
                    start: position,
                    end: position + stmts.len(),
                    variables: HashSet::new(),
                });
                
                for (i, stmt) in stmts.iter().enumerate() {
                    self.analyze_statement(stmt, position + i, scope_stack)?;
                }
                
                scope_stack.pop();
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr, position)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression, position: usize) -> anyhow::Result<()> {
        match expr {
            Expression::Reference { expression, .. } => {
                self.analyze_expression(expression, position)?;
            }
            Expression::Dereference(expr) => {
                self.analyze_expression(expr, position)?;
            }
            Expression::Call { callee, args } => {
                self.analyze_expression(callee, position)?;
                for arg in args {
                    self.analyze_expression(arg, position)?;
                }
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left, position)?;
                self.analyze_expression(right, position)?;
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expression(operand, position)?;
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.analyze_expression(condition, position)?;
                self.analyze_expression(then_branch, position)?;
                if let Some(else_expr) = else_branch {
                    self.analyze_expression(else_expr, position)?;
                }
            }
            Expression::Match { expression, arms } => {
                self.analyze_expression(expression, position)?;
                for arm in arms {
                    self.analyze_expression(&arm.body, position)?;
                }
            }
            Expression::Block(stmts) => {
                for stmt in stmts {
                    self.analyze_statement(stmt, position)?;
                }
            }
            Expression::Assignment { target, value } => {
                self.analyze_expression(target, position)?;
                self.analyze_expression(value, position)?;
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.analyze_expression(&field.value, position)?;
                }
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.analyze_expression(element, position)?;
                }
            }
            Expression::TupleLiteral(elements) => {
                for element in elements {
                    self.analyze_expression(element, position)?;
                }
            }
            Expression::Closure { body, .. } => {
                self.analyze_expression(body, position)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_type_lifetimes(&mut self, ty: &Type, lifetime: &Lifetime) -> anyhow::Result<()> {
        match ty {
            Type::Reference { inner, .. } => {
                self.analyze_type_lifetimes(inner, lifetime)?;
            }
            Type::Pointer { inner, .. } => {
                self.analyze_type_lifetimes(inner, lifetime)?;
            }
            Type::Array { element_type, .. } => {
                self.analyze_type_lifetimes(element_type, lifetime)?;
            }
            Type::Slice(element_type) => {
                self.analyze_type_lifetimes(element_type, lifetime)?;
            }
            Type::Tuple(types) => {
                for ty in types {
                    self.analyze_type_lifetimes(ty, lifetime)?;
                }
            }
            Type::Function { params, return_type } => {
                for ty in params {
                    self.analyze_type_lifetimes(ty, lifetime)?;
                }
                self.analyze_type_lifetimes(return_type, lifetime)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn create_lifetime(&mut self, name: String, start: usize, scope_level: usize) -> Lifetime {
        let lifetime = Lifetime {
            id: self.next_lifetime_id,
            name,
            scope: Scope {
                start,
                end: start + 1,
                variables: HashSet::new(),
            },
            outlives: HashSet::new(),
        };
        
        self.next_lifetime_id += 1;
        self.lifetimes.insert(lifetime.id, lifetime.clone());
        lifetime
    }

    pub fn get_lifetime(&self, variable: &str) -> Option<&Lifetime> {
        self.variable_lifetimes.get(variable)
            .and_then(|id| self.lifetimes.get(id))
    }

    pub fn add_outlives_relation(&mut self, longer: usize, shorter: usize) {
        if let Some(lifetime) = self.lifetimes.get_mut(&longer) {
            lifetime.outlives.insert(shorter);
        }
    }

    pub fn check_lifetime_validity(&self) -> anyhow::Result<()> {
        for (id, lifetime) in &self.lifetimes {
            for &outlives_id in &lifetime.outlives {
                if let Some(outlives_lifetime) = self.lifetimes.get(&outlives_id) {
                    if lifetime.scope.end <= outlives_lifetime.scope.start {
                        return Err(anyhow::anyhow!(
                            "Lifetime '{}' outlives '{}' but ends before it starts",
                            lifetime.name, outlives_lifetime.name
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}