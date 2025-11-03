use std::collections::{HashMap, HashSet};

pub struct MemoryModel {
    allocations: HashMap<AllocationId, Allocation>,
    next_id: usize,
    stack_frames: Vec<StackFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllocationId(usize);

#[derive(Debug, Clone)]
pub struct Allocation {
    pub id: AllocationId,
    pub size: usize,
    pub alignment: usize,
    pub location: MemoryLocation,
    pub mutable: bool,
    pub live: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryLocation {
    Stack,
    Heap,
    Static,
    ThreadLocal,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub variables: HashMap<String, AllocationId>,
    pub return_address: Option<usize>,
}

impl MemoryModel {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            next_id: 0,
            stack_frames: vec![StackFrame::new()],
        }
    }
    
    pub fn allocate(&mut self, size: usize, alignment: usize, location: MemoryLocation) -> AllocationId {
        let id = AllocationId(self.next_id);
        self.next_id += 1;
        
        let allocation = Allocation {
            id,
            size,
            alignment,
            location,
            mutable: true,
            live: true,
        };
        
        self.allocations.insert(id, allocation);
        id
    }
    
    pub fn deallocate(&mut self, id: AllocationId) -> Result<(), String> {
        if let Some(alloc) = self.allocations.get_mut(&id) {
            if !alloc.live {
                return Err("Double free detected".to_string());
            }
            alloc.live = false;
            Ok(())
        } else {
            Err("Invalid allocation ID".to_string())
        }
    }
    
    pub fn push_frame(&mut self) {
        self.stack_frames.push(StackFrame::new());
    }
    
    pub fn pop_frame(&mut self) -> Result<(), String> {
        if self.stack_frames.len() <= 1 {
            return Err("Cannot pop root frame".to_string());
        }
        
        if let Some(frame) = self.stack_frames.pop() {
            for allocation_id in frame.variables.values() {
                self.deallocate(*allocation_id)?;
            }
        }
        
        Ok(())
    }
    
    pub fn declare_variable(&mut self, name: String, allocation_id: AllocationId) -> Result<(), String> {
        if let Some(frame) = self.stack_frames.last_mut() {
            frame.variables.insert(name, allocation_id);
            Ok(())
        } else {
            Err("No active stack frame".to_string())
        }
    }
    
    pub fn get_variable(&self, name: &str) -> Option<AllocationId> {
        for frame in self.stack_frames.iter().rev() {
            if let Some(&id) = frame.variables.get(name) {
                return Some(id);
            }
        }
        None
    }
    
    pub fn check_alignment(&self, id: AllocationId, required_alignment: usize) -> bool {
        if let Some(alloc) = self.allocations.get(&id) {
            alloc.alignment >= required_alignment
        } else {
            false
        }
    }
    
    pub fn total_allocated(&self) -> usize {
        self.allocations
            .values()
            .filter(|a| a.live)
            .map(|a| a.size)
            .sum()
    }
    
    pub fn find_leaks(&self) -> Vec<AllocationId> {
        self.allocations
            .values()
            .filter(|a| a.live && a.location == MemoryLocation::Heap)
            .map(|a| a.id)
            .collect()
    }
}

impl StackFrame {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            return_address: None,
        }
    }
}

impl Default for MemoryModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AliasingAnalyzer {
    aliases: HashMap<String, HashSet<String>>,
}

impl AliasingAnalyzer {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }
    
    pub fn add_alias(&mut self, var1: String, var2: String) {
        self.aliases.entry(var1.clone()).or_insert_with(HashSet::new).insert(var2.clone());
        self.aliases.entry(var2).or_insert_with(HashSet::new).insert(var1);
    }
    
    pub fn may_alias(&self, var1: &str, var2: &str) -> bool {
        if var1 == var2 {
            return true;
        }
        
        self.aliases
            .get(var1)
            .map(|aliases| aliases.contains(var2))
            .unwrap_or(false)
    }
    
    pub fn get_aliases(&self, var: &str) -> Vec<String> {
        self.aliases
            .get(var)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for AliasingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemorySafetyChecker {
    use_after_free: Vec<String>,
    double_free: Vec<String>,
    memory_leaks: Vec<String>,
}

impl MemorySafetyChecker {
    pub fn new() -> Self {
        Self {
            use_after_free: Vec::new(),
            double_free: Vec::new(),
            memory_leaks: Vec::new(),
        }
    }
    
    pub fn check(&mut self, model: &MemoryModel) -> Result<(), String> {
        self.check_use_after_free(model)?;
        self.check_double_free(model)?;
        self.check_memory_leaks(model)?;
        
        if !self.use_after_free.is_empty() {
            return Err(format!("Use-after-free detected: {:?}", self.use_after_free));
        }
        
        if !self.double_free.is_empty() {
            return Err(format!("Double-free detected: {:?}", self.double_free));
        }
        
        if !self.memory_leaks.is_empty() {
            return Err(format!("Memory leaks detected: {:?}", self.memory_leaks));
        }
        
        Ok(())
    }
    
    fn check_use_after_free(&mut self, model: &MemoryModel) -> Result<(), String> {
        Ok(())
    }
    
    fn check_double_free(&mut self, model: &MemoryModel) -> Result<(), String> {
        Ok(())
    }
    
    fn check_memory_leaks(&mut self, model: &MemoryModel) -> Result<(), String> {
        let leaks = model.find_leaks();
        
        for leak in leaks {
            self.memory_leaks.push(format!("{:?}", leak));
        }
        
        Ok(())
    }
}

impl Default for MemorySafetyChecker {
    fn default() -> Self {
        Self::new()
    }
}
