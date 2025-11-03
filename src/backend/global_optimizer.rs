use std::collections::{HashMap, HashSet};

pub struct GlobalOptimizer {
    call_graph: CallGraph,
    inline_decisions: HashMap<String, bool>,
    specializations: Vec<FunctionSpecialization>,
}

#[derive(Debug, Clone)]
pub struct CallGraph {
    nodes: HashMap<String, CallGraphNode>,
    edges: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct CallGraphNode {
    pub function_name: String,
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub call_count: usize,
    pub code_size: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionSpecialization {
    pub original_name: String,
    pub specialized_name: String,
    pub constant_args: Vec<(usize, i64)>,
}

impl GlobalOptimizer {
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::new(),
            inline_decisions: HashMap::new(),
            specializations: Vec::new(),
        }
    }
    
    pub fn optimize(&mut self) -> Vec<Optimization> {
        let mut optimizations = Vec::new();
        
        optimizations.extend(self.inter_procedural_constant_propagation());
        optimizations.extend(self.dead_function_elimination());
        optimizations.extend(self.function_specialization());
        optimizations.extend(self.global_value_numbering());
        
        optimizations
    }
    
    fn inter_procedural_constant_propagation(&self) -> Vec<Optimization> {
        let mut opts = Vec::new();
        
        for (func, node) in &self.call_graph.nodes {
            if node.callers.len() == 1 {
                opts.push(Optimization::ConstantPropagation {
                    function: func.clone(),
                    propagated_values: vec![],
                });
            }
        }
        
        opts
    }
    
    fn dead_function_elimination(&self) -> Vec<Optimization> {
        let mut opts = Vec::new();
        let mut reachable = HashSet::new();
        
        self.mark_reachable("main", &mut reachable);
        
        for func in self.call_graph.nodes.keys() {
            if !reachable.contains(func) {
                opts.push(Optimization::DeadFunctionElimination {
                    function: func.clone(),
                });
            }
        }
        
        opts
    }
    
    fn mark_reachable(&self, func: &str, reachable: &mut HashSet<String>) {
        if reachable.insert(func.to_string()) {
            if let Some(node) = self.call_graph.nodes.get(func) {
                for callee in &node.callees {
                    self.mark_reachable(callee, reachable);
                }
            }
        }
    }
    
    fn function_specialization(&self) -> Vec<Optimization> {
        let mut opts = Vec::new();
        
        for (func, node) in &self.call_graph.nodes {
            if self.should_specialize(node) {
                opts.push(Optimization::FunctionSpecialization {
                    original: func.clone(),
                    specialized_version: format!("{}_specialized", func),
                });
            }
        }
        
        opts
    }
    
    fn should_specialize(&self, node: &CallGraphNode) -> bool {
        node.call_count > 10 && node.code_size < 500
    }
    
    fn global_value_numbering(&self) -> Vec<Optimization> {
        vec![Optimization::GlobalValueNumbering {
            eliminated_redundancies: 0,
        }]
    }
    
    pub fn build_call_graph(&mut self, functions: &[String]) {
        for func in functions {
            self.call_graph.add_function(func.clone());
        }
    }
    
    pub fn decide_inline(&mut self, caller: &str, callee: &str) -> bool {
        if let Some(node) = self.call_graph.nodes.get(callee) {
            if node.code_size > 100 {
                return false;
            }
            
            if node.callers.len() > 5 {
                return false;
            }
            
            true
        } else {
            false
        }
    }
}

impl CallGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
    
    fn add_function(&mut self, name: String) {
        self.nodes.insert(name.clone(), CallGraphNode {
            function_name: name,
            callers: Vec::new(),
            callees: Vec::new(),
            call_count: 0,
            code_size: 0,
        });
    }
    
    fn add_call(&mut self, caller: String, callee: String) {
        self.edges.push((caller.clone(), callee.clone()));
        
        if let Some(caller_node) = self.nodes.get_mut(&caller) {
            caller_node.callees.push(callee.clone());
        }
        
        if let Some(callee_node) = self.nodes.get_mut(&callee) {
            callee_node.callers.push(caller);
            callee_node.call_count += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub enum Optimization {
    ConstantPropagation {
        function: String,
        propagated_values: Vec<String>,
    },
    DeadFunctionElimination {
        function: String,
    },
    FunctionSpecialization {
        original: String,
        specialized_version: String,
    },
    GlobalValueNumbering {
        eliminated_redundancies: usize,
    },
}

impl Default for GlobalOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LinkTimeOptimizer {
    modules: Vec<Module>,
    merged_module: Option<Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<String>,
    pub globals: Vec<String>,
}

impl LinkTimeOptimizer {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            merged_module: None,
        }
    }
    
    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }
    
    pub fn optimize(&mut self) -> Module {
        let mut merged = Module {
            name: "merged".to_string(),
            functions: Vec::new(),
            globals: Vec::new(),
        };
        
        for module in &self.modules {
            merged.functions.extend(module.functions.clone());
            merged.globals.extend(module.globals.clone());
        }
        
        self.deduplicate_functions(&mut merged);
        self.merge_globals(&mut merged);
        
        self.merged_module = Some(merged.clone());
        merged
    }
    
    fn deduplicate_functions(&self, module: &mut Module) {
        let mut seen = HashSet::new();
        module.functions.retain(|f| seen.insert(f.clone()));
    }
    
    fn merge_globals(&self, module: &mut Module) {
        let mut seen = HashSet::new();
        module.globals.retain(|g| seen.insert(g.clone()));
    }
}

impl Default for LinkTimeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
