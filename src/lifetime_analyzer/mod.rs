use std::collections::{HashMap, HashSet};
use crate::parser::{Function, Statement, Expression};

pub struct LifetimeAnalyzer {
    lifetimes: HashMap<String, Lifetime>,
    constraints: Vec<LifetimeConstraint>,
    next_lifetime_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub enum LifetimeConstraint {
    Outlives { shorter: Lifetime, longer: Lifetime },
    Equal { left: Lifetime, right: Lifetime },
}

impl LifetimeAnalyzer {
    pub fn new() -> Self {
        Self {
            lifetimes: HashMap::new(),
            constraints: Vec::new(),
            next_lifetime_id: 0,
        }
    }
    
    pub fn analyze_function(&mut self, function: &Function) -> Result<(), String> {
        for param in &function.params {
            let lifetime = self.fresh_lifetime(&param.name);
            self.lifetimes.insert(param.name.clone(), lifetime);
        }
        
        for stmt in &function.body {
            self.analyze_statement(stmt)?;
        }
        
        self.solve_constraints()
    }
    
    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let { name, value, .. } => {
                let lifetime = self.fresh_lifetime(name);
                self.lifetimes.insert(name.clone(), lifetime.clone());
                
                if let Some(expr) = value {
                    let expr_lifetime = self.analyze_expression(expr)?;
                    self.add_constraint(LifetimeConstraint::Outlives {
                        shorter: expr_lifetime,
                        longer: lifetime,
                    });
                }
            }
            
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            
            Statement::Return(Some(expr)) => {
                self.analyze_expression(expr)?;
            }
            
            Statement::If { condition, then_body, else_body } => {
                self.analyze_expression(condition)?;
                
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
                self.analyze_expression(condition)?;
                
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn analyze_expression(&mut self, expr: &Expression) -> Result<Lifetime, String> {
        match expr {
            Expression::Ident(name) | Expression::Identifier(name) => {
                self.lifetimes
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable '{}'", name))
            }
            
            Expression::Binary { left, right, .. } => {
                let left_lifetime = self.analyze_expression(left)?;
                let right_lifetime = self.analyze_expression(right)?;
                
                let result_lifetime = self.fresh_lifetime("binary_result");
                
                self.add_constraint(LifetimeConstraint::Outlives {
                    shorter: left_lifetime,
                    longer: result_lifetime.clone(),
                });
                
                self.add_constraint(LifetimeConstraint::Outlives {
                    shorter: right_lifetime,
                    longer: result_lifetime.clone(),
                });
                
                Ok(result_lifetime)
            }
            
            Expression::Unary { expr, .. } => {
                self.analyze_expression(expr)
            }
            
            Expression::Call { func, args } | Expression::CallAlt { callee: func, args } => {
                self.analyze_expression(func)?;
                
                for arg in args {
                    self.analyze_expression(arg)?;
                }
                
                Ok(self.fresh_lifetime("call_result"))
            }
            
            _ => Ok(self.fresh_lifetime("expr")),
        }
    }
    
    fn fresh_lifetime(&mut self, name: &str) -> Lifetime {
        let id = self.next_lifetime_id;
        self.next_lifetime_id += 1;
        
        Lifetime {
            name: format!("'{}{}", name, id),
            id,
        }
    }
    
    fn add_constraint(&mut self, constraint: LifetimeConstraint) {
        self.constraints.push(constraint);
    }
    
    fn solve_constraints(&self) -> Result<(), String> {
        let mut outlives_graph: HashMap<usize, HashSet<usize>> = HashMap::new();
        
        for constraint in &self.constraints {
            match constraint {
                LifetimeConstraint::Outlives { shorter, longer } => {
                    outlives_graph
                        .entry(shorter.id)
                        .or_insert_with(HashSet::new)
                        .insert(longer.id);
                }
                
                LifetimeConstraint::Equal { left, right } => {
                    outlives_graph
                        .entry(left.id)
                        .or_insert_with(HashSet::new)
                        .insert(right.id);
                    
                    outlives_graph
                        .entry(right.id)
                        .or_insert_with(HashSet::new)
                        .insert(left.id);
                }
            }
        }
        
        if self.has_cycle(&outlives_graph) {
            return Err("Cyclic lifetime constraints detected".to_string());
        }
        
        Ok(())
    }
    
    fn has_cycle(&self, graph: &HashMap<usize, HashSet<usize>>) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &node in graph.keys() {
            if self.has_cycle_util(node, graph, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        
        false
    }
    
    fn has_cycle_util(
        &self,
        node: usize,
        graph: &HashMap<usize, HashSet<usize>>,
        visited: &mut HashSet<usize>,
        rec_stack: &mut HashSet<usize>,
    ) -> bool {
        if rec_stack.contains(&node) {
            return true;
        }
        
        if visited.contains(&node) {
            return false;
        }
        
        visited.insert(node);
        rec_stack.insert(node);
        
        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if self.has_cycle_util(neighbor, graph, visited, rec_stack) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(&node);
        false
    }
    
    pub fn elide_lifetimes(&self, function: &Function) -> HashMap<String, Lifetime> {
        let mut elided = HashMap::new();
        
        if function.params.len() == 1 {
            for param in &function.params {
                if let Some(lifetime) = self.lifetimes.get(&param.name) {
                    elided.insert(param.name.clone(), lifetime.clone());
                }
            }
        }
        
        elided
    }
}

impl Default for LifetimeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegionInference {
    regions: HashMap<usize, Region>,
    region_constraints: Vec<RegionConstraint>,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub id: usize,
    pub scope_depth: usize,
}

#[derive(Debug, Clone)]
pub enum RegionConstraint {
    SubRegion { sub: usize, sup: usize },
}

impl RegionInference {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            region_constraints: Vec::new(),
        }
    }
    
    pub fn infer_regions(&mut self) -> Result<(), String> {
        let mut changed = true;
        
        while changed {
            changed = false;
            
            for constraint in &self.region_constraints {
                match constraint {
                    RegionConstraint::SubRegion { sub, sup } => {
                        if let (Some(sub_region), Some(sup_region)) = 
                            (self.regions.get(sub), self.regions.get(sup)) {
                            
                            if sub_region.scope_depth > sup_region.scope_depth {
                                return Err(format!(
                                    "Region {} outlives region {}",
                                    sub, sup
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for RegionInference {
    fn default() -> Self {
        Self::new()
    }
}
