use crate::analysis::control_flow::{ControlFlowGraph, BlockId};
use crate::parser::{Statement, Expression};
use std::collections::{HashMap, HashSet};

pub struct DataFlowAnalyzer {
    reaching_definitions: HashMap<BlockId, HashSet<Definition>>,
    live_variables: HashMap<BlockId, HashSet<String>>,
    available_expressions: HashMap<BlockId, HashSet<AvailableExpr>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub variable: String,
    pub block: BlockId,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AvailableExpr {
    pub expr: String,
    pub block: BlockId,
}

impl DataFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            reaching_definitions: HashMap::new(),
            live_variables: HashMap::new(),
            available_expressions: HashMap::new(),
        }
    }
    
    pub fn analyze(&mut self, cfg: &ControlFlowGraph) {
        self.compute_reaching_definitions(cfg);
        self.compute_live_variables(cfg);
        self.compute_available_expressions(cfg);
    }
    
    fn compute_reaching_definitions(&mut self, cfg: &ControlFlowGraph) {
        let mut gen_sets: HashMap<BlockId, HashSet<Definition>> = HashMap::new();
        let mut kill_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();
        
        for block in &cfg.blocks {
            let mut gen = HashSet::new();
            let mut kill = HashSet::new();
            
            for (idx, stmt) in block.statements.iter().enumerate() {
                if let Some(var) = self.get_defined_variable(stmt) {
                    kill.insert(var.clone());
                    gen.insert(Definition {
                        variable: var,
                        block: block.id,
                        index: idx,
                    });
                }
            }
            
            gen_sets.insert(block.id, gen);
            kill_sets.insert(block.id, kill);
        }
        
        for block in &cfg.blocks {
            self.reaching_definitions.insert(block.id, HashSet::new());
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for block in &cfg.blocks {
                let mut in_set = HashSet::new();
                
                for &pred in &block.predecessors {
                    if let Some(pred_out) = self.reaching_definitions.get(&pred) {
                        in_set.extend(pred_out.iter().cloned());
                    }
                }
                
                let kill = &kill_sets[&block.id];
                let gen = &gen_sets[&block.id];
                
                let mut out_set: HashSet<_> = in_set
                    .into_iter()
                    .filter(|def| !kill.contains(&def.variable))
                    .collect();
                out_set.extend(gen.iter().cloned());
                
                if out_set != self.reaching_definitions[&block.id] {
                    self.reaching_definitions.insert(block.id, out_set);
                    changed = true;
                }
            }
        }
    }
    
    fn compute_live_variables(&mut self, cfg: &ControlFlowGraph) {
        let mut use_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();
        let mut def_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();
        
        for block in &cfg.blocks {
            let mut uses = HashSet::new();
            let mut defs = HashSet::new();
            
            for stmt in &block.statements {
                let stmt_uses = self.get_used_variables(stmt);
                for var in stmt_uses {
                    if !defs.contains(&var) {
                        uses.insert(var);
                    }
                }
                
                if let Some(var) = self.get_defined_variable(stmt) {
                    defs.insert(var);
                }
            }
            
            use_sets.insert(block.id, uses);
            def_sets.insert(block.id, defs);
        }
        
        for block in &cfg.blocks {
            self.live_variables.insert(block.id, HashSet::new());
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for block in cfg.blocks.iter().rev() {
                let mut out_set = HashSet::new();
                
                for &succ in &block.successors {
                    if let Some(succ_in) = self.live_variables.get(&succ) {
                        out_set.extend(succ_in.iter().cloned());
                    }
                }
                
                let def = &def_sets[&block.id];
                let use_set = &use_sets[&block.id];
                
                let mut in_set: HashSet<_> = out_set
                    .into_iter()
                    .filter(|var| !def.contains(var))
                    .collect();
                in_set.extend(use_set.iter().cloned());
                
                if in_set != self.live_variables[&block.id] {
                    self.live_variables.insert(block.id, in_set);
                    changed = true;
                }
            }
        }
    }
    
    fn compute_available_expressions(&mut self, cfg: &ControlFlowGraph) {
        let mut gen_sets: HashMap<BlockId, HashSet<AvailableExpr>> = HashMap::new();
        let mut kill_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();
        
        for block in &cfg.blocks {
            let mut gen = HashSet::new();
            let mut kill = HashSet::new();
            
            for stmt in &block.statements {
                if let Some(var) = self.get_defined_variable(stmt) {
                    kill.insert(var);
                }
                
                for expr in self.get_expressions(stmt) {
                    gen.insert(AvailableExpr {
                        expr,
                        block: block.id,
                    });
                }
            }
            
            gen_sets.insert(block.id, gen);
            kill_sets.insert(block.id, kill);
        }
        
        for block in &cfg.blocks {
            if block.id == cfg.entry_block {
                self.available_expressions.insert(block.id, HashSet::new());
            } else {
                let all_exprs: HashSet<_> = gen_sets.values()
                    .flat_map(|s| s.iter().cloned())
                    .collect();
                self.available_expressions.insert(block.id, all_exprs);
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for block in &cfg.blocks {
                if block.id == cfg.entry_block {
                    continue;
                }
                
                let mut in_set = if let Some(&first_pred) = block.predecessors.iter().next() {
                    self.available_expressions[&first_pred].clone()
                } else {
                    HashSet::new()
                };
                
                for &pred in &block.predecessors {
                    let pred_out = &self.available_expressions[&pred];
                    in_set.retain(|e| pred_out.contains(e));
                }
                
                let kill = &kill_sets[&block.id];
                let gen = &gen_sets[&block.id];
                
                let mut out_set: HashSet<_> = in_set
                    .into_iter()
                    .filter(|e| !self.expr_uses_variable(&e.expr, kill))
                    .collect();
                out_set.extend(gen.iter().cloned());
                
                if out_set != self.available_expressions[&block.id] {
                    self.available_expressions.insert(block.id, out_set);
                    changed = true;
                }
            }
        }
    }
    
    fn get_defined_variable(&self, stmt: &Statement) -> Option<String> {
        match stmt {
            Statement::Let { name, .. } => Some(name.clone()),
            _ => None,
        }
    }
    
    fn get_used_variables(&self, stmt: &Statement) -> HashSet<String> {
        let mut vars = HashSet::new();
        
        match stmt {
            Statement::Let { value, .. } => {
                if let Some(expr) = value {
                    self.collect_expr_variables(expr, &mut vars);
                }
            }
            Statement::Return(Some(expr)) => {
                self.collect_expr_variables(expr, &mut vars);
            }
            Statement::Expression(expr) => {
                self.collect_expr_variables(expr, &mut vars);
            }
            Statement::While { condition, body } => {
                self.collect_expr_variables(condition, &mut vars);
                for stmt in body {
                    vars.extend(self.get_used_variables(stmt));
                }
            }
            Statement::If { condition, then_body, else_body } => {
                self.collect_expr_variables(condition, &mut vars);
                for stmt in then_body {
                    vars.extend(self.get_used_variables(stmt));
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        vars.extend(self.get_used_variables(stmt));
                    }
                }
            }
            _ => {}
        }
        
        vars
    }
    
    fn collect_expr_variables(&self, expr: &Expression, vars: &mut HashSet<String>) {
        match expr {
            Expression::Ident(name) | Expression::Identifier(name) => {
                vars.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.collect_expr_variables(left, vars);
                self.collect_expr_variables(right, vars);
            }
            Expression::Unary { expr, .. } => {
                self.collect_expr_variables(expr, vars);
            }
            Expression::Call { func, args } | Expression::CallAlt { callee: func, args } => {
                self.collect_expr_variables(func, vars);
                for arg in args {
                    self.collect_expr_variables(arg, vars);
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.collect_expr_variables(object, vars);
            }
            Expression::Block(stmts) => {
                for stmt in stmts {
                    vars.extend(self.get_used_variables(stmt));
                }
            }
            _ => {}
        }
    }
    
    fn get_expressions(&self, stmt: &Statement) -> Vec<String> {
        let mut exprs = Vec::new();
        
        match stmt {
            Statement::Let { value: Some(expr), .. } => {
                self.collect_expressions(expr, &mut exprs);
            }
            Statement::Expression(expr) => {
                self.collect_expressions(expr, &mut exprs);
            }
            _ => {}
        }
        
        exprs
    }
    
    fn collect_expressions(&self, expr: &Expression, exprs: &mut Vec<String>) {
        match expr {
            Expression::Binary { op, left, right } => {
                let expr_str = format!("{:?} {:?} {:?}", left, op, right);
                exprs.push(expr_str);
                self.collect_expressions(left, exprs);
                self.collect_expressions(right, exprs);
            }
            Expression::Unary { op, expr: inner } => {
                let expr_str = format!("{:?} {:?}", op, inner);
                exprs.push(expr_str);
                self.collect_expressions(inner, exprs);
            }
            _ => {}
        }
    }
    
    fn expr_uses_variable(&self, expr: &str, kill_set: &HashSet<String>) -> bool {
        kill_set.iter().any(|var| expr.contains(var))
    }
    
    pub fn get_reaching_definitions(&self, block: BlockId) -> Option<&HashSet<Definition>> {
        self.reaching_definitions.get(&block)
    }
    
    pub fn get_live_variables(&self, block: BlockId) -> Option<&HashSet<String>> {
        self.live_variables.get(&block)
    }
    
    pub fn get_available_expressions(&self, block: BlockId) -> Option<&HashSet<AvailableExpr>> {
        self.available_expressions.get(&block)
    }
    
    pub fn find_dead_stores(&self, cfg: &ControlFlowGraph) -> Vec<DeadStore> {
        let mut dead_stores = Vec::new();
        
        for block in &cfg.blocks {
            if let Some(live_vars) = self.live_variables.get(&block.id) {
                for (idx, stmt) in block.statements.iter().enumerate() {
                    if let Some(var) = self.get_defined_variable(stmt) {
                        if !live_vars.contains(&var) {
                            dead_stores.push(DeadStore {
                                variable: var,
                                block: block.id,
                                index: idx,
                            });
                        }
                    }
                }
            }
        }
        
        dead_stores
    }
}

#[derive(Debug, Clone)]
pub struct DeadStore {
    pub variable: String,
    pub block: BlockId,
    pub index: usize,
}

impl Default for DataFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
