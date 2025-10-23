use crate::parser::{Program, Item, Function, Statement, Expression};
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct LifetimeAnalyzer {
    next_lifetime_id: usize,
    lifetime_constraints: Vec<LifetimeConstraint>,
    lifetime_map: HashMap<String, LifetimeId>,
    concrete_lifetimes: HashMap<LifetimeId, ConcreteLifetime>,
    source_locations: HashMap<LifetimeId, SourceLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifetimeId(usize);

#[derive(Debug, Clone)]
pub enum LifetimeConstraint {
    Outlives { longer: LifetimeId, shorter: LifetimeId },
    Equal { left: LifetimeId, right: LifetimeId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcreteLifetime {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

impl LifetimeAnalyzer {
    pub fn new() -> Self {
        Self {
            next_lifetime_id: 0,
            lifetime_constraints: Vec::new(),
            lifetime_map: HashMap::new(),
            concrete_lifetimes: HashMap::new(),
            source_locations: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        // Phase 1: Collect constraints from AST
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.analyze_function(func)?;
                }
                Item::Struct(_) => {}
            }
        }
        
        // Phase 2: Solve constraints using unification
        self.solve_constraints()?;
        
        // Phase 3: Assign concrete lifetimes
        self.assign_concrete_lifetimes()?;
        
        // Phase 4: Validate lifetime constraints
        self.validate_lifetimes()?;
        
        Ok(())
    }

    fn analyze_function(&mut self, func: &Function) -> Result<()> {
        // Create lifetimes for function parameters
        for param in &func.params {
            let lifetime = self.new_lifetime();
            self.lifetime_map.insert(param.name.clone(), lifetime);
            self.source_locations.insert(
                lifetime,
                SourceLocation {
                    name: param.name.clone(),
                    line: 0, // Would need actual source location from parser
                    column: 0,
                },
            );
        }

        // Analyze function body
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value, .. } => {
                let var_lifetime = self.new_lifetime();
                self.lifetime_map.insert(name.clone(), var_lifetime);
                self.source_locations.insert(
                    var_lifetime,
                    SourceLocation {
                        name: name.clone(),
                        line: 0,
                        column: 0,
                    },
                );

                let expr_lifetime = self.infer_expression_lifetime(value)?;
                // Variable must outlive its initializer
                self.add_constraint(LifetimeConstraint::Outlives {
                    longer: var_lifetime,
                    shorter: expr_lifetime,
                });
            }
            Statement::If { condition, then_body, else_body } => {
                self.infer_expression_lifetime(condition)?;
                
                for stmt in then_body {
                    self.analyze_statement(stmt)?;
                }
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                self.infer_expression_lifetime(condition)?;
                
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::Return(Some(expr)) => {
                self.infer_expression_lifetime(expr)?;
            }
            Statement::Expression(expr) => {
                self.infer_expression_lifetime(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn infer_expression_lifetime(&mut self, expr: &Expression) -> Result<LifetimeId> {
        match expr {
            Expression::Ident(name) => {
                if let Some(&lifetime) = self.lifetime_map.get(name) {
                    Ok(lifetime)
                } else {
                    let lifetime = self.new_lifetime();
                    self.lifetime_map.insert(name.clone(), lifetime);
                    Ok(lifetime)
                }
            }
            Expression::Binary { left, right, .. } => {
                let left_lifetime = self.infer_expression_lifetime(left)?;
                let right_lifetime = self.infer_expression_lifetime(right)?;
                
                let result_lifetime = self.new_lifetime();
                self.add_constraint(LifetimeConstraint::Outlives {
                    longer: result_lifetime,
                    shorter: left_lifetime,
                });
                self.add_constraint(LifetimeConstraint::Outlives {
                    longer: result_lifetime,
                    shorter: right_lifetime,
                });
                
                Ok(result_lifetime)
            }
            Expression::Call { func, args } => {
                if let Expression::Ident(_) = &**func {
                    for arg in args {
                        self.infer_expression_lifetime(arg)?;
                    }
                }
                Ok(self.new_lifetime())
            }
            _ => Ok(self.new_lifetime()),
        }
    }

    /// Solve lifetime constraints using unification-based approach
    fn solve_constraints(&mut self) -> Result<()> {
        // Build constraint graph
        let mut outlives_graph: HashMap<LifetimeId, HashSet<LifetimeId>> = HashMap::new();
        let mut equal_sets: HashMap<LifetimeId, LifetimeId> = HashMap::new();
        
        // Process equality constraints first (unification)
        for constraint in &self.lifetime_constraints {
            if let LifetimeConstraint::Equal { left, right } = constraint {
                self.unify_lifetimes(&mut equal_sets, *left, *right);
            }
        }
        
        // Process outlives constraints
        for constraint in &self.lifetime_constraints {
            if let LifetimeConstraint::Outlives { longer, shorter } = constraint {
                let longer_rep = self.find_representative(&equal_sets, *longer);
                let shorter_rep = self.find_representative(&equal_sets, *shorter);
                
                outlives_graph
                    .entry(longer_rep)
                    .or_insert_with(HashSet::new)
                    .insert(shorter_rep);
            }
        }
        
        // Check for cycles in outlives graph
        self.check_cycles(&outlives_graph)?;
        
        Ok(())
    }
    
    /// Unify two lifetimes (make them equal)
    fn unify_lifetimes(
        &self,
        equal_sets: &mut HashMap<LifetimeId, LifetimeId>,
        left: LifetimeId,
        right: LifetimeId,
    ) {
        let left_rep = self.find_representative(equal_sets, left);
        let right_rep = self.find_representative(equal_sets, right);
        
        if left_rep != right_rep {
            equal_sets.insert(right_rep, left_rep);
        }
    }
    
    /// Find the representative lifetime in the union-find structure
    fn find_representative(
        &self,
        equal_sets: &HashMap<LifetimeId, LifetimeId>,
        lifetime: LifetimeId,
    ) -> LifetimeId {
        if let Some(&parent) = equal_sets.get(&lifetime) {
            if parent != lifetime {
                return self.find_representative(equal_sets, parent);
            }
        }
        lifetime
    }
    
    /// Check for cycles in the outlives graph
    fn check_cycles(&self, graph: &HashMap<LifetimeId, HashSet<LifetimeId>>) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &lifetime in graph.keys() {
            if !visited.contains(&lifetime) {
                if self.has_cycle_dfs(lifetime, graph, &mut visited, &mut rec_stack)? {
                    return self.report_cycle_error(lifetime);
                }
            }
        }
        
        Ok(())
    }
    
    /// DFS-based cycle detection
    fn has_cycle_dfs(
        &self,
        current: LifetimeId,
        graph: &HashMap<LifetimeId, HashSet<LifetimeId>>,
        visited: &mut HashSet<LifetimeId>,
        rec_stack: &mut HashSet<LifetimeId>,
    ) -> Result<bool> {
        visited.insert(current);
        rec_stack.insert(current);
        
        if let Some(neighbors) = graph.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if self.has_cycle_dfs(neighbor, graph, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&neighbor) {
                    return Ok(true);
                }
            }
        }
        
        rec_stack.remove(&current);
        Ok(false)
    }
    
    /// Assign concrete lifetime regions based on constraints
    fn assign_concrete_lifetimes(&mut self) -> Result<()> {
        // Build topological order of lifetimes
        let mut graph: HashMap<LifetimeId, HashSet<LifetimeId>> = HashMap::new();
        let mut in_degree: HashMap<LifetimeId, usize> = HashMap::new();
        
        // Collect all lifetimes
        let all_lifetimes: HashSet<LifetimeId> = self
            .lifetime_map
            .values()
            .copied()
            .collect();
        
        for &lifetime in &all_lifetimes {
            in_degree.entry(lifetime).or_insert(0);
        }
        
        // Build graph from outlives constraints
        for constraint in &self.lifetime_constraints {
            if let LifetimeConstraint::Outlives { longer, shorter } = constraint {
                graph
                    .entry(*longer)
                    .or_insert_with(HashSet::new)
                    .insert(*shorter);
                *in_degree.entry(*shorter).or_insert(0) += 1;
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut queue: VecDeque<LifetimeId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&lifetime, _)| lifetime)
            .collect();
        
        let mut position = 0;
        
        while let Some(lifetime) = queue.pop_front() {
            // Assign concrete lifetime based on topological order
            self.concrete_lifetimes.insert(
                lifetime,
                ConcreteLifetime {
                    start: position,
                    end: position + 100, // Default scope size
                },
            );
            
            position += 1;
            
            if let Some(neighbors) = graph.get(&lifetime) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate that all lifetime constraints are satisfied
    fn validate_lifetimes(&self) -> Result<()> {
        for constraint in &self.lifetime_constraints {
            match constraint {
                LifetimeConstraint::Outlives { longer, shorter } => {
                    if let (Some(longer_lifetime), Some(shorter_lifetime)) = (
                        self.concrete_lifetimes.get(longer),
                        self.concrete_lifetimes.get(shorter),
                    ) {
                        // Longer lifetime must start before or at the same time as shorter
                        // and end after or at the same time as shorter
                        if longer_lifetime.start > shorter_lifetime.start
                            || longer_lifetime.end < shorter_lifetime.end
                        {
                            return self.report_lifetime_error(*longer, *shorter);
                        }
                    }
                }
                LifetimeConstraint::Equal { left, right } => {
                    if let (Some(left_lifetime), Some(right_lifetime)) = (
                        self.concrete_lifetimes.get(left),
                        self.concrete_lifetimes.get(right),
                    ) {
                        if left_lifetime != right_lifetime {
                            return self.report_equality_error(*left, *right);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Report a cycle error with source location information
    fn report_cycle_error(&self, lifetime: LifetimeId) -> Result<()> {
        if let Some(location) = self.source_locations.get(&lifetime) {
            bail!(
                "Cyclic lifetime dependency detected for variable '{}' at line {}, column {}",
                location.name,
                location.line,
                location.column
            );
        } else {
            bail!("Cyclic lifetime dependency detected");
        }
    }
    
    /// Report a lifetime constraint violation with source locations
    fn report_lifetime_error(&self, longer: LifetimeId, shorter: LifetimeId) -> Result<()> {
        let longer_loc = self.source_locations.get(&longer);
        let shorter_loc = self.source_locations.get(&shorter);
        
        match (longer_loc, shorter_loc) {
            (Some(longer_info), Some(shorter_info)) => {
                bail!(
                    "Lifetime constraint violation: '{}' (at line {}) must outlive '{}' (at line {})",
                    longer_info.name,
                    longer_info.line,
                    shorter_info.name,
                    shorter_info.line
                );
            }
            _ => {
                bail!("Lifetime constraint violation detected");
            }
        }
    }
    
    /// Report an equality constraint violation
    fn report_equality_error(&self, left: LifetimeId, right: LifetimeId) -> Result<()> {
        let left_loc = self.source_locations.get(&left);
        let right_loc = self.source_locations.get(&right);
        
        match (left_loc, right_loc) {
            (Some(left_info), Some(right_info)) => {
                bail!(
                    "Lifetime equality constraint violated: '{}' and '{}' must have the same lifetime",
                    left_info.name,
                    right_info.name
                );
            }
            _ => {
                bail!("Lifetime equality constraint violated");
            }
        }
    }

    fn new_lifetime(&mut self) -> LifetimeId {
        let id = LifetimeId(self.next_lifetime_id);
        self.next_lifetime_id += 1;
        id
    }

    fn add_constraint(&mut self, constraint: LifetimeConstraint) {
        self.lifetime_constraints.push(constraint);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expression, BinaryOp, Type};

    fn create_simple_program() -> Program {
        Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: None,
                        value: Expression::IntLit(42),
                    },
                ],
            })],
        }
    }

    #[test]
    fn test_simple_lifetime_analysis() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = create_simple_program();
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok(), "Simple program should pass lifetime analysis");
    }

    #[test]
    fn test_lifetime_constraint_collection() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::Let {
                        name: "x".to_string(),
                        mutable: false,
                        ty: None,
                        value: Expression::IntLit(10),
                    },
                    Statement::Let {
                        name: "y".to_string(),
                        mutable: false,
                        ty: None,
                        value: Expression::Ident("x".to_string()),
                    },
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
        assert!(!analyzer.lifetime_constraints.is_empty(), "Should collect constraints");
    }

    #[test]
    fn test_lifetime_with_binary_expression() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::Let {
                        name: "result".to_string(),
                        mutable: false,
                        ty: None,
                        value: Expression::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Ident("a".to_string())),
                            right: Box::new(Expression::Ident("b".to_string())),
                        },
                    },
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lifetime_with_if_statement() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::If {
                        condition: Expression::BoolLit(true),
                        then_body: vec![
                            Statement::Let {
                                name: "x".to_string(),
                                mutable: false,
                                ty: None,
                                value: Expression::IntLit(10),
                            },
                        ],
                        else_body: None,
                    },
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lifetime_with_while_loop() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::While {
                        condition: Expression::BoolLit(false),
                        body: vec![
                            Statement::Let {
                                name: "x".to_string(),
                                mutable: false,
                                ty: None,
                                value: Expression::IntLit(10),
                            },
                        ],
                    },
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lifetime_with_function_params() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![
                    crate::parser::Parameter {
                        name: "x".to_string(),
                        ty: Type::I32,
                    },
                    crate::parser::Parameter {
                        name: "y".to_string(),
                        ty: Type::I32,
                    },
                ],
                return_type: Some(Type::I32),
                body: vec![
                    Statement::Return(Some(Expression::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Ident("x".to_string())),
                        right: Box::new(Expression::Ident("y".to_string())),
                    })),
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_concrete_lifetime_assignment() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = create_simple_program();
        
        analyzer.analyze(&program).unwrap();
        
        // Check that concrete lifetimes were assigned
        assert!(!analyzer.concrete_lifetimes.is_empty(), "Should assign concrete lifetimes");
    }

    #[test]
    fn test_lifetime_unification() {
        let mut analyzer = LifetimeAnalyzer::new();
        let lifetime1 = analyzer.new_lifetime();
        let lifetime2 = analyzer.new_lifetime();
        
        let mut equal_sets = HashMap::new();
        analyzer.unify_lifetimes(&mut equal_sets, lifetime1, lifetime2);
        
        let rep1 = analyzer.find_representative(&equal_sets, lifetime1);
        let rep2 = analyzer.find_representative(&equal_sets, lifetime2);
        
        assert_eq!(rep1, rep2, "Unified lifetimes should have same representative");
    }

    #[test]
    fn test_multiple_functions() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "func1".to_string(),
                    params: vec![],
                    return_type: None,
                    body: vec![
                        Statement::Let {
                            name: "x".to_string(),
                            mutable: false,
                            ty: None,
                            value: Expression::IntLit(10),
                        },
                    ],
                }),
                Item::Function(Function {
                    name: "func2".to_string(),
                    params: vec![],
                    return_type: None,
                    body: vec![
                        Statement::Let {
                            name: "y".to_string(),
                            mutable: false,
                            ty: None,
                            value: Expression::IntLit(20),
                        },
                    ],
                }),
            ],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_control_flow() {
        let mut analyzer = LifetimeAnalyzer::new();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![
                    Statement::If {
                        condition: Expression::BoolLit(true),
                        then_body: vec![
                            Statement::While {
                                condition: Expression::BoolLit(false),
                                body: vec![
                                    Statement::Let {
                                        name: "x".to_string(),
                                        mutable: false,
                                        ty: None,
                                        value: Expression::IntLit(10),
                                    },
                                ],
                            },
                        ],
                        else_body: None,
                    },
                ],
            })],
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }
}
