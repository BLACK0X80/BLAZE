use crate::parser::{Program, Item, Function, Statement, Expression};
use crate::semantic::SymbolTable;
use anyhow::{Result, bail};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dominators;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct BorrowChecker {
    next_loan_id: usize,
    next_lifetime_id: usize,
    next_block_id: usize,
    cfg: DiGraph<BasicBlock, ControlFlowEdge>,
    dominator_tree: Option<dominators::Dominators<NodeIndex>>,
    loan_sets: HashMap<NodeIndex, HashSet<usize>>,
    gen_sets: HashMap<NodeIndex, HashSet<usize>>,
    kill_sets: HashMap<NodeIndex, HashSet<usize>>,
    active_loans: HashMap<String, Vec<Loan>>,
    all_loans: Vec<Loan>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub statements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ControlFlowEdge;

#[derive(Debug, Clone)]
pub struct Loan {
    pub id: usize,
    pub location: String,
    pub mutable: bool,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone)]
pub struct Lifetime {
    pub id: usize,
    pub scope: Scope,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub start: usize,
    pub end: usize,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            next_loan_id: 0,
            next_lifetime_id: 0,
            next_block_id: 0,
            cfg: DiGraph::new(),
            dominator_tree: None,
            loan_sets: HashMap::new(),
            gen_sets: HashMap::new(),
            kill_sets: HashMap::new(),
            active_loans: HashMap::new(),
            all_loans: Vec::new(),
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
        self.build_cfg(func)?;
        
        if let Some(entry_node) = self.cfg.node_indices().next() {
            self.dominator_tree = Some(dominators::simple_fast(&self.cfg, entry_node));
        }
        
        self.analyze_borrows_in_function(func)?;
        
        self.initialize_gen_kill_sets()?;
        
        self.compute_dataflow()?;
        self.check_loan_conflicts()?;
        
