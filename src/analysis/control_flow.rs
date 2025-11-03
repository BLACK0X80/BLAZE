use crate::parser::{Function, Statement, Expression};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
    pub exit_blocks: HashSet<BlockId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(usize);

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub statements: Vec<Statement>,
    pub predecessors: HashSet<BlockId>,
    pub successors: HashSet<BlockId>,
    pub dominators: HashSet<BlockId>,
    pub immediate_dominator: Option<BlockId>,
}

pub struct CFGBuilder {
    blocks: Vec<BasicBlock>,
    current_block: BlockId,
    next_block_id: usize,
}

impl CFGBuilder {
    pub fn new() -> Self {
        let entry_block = BasicBlock {
            id: BlockId(0),
            statements: Vec::new(),
            predecessors: HashSet::new(),
            successors: HashSet::new(),
            dominators: HashSet::new(),
            immediate_dominator: None,
        };
        
        Self {
            blocks: vec![entry_block],
            current_block: BlockId(0),
            next_block_id: 1,
        }
    }
    
    pub fn build_from_function(&mut self, function: &Function) -> ControlFlowGraph {
        for stmt in &function.body {
            self.process_statement(stmt);
        }
        
        let exit_blocks = self.find_exit_blocks();
        self.compute_dominators();
        
        ControlFlowGraph {
            blocks: self.blocks.clone(),
            entry_block: BlockId(0),
            exit_blocks,
        }
    }
    
    fn process_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::If { condition, then_body, else_body } => {
                let then_block = self.create_block();
                let else_block = self.create_block();
                let merge_block = self.create_block();
                
                self.add_edge(self.current_block, then_block);
                self.add_edge(self.current_block, else_block);
                
                self.current_block = then_block;
                for stmt in then_body {
                    self.process_statement(stmt);
                }
                self.add_edge(then_block, merge_block);
                
                if let Some(else_stmts) = else_body {
                    self.current_block = else_block;
                    for stmt in else_stmts {
                        self.process_statement(stmt);
                    }
                    self.add_edge(else_block, merge_block);
                } else {
                    self.add_edge(else_block, merge_block);
                }
                
                self.current_block = merge_block;
            }
            
            Statement::While { condition, body } => {
                let header_block = self.create_block();
                let body_block = self.create_block();
                let exit_block = self.create_block();
                
                self.add_edge(self.current_block, header_block);
                self.add_edge(header_block, body_block);
                self.add_edge(header_block, exit_block);
                
                self.current_block = body_block;
                for stmt in body {
                    self.process_statement(stmt);
                }
                self.add_edge(body_block, header_block);
                
                self.current_block = exit_block;
            }
            
            Statement::For { variable, iterable, body } => {
                let init_block = self.create_block();
                let condition_block = self.create_block();
                let body_block = self.create_block();
                let increment_block = self.create_block();
                let exit_block = self.create_block();
                
                self.add_edge(self.current_block, init_block);
                self.add_edge(init_block, condition_block);
                self.add_edge(condition_block, body_block);
                self.add_edge(condition_block, exit_block);
                
                self.current_block = body_block;
                for stmt in body {
                    self.process_statement(stmt);
                }
                self.add_edge(body_block, increment_block);
                self.add_edge(increment_block, condition_block);
                
                self.current_block = exit_block;
            }
            
            Statement::Loop { body } => {
                let header_block = self.create_block();
                let body_block = self.create_block();
                
                self.add_edge(self.current_block, header_block);
                self.add_edge(header_block, body_block);
                
                self.current_block = body_block;
                for stmt in body {
                    self.process_statement(stmt);
                }
                self.add_edge(body_block, header_block);
            }
            
            Statement::Break(_) => {
                let exit_block = self.create_block();
                self.add_edge(self.current_block, exit_block);
                self.current_block = exit_block;
            }
            
            Statement::Continue => {
            }
            
            Statement::Return(_) => {
                let return_block = self.create_block();
                self.add_edge(self.current_block, return_block);
                self.current_block = return_block;
            }
            
            Statement::Block(stmts) => {
                for stmt in stmts {
                    self.process_statement(stmt);
                }
            }
            
