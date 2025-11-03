use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use anyhow::{Result, Context};
use rayon::prelude::*;

use super::{BuildConfig, BuildTarget, Artifact, TargetType, BuildMode};

pub struct BuildCompiler {
    config: BuildConfig,
}

impl BuildCompiler {
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }
    
    pub fn compile_target(&self, target: &BuildTarget) -> Result<Artifact> {
        tracing::info!("Compiling target: {}", target.name);
        
        let mut object_files = Vec::new();
        
        if self.config.parallel_build {
            object_files = target.sources
                .par_iter()
                .map(|source| self.compile_source(source))
                .collect::<Result<Vec<_>>>()?;
        } else {
            for source in &target.sources {
                object_files.push(self.compile_source(source)?);
            }
        }
        
        self.link_objects(&object_files, target)
    }
    
    fn compile_source(&self, source: &Path) -> Result<PathBuf> {
        let output = self.config.output_dir
            .join(source.file_stem().unwrap())
            .with_extension("o");
        
        fs::create_dir_all(output.parent().unwrap())?;
        
        tracing::debug!("Compiling {} -> {}", source.display(), output.display());
        
        Ok(output)
    }
    
    fn link_objects(&self, objects: &[PathBuf], target: &BuildTarget) -> Result<Artifact> {
        let output = &target.output_path;
        
        fs::create_dir_all(output.parent().unwrap())?;
        
        let linker = if cfg!(target_os = "windows") {
            "link.exe"
        } else if cfg!(target_os = "macos") {
            "ld"
        } else {
            "ld"
        };
        
        let mut link_cmd = Command::new(linker);
        
        for obj in objects {
            link_cmd.arg(obj);
        }
        
        match target.target_type {
            TargetType::Executable => {
                #[cfg(target_os = "windows")]
                link_cmd.arg("/OUT:").arg(output);
                
                #[cfg(not(target_os = "windows"))]
                link_cmd.arg("-o").arg(output);
            }
            TargetType::StaticLibrary => {
                #[cfg(target_os = "windows")]
                {
                    link_cmd = Command::new("lib.exe");
                    link_cmd.arg("/OUT:").arg(output);
                    for obj in objects {
                        link_cmd.arg(obj);
                    }
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    link_cmd = Command::new("ar");
                    link_cmd.arg("rcs").arg(output);
                    for obj in objects {
                        link_cmd.arg(obj);
                    }
                }
            }
            TargetType::DynamicLibrary => {
                #[cfg(target_os = "windows")]
                {
                    link_cmd.arg("/DLL");
                    link_cmd.arg("/OUT:").arg(output);
                }
                
                #[cfg(target_os = "macos")]
                {
                    link_cmd.arg("-dylib");
                    link_cmd.arg("-o").arg(output);
                }
                
                #[cfg(target_os = "linux")]
                {
                    link_cmd.arg("-shared");
                    link_cmd.arg("-o").arg(output);
                }
            }
            _ => {}
        }
        
        link_cmd.arg("-L").arg(&self.config.output_dir);
        
        let runtime_lib = self.get_runtime_library()?;
        link_cmd.arg(&runtime_lib);
        
        for dep in &target.dependencies {
            link_cmd.arg(format!("-l{}", dep));
        }
        
        tracing::debug!("Linking command: {:?}", link_cmd);
        
        let link_output = link_cmd.output()
            .context("Failed to execute linker")?;
        
        if !link_output.status.success() {
            let stderr = String::from_utf8_lossy(&link_output.stderr);
            anyhow::bail!("Linking failed:\n{}", stderr);
        }
        
        tracing::info!("Successfully linked: {}", output.display());
        
        Ok(Artifact {
            path: output.clone(),
            artifact_type: target.target_type,
        })
    }
    
    fn get_runtime_library(&self) -> Result<PathBuf> {
        let runtime_dir = std::env::current_exe()?
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("runtime");
        
        #[cfg(target_os = "windows")]
        let lib_name = "blaze_runtime.lib";
        
        #[cfg(not(target_os = "windows"))]
        let lib_name = "libblaze_runtime.a";
        
        let lib_path = runtime_dir.join("target").join("release").join(lib_name);
        
        if !lib_path.exists() {
            self.build_runtime(&runtime_dir)?;
        }
        
        Ok(lib_path)
    }
    
    fn build_runtime(&self, runtime_dir: &Path) -> Result<()> {
        tracing::info!("Building runtime library...");
        
        let output = Command::new("cargo")
            .current_dir(runtime_dir)
            .args(&["build", "--release"])
            .output()
            .context("Failed to build runtime library")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Runtime build failed:\n{}", stderr);
        }
        
        tracing::info!("Runtime library built successfully");
        
        Ok(())
    }
    
    pub fn incremental_compile(&self, target: &BuildTarget, changed_files: &[PathBuf]) -> Result<Artifact> {
        let cache_file = self.config.output_dir.join(".build_cache");
        let cache: BuildCache = if cache_file.exists() {
            let content = fs::read_to_string(&cache_file)?;
            serde_json::from_str(&content)?
        } else {
            BuildCache::default()
        };
        
        let mut object_files = Vec::new();
        
        for source in &target.sources {
            if changed_files.contains(source) || !cache.is_up_to_date(source)? {
                let obj = self.compile_source(source)?;
                object_files.push(obj);
            } else if let Some(cached_obj) = cache.get_object(source) {
                object_files.push(cached_obj.clone());
            }
        }
        
        let artifact = self.link_objects(&object_files, target)?;
        
        let mut new_cache = cache;
        for (source, obj) in target.sources.iter().zip(object_files.iter()) {
            new_cache.update_entry(source, obj)?;
        }
        
        let cache_content = serde_json::to_string_pretty(&new_cache)?;
        fs::write(&cache_file, cache_content)?;
        
        Ok(artifact)
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct BuildCache {
    entries: std::collections::HashMap<PathBuf, CacheEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    source_path: PathBuf,
    object_path: PathBuf,
    timestamp: std::time::SystemTime,
    checksum: String,
}

impl BuildCache {
    fn is_up_to_date(&self, source: &Path) -> Result<bool> {
        if let Some(entry) = self.entries.get(source) {
            let metadata = fs::metadata(source)?;
            let modified = metadata.modified()?;
            
            if modified <= entry.timestamp {
                let content = fs::read(source)?;
                let checksum = Self::compute_checksum(&content);
                
                return Ok(checksum == entry.checksum);
            }
        }
        
        Ok(false)
    }
    
    fn get_object(&self, source: &Path) -> Option<&PathBuf> {
        self.entries.get(source).map(|e| &e.object_path)
    }
    
    fn update_entry(&mut self, source: &Path, object: &Path) -> Result<()> {
        let metadata = fs::metadata(source)?;
        let content = fs::read(source)?;
        
        self.entries.insert(
            source.to_path_buf(),
            CacheEntry {
                source_path: source.to_path_buf(),
                object_path: object.to_path_buf(),
                timestamp: metadata.modified()?,
                checksum: Self::compute_checksum(&content),
            },
        );
        
        Ok(())
    }
    
    fn compute_checksum(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}
