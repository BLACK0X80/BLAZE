use std::collections::{HashMap, HashSet, VecDeque};
use crate::parser::{Program, Item, Function, Statement, Expression, Type as AstType, BinaryOperator, UnaryOperator, Literal, Pattern};
use crate::error::{CompileError, Result};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    id: usize,
    level: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    Var(TypeVariable),
    Concrete(AstType),
    Function(Vec<InferredType>, Box<InferredType>),
    Generic(String, Vec<InferredType>),
    ForAll(Vec<TypeVariable>, Box<InferredType>),
}

#[derive(Debug, Clone)]
pub struct Constraint {
    left: InferredType,
    right: InferredType,
    location: Option<(usize, usize)>,
}

pub struct TypeInference {
    next_var: usize,
    current_level: usize,
    constraints: VecDeque<Constraint>,
    substitutions: Arc<RwLock<HashMap<TypeVariable, InferredType>>>,
    environment: HashMap<String, InferredType>,
    generic_instances: HashMap<String, Vec<InferredType>>,
}

impl TypeInference {
    pub fn new() -> Self {
        Self {
            next_var: 0,
            current_level: 0,
            constraints: VecDeque::new(),
            substitutions: Arc::new(RwLock::new(HashMap::new())),
            environment: HashMap::new(),
            generic_instances: HashMap::new(),
        }
    }

