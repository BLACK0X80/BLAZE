use std::collections::HashMap;
use crate::parser::{Program, Item, Function, Statement, Expression, Type};

#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub scope_depth: usize,
    pub is_imported: bool,
}

pub struct ScopeResolver {
    scopes: Vec<HashMap<String, ResolvedSymbol>>,
    imports: HashMap<String, String>,
    current_depth: usize,
}

impl ScopeResolver {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            imports: HashMap::new(),
            current_depth: 0,
        }
    }

    pub fn resolve(&mut self, program: &Program) -> anyhow::Result<()> {
        for item in &program.items {
            self.resolve_item(item)?;
        }
        Ok(())
    }

    fn resolve_item(&mut self, item: &Item) -> anyhow::Result<()> {
        match item {
            Item::Function(func) => {
                self.resolve_function(func)?;
            }
            Item::Const(const_def) => {
                self.add_symbol(const_def.name.clone(), const_def.ty.clone(), false, false)?;
            }
            Item::Static(static_def) => {
                self.add_symbol(static_def.name.clone(), static_def.ty.clone(), false, false)?;
            }
            Item::Struct(struct_def) => {
                self.add_symbol(struct_def.name.clone(), Type::Identifier(struct_def.name.clone()), false, false)?;
            }
            Item::Enum(enum_def) => {
                self.add_symbol(enum_def.name.clone(), Type::Identifier(enum_def.name.clone()), false, false)?;
            }
            Item::Use(use_stmt) => {
                self.resolve_use(use_stmt)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_function(&mut self, func: &Function) -> anyhow::Result<()> {
        self.enter_scope();
        
        for param in &func.params {
            self.add_symbol(param.name.clone(), param.ty.clone(), false, false)?;
        }
        
        for stmt in &func.body {
            self.resolve_statement(stmt)?;
        }
        
        self.exit_scope();
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Statement) -> anyhow::Result<()> {
        match stmt {
            Statement::Let { name, ty, mutable, .. } => {
                if let Some(ty) = ty {
                    self.add_symbol(name.clone(), ty.clone(), *mutable, false)?;
                }
            }
            Statement::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.resolve_statement(stmt)?;
                }
                self.exit_scope();
            }
            Statement::Expression(expr) => {
                self.resolve_expression(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_expression(&mut self, expr: &Expression) -> anyhow::Result<()> {
        match expr {
            Expression::Identifier(name) => {
                self.resolve_identifier(name)?;
            }
            Expression::Call { callee, args } => {
                self.resolve_expression(callee)?;
                for arg in args {
                    self.resolve_expression(arg)?;
                }
            }
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.resolve_expression(operand)?;
            }
            Expression::Index { object, index } => {
                self.resolve_expression(object)?;
                self.resolve_expression(index)?;
            }
            Expression::FieldAccess { object, .. } => {
                self.resolve_expression(object)?;
            }
            Expression::MethodCall { object, .. } => {
                self.resolve_expression(object)?;
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.resolve_expression(condition)?;
                self.resolve_expression(then_branch)?;
                if let Some(else_expr) = else_branch {
                    self.resolve_expression(else_expr)?;
                }
            }
            Expression::Match { expression, arms } => {
                self.resolve_expression(expression)?;
                for arm in arms {
                    self.resolve_expression(&arm.body)?;
                }
            }
            Expression::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.resolve_statement(stmt)?;
                }
                self.exit_scope();
            }
            Expression::Assignment { target, value } => {
                self.resolve_expression(target)?;
                self.resolve_expression(value)?;
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.resolve_expression(&field.value)?;
                }
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.resolve_expression(element)?;
                }
            }
            Expression::TupleLiteral(elements) => {
                for element in elements {
                    self.resolve_expression(element)?;
                }
            }
            Expression::Closure { body, .. } => {
                self.resolve_expression(body)?;
            }
            Expression::Reference { expression, .. } => {
                self.resolve_expression(expression)?;
            }
            Expression::Dereference(expr) => {
                self.resolve_expression(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_identifier(&self, name: &str) -> anyhow::Result<()> {
        if self.lookup_symbol(name).is_none() {
            return Err(anyhow::anyhow!("Undefined symbol: {}", name));
        }
        Ok(())
    }

    fn resolve_use(&mut self, use_stmt: &crate::parser::Use) -> anyhow::Result<()> {
        let path = use_stmt.path.join("::");
        let last_component = use_stmt.path.last()
            .ok_or_else(|| anyhow::anyhow!("Empty use path"))?;
        
        self.imports.insert(last_component.clone(), path);
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.current_depth += 1;
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if self.current_depth > 0 {
            self.scopes.pop();
            self.current_depth -= 1;
        }
    }

    fn add_symbol(&mut self, name: String, ty: Type, mutable: bool, is_imported: bool) -> anyhow::Result<()> {
        let symbol = ResolvedSymbol {
            name: name.clone(),
            ty,
            mutable,
            scope_depth: self.current_depth,
            is_imported,
        };
        
        if self.scopes[self.current_depth].contains_key(&name) {
            return Err(anyhow::anyhow!("Symbol '{}' already defined in this scope", name));
        }
        
        self.scopes[self.current_depth].insert(name, symbol);
        Ok(())
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<&ResolvedSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn is_imported(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    pub fn get_import_path(&self, name: &str) -> Option<&String> {
        self.imports.get(name)
    }
}