use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;

pub struct GarbageCollector {
    heap: Vec<Allocation>,
    roots: HashSet<usize>,
    gc_threshold: usize,
    allocated_bytes: usize,
    algorithm: GCAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GCAlgorithm {
    MarkAndSweep,
    Copying,
    Generational,
    Incremental,
}

#[derive(Debug, Clone)]
struct Allocation {
    id: usize,
    size: usize,
    marked: bool,
    data: Vec<u8>,
    references: Vec<usize>,
}

impl GarbageCollector {
    pub fn new(algorithm: GCAlgorithm) -> Self {
        Self {
            heap: Vec::new(),
            roots: HashSet::new(),
            gc_threshold: 1024 * 1024,
            allocated_bytes: 0,
            algorithm,
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Result<usize, String> {
        if self.allocated_bytes + size > self.gc_threshold {
            self.collect()?;
        }
        
        let id = self.heap.len();
        let allocation = Allocation {
            id,
            size,
            marked: false,
            data: vec![0; size],
            references: Vec::new(),
        };
        
        self.heap.push(allocation);
        self.allocated_bytes += size;
        Ok(id)
    }
    
    pub fn add_root(&mut self, id: usize) {
        self.roots.insert(id);
    }
    
    pub fn remove_root(&mut self, id: usize) {
        self.roots.remove(&id);
    }
    
    pub fn collect(&mut self) -> Result<(), String> {
        match self.algorithm {
            GCAlgorithm::MarkAndSweep => self.mark_and_sweep(),
            GCAlgorithm::Copying => self.copying_collect(),
            GCAlgorithm::Generational => self.generational_collect(),
            GCAlgorithm::Incremental => self.incremental_collect(),
        }
    }
    
    fn mark_and_sweep(&mut self) -> Result<(), String> {
        for alloc in &mut self.heap {
            alloc.marked = false;
        }
        
        let mut work_list: Vec<usize> = self.roots.iter().copied().collect();
        
        while let Some(id) = work_list.pop() {
            if let Some(alloc) = self.heap.get_mut(id) {
                if !alloc.marked {
                    alloc.marked = true;
                    work_list.extend(&alloc.references);
                }
            }
        }
        
        let mut freed_bytes = 0;
        self.heap.retain(|alloc| {
            if alloc.marked {
                true
            } else {
                freed_bytes += alloc.size;
                false
            }
        });
        
        self.allocated_bytes -= freed_bytes;
        Ok(())
    }
    
    fn copying_collect(&mut self) -> Result<(), String> {
        let mut new_heap = Vec::new();
        let mut id_mapping = HashMap::new();
        
        for &root_id in &self.roots {
            if let Some(alloc) = self.heap.get(root_id) {
                let new_id = new_heap.len();
                id_mapping.insert(root_id, new_id);
                new_heap.push(alloc.clone());
            }
        }
        
        let mut i = 0;
        while i < new_heap.len() {
            let refs = new_heap[i].references.clone();
            
            for &ref_id in &refs {
                if !id_mapping.contains_key(&ref_id) {
                    if let Some(alloc) = self.heap.get(ref_id) {
                        let new_id = new_heap.len();
                        id_mapping.insert(ref_id, new_id);
                        new_heap.push(alloc.clone());
                    }
                }
            }
            
            i += 1;
        }
        
        self.heap = new_heap;
        self.allocated_bytes = self.heap.iter().map(|a| a.size).sum();
        Ok(())
    }
    
    fn generational_collect(&mut self) -> Result<(), String> {
        self.mark_and_sweep()
    }
    
    fn incremental_collect(&mut self) -> Result<(), String> {
        let batch_size = 10;
        
        for i in 0..batch_size.min(self.heap.len()) {
            if let Some(alloc) = self.heap.get_mut(i) {
                alloc.marked = false;
            }
        }
        
        Ok(())
    }
    
    pub fn stats(&self) -> GCStats {
        GCStats {
            total_allocated: self.allocated_bytes,
            heap_size: self.heap.len(),
            roots_count: self.roots.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GCStats {
    pub total_allocated: usize,
    pub heap_size: usize,
    pub roots_count: usize,
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new(GCAlgorithm::MarkAndSweep)
    }
}