        Ok(())
    }

    fn initialize_gen_kill_sets(&mut self) -> Result<()> {
        for node in self.cfg.node_indices() {
            let mut gen_set = HashSet::new();
            let mut kill_set = HashSet::new();
            
            if let Some(block) = self.cfg.node_weight(node) {
                for stmt_desc in &block.statements {
                    for loan in &self.all_loans {
                        if stmt_desc.contains(&loan.location) {
                            gen_set.insert(loan.id);
                        }
                    }
                }
            }
            
            self.gen_sets.insert(node, gen_set);
            self.kill_sets.insert(node, kill_set);
        }
        
        Ok(())
    }

    fn analyze_borrows_in_function(&mut self, func: &Function) -> Result<()> {
        self.analyze_statements_with_scope(&func.body, 0)?;
        Ok(())
    }

    fn analyze_statements_with_scope(&mut self, statements: &[Statement], scope_start: usize) -> Result<usize> {
        let mut current_scope = scope_start;
        
        for stmt in statements {
            current_scope = self.analyze_statement_with_scope(stmt, current_scope)?;
        }
        
        Ok(current_scope)
    }

    fn analyze_statement_with_scope(&mut self, stmt: &Statement, scope_start: usize) -> Result<usize> {
        match stmt {
            Statement::Let { name, value, mutable, .. } => {
                let scope_end = scope_start + 1;
                self.analyze_expression_with_lifetime(value, name, *mutable, scope_start, scope_end)?;
                Ok(scope_end)
            }
            Statement::Expression(expr) => {
                let scope_end = scope_start + 1;
                self.analyze_expression_with_lifetime(expr, "<expr>", false, scope_start, scope_end)?;
                Ok(scope_end)
            }
            Statement::Return(Some(expr)) => {
                let scope_end = scope_start + 1;
                self.analyze_expression_with_lifetime(expr, "<return>", false, scope_start, scope_end)?;
                Ok(scope_end)
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_end = scope_start + 1;
                self.analyze_expression_with_lifetime(condition, "<if_cond>", false, scope_start, cond_end)?;
                
                let then_end = self.analyze_statements_with_scope(then_body, cond_end)?;
                let else_end = if let Some(else_stmts) = else_body {
                    self.analyze_statements_with_scope(else_stmts, cond_end)?
                } else {
                    cond_end
                };
                
                Ok(then_end.max(else_end))
            }
            Statement::While { condition, body } => {
                let cond_end = scope_start + 1;
                self.analyze_expression_with_lifetime(condition, "<while_cond>", false, scope_start, cond_end)?;
                
                let body_end = self.analyze_statements_with_scope(body, cond_end)?;
                Ok(body_end)
            }
            Statement::Return(None) => {
                Ok(scope_start + 1)
            }
        }
    }

    fn analyze_expression_with_lifetime(
        &mut self,
        expr: &Expression,
        location: &str,
        mutable: bool,
        scope_start: usize,
        scope_end: usize,
    ) -> Result<()> {
        match expr {
            Expression::Ident(name) => {
                let lifetime = self.new_lifetime(scope_start, scope_end);
                let loan = Loan {
                    id: self.next_loan_id,
                    location: name.clone(),
                    mutable,
                    lifetime,
                };
                self.next_loan_id += 1;
                
                self.active_loans.entry(location.to_string())
                    .or_insert_with(Vec::new)
                    .push(loan.clone());
                self.all_loans.push(loan);
            }
            
            Expression::Binary { left, right, .. } => {
                self.analyze_expression_with_lifetime(left, location, false, scope_start, scope_end)?;
                self.analyze_expression_with_lifetime(right, location, false, scope_start, scope_end)?;
            }
            
            Expression::Unary { expr, .. } => {
                self.analyze_expression_with_lifetime(expr, location, false, scope_start, scope_end)?;
            }
            
            Expression::Call { func, args } => {
                self.analyze_expression_with_lifetime(func, location, false, scope_start, scope_end)?;
                for arg in args {
                    self.analyze_expression_with_lifetime(arg, location, false, scope_start, scope_end)?;
                }
            }
            
            Expression::IntLit(_) | Expression::FloatLit(_) | 
            Expression::StringLit(_) | Expression::CharLit(_) | 
            Expression::BoolLit(_) => {
                // Literals don't create borrows
            }
        }
        Ok(())
    }

    fn build_cfg(&mut self, func: &Function) -> Result<()> {
        self.cfg.clear();
        self.next_block_id = 0;
        
        let entry_block = self.new_basic_block();
        let entry_node = self.cfg.add_node(entry_block);
        
        let exit_block = self.new_basic_block();
        let exit_node = self.cfg.add_node(exit_block);
        
        self.build_cfg_for_statements(&func.body, entry_node, exit_node)?;
        
        Ok(())
    }

    fn build_cfg_for_statements(
        &mut self,
        statements: &[Statement],
        entry: NodeIndex,
        exit: NodeIndex,
    ) -> Result<()> {
        if statements.is_empty() {
            self.cfg.add_edge(entry, exit, ControlFlowEdge);
            return Ok(());
        }

        let mut current = entry;
        
        for (i, stmt) in statements.iter().enumerate() {
            let is_last = i == statements.len() - 1;
            let next = if is_last {
                exit
            } else {
                let block = self.new_basic_block();
                self.cfg.add_node(block)
            };
            
            current = self.build_cfg_for_statement(stmt, current, next)?;
        }
        
        Ok(())
    }

    fn build_cfg_for_statement(
        &mut self,
        stmt: &Statement,
        entry: NodeIndex,
        exit: NodeIndex,
    ) -> Result<NodeIndex> {
        match stmt {
            Statement::Let { name, value, .. } => {
                if let Some(block) = self.cfg.node_weight_mut(entry) {
                    block.statements.push(format!("let {}", name));
                }
                self.cfg.add_edge(entry, exit, ControlFlowEdge);
                Ok(exit)
            }
            
            Statement::Expression(_) => {
                if let Some(block) = self.cfg.node_weight_mut(entry) {
                    block.statements.push("expression".to_string());
                }
                self.cfg.add_edge(entry, exit, ControlFlowEdge);
                Ok(exit)
            }
            
            Statement::Return(_) => {
                if let Some(block) = self.cfg.node_weight_mut(entry) {
                    block.statements.push("return".to_string());
                }
                self.cfg.add_edge(entry, exit, ControlFlowEdge);
                Ok(exit)
            }
            
            Statement::If { condition: _, then_body, else_body } => {
                let then_entry = self.new_basic_block();
                let then_entry_node = self.cfg.add_node(then_entry);
                
                let else_entry = self.new_basic_block();
                let else_entry_node = self.cfg.add_node(else_entry);
                
                let merge = self.new_basic_block();
                let merge_node = self.cfg.add_node(merge);
                
                self.cfg.add_edge(entry, then_entry_node, ControlFlowEdge);
                self.cfg.add_edge(entry, else_entry_node, ControlFlowEdge);
                
                self.build_cfg_for_statements(then_body, then_entry_node, merge_node)?;
                
                if let Some(else_stmts) = else_body {
                    self.build_cfg_for_statements(else_stmts, else_entry_node, merge_node)?;
                } else {
                    self.cfg.add_edge(else_entry_node, merge_node, ControlFlowEdge);
                }
                
                self.cfg.add_edge(merge_node, exit, ControlFlowEdge);
                Ok(exit)
            }
            
            Statement::While { condition: _, body } => {
                let loop_header = self.new_basic_block();
                let loop_header_node = self.cfg.add_node(loop_header);
                
                let loop_body_entry = self.new_basic_block();
                let loop_body_entry_node = self.cfg.add_node(loop_body_entry);
                
                let loop_exit = self.new_basic_block();
                let loop_exit_node = self.cfg.add_node(loop_exit);
                
                self.cfg.add_edge(entry, loop_header_node, ControlFlowEdge);
                
                self.cfg.add_edge(loop_header_node, loop_body_entry_node, ControlFlowEdge);
                self.cfg.add_edge(loop_header_node, loop_exit_node, ControlFlowEdge);
                
                self.build_cfg_for_statements(body, loop_body_entry_node, loop_header_node)?;
                
                self.cfg.add_edge(loop_exit_node, exit, ControlFlowEdge);
                Ok(exit)
            }
        }
    }

    fn compute_dataflow(&mut self) -> Result<()> {
        let mut worklist: VecDeque<NodeIndex> = VecDeque::new();
        let mut in_worklist: HashSet<NodeIndex> = HashSet::new();
        
        for node in self.cfg.node_indices() {
            worklist.push_back(node);
            in_worklist.insert(node);
            self.loan_sets.insert(node, HashSet::new());
        }
        
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 1000;
        
        while let Some(node) = worklist.pop_front() {
            in_worklist.remove(&node);
            iteration += 1;
            
            if iteration > MAX_ITERATIONS {
                bail!("Dataflow analysis did not converge after {} iterations", MAX_ITERATIONS);
            }
            
            let mut in_set = HashSet::new();
            
            for edge in self.cfg.edges_directed(node, petgraph::Direction::Incoming) {
                let pred = edge.source();
                if let Some(pred_out) = self.loan_sets.get(&pred) {
                    in_set.extend(pred_out.iter().copied());
                }
            }
            
            let gen = self.gen_sets.get(&node).cloned().unwrap_or_default();
            let kill = self.kill_sets.get(&node).cloned().unwrap_or_default();
            
            let mut out_set = in_set.clone();
            out_set.extend(&gen);
            for killed in &kill {
                out_set.remove(killed);
            }
            
            let old_out = self.loan_sets.get(&node).cloned().unwrap_or_default();
            if out_set != old_out {
                self.loan_sets.insert(node, out_set);
                
                for edge in self.cfg.edges_directed(node, petgraph::Direction::Outgoing) {
                    let succ = edge.target();
                    if !in_worklist.contains(&succ) {
                        worklist.push_back(succ);
                        in_worklist.insert(succ);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn check_loan_conflicts(&self) -> Result<()> {
        for node in self.cfg.node_indices() {
            if let Some(loans) = self.loan_sets.get(&node) {
                let loan_vec: Vec<_> = loans.iter().copied().collect();
                for i in 0..loan_vec.len() {
                    for j in (i + 1)..loan_vec.len() {
                        if self.loans_conflict(loan_vec[i], loan_vec[j]) {
                            let loan1 = self.get_loan_by_id(loan_vec[i]).unwrap();
                            let loan2 = self.get_loan_by_id(loan_vec[j]).unwrap();
                            
                            self.report_borrow_conflict(&loan1, &loan2)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn report_borrow_conflict(&self, loan1: &Loan, loan2: &Loan) -> Result<()> {
        let (first_loan, second_loan) = if loan1.lifetime.scope.start < loan2.lifetime.scope.start {
            (loan1, loan2)
        } else {
            (loan2, loan1)
        };

        let error_msg = if first_loan.mutable && second_loan.mutable {
            format!(
                "cannot borrow `{}` as mutable more than once at a time\n\
                 first mutable borrow occurs at scope {}-{}\n\
                 second mutable borrow occurs at scope {}-{}\n\
                 \n\
                 help: consider using the first mutable borrow throughout, or ensure borrows don't overlap",
                first_loan.location,
                first_loan.lifetime.scope.start,
                first_loan.lifetime.scope.end,
                second_loan.lifetime.scope.start,
                second_loan.lifetime.scope.end
            )
        } else if first_loan.mutable || second_loan.mutable {
            let (mutable_loan, immutable_loan) = if first_loan.mutable {
                (first_loan, second_loan)
            } else {
                (second_loan, first_loan)
            };
            
            format!(
                "cannot borrow `{}` as {} because it is also borrowed as {}\n\
                 {} borrow occurs at scope {}-{}\n\
                 {} borrow occurs at scope {}-{}\n\
                 \n\
                 help: {} borrow must end before {} borrow begins",
                mutable_loan.location,
                if mutable_loan == first_loan { "mutable" } else { "immutable" },
                if mutable_loan == first_loan { "immutable" } else { "mutable" },
                if immutable_loan.mutable { "mutable" } else { "immutable" },
                immutable_loan.lifetime.scope.start,
                immutable_loan.lifetime.scope.end,
                if mutable_loan.mutable { "mutable" } else { "immutable" },
                mutable_loan.lifetime.scope.start,
                mutable_loan.lifetime.scope.end,
                if first_loan == immutable_loan { "first" } else { "second" },
                if first_loan == mutable_loan { "first" } else { "second" }
            )
        } else {
            format!(
                "internal error: conflict detected between two immutable borrows of `{}`",
                first_loan.location
            )
        };

        bail!("{}", error_msg)
    }

    fn loans_conflict(&self, loan1_id: usize, loan2_id: usize) -> bool {
        let loan1 = match self.get_loan_by_id(loan1_id) {
            Some(l) => l,
            None => return false,
        };

        let loan2 = match self.get_loan_by_id(loan2_id) {
            Some(l) => l,
            None => return false,
        };

        if loan1.location != loan2.location {
            return false;
        }

        if loan1.mutable || loan2.mutable {
            if Self::lifetimes_overlap(&loan1.lifetime, &loan2.lifetime) {
                return true;
            }
        }

        false
    }

    fn lifetimes_overlap(l1: &Lifetime, l2: &Lifetime) -> bool {
        !(l1.scope.end <= l2.scope.start || l2.scope.end <= l1.scope.start)
    }

    fn get_loan_by_id(&self, id: usize) -> Option<Loan> {
        self.all_loans.iter().find(|l| l.id == id).cloned()
    }

    fn new_basic_block(&mut self) -> BasicBlock {
        let id = self.next_block_id;
        self.next_block_id += 1;
        BasicBlock {
            id,
            statements: Vec::new(),
        }
    }

    fn new_lifetime(&mut self, start: usize, end: usize) -> Lifetime {
        let id = self.next_lifetime_id;
        self.next_lifetime_id += 1;
        Lifetime {
            id,
            scope: Scope { start, end },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expression, BinaryOp};

    fn create_test_symbol_table() -> SymbolTable {
        SymbolTable::new()
    }

    #[test]
    fn test_cfg_construction_simple() {
        let mut checker = BorrowChecker::new();
        let func = Function {
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
                Statement::Return(Some(Expression::Ident("x".to_string()))),
            ],
        };

        let result = checker.build_cfg(&func);
        assert!(result.is_ok());
        assert!(checker.cfg.node_count() >= 2);
    }

    #[test]
    fn test_cfg_construction_if_statement() {
        let mut checker = BorrowChecker::new();
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::If {
                    condition: Expression::BoolLit(true),
                    then_body: vec![
                        Statement::Expression(Expression::IntLit(1)),
                    ],
                    else_body: Some(vec![
                        Statement::Expression(Expression::IntLit(2)),
                    ]),
                },
            ],
        };

        let result = checker.build_cfg(&func);
        assert!(result.is_ok());
        assert!(checker.cfg.node_count() >= 5);
    }

    #[test]
    fn test_cfg_construction_while_loop() {
        let mut checker = BorrowChecker::new();
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::While {
                    condition: Expression::BoolLit(true),
                    body: vec![
                        Statement::Expression(Expression::IntLit(1)),
                    ],
                },
            ],
        };

        let result = checker.build_cfg(&func);
        assert!(result.is_ok());
        assert!(checker.cfg.node_count() >= 4);
    }

    #[test]
    fn test_loan_tracking_simple() {
        let mut checker = BorrowChecker::new();
        let func = Function {
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
                Statement::Let {
                    name: "y".to_string(),
                    mutable: false,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
            ],
        };

        let result = checker.analyze_borrows_in_function(&func);
        assert!(result.is_ok());
        assert!(checker.all_loans.len() > 0);
    }

    #[test]
    fn test_loan_tracking_binary_expression() {
        let mut checker = BorrowChecker::new();
        let func = Function {
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
                        left: Box::new(Expression::Ident("x".to_string())),
                        right: Box::new(Expression::Ident("y".to_string())),
                    },
                },
            ],
        };

        let result = checker.analyze_borrows_in_function(&func);
        assert!(result.is_ok());
        assert!(checker.all_loans.len() >= 2);
    }

    #[test]
    fn test_no_conflict_immutable_borrows() {
        let mut checker = BorrowChecker::new();
        let symbol_table = create_test_symbol_table();
        
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "y".to_string(),
                    mutable: false,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
                Statement::Let {
                    name: "z".to_string(),
                    mutable: false,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
            ],
        };

        let result = checker.check_function(&func, &symbol_table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_mutable_and_immutable() {
        let mut checker = BorrowChecker::new();
        let symbol_table = create_test_symbol_table();
        
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "y".to_string(),
                    mutable: false,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
                Statement::Let {
                    name: "z".to_string(),
                    mutable: true,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
            ],
        };

        let result = checker.check_function(&func, &symbol_table);
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("borrow") || error_msg.contains("conflict"));
        }
    }

    #[test]
    fn test_conflict_multiple_mutable() {
        let mut checker = BorrowChecker::new();
        let symbol_table = create_test_symbol_table();
        
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::Let {
                    name: "y".to_string(),
                    mutable: true,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
                Statement::Let {
                    name: "z".to_string(),
                    mutable: true,
                    ty: None,
                    value: Expression::Ident("x".to_string()),
                },
            ],
        };

        let result = checker.check_function(&func, &symbol_table);
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("mutable"));
        }
    }

    #[test]
    fn test_dataflow_convergence() {
        let mut checker = BorrowChecker::new();
        let func = Function {
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
                Statement::While {
                    condition: Expression::BoolLit(true),
                    body: vec![
                        Statement::Let {
                            name: "y".to_string(),
                            mutable: false,
                            ty: None,
                            value: Expression::Ident("x".to_string()),
                        },
                    ],
                },
            ],
        };

        checker.build_cfg(&func).unwrap();
        checker.analyze_borrows_in_function(&func).unwrap();
        checker.initialize_gen_kill_sets().unwrap();
        
        let result = checker.compute_dataflow();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_control_flow() {
        let mut checker = BorrowChecker::new();
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: vec![
                Statement::If {
                    condition: Expression::BoolLit(true),
                    then_body: vec![
                        Statement::While {
                            condition: Expression::BoolLit(true),
                            body: vec![
                                Statement::Expression(Expression::IntLit(1)),
                            ],
                        },
                    ],
                    else_body: None,
                },
            ],
        };

        let result = checker.build_cfg(&func);
        assert!(result.is_ok());
        assert!(checker.cfg.node_count() >= 6);
    }
}
