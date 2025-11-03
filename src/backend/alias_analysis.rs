use std::collections::{HashMap, HashSet};

pub struct AliasAnalyzer {
    points_to: HashMap<String, PointsToSet>,
    alias_pairs: HashSet<(String, String)>,
    analysis_type: AnalysisType,
}

#[derive(Debug, Clone)]
pub struct PointsToSet {
    pub variable: String,
    pub targets: HashSet<MemoryLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryLocation {
    pub id: usize,
    pub location_type: LocationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationType {
    Stack,
    Heap,
    Global,
    Parameter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisType {
    FlowInsensitive,
    FlowSensitive,
    ContextSensitive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AliasResult {
    MustAlias,
    MayAlias,
    NoAlias,
}

impl AliasAnalyzer {
    pub fn new(analysis_type: AnalysisType) -> Self {
        Self {
            points_to: HashMap::new(),
            alias_pairs: HashSet::new(),
            analysis_type,
        }
    }
    
    pub fn analyze_pointer(&mut self, var: String, target: MemoryLocation) {
        self.points_to
            .entry(var)
            .or_insert_with(|| PointsToSet {
                variable: String::new(),
                targets: HashSet::new(),
            })
            .targets
            .insert(target);
    }
    
    pub fn query_alias(&self, var1: &str, var2: &str) -> AliasResult {
        if var1 == var2 {
            return AliasResult::MustAlias;
        }
        
        let pts1 = self.points_to.get(var1);
        let pts2 = self.points_to.get(var2);
        
        match (pts1, pts2) {
            (Some(p1), Some(p2)) => {
                let intersection: HashSet<_> = p1.targets.intersection(&p2.targets).collect();
                
                if !intersection.is_empty() {
                    if p1.targets.len() == 1 && p2.targets.len() == 1 && p1.targets == p2.targets {
                        AliasResult::MustAlias
                    } else {
                        AliasResult::MayAlias
                    }
                } else {
                    AliasResult::NoAlias
                }
            }
            _ => AliasResult::MayAlias,
        }
    }
    
    pub fn get_points_to_set(&self, var: &str) -> Option<&HashSet<MemoryLocation>> {
        self.points_to.get(var).map(|pts| &pts.targets)
    }
    
    pub fn compute_mod_ref_sets(&self, function: &str) -> (HashSet<MemoryLocation>, HashSet<MemoryLocation>) {
        let mut mod_set = HashSet::new();
        let mut ref_set = HashSet::new();
        
        for pts in self.points_to.values() {
            ref_set.extend(pts.targets.iter().cloned());
        }
        
        (mod_set, ref_set)
    }
}

impl Default for AliasAnalyzer {
    fn default() -> Self {
        Self::new(AnalysisType::FlowInsensitive)
    }
}

pub struct AndersonAnalysis {
    constraints: Vec<PointerConstraint>,
    solution: HashMap<String, HashSet<usize>>,
}

#[derive(Debug, Clone)]
pub enum PointerConstraint {
    AddressOf { target: String, source: usize },
    Copy { target: String, source: String },
    Load { target: String, source: String },
    Store { target: String, source: String },
}

impl AndersonAnalysis {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            solution: HashMap::new(),
        }
    }
    
    pub fn add_constraint(&mut self, constraint: PointerConstraint) {
        self.constraints.push(constraint);
    }
    
    pub fn solve(&mut self) {
        let mut changed = true;
        
        while changed {
            changed = false;
            
            for constraint in &self.constraints.clone() {
                match constraint {
                    PointerConstraint::AddressOf { target, source } => {
                        let set = self.solution.entry(target.clone()).or_insert_with(HashSet::new);
                        if set.insert(*source) {
                            changed = true;
                        }
                    }
                    
                    PointerConstraint::Copy { target, source } => {
                        if let Some(source_set) = self.solution.get(source).cloned() {
                            let target_set = self.solution.entry(target.clone()).or_insert_with(HashSet::new);
                            let old_size = target_set.len();
                            target_set.extend(source_set);
                            if target_set.len() > old_size {
                                changed = true;
                            }
                        }
                    }
                    
                    _ => {}
                }
            }
        }
    }
    
    pub fn get_points_to(&self, var: &str) -> Option<&HashSet<usize>> {
        self.solution.get(var)
    }
}

impl Default for AndersonAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SteensgaardAnalysis {
    union_find: UnionFind,
    points_to: HashMap<usize, usize>,
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl SteensgaardAnalysis {
    pub fn new(size: usize) -> Self {
        Self {
            union_find: UnionFind::new(size),
            points_to: HashMap::new(),
        }
    }
    
    pub fn unify(&mut self, a: usize, b: usize) {
        let root_a = self.union_find.find(a);
        let root_b = self.union_find.find(b);
        
        if root_a != root_b {
            self.union_find.union(root_a, root_b);
        }
    }
    
    pub fn add_points_to(&mut self, pointer: usize, target: usize) {
        let root = self.union_find.find(pointer);
        self.points_to.insert(root, target);
    }
    
    pub fn may_alias(&mut self, a: usize, b: usize) -> bool {
        self.union_find.find(a) == self.union_find.find(b)
    }
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }
    
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }
    
    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        
        if root_x == root_y {
            return;
        }
        
        if self.rank[root_x] < self.rank[root_y] {
            self.parent[root_x] = root_y;
        } else if self.rank[root_x] > self.rank[root_y] {
            self.parent[root_y] = root_x;
        } else {
            self.parent[root_y] = root_x;
            self.rank[root_x] += 1;
        }
    }
}
