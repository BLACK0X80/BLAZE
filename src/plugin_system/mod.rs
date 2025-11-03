use std::collections::HashMap;
use std::path::PathBuf;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    hooks: HashMap<PluginHook, Vec<String>>,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self) -> Result<(), String>;
    fn on_compile_start(&mut self, _context: &CompileContext) {}
    fn on_compile_end(&mut self, _context: &CompileContext) {}
    fn on_parse_start(&mut self, _file: &str) {}
    fn on_parse_end(&mut self, _file: &str) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginHook {
    CompileStart,
    CompileEnd,
    ParseStart,
    ParseEnd,
    OptimizationStart,
    OptimizationEnd,
    CodeGenStart,
    CodeGenEnd,
}

#[derive(Debug, Clone)]
pub struct CompileContext {
    pub source_file: PathBuf,
    pub output_file: PathBuf,
    pub optimization_level: u8,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hooks: HashMap::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let name = plugin.name().to_string();
        
        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' already registered", name));
        }
        
        self.plugins.insert(name, plugin);
        Ok(())
    }
    
    pub fn load_plugin(&mut self, path: PathBuf) -> Result<(), String> {
        Ok(())
    }
    
    pub fn initialize_all(&mut self) -> Result<(), String> {
        for plugin in self.plugins.values_mut() {
            plugin.initialize()?;
        }
        Ok(())
    }
    
    pub fn trigger_hook(&mut self, hook: PluginHook, context: &CompileContext) {
        for plugin in self.plugins.values_mut() {
            match hook {
                PluginHook::CompileStart => plugin.on_compile_start(context),
                PluginHook::CompileEnd => plugin.on_compile_end(context),
                _ => {}
            }
        }
    }
    
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
    
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        self.plugins.remove(name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PluginRegistry {
    available_plugins: Vec<PluginMetadata>,
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub repository: Option<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            available_plugins: Vec::new(),
        }
    }
    
    pub fn register_metadata(&mut self, metadata: PluginMetadata) {
        self.available_plugins.push(metadata);
    }
    
    pub fn search(&self, query: &str) -> Vec<&PluginMetadata> {
        self.available_plugins
            .iter()
            .filter(|p| p.name.contains(query) || p.description.contains(query))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
