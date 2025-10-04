use std::collections::{HashMap, HashSet, VecDeque};
use crate::parser::{Program, Statement, Expression, Type};
use crate::semantic::SymbolTable;
use crate::error::BorrowError;
use anyhow::Result;
use petgraph::{Graph, Directed, NodeIndex};
use petgraph::algo::dominators;

#[derive(Debug, Clone, PartialEq)]
pub struct Loan {
    pub id: usize,
    pub location: String,
    pub mutable: bool,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub id: usize,
    pub name: String,
    pub scope: Scope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub start: usize,
    pub end: usize,
    pub variables: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct BorrowState {
    pub loans: HashMap<String, Vec<Loan>>,
    pub moved: HashSet<String>,
    pub conflicts: Vec<(Loan, Loan)>,
}

pub struct BorrowChecker {
    next_loan_id: usize,
    next_lifetime_id: usize,
    cfg: Graph<BasicBlock, ControlFlowEdge, Directed>,
    dominator_tree: Option<dominators::DominatorTree<NodeIndex>>,
    loan_sets: HashMap<NodeIndex, HashSet<usize>>,
    gen_sets: HashMap<NodeIndex, HashSet<usize>>,
    kill_sets: HashMap<NodeIndex, HashSet<usize>>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: NodeIndex,
    pub statements: Vec<Statement>,
    pub predecessors: Vec<NodeIndex>,
    pub successors: Vec<NodeIndex>,
}

#[derive(Debug, Clone)]
pub enum ControlFlowEdge {
    True,
    False,
    Unconditional,
    Loop,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            next_loan_id: 0,
            next_lifetime_id: 0,
            cfg: Graph::new(),
            dominator_tree: None,
            loan_sets: HashMap::new(),
            gen_sets: HashMap::new(),
            kill_sets: HashMap::new(),
        }
    }

    pub fn check(&mut self, program: &Program, symbol_table: &SymbolTable) -> Result<()> {
        self.build_cfg(program)?;
        self.compute_dominators();
        self.compute_gen_kill_sets();
        self.dataflow_analysis();
        self.check_conflicts()?;
        Ok(())
    }

    fn build_cfg(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            if let crate::parser::Item::Function(func) = item {
                self.build_function_cfg(func)?;
            }
        }
        Ok(())
    }

    fn build_function_cfg(&mut self, func: &crate::parser::Function) -> Result<()> {
        let mut blocks = Vec::new();
        let mut current_block = BasicBlock {
            id: NodeIndex::new(0),
            statements: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        for stmt in &func.body {
            match stmt {
                Statement::While { condition, body } => {
                    if !current_block.statements.is_empty() {
                        let block_id = self.cfg.add_node(current_block);
                        blocks.push(block_id);
                        current_block = BasicBlock {
                            id: block_id,
                            statements: Vec::new(),
                            predecessors: Vec::new(),
                            successors: Vec::new(),
                        };
                    }

                    let condition_block = self.cfg.add_node(BasicBlock {
                        id: NodeIndex::new(0),
                        statements: vec![Statement::Expression(condition.clone())],
                        predecessors: Vec::new(),
                        successors: Vec::new(),
                    });

                    let body_blocks = self.build_statement_cfg(body)?;
                    let exit_block = self.cfg.add_node(BasicBlock {
                        id: NodeIndex::new(0),
                        statements: Vec::new(),
                        predecessors: Vec::new(),
                        successors: Vec::new(),
                    });

                    self.cfg.add_edge(condition_block, body_blocks[0], ControlFlowEdge::True);
                    self.cfg.add_edge(condition_block, exit_block, ControlFlowEdge::False);
                    self.cfg.add_edge(body_blocks[body_blocks.len() - 1], condition_block, ControlFlowEdge::Loop);

                    blocks.push(condition_block);
                    blocks.extend(body_blocks);
                    blocks.push(exit_block);
                }
                Statement::If { condition, then_branch, else_branch } => {
                    if !current_block.statements.is_empty() {
                        let block_id = self.cfg.add_node(current_block);
                        blocks.push(block_id);
                        current_block = BasicBlock {
                            id: block_id,
                            statements: Vec::new(),
                            predecessors: Vec::new(),
                            successors: Vec::new(),
                        };
                    }

                    let condition_block = self.cfg.add_node(BasicBlock {
                        id: NodeIndex::new(0),
                        statements: vec![Statement::Expression(condition.clone())],
                        predecessors: Vec::new(),
                        successors: Vec::new(),
                    });

                    let then_blocks = self.build_statement_cfg(then_branch)?;
                    let else_blocks = if let Some(else_branch) = else_branch {
                        self.build_statement_cfg(&[else_branch.clone()])?
                    } else {
                        Vec::new()
                    };

                    let exit_block = self.cfg.add_node(BasicBlock {
                        id: NodeIndex::new(0),
                        statements: Vec::new(),
                        predecessors: Vec::new(),
                        successors: Vec::new(),
                    });

                    self.cfg.add_edge(condition_block, then_blocks[0], ControlFlowEdge::True);
                    if !else_blocks.is_empty() {
                        self.cfg.add_edge(condition_block, else_blocks[0], ControlFlowEdge::False);
                    } else {
                        self.cfg.add_edge(condition_block, exit_block, ControlFlowEdge::False);
                    }

                    blocks.push(condition_block);
                    blocks.extend(then_blocks);
                    blocks.extend(else_blocks);
                    blocks.push(exit_block);
                }
                _ => {
                    current_block.statements.push(stmt.clone());
                }
            }
        }

        if !current_block.statements.is_empty() {
            let block_id = self.cfg.add_node(current_block);
            blocks.push(block_id);
        }

        Ok(())
    }

    fn build_statement_cfg(&mut self, statements: &[Statement]) -> Result<Vec<NodeIndex>> {
        let mut blocks = Vec::new();
        let mut current_block = BasicBlock {
            id: NodeIndex::new(0),
            statements: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        for stmt in statements {
            current_block.statements.push(stmt.clone());
        }

        if !current_block.statements.is_empty() {
            let block_id = self.cfg.add_node(current_block);
            blocks.push(block_id);
        }

        Ok(blocks)
    }

    fn compute_dominators(&mut self) {
        if let Some(root) = self.cfg.node_indices().next() {
            self.dominator_tree = Some(dominators::simple_fast(&self.cfg, root));
        }
    }

    fn compute_gen_kill_sets(&mut self) {
        for node in self.cfg.node_indices() {
            let mut gen = HashSet::new();
            let mut kill = HashSet::new();

            if let Some(block) = self.cfg.node_weight(node) {
                for stmt in &block.statements {
                    self.analyze_statement(stmt, &mut gen, &mut kill);
                }
            }

            self.gen_sets.insert(node, gen);
            self.kill_sets.insert(node, kill);
        }
    }

    fn analyze_statement(&self, stmt: &Statement, gen: &mut HashSet<usize>, kill: &mut HashSet<usize>) {
        match stmt {
            Statement::Let { name, value, mutable, .. } => {
                if let Some(value_expr) = value {
                    self.analyze_expression(value_expr, gen, kill);
                }
                
                if *mutable {
                    kill.insert(self.next_loan_id);
                    self.next_loan_id += 1;
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr, gen, kill);
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.analyze_expression(expr, gen, kill);
                }
            }
            _ => {}
        }
    }

    fn analyze_expression(&self, expr: &Expression, gen: &mut HashSet<usize>, kill: &mut HashSet<usize>) {
        match expr {
            Expression::Reference { mutable, expression } => {
                self.analyze_expression(expression, gen, kill);
                gen.insert(self.next_loan_id);
                self.next_loan_id += 1;
            }
            Expression::Dereference(expr) => {
                self.analyze_expression(expr, gen, kill);
            }
            Expression::Call { callee, args } => {
                self.analyze_expression(callee, gen, kill);
                for arg in args {
                    self.analyze_expression(arg, gen, kill);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left, gen, kill);
                self.analyze_expression(right, gen, kill);
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expression(operand, gen, kill);
            }
            _ => {}
        }
    }

    fn dataflow_analysis(&mut self) {
        let mut worklist: VecDeque<NodeIndex> = self.cfg.node_indices().collect();
        
        while let Some(node) = worklist.pop_front() {
            let mut in_set = HashSet::new();
            
            for pred in self.cfg.neighbors_directed(node, petgraph::Direction::Incoming) {
                if let Some(pred_loans) = self.loan_sets.get(&pred) {
                    in_set.extend(pred_loans);
                }
            }
            
            let gen = self.gen_sets.get(&node).cloned().unwrap_or_default();
            let kill = self.kill_sets.get(&node).cloned().unwrap_or_default();
            
            let mut out_set = in_set.clone();
            out_set.retain(|&loan_id| !kill.contains(&loan_id));
            out_set.extend(gen);
            
            let changed = self.loan_sets.get(&node).map_or(true, |current| current != &out_set);
            
            if changed {
                self.loan_sets.insert(node, out_set);
                for succ in self.cfg.neighbors_directed(node, petgraph::Direction::Outgoing) {
                    worklist.push_back(succ);
                }
            }
        }
    }

    fn check_conflicts(&self) -> Result<()> {
        for node in self.cfg.node_indices() {
            if let Some(loans) = self.loan_sets.get(&node) {
                let mut loan_list: Vec<usize> = loans.iter().cloned().collect();
                
                for i in 0..loan_list.len() {
                    for j in (i + 1)..loan_list.len() {
                        if self.loans_conflict(loan_list[i], loan_list[j]) {
                            return Err(BorrowError::ConflictingBorrows {
                                first: format!("loan_{}", loan_list[i]),
                                second: format!("loan_{}", loan_list[j]),
                            }.into());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn loans_conflict(&self, loan1: usize, loan2: usize) -> bool {
        loan1 != loan2
    }
}