            _ => {
                self.blocks[self.current_block.0].statements.push(stmt.clone());
            }
        }
    }
    
    fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        
        let block = BasicBlock {
            id,
            statements: Vec::new(),
            predecessors: HashSet::new(),
            successors: HashSet::new(),
            dominators: HashSet::new(),
            immediate_dominator: None,
        };
        
        self.blocks.push(block);
        id
    }
    
    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.blocks[from.0].successors.insert(to);
        self.blocks[to.0].predecessors.insert(from);
    }
    
    fn find_exit_blocks(&self) -> HashSet<BlockId> {
        self.blocks
            .iter()
            .filter(|b| b.successors.is_empty())
            .map(|b| b.id)
            .collect()
    }
    
    fn compute_dominators(&mut self) {
        let n = self.blocks.len();
        let entry = BlockId(0);
        
        for block in &mut self.blocks {
            if block.id == entry {
                block.dominators.insert(entry);
            } else {
                for i in 0..n {
                    block.dominators.insert(BlockId(i));
                }
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for i in 1..n {
                let block_id = BlockId(i);
                let mut new_doms = HashSet::new();
                
                if let Some(&first_pred) = self.blocks[i].predecessors.iter().next() {
                    new_doms = self.blocks[first_pred.0].dominators.clone();
                    
                    for &pred in &self.blocks[i].predecessors {
                        let pred_doms = &self.blocks[pred.0].dominators;
                        new_doms.retain(|d| pred_doms.contains(d));
                    }
                }
                
                new_doms.insert(block_id);
                
                if new_doms != self.blocks[i].dominators {
                    self.blocks[i].dominators = new_doms;
                    changed = true;
                }
            }
        }
        
        self.compute_immediate_dominators();
    }
    
    fn compute_immediate_dominators(&mut self) {
        for i in 1..self.blocks.len() {
            let block_id = BlockId(i);
            let dominators: Vec<_> = self.blocks[i]
                .dominators
                .iter()
                .filter(|&&d| d != block_id)
                .copied()
                .collect();
            
            for &dom in &dominators {
                let mut is_immediate = true;
                
                for &other_dom in &dominators {
                    if other_dom != dom && self.blocks[other_dom.0].dominators.contains(&dom) {
                        is_immediate = false;
                        break;
                    }
                }
                
                if is_immediate {
                    self.blocks[i].immediate_dominator = Some(dom);
                    break;
                }
            }
        }
    }
}

impl ControlFlowGraph {
    pub fn compute_post_dominators(&self) -> HashMap<BlockId, HashSet<BlockId>> {
        let mut post_doms: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let all_blocks: HashSet<BlockId> = self.blocks.iter().map(|b| b.id).collect();
        
        for block in &self.blocks {
            if self.exit_blocks.contains(&block.id) {
                post_doms.insert(block.id, [block.id].iter().copied().collect());
            } else {
                post_doms.insert(block.id, all_blocks.clone());
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for block in &self.blocks {
                if self.exit_blocks.contains(&block.id) {
                    continue;
                }
                
                let mut new_post_doms = HashSet::new();
                
                if !block.successors.is_empty() {
                    let mut iter = block.successors.iter();
                    if let Some(&first_succ) = iter.next() {
                        new_post_doms = post_doms[&first_succ].clone();
                        
                        for &succ in iter {
                            new_post_doms.retain(|d| post_doms[&succ].contains(d));
                        }
                    }
                }
                
                new_post_doms.insert(block.id);
                
                if new_post_doms != post_doms[&block.id] {
                    post_doms.insert(block.id, new_post_doms);
                    changed = true;
                }
            }
        }
        
        post_doms
    }
    
    pub fn find_natural_loops(&self) -> Vec<NaturalLoop> {
        let mut loops = Vec::new();
        
        for block in &self.blocks {
            for &succ in &block.successors {
                if block.dominators.contains(&succ) {
                    let loop_body = self.find_loop_body(block.id, succ);
                    loops.push(NaturalLoop {
                        header: succ,
                        back_edge: block.id,
                        body: loop_body,
                    });
                }
            }
        }
        
        loops
    }
    
    fn find_loop_body(&self, back_edge: BlockId, header: BlockId) -> HashSet<BlockId> {
        let mut body = HashSet::new();
        body.insert(header);
        body.insert(back_edge);
        
        let mut worklist = VecDeque::new();
        worklist.push_back(back_edge);
        
        while let Some(node) = worklist.pop_front() {
            for &pred in &self.blocks[node.0].predecessors {
                if !body.contains(&pred) {
                    body.insert(pred);
                    worklist.push_back(pred);
                }
            }
        }
        
        body
    }
}

#[derive(Debug, Clone)]
pub struct NaturalLoop {
    pub header: BlockId,
    pub back_edge: BlockId,
    pub body: HashSet<BlockId>,
}

impl Default for CFGBuilder {
    fn default() -> Self {
        Self::new()
    }
}
