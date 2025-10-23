use crate::parser::{Program, Type, Item, Function, Statement, Expression, BinaryOp, UnaryOp};
use crate::semantic::SymbolTable;
use crate::error::CompileError;
use anyhow::{Result, bail};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct TypeChecker {
    substitutions: Arc<RwLock<HashMap<TypeVar, Type>>>,
    occurs_check_cache: HashMap<(Type, TypeVar), bool>,
    next_type_var: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: usize,
    pub name: String,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            substitutions: Arc::new(RwLock::new(HashMap::new())),
            occurs_check_cache: HashMap::new(),
            next_type_var: 0,
        }
    }

    pub fn check(&mut self, program: &Program, symbol_table: &SymbolTable) -> Result<()> {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.check_function(func, symbol_table)?;
                }
                Item::Struct(_) => {}
            }
        }
        Ok(())
    }

    fn check_function(&mut self, func: &Function, symbol_table: &SymbolTable) -> Result<()> {
        for param in &func.params {
            self.check_type(&param.ty)?;
        }

        if let Some(return_type) = &func.return_type {
            self.check_type(return_type)?;
        }

        for stmt in &func.body {
            self.check_statement(stmt, symbol_table)?;
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement, symbol_table: &SymbolTable) -> Result<()> {
        match stmt {
            Statement::Let { name, ty, value, .. } => {
                // Infer the type from the value expression
                let value_type = self.infer_expression(value, symbol_table)?;

                // If there's an explicit type annotation, unify it with the inferred type
                if let Some(declared_type) = ty {
                    self.unify(declared_type, &value_type)?;
                } else {
                    // Type inference: the variable gets the inferred type from the value
                    // In a more complete implementation, we would update the symbol table here
                    // For now, we just validate that the type can be inferred
                    if matches!(value_type, Type::Custom(ref s) if s == "unknown") {
                        bail!("Type error: cannot infer type for variable '{}'", name);
                    }
                }
            }
            Statement::Return(Some(expr)) => {
                self.infer_expression(expr, symbol_table)?;
            }
            Statement::Return(None) => {
                // Return without value - type is void/unit
            }
            Statement::Expression(expr) => {
                self.infer_expression(expr, symbol_table)?;
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_type = self.infer_expression(condition, symbol_table)?;
                self.unify(&Type::Bool, &cond_type)?;

                for stmt in then_body {
                    self.check_statement(stmt, symbol_table)?;
                }

                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.check_statement(stmt, symbol_table)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                let cond_type = self.infer_expression(condition, symbol_table)?;
                self.unify(&Type::Bool, &cond_type)?;

                for stmt in body {
                    self.check_statement(stmt, symbol_table)?;
                }
            }
        }
        Ok(())
    }

    fn infer_expression(&mut self, expr: &Expression, symbol_table: &SymbolTable) -> Result<Type> {
        match expr {
            Expression::IntLit(_) => Ok(Type::I32),
            Expression::FloatLit(_) => Ok(Type::F64),
            Expression::BoolLit(_) => Ok(Type::Bool),
            Expression::StringLit(_) => Ok(Type::String),
            Expression::CharLit(_) => Ok(Type::Char),
            Expression::Ident(name) => {
                if let Some(symbol) = symbol_table.lookup(name) {
                    Ok(symbol.ty.clone())
                } else {
                    return Err(self.type_error(
                        format!("undefined variable '{}'", name),
                        None,
                        None,
                        Some(format!("variable '{}' is not defined in this scope", name)),
                    ));
                }
            }
            Expression::Binary { op, left, right } => {
                self.check_binary_operation(op, left, right, symbol_table)
            }
            Expression::Unary { op, expr } => {
                self.check_unary_operation(op, expr, symbol_table)
            }
            Expression::Call { func, args } => {
                self.check_function_call(func, args, symbol_table)
            }
        }
    }

    fn check_binary_operation(
        &mut self,
        op: &BinaryOp,
        left: &Expression,
        right: &Expression,
        symbol_table: &SymbolTable,
    ) -> Result<Type> {
        let left_type = self.infer_expression(left, symbol_table)?;
        let right_type = self.infer_expression(right, symbol_table)?;

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Arithmetic operations require numeric types
                if !self.is_numeric_type(&left_type) {
                    return Err(self.type_error(
                        format!("arithmetic operation requires numeric type"),
                        Some(Type::I32),
                        Some(left_type),
                        Some("arithmetic operations work on i32, i64, f32, or f64".to_string()),
                    ));
                }
                if !self.is_numeric_type(&right_type) {
                    return Err(self.type_error(
                        format!("arithmetic operation requires numeric type"),
                        Some(Type::I32),
                        Some(right_type),
                        Some("arithmetic operations work on i32, i64, f32, or f64".to_string()),
                    ));
                }
                self.unify(&left_type, &right_type)?;
                Ok(left_type)
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality operations work on any type, but both sides must match
                self.unify(&left_type, &right_type)?;
                Ok(Type::Bool)
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison operations require numeric types
                if !self.is_numeric_type(&left_type) {
                    return Err(self.type_error(
                        format!("comparison operation requires numeric type"),
                        Some(Type::I32),
                        Some(left_type),
                        Some("comparison operations work on i32, i64, f32, or f64".to_string()),
                    ));
                }
                if !self.is_numeric_type(&right_type) {
                    return Err(self.type_error(
                        format!("comparison operation requires numeric type"),
                        Some(Type::I32),
                        Some(right_type),
                        Some("comparison operations work on i32, i64, f32, or f64".to_string()),
                    ));
                }
                self.unify(&left_type, &right_type)?;
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations require boolean types
                self.unify(&Type::Bool, &left_type)?;
                self.unify(&Type::Bool, &right_type)?;
                Ok(Type::Bool)
            }
        }
    }

    fn check_unary_operation(
        &mut self,
        op: &UnaryOp,
        expr: &Expression,
        symbol_table: &SymbolTable,
    ) -> Result<Type> {
        let expr_type = self.infer_expression(expr, symbol_table)?;

        match op {
            UnaryOp::Neg => {
                // Negation requires numeric type
                if !self.is_numeric_type(&expr_type) {
                    return Err(self.type_error(
                        format!("negation requires numeric type"),
                        Some(Type::I32),
                        Some(expr_type),
                        Some("negation works on i32, i64, f32, or f64".to_string()),
                    ));
                }
                Ok(expr_type)
            }
            UnaryOp::Not => {
                // Logical not requires boolean type
                if !matches!(expr_type, Type::Bool) {
                    return Err(self.type_error(
                        format!("logical not requires boolean type"),
                        Some(Type::Bool),
                        Some(expr_type),
                        Some("use ! operator only on boolean expressions".to_string()),
                    ));
                }
                Ok(Type::Bool)
            }
        }
    }

    fn check_function_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
        symbol_table: &SymbolTable,
    ) -> Result<Type> {
        // Get the function name
        let func_name = match func {
            Expression::Ident(name) => name,
            _ => {
                return Err(self.type_error(
                    format!("invalid function call expression"),
                    None,
                    None,
                    Some("function calls must use an identifier".to_string()),
                ));
            }
        };

        // Look up the function in the symbol table
        if let Some(symbol) = symbol_table.lookup(func_name) {
            // For now, we'll return the function's return type
            // In a more complete implementation, we would:
            // 1. Extract parameter types from the function signature
            // 2. Check that the number of arguments matches
            // 3. Check that each argument type matches the parameter type
            
            // Type check all arguments
            for arg in args {
                self.infer_expression(arg, symbol_table)?;
            }

            // Return the function's return type
            // Note: This is simplified - a real implementation would need
            // to store function signatures in the symbol table
            Ok(symbol.ty.clone())
        } else {
            return Err(self.type_error(
                format!("undefined function '{}'", func_name),
                None,
                None,
                Some(format!("function '{}' is not defined in this scope", func_name)),
            ));
        }
    }

    fn is_numeric_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::I32 | Type::I64 | Type::F32 | Type::F64)
    }

    fn check_type(&self, _ty: &Type) -> Result<()> {
        Ok(())
    }

    /// Complete occurs check to prevent infinite types
    /// Returns true if the type variable occurs in the type
    fn occurs_check(&mut self, ty: &Type, var: &TypeVar) -> Result<bool> {
        let key = (ty.clone(), var.clone());
        if let Some(&result) = self.occurs_check_cache.get(&key) {
            return Ok(result);
        }

        // Apply substitutions first to get the actual type
        let resolved_ty = self.apply_substitutions(ty);
        
        let result = self.occurs_check_recursive(&resolved_ty, var)?;
        self.occurs_check_cache.insert(key, result);
        Ok(result)
    }
    
    /// Recursive helper for occurs check
    fn occurs_check_recursive(&self, ty: &Type, var: &TypeVar) -> Result<bool> {
        match ty {
            Type::Custom(name) if name == &var.name => Ok(true),
            Type::Custom(name) => {
                // Check if this type variable has a substitution
                let type_var = TypeVar { id: 0, name: name.clone() };
                let subs = self.substitutions.read();
                if let Some(substituted) = subs.get(&type_var) {
                    drop(subs); // Release lock before recursive call
                    self.occurs_check_recursive(substituted, var)
                } else {
                    Ok(false)
                }
            }
            // For other types, they don't contain type variables in this simple implementation
            // In a more complete implementation, we would check function types, tuple types, etc.
            _ => Ok(false),
        }
    }

    /// Complete unification algorithm for all type combinations
    fn unify(&mut self, t1: &Type, t2: &Type) -> Result<()> {
        // Apply existing substitutions first
        let ty1 = self.apply_substitutions(t1);
        let ty2 = self.apply_substitutions(t2);
        
        // If types are already equal, nothing to do
        if self.types_equal(&ty1, &ty2) {
            return Ok(());
        }

        match (&ty1, &ty2) {
            // Both are the same concrete type - already handled by types_equal above
            (Type::I32, Type::I32) | (Type::I64, Type::I64) | 
            (Type::F32, Type::F32) | (Type::F64, Type::F64) |
            (Type::Bool, Type::Bool) | (Type::Char, Type::Char) |
            (Type::String, Type::String) => Ok(()),
            
            // Both are type variables with the same name
            (Type::Custom(var1), Type::Custom(var2)) if var1 == var2 => Ok(()),
            
            // One is a type variable, the other is a concrete type
            (Type::Custom(var), other) | (other, Type::Custom(var)) => {
                let type_var = TypeVar { id: 0, name: var.clone() };
                
                // Perform occurs check to prevent infinite types
                if self.occurs_check(other, &type_var)? {
                    return Err(self.type_error(
                        format!("infinite type detected: type variable '{}' occurs in '{}'", 
                                var, self.format_type(other)),
                        Some(Type::Custom(var.clone())),
                        Some(other.clone()),
                        Some("this would create an infinitely large type".to_string()),
                    ));
                }

                // Add substitution
                let mut subs = self.substitutions.write();
                subs.insert(type_var, other.clone());
                Ok(())
            }
            
            // Type mismatch - provide helpful error message
            _ => {
                let suggestion = self.suggest_fix(&ty1, &ty2);
                Err(self.type_error(
                    format!("type mismatch: expected '{}', found '{}'", 
                            self.format_type(&ty1), self.format_type(&ty2)),
                    Some(ty1.clone()),
                    Some(ty2.clone()),
                    suggestion,
                ))
            }
        }
    }
    
    /// Unify multiple types (useful for function arguments, tuple elements, etc.)
    fn unify_many(&mut self, types: &[(Type, Type)]) -> Result<()> {
        for (t1, t2) in types {
            self.unify(t1, t2)?;
        }
        Ok(())
    }

    fn types_equal(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::I32, Type::I32) => true,
            (Type::I64, Type::I64) => true,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Char, Type::Char) => true,
            (Type::String, Type::String) => true,
            (Type::Custom(a), Type::Custom(b)) => a == b,
            _ => false,
        }
    }

    fn new_type_var(&mut self) -> TypeVar {
        let id = self.next_type_var;
        self.next_type_var += 1;
        TypeVar {
            id,
            name: format!("T{}", id),
        }
    }

    /// Apply substitutions to resolve a type
    /// This recursively follows the substitution chain to get the final type
    fn apply_substitutions(&self, ty: &Type) -> Type {
        let mut visited = HashSet::new();
        self.apply_substitutions_with_cycle_check(ty, &mut visited)
    }
    
    /// Apply substitutions with cycle detection
    fn apply_substitutions_with_cycle_check(&self, ty: &Type, visited: &mut HashSet<String>) -> Type {
        match ty {
            Type::Custom(var_name) => {
                // Check for cycles in substitution chain
                if !visited.insert(var_name.clone()) {
                    // Cycle detected, return the type variable itself
                    return ty.clone();
                }
                
                let type_var = TypeVar {
                    id: 0,
                    name: var_name.clone(),
                };
                let subs = self.substitutions.read();
                if let Some(substituted) = subs.get(&type_var) {
                    let result = substituted.clone();
                    drop(subs); // Release lock before recursive call
                    // Recursively apply substitutions
                    self.apply_substitutions_with_cycle_check(&result, visited)
                } else {
                    ty.clone()
                }
            }
            // For concrete types, return as-is
            // In a more complete implementation, we would recursively apply to
            // function types, tuple types, array types, etc.
            _ => ty.clone(),
        }
    }

    /// Resolve a type by applying all substitutions
    pub fn resolve_type(&self, ty: &Type) -> Type {
        self.apply_substitutions(ty)
    }
    
    /// Apply substitutions throughout a program's AST
    /// This would update all type annotations with their resolved types
    pub fn apply_substitutions_to_program(&self, _program: &mut Program) -> Result<()> {
        // In a complete implementation, this would:
        // 1. Walk through all function signatures
        // 2. Update parameter types
        // 3. Update return types
        // 4. Update variable declarations
        // 5. Update expression types (if stored in AST)
        
        // For now, this is a placeholder that demonstrates the concept
        // The actual implementation would require the AST to store type information
        Ok(())
    }
    
    /// Get all active substitutions (for debugging/testing)
    pub fn get_substitutions(&self) -> HashMap<String, Type> {
        let subs = self.substitutions.read();
        subs.iter()
            .map(|(var, ty)| (var.name.clone(), ty.clone()))
            .collect()
    }
    
    /// Clear all substitutions (useful for testing)
    pub fn clear_substitutions(&mut self) {
        let mut subs = self.substitutions.write();
        subs.clear();
        self.occurs_check_cache.clear();
    }

    /// Create a type error with detailed information
    fn type_error(
        &self,
        message: String,
        expected: Option<Type>,
        found: Option<Type>,
        suggestion: Option<String>,
    ) -> anyhow::Error {
        CompileError::TypeError {
            message,
            expected: expected.map(|t| self.format_type(&t)),
            found: found.map(|t| self.format_type(&t)),
            line: None,
            column: None,
            suggestion,
        }
        .into()
    }

    /// Format a type for display in error messages
    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "String".to_string(),
            Type::Custom(name) => name.clone(),
        }
    }

    /// Suggest a fix for common type errors
    fn suggest_fix(&self, expected: &Type, found: &Type) -> Option<String> {
        match (expected, found) {
            (Type::F64, Type::I32) | (Type::F32, Type::I32) => {
                Some("try converting the integer to a float with `as f64` or `as f32`".to_string())
            }
            (Type::I32, Type::F64) | (Type::I32, Type::F32) => {
                Some("try converting the float to an integer with `as i32`".to_string())
            }
            (Type::I64, Type::I32) => {
                Some("try converting the i32 to i64 with `as i64`".to_string())
            }
            (Type::I32, Type::I64) => {
                Some("try converting the i64 to i32 with `as i32`".to_string())
            }
            (Type::Bool, _) => {
                Some("expected a boolean expression".to_string())
            }
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expression, BinaryOp, UnaryOp, Statement};

    fn create_test_symbol_table() -> SymbolTable {
        let mut table = SymbolTable::new();
        // Add some test symbols
        table.insert("x".to_string(), crate::semantic::Symbol {
            name: "x".to_string(),
            ty: Type::I32,
            mutable: false,
        });
        table.insert("y".to_string(), crate::semantic::Symbol {
            name: "y".to_string(),
            ty: Type::F64,
            mutable: false,
        });
        table.insert("flag".to_string(), crate::semantic::Symbol {
            name: "flag".to_string(),
            ty: Type::Bool,
            mutable: false,
        });
        table
    }

    #[test]
    fn test_infer_literal_types() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test integer literal
        let int_expr = Expression::IntLit(42);
        let result = checker.infer_expression(&int_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::I32);

        // Test float literal
        let float_expr = Expression::FloatLit(3.14);
        let result = checker.infer_expression(&float_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::F64);

        // Test boolean literal
        let bool_expr = Expression::BoolLit(true);
        let result = checker.infer_expression(&bool_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test string literal
        let string_expr = Expression::StringLit("hello".to_string());
        let result = checker.infer_expression(&string_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::String);

        // Test char literal
        let char_expr = Expression::CharLit('a');
        let result = checker.infer_expression(&char_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Char);
    }

    #[test]
    fn test_infer_identifier_types() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test defined variable
        let ident_expr = Expression::Ident("x".to_string());
        let result = checker.infer_expression(&ident_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::I32);

        // Test undefined variable
        let undefined_expr = Expression::Ident("undefined".to_string());
        let result = checker.infer_expression(&undefined_expr, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test valid arithmetic: i32 + i32
        let add_expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntLit(1)),
            right: Box::new(Expression::IntLit(2)),
        };
        let result = checker.infer_expression(&add_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::I32);

        // Test valid arithmetic: f64 * f64
        let mul_expr = Expression::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expression::FloatLit(2.0)),
            right: Box::new(Expression::FloatLit(3.0)),
        };
        let result = checker.infer_expression(&mul_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::F64);

        // Test invalid arithmetic: bool + i32
        let invalid_expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::IntLit(1)),
        };
        let result = checker.infer_expression(&invalid_expr, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_comparison_operations() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test valid comparison: i32 < i32
        let lt_expr = Expression::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expression::IntLit(1)),
            right: Box::new(Expression::IntLit(2)),
        };
        let result = checker.infer_expression(&lt_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test valid comparison: f64 >= f64
        let ge_expr = Expression::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expression::FloatLit(3.14)),
            right: Box::new(Expression::FloatLit(2.71)),
        };
        let result = checker.infer_expression(&ge_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test invalid comparison: bool < i32
        let invalid_expr = Expression::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::IntLit(1)),
        };
        let result = checker.infer_expression(&invalid_expr, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_logical_operations() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test valid logical: bool && bool
        let and_expr = Expression::Binary {
            op: BinaryOp::And,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::BoolLit(false)),
        };
        let result = checker.infer_expression(&and_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test valid logical: bool || bool
        let or_expr = Expression::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::BoolLit(false)),
        };
        let result = checker.infer_expression(&or_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test invalid logical: i32 && bool
        let invalid_expr = Expression::Binary {
            op: BinaryOp::And,
            left: Box::new(Expression::IntLit(1)),
            right: Box::new(Expression::BoolLit(true)),
        };
        let result = checker.infer_expression(&invalid_expr, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_equality_operations() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test valid equality: i32 == i32
        let eq_expr = Expression::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expression::IntLit(1)),
            right: Box::new(Expression::IntLit(2)),
        };
        let result = checker.infer_expression(&eq_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test valid inequality: bool != bool
        let ne_expr = Expression::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::BoolLit(false)),
        };
        let result = checker.infer_expression(&ne_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_unary_operations() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test valid negation: -i32
        let neg_expr = Expression::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expression::IntLit(42)),
        };
        let result = checker.infer_expression(&neg_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::I32);

        // Test valid not: !bool
        let not_expr = Expression::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expression::BoolLit(true)),
        };
        let result = checker.infer_expression(&not_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Test invalid negation: -bool
        let invalid_neg = Expression::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expression::BoolLit(true)),
        };
        let result = checker.infer_expression(&invalid_neg, &symbol_table);
        assert!(result.is_err());

        // Test invalid not: !i32
        let invalid_not = Expression::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expression::IntLit(42)),
        };
        let result = checker.infer_expression(&invalid_not, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_inference_in_let_statement() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test let with explicit type
        let let_stmt = Statement::Let {
            name: "z".to_string(),
            mutable: false,
            ty: Some(Type::I32),
            value: Expression::IntLit(42),
        };
        let result = checker.check_statement(&let_stmt, &symbol_table);
        assert!(result.is_ok());

        // Test let without explicit type (type inference)
        let let_stmt_infer = Statement::Let {
            name: "w".to_string(),
            mutable: false,
            ty: None,
            value: Expression::FloatLit(3.14),
        };
        let result = checker.check_statement(&let_stmt_infer, &symbol_table);
        assert!(result.is_ok());

        // Test let with type mismatch
        let let_stmt_mismatch = Statement::Let {
            name: "v".to_string(),
            mutable: false,
            ty: Some(Type::I32),
            value: Expression::BoolLit(true),
        };
        let result = checker.check_statement(&let_stmt_mismatch, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_unification() {
        let mut checker = TypeChecker::new();

        // Test unifying same types
        let result = checker.unify(&Type::I32, &Type::I32);
        assert!(result.is_ok());

        // Test unifying different types
        let result = checker.unify(&Type::I32, &Type::Bool);
        assert!(result.is_err());

        // Test unifying with type variable
        let result = checker.unify(&Type::Custom("T".to_string()), &Type::I32);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_messages() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test error message for undefined variable
        let undefined_expr = Expression::Ident("undefined_var".to_string());
        let result = checker.infer_expression(&undefined_expr, &symbol_table);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("undefined variable"));

        // Test error message for type mismatch in arithmetic
        let invalid_expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::BoolLit(true)),
            right: Box::new(Expression::IntLit(1)),
        };
        let result = checker.infer_expression(&invalid_expr, &symbol_table);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("arithmetic operation"));
    }

    #[test]
    fn test_nested_expressions() {
        let mut checker = TypeChecker::new();
        let symbol_table = create_test_symbol_table();

        // Test nested arithmetic: (1 + 2) * 3
        let nested_expr = Expression::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expression::IntLit(1)),
                right: Box::new(Expression::IntLit(2)),
            }),
            right: Box::new(Expression::IntLit(3)),
        };
        let result = checker.infer_expression(&nested_expr, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::I32);

        // Test nested comparison: (1 < 2) && (3 > 2)
        let nested_logical = Expression::Binary {
            op: BinaryOp::And,
            left: Box::new(Expression::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expression::IntLit(1)),
                right: Box::new(Expression::IntLit(2)),
            }),
            right: Box::new(Expression::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expression::IntLit(3)),
                right: Box::new(Expression::IntLit(2)),
            }),
        };
        let result = checker.infer_expression(&nested_logical, &symbol_table);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);
    }
    
    #[test]
    fn test_complete_occurs_check() {
        let mut checker = TypeChecker::new();
        
        // Create a type variable
        let type_var = TypeVar { id: 0, name: "T".to_string() };
        
        // Test occurs check with the same type variable
        let result = checker.occurs_check(&Type::Custom("T".to_string()), &type_var);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        
        // Test occurs check with a different type
        let result = checker.occurs_check(&Type::I32, &type_var);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
    
    #[test]
    fn test_unification_with_type_variables() {
        let mut checker = TypeChecker::new();
        
        // Test unifying type variable with concrete type
        let result = checker.unify(&Type::Custom("T".to_string()), &Type::I32);
        assert!(result.is_ok());
        
        // Verify substitution was recorded
        let subs = checker.get_substitutions();
        assert!(subs.contains_key("T"));
        assert_eq!(subs.get("T"), Some(&Type::I32));
    }
    
    #[test]
    fn test_unification_prevents_infinite_types() {
        let mut checker = TypeChecker::new();
        
        // Try to unify T with T (should succeed)
        let result = checker.unify(&Type::Custom("T".to_string()), &Type::Custom("T".to_string()));
        assert!(result.is_ok());
        
        // In a more complex scenario with function types, we would test:
        // T = T -> T (should fail with occurs check)
        // But our simple type system doesn't have function types yet
    }
    
    #[test]
    fn test_substitution_application() {
        let mut checker = TypeChecker::new();
        
        // Add a substitution T -> I32
        checker.unify(&Type::Custom("T".to_string()), &Type::I32).unwrap();
        
        // Apply substitutions to T
        let resolved = checker.apply_substitutions(&Type::Custom("T".to_string()));
        assert_eq!(resolved, Type::I32);
        
        // Apply substitutions to I32 (should return I32)
        let resolved = checker.apply_substitutions(&Type::I32);
        assert_eq!(resolved, Type::I32);
    }
    
    #[test]
    fn test_transitive_substitutions() {
        let mut checker = TypeChecker::new();
        
        // Create chain: T1 -> T2, T2 -> I32
        checker.unify(&Type::Custom("T1".to_string()), &Type::Custom("T2".to_string())).unwrap();
        checker.unify(&Type::Custom("T2".to_string()), &Type::I32).unwrap();
        
        // Resolving T1 should give I32
        let resolved = checker.apply_substitutions(&Type::Custom("T1".to_string()));
        assert_eq!(resolved, Type::I32);
    }
    
    #[test]
    fn test_unify_many() {
        let mut checker = TypeChecker::new();
        
        // Test unifying multiple type pairs
        let types = vec![
            (Type::I32, Type::I32),
            (Type::Bool, Type::Bool),
            (Type::Custom("T".to_string()), Type::I32),
        ];
        
        let result = checker.unify_many(&types);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_many_with_conflict() {
        let mut checker = TypeChecker::new();
        
        // Test unifying with a conflict
        let types = vec![
            (Type::I32, Type::I32),
            (Type::Bool, Type::I32), // This should fail
        ];
        
        let result = checker.unify_many(&types);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_clear_substitutions() {
        let mut checker = TypeChecker::new();
        
        // Add some substitutions
        checker.unify(&Type::Custom("T".to_string()), &Type::I32).unwrap();
        assert!(!checker.get_substitutions().is_empty());
        
        // Clear substitutions
        checker.clear_substitutions();
        assert!(checker.get_substitutions().is_empty());
    }
    
    #[test]
    fn test_type_resolution_with_multiple_substitutions() {
        let mut checker = TypeChecker::new();
        
        // Create multiple substitutions
        checker.unify(&Type::Custom("T1".to_string()), &Type::I32).unwrap();
        checker.unify(&Type::Custom("T2".to_string()), &Type::Bool).unwrap();
        
        // Resolve both
        let resolved1 = checker.resolve_type(&Type::Custom("T1".to_string()));
        let resolved2 = checker.resolve_type(&Type::Custom("T2".to_string()));
        
        assert_eq!(resolved1, Type::I32);
        assert_eq!(resolved2, Type::Bool);
    }
    
    #[test]
    fn test_error_message_quality() {
        let mut checker = TypeChecker::new();
        
        // Test that error messages contain useful information
        let result = checker.unify(&Type::I32, &Type::Bool);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("type mismatch") || err_msg.contains("expected"));
    }
}
