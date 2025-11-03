use std::path::{Path, PathBuf};
use std::fs;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use anyhow::{Result, Context};
use flate2::read::GzDecoder;
use tar::Archive;

use super::{Package, Version, Dependency};

pub struct Registry {
    url: String,
    client: Client,
    cache_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryPackage {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    license: String,
    repository: Option<String>,
    homepage: Option<String>,
    keywords: Vec<String>,
    dependencies: Vec<RegistryDependency>,
    checksum: String,
    download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryDependency {
    name: String,
    version_req: String,
    optional: bool,
    features: Vec<String>,
}

impl Registry {
    pub fn new(url: String) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("blaze")
            .join("packages");
        
        fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            url,
            client: Client::builder()
                .user_agent("BLAZE Package Manager/0.1.0")
                .build()?,
            cache_dir,
        })
    }
    
    pub fn fetch_package(&self, name: &str, version: &Version) -> Result<Package> {
        let url = format!("{}/api/v1/packages/{}/{}", self.url, name, version);
        
        let response = self.client
            .get(&url)
            .send()
            .context("Failed to fetch package metadata")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Package not found: {} {}", name, version);
        }
        
        let registry_pkg: RegistryPackage = response.json()
            .context("Failed to parse package metadata")?;
        
        Ok(Package {
            name: registry_pkg.name,
            version: Version::parse(&registry_pkg.version)?,
            authors: registry_pkg.authors,
            description: registry_pkg.description,
            license: registry_pkg.license,
            repository: registry_pkg.repository,
            homepage: registry_pkg.homepage,
            keywords: registry_pkg.keywords,
        })
    }
    
    pub fn download_package(&self, name: &str, version: &Version) -> Result<PathBuf> {
        let cached_path = self.cache_dir.join(format!("{}-{}", name, version));
        
        if cached_path.exists() {
            return Ok(cached_path);
        }
        
        let url = format!("{}/api/v1/packages/{}/{}", self.url, name, version);
        
        let response = self.client
            .get(&url)
            .send()
            .context("Failed to fetch package metadata")?;
        
        let registry_pkg: RegistryPackage = response.json()?;
        
        let download_response = self.client
            .get(&registry_pkg.download_url)
            .send()
            .context("Failed to download package")?;
        
        let bytes = download_response.bytes()?;
        
        self.verify_checksum(&bytes, &registry_pkg.checksum)?;
        
        let temp_dir = tempfile::tempdir()?;
        let tar_gz_path = temp_dir.path().join("package.tar.gz");
        fs::write(&tar_gz_path, &bytes)?;
        
        let tar_gz = fs::File::open(&tar_gz_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        
        fs::create_dir_all(&cached_path)?;
        archive.unpack(&cached_path)?;
        
        Ok(cached_path)
    }
    
    pub fn search_packages(&self, query: &str) -> Result<Vec<Package>> {
        let url = format!("{}/api/v1/search?q={}", self.url, query);
        
        let response = self.client
            .get(&url)
            .send()
            .context("Failed to search packages")?;
        
        let packages: Vec<RegistryPackage> = response.json()?;
        
        packages.into_iter()
            .map(|p| Ok(Package {
                name: p.name,
                version: Version::parse(&p.version)?,
                authors: p.authors,
                description: p.description,
                license: p.license,
                repository: p.repository,
                homepage: p.homepage,
                keywords: p.keywords,
            }))
            .collect()
    }
    
    pub fn fetch_latest_version(&self, name: &str) -> Result<Version> {
        let url = format!("{}/api/v1/packages/{}/versions/latest", self.url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .context("Failed to fetch latest version")?;
        
        #[derive(Deserialize)]
        struct LatestVersion {
            version: String,
        }
        
        let latest: LatestVersion = response.json()?;
        Version::parse(&latest.version)
    }
    
    pub fn fetch_dependencies(&self, name: &str, version: &Version) -> Result<Vec<Dependency>> {
        let url = format!("{}/api/v1/packages/{}/{}", self.url, name, version);
        
        let response = self.client
            .get(&url)
            .send()
            .context("Failed to fetch package dependencies")?;
        
        let registry_pkg: RegistryPackage = response.json()?;
        
        registry_pkg.dependencies.into_iter()
            .map(|dep| {
                use super::VersionRequirement;
                
                let version_req = if dep.version_req.starts_with('^') {
                    let v = Version::parse(&dep.version_req[1..])?;
                    VersionRequirement::Compatible(v)
                } else if dep.version_req.starts_with('=') {
                    let v = Version::parse(&dep.version_req[1..])?;
                    VersionRequirement::Exact(v)
                } else if dep.version_req.starts_with('>') {
                    let v = Version::parse(&dep.version_req[1..])?;
                    VersionRequirement::GreaterThan(v)
                } else {
                    Version::parse(&dep.version_req)
                        .map(VersionRequirement::Compatible)?
                };
                
                Ok(Dependency {
                    name: dep.name,
                    version_req,
                    optional: dep.optional,
                    features: dep.features,
                })
            })
            .collect()
    }
    
    fn verify_checksum(&self, data: &[u8], expected: &str) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let actual = hex::encode(result);
        
        if actual != expected {
            anyhow::bail!("Checksum mismatch: expected {}, got {}", expected, actual);
        }
        
        Ok(())
    }
    
    pub fn publish_package(&self, package_path: &Path, token: &str) -> Result<()> {
        let manifest_path = package_path.join("Blaze.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .context("Failed to read Blaze.toml")?;
        
        let tar_gz = self.create_package_archive(package_path)?;
        
        let form = reqwest::blocking::multipart::Form::new()
            .text("manifest", manifest)
            .part("package", reqwest::blocking::multipart::Part::bytes(tar_gz));
        
        let url = format!("{}/api/v1/publish", self.url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .context("Failed to publish package")?;
        
        if !response.status().is_success() {
            let error_text = response.text()?;
            anyhow::bail!("Failed to publish package: {}", error_text);
        }
        
        Ok(())
    }
    
    fn create_package_archive(&self, package_path: &Path) -> Result<Vec<u8>> {
        use std::io::Write;
        
        let mut tar_builder = tar::Builder::new(Vec::new());
        
        for entry in walkdir::WalkDir::new(package_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let path = e.path();
                !path.starts_with(package_path.join("target")) &&
                !path.starts_with(package_path.join(".git"))
            })
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let rel_path = path.strip_prefix(package_path)?;
                tar_builder.append_path_with_name(path, rel_path)?;
            }
        }
        
        let tar_data = tar_builder.into_inner()?;
        
        let mut gz_encoder = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default()
        );
        gz_encoder.write_all(&tar_data)?;
        
        Ok(gz_encoder.finish()?)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new("https://packages.blaze-lang.org".to_string())
            .expect("Failed to create default registry")
    }
}
