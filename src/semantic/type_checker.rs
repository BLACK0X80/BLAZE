use std::collections::{HashMap, HashSet, VecDeque};
use crate::parser::{Program, Item, Function, Statement, Expression, Type, BinaryOperator, UnaryOperator, Literal, PrimitiveType};
use crate::semantic::SymbolTable;
use crate::error::TypeError;
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeVar {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Equality(Type, Type),
    Subtype(Type, Type),
    Trait(Type, String),
}

#[derive(Debug, Clone)]
pub struct Substitution {
    pub var: TypeVar,
    pub ty: Type,
}

pub struct TypeChecker {
    type_vars: HashMap<String, TypeVar>,
    next_type_var: usize,
    substitutions: RwLock<Vec<Substitution>>,
    type_cache: DashMap<Expression, Type>,
    trait_impls: DashMap<(Type, String), bool>,
    constraints: VecDeque<TypeConstraint>,
    occurs_check_cache: DashMap<(Type, TypeVar), bool>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_vars: HashMap::new(),
            next_type_var: 0,
            substitutions: RwLock::new(Vec::new()),
            type_cache: DashMap::new(),
            trait_impls: DashMap::new(),
            constraints: VecDeque::new(),
            occurs_check_cache: DashMap::new(),
        }
    }

    pub fn check(&mut self, program: &Program, symbol_table: &SymbolTable) -> Result<()> {
        for item in &program.items {
            self.check_item(item, symbol_table)?;
        }
        
        self.solve_constraints()?;
        Ok(())
    }

    fn check_item(&mut self, item: &Item, symbol_table: &SymbolTable) -> Result<()> {
        match item {
            Item::Function(func) => self.check_function(func, symbol_table),
            Item::Const(const_def) => {
                let expr_type = self.check_expression(&const_def.value, symbol_table)?;
                self.unify(&const_def.ty, &expr_type)?;
                Ok(())
            }
            Item::Static(static_def) => {
                let expr_type = self.check_expression(&static_def.value, symbol_table)?;
                self.unify(&static_def.ty, &expr_type)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_function(&mut self, func: &Function, symbol_table: &SymbolTable) -> Result<()> {
        for stmt in &func.body {
            self.check_statement(stmt, symbol_table)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement, symbol_table: &SymbolTable) -> Result<()> {
        match stmt {
            Statement::Let { name: _, ty, value, mutable: _ } => {
                if let Some(value_expr) = value {
                    let value_type = self.check_expression(value_expr, symbol_table)?;
                    if let Some(declared_type) = ty {
                        self.unify(declared_type, &value_type)?;
                    }
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.check_expression(expr, symbol_table)?;
                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expression(expr, symbol_table)?;
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                let cond_type = self.check_expression(condition, symbol_table)?;
                self.unify(&Type::Primitive(PrimitiveType::Bool), &cond_type)?;
                for stmt in body {
                    self.check_statement(stmt, symbol_table)?;
                }
                Ok(())
            }
            Statement::For { variable: _, iterable, body } => {
                self.check_expression(iterable, symbol_table)?;
                for stmt in body {
                    self.check_statement(stmt, symbol_table)?;
                }
                Ok(())
            }
            Statement::Loop { body } => {
                for stmt in body {
                    self.check_statement(stmt, symbol_table)?;
                }
                Ok(())
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt, symbol_table)?;
                }
                Ok(())
            }
            Statement::Break(_) | Statement::Continue => Ok(()),
        }
    }

    fn check_expression(&mut self, expr: &Expression, symbol_table: &SymbolTable) -> Result<Type> {
        if let Some(cached_type) = self.type_cache.get(expr) {
            return Ok(cached_type.clone());
        }

        let expr_type = match expr {
            Expression::Literal(lit) => self.literal_type(lit),
            Expression::Identifier(name) => {
                if let Some(symbol) = symbol_table.lookup_variable(name) {
                    symbol.ty.clone()
                } else {
                    return Err(TypeError::UndefinedVariable {
                        name: name.clone(),
                    }.into());
                }
            }
            Expression::Binary { left, operator, right } => {
                let left_type = self.check_expression(left, symbol_table)?;
                let right_type = self.check_expression(right, symbol_table)?;
                self.check_binary_op(&left_type, operator, &right_type)?
            }
            Expression::Unary { operator, operand } => {
                let operand_type = self.check_expression(operand, symbol_table)?;
                self.check_unary_op(operator, &operand_type)?
            }
            Expression::Call { callee, args } => {
                let callee_type = self.check_expression(callee, symbol_table)?;
                let arg_types: Result<Vec<Type>> = args.iter()
                    .map(|arg| self.check_expression(arg, symbol_table))
                    .collect();
                let arg_types = arg_types?;
                
                match callee_type {
                    Type::Function { params, return_type } => {
                        if params.len() != arg_types.len() {
                            return Err(TypeError::ArgCountMismatch.into());
                        }
                        
                        for (param_type, arg_type) in params.iter().zip(arg_types.iter()) {
                            self.unify(param_type, arg_type)?;
                        }
                        
                        *return_type
                    }
                    _ => return Err(TypeError::NotCallable.into()),
                }
            }
            Expression::Index { object, index } => {
                let object_type = self.check_expression(object, symbol_table)?;
                let index_type = self.check_expression(index, symbol_table)?;
                
                self.unify(&Type::Primitive(PrimitiveType::U64), &index_type)?;
                
                match object_type {
                    Type::Array { element_type, size: _ } => *element_type,
                    Type::Slice(element_type) => *element_type,
                    _ => return Err(TypeError::InvalidBinaryOp.into()),
                }
            }
            Expression::FieldAccess { object, field: _ } => {
                self.check_expression(object, symbol_table)?
            }
            Expression::MethodCall { object, method: _, args } => {
                self.check_expression(object, symbol_table)?;
                for arg in args {
                    self.check_expression(arg, symbol_table)?;
                }
                Type::Unit
            }
            Expression::If { condition, then_branch, else_branch } => {
                let cond_type = self.check_expression(condition, symbol_table)?;
                self.unify(&Type::Primitive(PrimitiveType::Bool), &cond_type)?;
                
                let then_type = self.check_expression(then_branch, symbol_table)?;
                
                if let Some(else_expr) = else_branch {
                    let else_type = self.check_expression(else_expr, symbol_table)?;
                    self.unify(&then_type, &else_type)?;
                    then_type
                } else {
                    Type::Unit
                }
            }
            Expression::Match { expression, arms } => {
                self.check_expression(expression, symbol_table)?;
                
                if arms.is_empty() {
                    return Ok(Type::Unit);
                }
                
                let first_arm_type = self.check_expression(&arms[0].body, symbol_table)?;
                
                for arm in &arms[1..] {
                    let arm_type = self.check_expression(&arm.body, symbol_table)?;
                    self.unify(&first_arm_type, &arm_type)?;
                }
                
                first_arm_type
            }
            Expression::Block(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt, symbol_table)?;
                }
                Type::Unit
            }
            Expression::Assignment { target: _, value } => {
                self.check_expression(value, symbol_table)?;
                Type::Unit
            }
            Expression::StructLiteral { name: _, fields } => {
                for field in fields {
                    self.check_expression(&field.value, symbol_table)?;
                }
                Type::Unit
            }
            Expression::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Ok(Type::Array {
                        element_type: Box::new(self.fresh_type_var()),
                        size: Some(Expression::Literal(Literal::Integer(0))),
                    });
                }
                
                let first_type = self.check_expression(&elements[0], symbol_table)?;
                for element in &elements[1..] {
                    let element_type = self.check_expression(element, symbol_table)?;
                    self.unify(&first_type, &element_type)?;
                }
                
                Type::Array {
                    element_type: Box::new(first_type),
                    size: Some(Expression::Literal(Literal::Integer(elements.len() as i64))),
                }
            }
            Expression::TupleLiteral(elements) => {
                let types: Result<Vec<Type>> = elements.iter()
                    .map(|elem| self.check_expression(elem, symbol_table))
                    .collect();
                Type::Tuple(types?)
            }
            Expression::Closure { params: _, body } => {
                self.check_expression(body, symbol_table)?;
                Type::Unit
            }
            Expression::Reference { mutable, expression } => {
                let inner_type = self.check_expression(expression, symbol_table)?;
                Type::Reference {
                    mutable: *mutable,
                    inner: Box::new(inner_type),
                }
            }
            Expression::Dereference(expr) => {
                let expr_type = self.check_expression(expr, symbol_table)?;
                match expr_type {
                    Type::Reference { mutable: _, inner } => *inner,
                    Type::Pointer { mutable: _, inner } => *inner,
                    _ => return Err(TypeError::InvalidBinaryOp.into()),
                }
            }
        };

        self.type_cache.insert(expr.clone(), expr_type.clone());
        Ok(expr_type)
    }

    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::Primitive(PrimitiveType::I32),
            Literal::Float(_) => Type::Primitive(PrimitiveType::F64),
            Literal::String(_) => Type::Primitive(PrimitiveType::Str),
            Literal::Char(_) => Type::Primitive(PrimitiveType::Char),
            Literal::Boolean(_) => Type::Primitive(PrimitiveType::Bool),
            Literal::Unit => Type::Unit,
        }
    }

    fn check_binary_op(&mut self, left: &Type, op: &BinaryOperator, right: &Type) -> Result<Type> {
        match op {
            BinaryOperator::Add | BinaryOperator::Subtract | 
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                self.unify(left, right)?;
                Ok(left.clone())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual |
            BinaryOperator::Less | BinaryOperator::LessEqual |
            BinaryOperator::Greater | BinaryOperator::GreaterEqual => {
                self.unify(left, right)?;
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                let bool_type = Type::Primitive(PrimitiveType::Bool);
                self.unify(left, &bool_type)?;
                self.unify(right, &bool_type)?;
                Ok(bool_type)
            }
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr | BinaryOperator::BitwiseXor |
            BinaryOperator::LeftShift | BinaryOperator::RightShift => {
                self.unify(left, right)?;
                Ok(left.clone())
            }
        }
    }

    fn check_unary_op(&mut self, op: &UnaryOperator, operand: &Type) -> Result<Type> {
        match op {
            UnaryOperator::Not => {
                let bool_type = Type::Primitive(PrimitiveType::Bool);
                self.unify(operand, &bool_type)?;
                Ok(bool_type)
            }
            UnaryOperator::Minus => Ok(operand.clone()),
            UnaryOperator::Reference => {
                Ok(Type::Reference {
                    mutable: false,
                    inner: Box::new(operand.clone()),
                })
            }
            UnaryOperator::MutableReference => {
                Ok(Type::Reference {
                    mutable: true,
                    inner: Box::new(operand.clone()),
                })
            }
            UnaryOperator::Dereference => {
                match operand {
                    Type::Reference { mutable: _, inner } => Ok(*inner.clone()),
                    Type::Pointer { mutable: _, inner } => Ok(*inner.clone()),
                    _ => Err(TypeError::InvalidBinaryOp.into()),
                }
            }
        }
    }

    fn unify(&mut self, t1: &Type, t2: &Type) -> Result<()> {
        if self.types_equal(t1, t2) {
            return Ok(());
        }

        let mut worklist = vec![(t1.clone(), t2.clone())];
        let mut substitutions = self.substitutions.write();

        while let Some((ty1, ty2)) = worklist.pop() {
            match (&ty1, &ty2) {
                (Type::Identifier(var1), Type::Identifier(var2)) if var1 == var2 => continue,
                (Type::Identifier(var), other) | (other, Type::Identifier(var)) => {
                    if self.occurs_check(other, &TypeVar { id: 0, name: var.clone() })? {
                        return Err(TypeError::InfiniteType.into());
                    }
                    
                    let substitution = Substitution {
                        var: TypeVar { id: 0, name: var.clone() },
                        ty: other.clone(),
                    };
                    substitutions.push(substitution);
                }
                (Type::Reference { mutable: m1, inner: i1 }, Type::Reference { mutable: m2, inner: i2 }) => {
                    if m1 != m2 {
                        return Err(TypeError::Mismatch {
                            expected: format!("{:?}", ty1),
                            found: format!("{:?}", ty2),
                        }.into());
                    }
                    worklist.push((*i1.clone(), *i2.clone()));
                }
                (Type::Pointer { mutable: m1, inner: i1 }, Type::Pointer { mutable: m2, inner: i2 }) => {
                    if m1 != m2 {
                        return Err(TypeError::Mismatch {
                            expected: format!("{:?}", ty1),
                            found: format!("{:?}", ty2),
                        }.into());
                    }
                    worklist.push((*i1.clone(), *i2.clone()));
                }
                (Type::Array { element_type: e1, size: _ }, Type::Array { element_type: e2, size: _ }) => {
                    worklist.push((*e1.clone(), *e2.clone()));
                }
                (Type::Slice(e1), Type::Slice(e2)) => {
                    worklist.push((*e1.clone(), *e2.clone()));
                }
                (Type::Tuple(t1), Type::Tuple(t2)) => {
                    if t1.len() != t2.len() {
                        return Err(TypeError::Mismatch {
                            expected: format!("{:?}", ty1),
                            found: format!("{:?}", ty2),
                        }.into());
                    }
                    for (a, b) in t1.iter().zip(t2.iter()) {
                        worklist.push((a.clone(), b.clone()));
                    }
                }
                (Type::Function { params: p1, return_type: r1 }, Type::Function { params: p2, return_type: r2 }) => {
                    if p1.len() != p2.len() {
                        return Err(TypeError::Mismatch {
                            expected: format!("{:?}", ty1),
                            found: format!("{:?}", ty2),
                        }.into());
                    }
                    for (a, b) in p1.iter().zip(p2.iter()) {
                        worklist.push((a.clone(), b.clone()));
                    }
                    worklist.push((*r1.clone(), *r2.clone()));
                }
                _ => {
                    return Err(TypeError::Mismatch {
                        expected: format!("{:?}", ty1),
                        found: format!("{:?}", ty2),
                    }.into());
                }
            }
        }

        Ok(())
    }

    fn occurs_check(&mut self, ty: &Type, var: &TypeVar) -> Result<bool> {
        let key = (ty.clone(), var.clone());
        if let Some(&result) = self.occurs_check_cache.get(&key) {
            return Ok(result);
        }

        let result = match ty {
            Type::Identifier(name) => name == &var.name,
            Type::Reference { inner, .. } | Type::Pointer { inner, .. } => {
                self.occurs_check(inner, var)?
            }
            Type::Array { element_type, .. } => {
                self.occurs_check(element_type, var)?
            }
            Type::Slice(element_type) => {
                self.occurs_check(element_type, var)?
            }
            Type::Tuple(types) => {
                types.iter().any(|t| self.occurs_check(t, var)?)
            }
            Type::Function { params, return_type } => {
                params.iter().any(|t| self.occurs_check(t, var)?) ||
                self.occurs_check(return_type, var)?
            }
            _ => false,
        };

        self.occurs_check_cache.insert(key, result);
        Ok(result)
    }

    fn types_equal(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (Type::Unit, Type::Unit) => true,
            (Type::Identifier(n1), Type::Identifier(n2)) => n1 == n2,
            (Type::Reference { mutable: m1, inner: i1 }, Type::Reference { mutable: m2, inner: i2 }) => {
                m1 == m2 && self.types_equal(i1, i2)
            }
            (Type::Pointer { mutable: m1, inner: i1 }, Type::Pointer { mutable: m2, inner: i2 }) => {
                m1 == m2 && self.types_equal(i1, i2)
            }
            (Type::Array { element_type: e1, size: _ }, Type::Array { element_type: e2, size: _ }) => {
                self.types_equal(e1, e2)
            }
            (Type::Slice(e1), Type::Slice(e2)) => self.types_equal(e1, e2),
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len() && t1.iter().zip(t2.iter()).all(|(a, b)| self.types_equal(a, b))
            }
            (Type::Function { params: p1, return_type: r1 }, Type::Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len() && 
                p1.iter().zip(p2.iter()).all(|(a, b)| self.types_equal(a, b)) &&
                self.types_equal(r1, r2)
            }
            _ => false,
        }
    }

    fn fresh_type_var(&mut self) -> Type {
        let var_name = format!("T{}", self.next_type_var);
        self.next_type_var += 1;
        Type::Identifier(var_name)
    }

    fn solve_constraints(&mut self) -> Result<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            match constraint {
                TypeConstraint::Equality(t1, t2) => {
                    self.unify(&t1, &t2)?;
                }
                TypeConstraint::Subtype(sub, super_ty) => {
                    self.constraints.push_back(TypeConstraint::Equality(sub, super_ty));
                }
                TypeConstraint::Trait(ty, trait_name) => {
                    if !self.trait_impls.contains_key(&(ty.clone(), trait_name.clone())) {
                        return Err(TypeError::Mismatch {
                            expected: format!("type implementing {}", trait_name),
                            found: format!("{:?}", ty),
                        }.into());
                    }
                }
            }
        }
        Ok(())
    }
}