    pub fn infer(&mut self, program: &Program) -> Result<HashMap<String, InferredType>> {
        let mut results = HashMap::new();

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let ty = self.infer_function(func)?;
                    results.insert(func.name.clone(), ty);
                }
                _ => {}
            }
        }

        self.solve_constraints()?;
        self.apply_substitutions(&mut results);

        Ok(results)
    }

    fn fresh_var(&mut self) -> InferredType {
        let var = TypeVariable {
            id: self.next_var,
            level: self.current_level,
        };
        self.next_var += 1;
        InferredType::Var(var)
    }

    fn enter_level(&mut self) {
        self.current_level += 1;
    }

    fn exit_level(&mut self) {
        self.current_level -= 1;
    }

    fn infer_function(&mut self, func: &Function) -> Result<InferredType> {
        self.enter_level();

        let mut param_types = Vec::new();
        for param in &func.params {
            let ty = self.ast_type_to_inferred(&param.ty);
            self.environment.insert(param.name.clone(), ty.clone());
            param_types.push(ty);
        }

        let return_type = if let Some(ret_ty) = &func.return_type {
            self.ast_type_to_inferred(ret_ty)
        } else {
            self.fresh_var()
        };

        for stmt in &func.body {
            self.infer_statement(stmt)?;
        }

        self.exit_level();

        let func_type = InferredType::Function(param_types, Box::new(return_type));

        if !func.generics.is_empty() {
            let type_vars: Vec<TypeVariable> = (0..func.generics.len())
                .map(|i| TypeVariable {
                    id: self.next_var + i,
                    level: self.current_level,
                })
                .collect();
            self.next_var += func.generics.len();
            Ok(InferredType::ForAll(type_vars, Box::new(func_type)))
        } else {
            Ok(func_type)
        }
    }

    fn infer_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, ty, value, .. } => {
                let inferred_ty = if let Some(val) = value {
                    self.infer_expression(val)?
                } else {
                    self.fresh_var()
                };

                if let Some(declared_ty) = ty {
                    let declared = self.ast_type_to_inferred(declared_ty);
                    self.add_constraint(declared, inferred_ty.clone(), None);
                }

                self.environment.insert(name.clone(), inferred_ty);
                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.infer_expression(e)?;
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.infer_expression(expr)?;
                Ok(())
            }
            Statement::While { condition, body } => {
                let cond_ty = self.infer_expression(condition)?;
                self.add_constraint(
                    InferredType::Concrete(AstType::Primitive(crate::parser::PrimitiveType::Bool)),
                    cond_ty,
                    None,
                );
                for stmt in body {
                    self.infer_statement(stmt)?;
                }
                Ok(())
            }
            Statement::For { variable, iterable, body } => {
                let iter_ty = self.infer_expression(iterable)?;
                let elem_ty = self.fresh_var();
                self.environment.insert(variable.clone(), elem_ty);
                for stmt in body {
                    self.infer_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Loop { body } => {
                for stmt in body {
                    self.infer_statement(stmt)?;
                }
                Ok(())
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_ty = self.infer_expression(condition)?;
                self.add_constraint(
                    InferredType::Concrete(AstType::Primitive(crate::parser::PrimitiveType::Bool)),
                    cond_ty,
                    None,
                );
                for stmt in then_body {
                    self.infer_statement(stmt)?;
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.infer_statement(stmt)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn infer_expression(&mut self, expr: &Expression) -> Result<InferredType> {
        match expr {
            Expression::Literal(lit) => Ok(self.literal_type(lit)),
            Expression::Identifier(name) | Expression::Ident(name) => {
                self.environment.get(name).cloned().ok_or_else(|| {
                    CompileError::SemanticError {
                        message: format!("Undefined variable: {}", name),
                        line: None,
                        column: None,
                        source_snippet: None,
                        suggestion: Some(format!("Did you mean to declare '{}' first?", name)),
                        related_info: vec![],
                    }
                })
            }
            Expression::Binary { op, left, right } | Expression::BinaryOp { operator: op, left, right } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;
                self.infer_binary_op(&left_ty, op, &right_ty)
            }
            Expression::Unary { op, expr: operand } | Expression::UnaryOp { operator: op, operand } => {
                let operand_ty = self.infer_expression(operand)?;
                self.infer_unary_op(op, &operand_ty)
            }
            Expression::Call { func, args } | Expression::CallAlt { callee: func, args } => {
                let func_ty = self.infer_expression(func)?;
                let arg_types: Result<Vec<InferredType>> = args.iter()
                    .map(|arg| self.infer_expression(arg))
                    .collect();
                let arg_types = arg_types?;

                let return_ty = self.fresh_var();
                let expected_func_ty = InferredType::Function(arg_types, Box::new(return_ty.clone()));
                self.add_constraint(func_ty, expected_func_ty, None);

                Ok(return_ty)
            }
            Expression::If { condition, then_branch, else_branch } => {
                let cond_ty = self.infer_expression(condition)?;
                self.add_constraint(
                    InferredType::Concrete(AstType::Primitive(crate::parser::PrimitiveType::Bool)),
                    cond_ty,
                    None,
                );

                let then_ty = self.infer_expression(then_branch)?;
                if let Some(else_expr) = else_branch {
                    let else_ty = self.infer_expression(else_expr)?;
                    self.add_constraint(then_ty.clone(), else_ty, None);
                    Ok(then_ty)
                } else {
                    Ok(InferredType::Concrete(AstType::Unit))
                }
            }
            Expression::Block(stmts) => {
                for stmt in stmts {
                    self.infer_statement(stmt)?;
                }
                Ok(InferredType::Concrete(AstType::Unit))
            }
            Expression::Match { expression, arms } => {
                let expr_ty = self.infer_expression(expression)?;

                if arms.is_empty() {
                    return Ok(InferredType::Concrete(AstType::Unit));
                }

                let first_arm_ty = self.infer_expression(&arms[0].body)?;
                for arm in &arms[1..] {
                    let arm_ty = self.infer_expression(&arm.body)?;
                    self.add_constraint(first_arm_ty.clone(), arm_ty, None);
                }

                Ok(first_arm_ty)
            }
            Expression::FieldAccess { object, .. } => {
                self.infer_expression(object)?;
                Ok(self.fresh_var())
            }
            Expression::MethodCall { object, args, .. } => {
                self.infer_expression(object)?;
                for arg in args {
                    self.infer_expression(arg)?;
                }
                Ok(self.fresh_var())
            }
            Expression::Index { object, index } => {
                let obj_ty = self.infer_expression(object)?;
                let idx_ty = self.infer_expression(index)?;
                Ok(self.fresh_var())
            }
            Expression::Assignment { target, value } => {
                let target_ty = self.infer_expression(target)?;
                let value_ty = self.infer_expression(value)?;
                self.add_constraint(target_ty, value_ty, None);
                Ok(InferredType::Concrete(AstType::Unit))
            }
            Expression::Closure { params, body } => {
                let param_types: Vec<InferredType> = params.iter()
                    .map(|p| {
                        if let Some(ty) = &p.ty {
                            self.ast_type_to_inferred(ty)
                        } else {
                            self.fresh_var()
                        }
                    })
                    .collect();

                for (param, ty) in params.iter().zip(param_types.iter()) {
                    self.environment.insert(param.name.clone(), ty.clone());
                }

                let body_ty = self.infer_expression(body)?;
                Ok(InferredType::Function(param_types, Box::new(body_ty)))
            }
            Expression::TupleLiteral(exprs) => {
                let types: Result<Vec<InferredType>> = exprs.iter()
                    .map(|e| self.infer_expression(e))
                    .collect();
                let types = types?;
                let ast_types: Vec<AstType> = types.iter()
                    .map(|t| self.inferred_to_ast_type(t))
                    .collect();
                Ok(InferredType::Concrete(AstType::Tuple(ast_types)))
            }
            Expression::ArrayLiteral(exprs) => {
                if exprs.is_empty() {
                    let elem_ty = self.fresh_var();
                    return Ok(InferredType::Concrete(AstType::Array {
                        element: Box::new(self.inferred_to_ast_type(&elem_ty)),
                        size: None,
                    }));
                }

                let first_ty = self.infer_expression(&exprs[0])?;
                for expr in &exprs[1..] {
                    let ty = self.infer_expression(expr)?;
                    self.add_constraint(first_ty.clone(), ty, None);
                }

                Ok(InferredType::Concrete(AstType::Array {
                    element: Box::new(self.inferred_to_ast_type(&first_ty)),
                    size: None,
                }))
            }
            Expression::StructLiteral { name, fields } => {
                for field in fields {
                    self.infer_expression(&field.value)?;
                }
                Ok(InferredType::Concrete(AstType::Custom(name.clone())))
            }
            _ => Ok(self.fresh_var()),
        }
    }

    fn literal_type(&self, lit: &Literal) -> InferredType {
        use crate::parser::PrimitiveType;
        InferredType::Concrete(match lit {
            Literal::Integer(_) => AstType::Primitive(PrimitiveType::I64),
            Literal::Float(_) => AstType::Primitive(PrimitiveType::F64),
            Literal::String(_) => AstType::Primitive(PrimitiveType::Str),
            Literal::Char(_) => AstType::Primitive(PrimitiveType::Char),
            Literal::Boolean(_) => AstType::Primitive(PrimitiveType::Bool),
            Literal::Unit => AstType::Unit,
        })
    }

    fn infer_binary_op(&mut self, left: &InferredType, op: &dyn std::fmt::Debug, right: &InferredType) -> Result<InferredType> {
        use crate::parser::PrimitiveType;
        self.add_constraint(left.clone(), right.clone(), None);
        Ok(InferredType::Concrete(AstType::Primitive(PrimitiveType::I64)))
    }

    fn infer_unary_op(&mut self, op: &dyn std::fmt::Debug, operand: &InferredType) -> Result<InferredType> {
        Ok(operand.clone())
    }

    fn add_constraint(&mut self, left: InferredType, right: InferredType, location: Option<(usize, usize)>) {
        self.constraints.push_back(Constraint { left, right, location });
    }

    fn solve_constraints(&mut self) -> Result<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            self.unify(&constraint.left, &constraint.right)?;
        }
        Ok(())
    }

    fn unify(&mut self, t1: &InferredType, t2: &InferredType) -> Result<()> {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);

        match (&t1, &t2) {
            (InferredType::Var(v1), InferredType::Var(v2)) if v1 == v2 => Ok(()),
            (InferredType::Var(v), other) | (other, InferredType::Var(v)) => {
                if self.occurs_check(v, other) {
                    return Err(CompileError::TypeError {
                        message: "Infinite type detected".to_string(),
                        expected: None,
                        found: None,
                        line: None,
                        column: None,
                        source_snippet: None,
                        suggestion: Some("Check for recursive type definitions".to_string()),
                        help: None,
                    });
                }
                let mut subs = self.substitutions.write();
                subs.insert(v.clone(), other.clone());
                Ok(())
            }
            (InferredType::Concrete(c1), InferredType::Concrete(c2)) if c1 == c2 => Ok(()),
            (InferredType::Function(p1, r1), InferredType::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(CompileError::TypeError {
                        message: format!("Function parameter count mismatch: expected {}, found {}", p1.len(), p2.len()),
                        expected: Some(format!("{} parameters", p1.len())),
                        found: Some(format!("{} parameters", p2.len())),
                        line: None,
                        column: None,
                        source_snippet: None,
                        suggestion: None,
                        help: None,
                    });
                }
                for (p1, p2) in p1.iter().zip(p2.iter()) {
                    self.unify(p1, p2)?;
                }
                self.unify(r1, r2)
            }
            (InferredType::Generic(n1, a1), InferredType::Generic(n2, a2)) if n1 == n2 => {
                if a1.len() != a2.len() {
                    return Err(CompileError::TypeError {
                        message: format!("Generic type argument count mismatch"),
                        expected: Some(format!("{} arguments", a1.len())),
                        found: Some(format!("{} arguments", a2.len())),
                        line: None,
                        column: None,
                        source_snippet: None,
                        suggestion: None,
                        help: None,
                    });
                }
                for (a1, a2) in a1.iter().zip(a2.iter()) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            }
            _ => Err(CompileError::TypeError {
                message: "Type mismatch".to_string(),
                expected: Some(format!("{:?}", t1)),
                found: Some(format!("{:?}", t2)),
                line: None,
                column: None,
                source_snippet: None,
                suggestion: None,
                help: None,
            }),
        }
    }

    fn resolve(&self, ty: &InferredType) -> InferredType {
        if let InferredType::Var(v) = ty {
            let subs = self.substitutions.read();
            if let Some(t) = subs.get(v) {
                return self.resolve(t);
            }
        }
        ty.clone()
    }

    fn occurs_check(&self, var: &TypeVariable, ty: &InferredType) -> bool {
        match ty {
            InferredType::Var(v) => v == var,
            InferredType::Function(params, ret) => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, ret)
            }
            InferredType::Generic(_, args) => {
                args.iter().any(|a| self.occurs_check(var, a))
            }
            InferredType::ForAll(vars, ty) => {
                !vars.contains(var) && self.occurs_check(var, ty)
            }
            _ => false,
        }
    }

    fn ast_type_to_inferred(&self, ty: &AstType) -> InferredType {
        InferredType::Concrete(ty.clone())
    }

    fn inferred_to_ast_type(&self, ty: &InferredType) -> AstType {
        match ty {
            InferredType::Concrete(t) => t.clone(),
            InferredType::Var(_) => AstType::Identifier("_".to_string()),
            InferredType::Function(params, ret) => {
                let param_types: Vec<AstType> = params.iter()
                    .map(|p| self.inferred_to_ast_type(p))
                    .collect();
                AstType::Function {
                    params: param_types,
                    return_type: Box::new(self.inferred_to_ast_type(ret)),
                }
            }
            InferredType::Generic(name, args) => {
                let arg_types: Vec<AstType> = args.iter()
                    .map(|a| self.inferred_to_ast_type(a))
                    .collect();
                AstType::Generic {
                    name: name.clone(),
                    args: arg_types,
                }
            }
            InferredType::ForAll(_, ty) => self.inferred_to_ast_type(ty),
        }
    }

    fn apply_substitutions(&self, results: &mut HashMap<String, InferredType>) {
        for (_, ty) in results.iter_mut() {
            *ty = self.resolve(ty);
        }
    }
}
