use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

pub struct IncrementalCompiler {
    file_hashes: HashMap<PathBuf, FileHash>,
    dependency_graph: HashMap<PathBuf, HashSet<PathBuf>>,
    cache: CompilationCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileHash {
    pub hash: u64,
    pub timestamp: SystemTime,
}

pub struct CompilationCache {
    ast_cache: HashMap<PathBuf, Vec<u8>>,
    ir_cache: HashMap<PathBuf, Vec<u8>>,
    metadata_cache: HashMap<PathBuf, ModuleMetadata>,
}

#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub checksum: u64,
}

impl IncrementalCompiler {
    pub fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
            dependency_graph: HashMap::new(),
            cache: CompilationCache::new(),
        }
    }
    
    pub fn needs_recompile(&mut self, file: &PathBuf) -> Result<bool, String> {
        let current_hash = self.compute_file_hash(file)?;
        
        match self.file_hashes.get(file) {
            Some(cached_hash) if cached_hash == &current_hash => Ok(false),
            _ => {
                self.file_hashes.insert(file.clone(), current_hash);
                Ok(true)
            }
        }
    }
    
    fn compute_file_hash(&self, file: &PathBuf) -> Result<FileHash, String> {
        let metadata = std::fs::metadata(file)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        let timestamp = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?;
        
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let hash = self.hash_string(&content);
        
        Ok(FileHash { hash, timestamp })
    }
    
    fn hash_string(&self, s: &str) -> u64 {
        let mut hash = 0u64;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
    
    pub fn add_dependency(&mut self, file: PathBuf, dependency: PathBuf) {
        self.dependency_graph
            .entry(file)
            .or_insert_with(HashSet::new)
            .insert(dependency);
    }
    
    pub fn get_affected_files(&self, changed_file: &PathBuf) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();
        let mut to_check = vec![changed_file.clone()];
        
        while let Some(file) = to_check.pop() {
            for (dependent, dependencies) in &self.dependency_graph {
                if dependencies.contains(&file) && !affected.contains(dependent) {
                    affected.insert(dependent.clone());
                    to_check.push(dependent.clone());
                }
            }
        }
        
        affected
    }
    
    pub fn cache_ast(&mut self, file: PathBuf, ast: Vec<u8>) {
        self.cache.ast_cache.insert(file, ast);
    }
    
    pub fn get_cached_ast(&self, file: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.ast_cache.get(file)
    }
    
    pub fn cache_ir(&mut self, file: PathBuf, ir: Vec<u8>) {
        self.cache.ir_cache.insert(file, ir);
    }
    
    pub fn get_cached_ir(&self, file: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.ir_cache.get(file)
    }
    
    pub fn clear_cache(&mut self) {
        self.cache.ast_cache.clear();
        self.cache.ir_cache.clear();
        self.cache.metadata_cache.clear();
    }
}

impl CompilationCache {
    fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            ir_cache: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }
}

impl Default for IncrementalCompiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HotReloader {
    watchers: HashMap<PathBuf, FileWatcher>,
    reload_callbacks: Vec<Box<dyn Fn(&PathBuf)>>,
}

struct FileWatcher {
    path: PathBuf,
    last_modified: SystemTime,
}

impl HotReloader {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            reload_callbacks: Vec::new(),
        }
    }
    
    pub fn watch(&mut self, path: PathBuf) -> Result<(), String> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to watch file: {}", e))?;
        
        let last_modified = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?;
        
        self.watchers.insert(path.clone(), FileWatcher {
            path,
            last_modified,
        });
        
        Ok(())
    }
    
    pub fn check_changes(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        
        for (path, watcher) in &mut self.watchers {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > watcher.last_modified {
                        watcher.last_modified = modified;
                        changed.push(path.clone());
                    }
                }
            }
        }
        
        for path in &changed {
            for callback in &self.reload_callbacks {
                callback(path);
            }
        }
        
        changed
    }
    
    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: Fn(&PathBuf) + 'static,
    {
        self.reload_callbacks.push(Box::new(callback));
    }
}

impl Default for HotReloader {
    fn default() -> Self {
        Self::new()
    }
}
