pub mod compiler;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub use compiler::BuildCompiler;

pub struct BuildSystem {
    config: BuildConfig,
    targets: HashMap<String, BuildTarget>,
    dependencies: DependencyResolver,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub project_name: String,
    pub version: String,
    pub source_dirs: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub optimization_level: u8,
    pub build_mode: BuildMode,
    pub features: Vec<String>,
    pub parallel_build: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
    Test,
    Bench,
}

#[derive(Debug, Clone)]
pub struct BuildTarget {
    pub name: String,
    pub target_type: TargetType,
    pub sources: Vec<PathBuf>,
    pub dependencies: Vec<String>,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetType {
    Executable,
    Library,
    StaticLibrary,
    DynamicLibrary,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub artifact_type: TargetType,
}

impl BuildSystem {
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            targets: HashMap::new(),
            dependencies: DependencyResolver::new(),
        }
    }
    
    pub fn add_target(&mut self, target: BuildTarget) {
        self.targets.insert(target.name.clone(), target);
    }
    
    pub fn build(&mut self, target_name: Option<&str>) -> Result<BuildResult, String> {
        let targets = if let Some(name) = target_name {
            vec![self.targets.get(name).ok_or("Target not found")?.clone()]
        } else {
            self.targets.values().cloned().collect()
        };
        
        let mut results = Vec::new();
        
        for target in targets {
            let result = self.build_target(&target)?;
            results.push(result);
        }
        
        Ok(BuildResult {
            success: true,
            artifacts: results,
            duration: std::time::Duration::from_secs(0),
        })
    }
    
    fn build_target(&self, target: &BuildTarget) -> Result<Artifact, String> {
        self.compile_sources(target)
    }
    
    fn compile_sources(&self, target: &BuildTarget) -> Result<Artifact, String> {
        let mut objects = Vec::new();
        
        for source in &target.sources {
            let object = self.compile_file(source)?;
            objects.push(object);
        }
        
        self.link_objects(&objects, target)
    }
    
    fn compile_file(&self, source: &PathBuf) -> Result<PathBuf, String> {
        let output = self.config.output_dir.join(
            source.file_stem().unwrap().to_str().unwrap()
        ).with_extension("o");
        
        Ok(output)
    }
    
    fn link_objects(&self, objects: &[PathBuf], target: &BuildTarget) -> Result<Artifact, String> {
        Ok(Artifact {
            path: target.output_path.clone(),
            artifact_type: target.target_type,
        })
    }
    
    pub fn clean(&self) -> Result<(), String> {
        if self.config.output_dir.exists() {
            std::fs::remove_dir_all(&self.config.output_dir)
                .map_err(|e| format!("Failed to clean: {}", e))?;
        }
        Ok(())
    }
    
    pub fn install(&self, prefix: PathBuf) -> Result<(), String> {
        for target in self.targets.values() {
            let dest = prefix.join(&target.name);
            std::fs::copy(&target.output_path, dest)
                .map_err(|e| format!("Failed to install: {}", e))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub artifacts: Vec<Artifact>,
    pub duration: std::time::Duration,
}

pub struct DependencyResolver {
    dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
}

#[derive(Debug, Clone)]
pub enum DependencySource {
    Registry,
    Git { url: String, branch: Option<String> },
    Path(PathBuf),
}

impl DependencyResolver {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }
    
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.insert(dep.name.clone(), dep);
    }
    
    pub fn resolve(&self) -> Result<Vec<Dependency>, String> {
        Ok(self.dependencies.values().cloned().collect())
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            project_name: "project".to_string(),
            version: "0.1.0".to_string(),
            source_dirs: vec![PathBuf::from("src")],
            output_dir: PathBuf::from("target"),
            optimization_level: 0,
            build_mode: BuildMode::Debug,
            features: Vec::new(),
            parallel_build: true,
        }
    }
}

impl Default for BuildSystem {
    fn default() -> Self {
        Self::new(BuildConfig::default())
    }
}
