mod registry;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

pub use registry::Registry;

pub struct PackageManager {
    packages: HashMap<String, Package>,
    dependencies: HashMap<String, Vec<Dependency>>,
    registry_url: String,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub authors: Vec<String>,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: VersionRequirement,
    pub optional: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum VersionRequirement {
    Exact(Version),
    Range(Version, Version),
    GreaterThan(Version),
    Compatible(Version),
    Any,
}

impl PackageManager {
    pub fn new(registry_url: String) -> Self {
        Self {
            packages: HashMap::new(),
            dependencies: HashMap::new(),
            registry_url,
        }
    }
    
    pub fn install_package(&mut self, name: &str, version: &Version) -> Result<()> {
        let registry = Registry::default();
        let package = registry.fetch_package(name, version)?;
        
        let package_path = registry.download_package(name, version)?;
        
        let deps = registry.fetch_dependencies(name, version)?;
        
        for dep in deps {
            if !self.packages.contains_key(&dep.name) {
                let dep_version = self.resolve_version_requirement(&dep.version_req)?;
                self.install_package(&dep.name, &dep_version)?;
            }
        }
        
        self.dependencies.insert(name.to_string(), deps);
        self.packages.insert(name.to_string(), package);
        
        tracing::info!("Installed package: {} {}", name, version);
        Ok(())
    }
    
    fn resolve_version_requirement(&self, req: &VersionRequirement) -> Result<Version> {
        let registry = Registry::default();
        
        match req {
            VersionRequirement::Exact(v) => Ok(v.clone()),
            VersionRequirement::Compatible(v) => Ok(v.clone()),
            VersionRequirement::Any => {
                let name = "unknown";  // This should be passed in
                registry.fetch_latest_version(name)
            }
            VersionRequirement::GreaterThan(v) => Ok(v.clone()),
            VersionRequirement::Range(_, upper) => Ok(upper.clone()),
        }
    }
    
    pub fn resolve_dependencies(&self, packages: &[String]) -> Result<Vec<(String, Version)>> {
        let registry = Registry::default();
        let mut resolved = HashMap::new();
        let mut to_process: Vec<(String, Version)> = Vec::new();
        
        for pkg_name in packages {
            let version = registry.fetch_latest_version(pkg_name)?;
            to_process.push((pkg_name.clone(), version));
        }
        
        while let Some((name, version)) = to_process.pop() {
            if resolved.contains_key(&name) {
                continue;
            }
            
            let deps = registry.fetch_dependencies(&name, &version)?;
            
            for dep in deps {
                if !dep.optional {
                    let dep_version = self.resolve_version_requirement(&dep.version_req)?;
                    to_process.push((dep.name.clone(), dep_version));
                }
            }
            
            resolved.insert(name.clone(), version.clone());
        }
        
        Ok(resolved.into_iter().collect())
    }
    
    pub fn search_packages(&self, query: &str) -> Result<Vec<Package>> {
        let registry = Registry::default();
        registry.search_packages(query)
    }
    
    pub fn update_package(&mut self, name: &str) -> Result<()> {
        let registry = Registry::default();
        
        let current = self.packages.get(name)
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not installed", name))?
            .clone();
        
        let latest = registry.fetch_latest_version(name)?;
        
        if latest > current.version {
            tracing::info!("Updating {} from {} to {}", name, current.version, latest);
            self.install_package(name, &latest)?;
        } else {
            tracing::info!("Package {} is already up to date ({})", name, current.version);
        }
        
        Ok(())
    }
    
    pub fn publish_package(&self, package_path: &Path, token: &str) -> Result<()> {
        let registry = Registry::default();
        registry.publish_package(package_path, token)
    }
    
    pub fn remove_package(&mut self, name: &str) -> Result<()> {
        if !self.packages.contains_key(name) {
            anyhow::bail!("Package '{}' not installed", name);
        }
        
        if self.has_dependents(name) {
            anyhow::bail!("Cannot remove '{}': other packages depend on it", name);
        }
        
        self.packages.remove(name);
        self.dependencies.remove(name);
        
        tracing::info!("Removed package: {}", name);
        Ok(())
    }
    
    fn has_dependents(&self, name: &str) -> bool {
        self.dependencies.values().any(|deps| {
            deps.iter().any(|d| d.name == name)
        })
    }
    
    pub fn list_installed(&self) -> Vec<String> {
        self.packages.keys().cloned().collect()
    }
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }
    
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        
        if parts.len() < 3 {
            return Err("Invalid version format".to_string());
        }
        
        Ok(Self {
            major: parts[0].parse().map_err(|_| "Invalid major version")?,
            minor: parts[1].parse().map_err(|_| "Invalid minor version")?,
            patch: parts[2].parse().map_err(|_| "Invalid patch version")?,
            pre_release: None,
        })
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new("https://packages.blaze-lang.org".to_string())
    }
}
