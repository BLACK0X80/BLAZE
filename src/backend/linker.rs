use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct LinkerOptions {
    pub output_path: PathBuf,
    pub input_objects: Vec<PathBuf>,
    pub libraries: Vec<String>,
    pub library_paths: Vec<PathBuf>,
    pub static_link: bool,
    pub optimize_for_size: bool,
    pub strip_symbols: bool,
    pub target_triple: Option<String>,
}

pub struct Linker {
    options: LinkerOptions,
}

impl Linker {
    pub fn new(options: LinkerOptions) -> Self {
        Self { options }
    }
    
    pub fn link(&self) -> Result<()> {
        if cfg!(target_os = "windows") {
            self.link_windows()
        } else if cfg!(target_os = "macos") {
            self.link_macos()
        } else {
            self.link_linux()
        }
    }
    
    fn link_linux(&self) -> Result<()> {
        let mut cmd = Command::new("ld");
        
        cmd.arg("-o").arg(&self.options.output_path);
        
        for obj in &self.options.input_objects {
            cmd.arg(obj);
        }
        
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }
        
        for lib in &self.options.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        
        if self.options.static_link {
            cmd.arg("-static");
        } else {
            cmd.arg("-dynamic-linker").arg("/lib64/ld-linux-x86-64.so.2");
        }
        
        if self.options.strip_symbols {
            cmd.arg("-s");
        }
        
        if self.options.optimize_for_size {
            cmd.arg("--gc-sections");
        }
        
        cmd.arg("-lc");
        
        let output = cmd.output().context("Failed to execute linker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Linking failed: {}", stderr);
        }
        
        Ok(())
    }
    
    fn link_macos(&self) -> Result<()> {
        let mut cmd = Command::new("ld");
        
        cmd.arg("-o").arg(&self.options.output_path);
        
        for obj in &self.options.input_objects {
            cmd.arg(obj);
        }
        
        for lib_path in &self.options.library_paths {
            cmd.arg("-L").arg(lib_path);
        }
        
        for lib in &self.options.libraries {
            cmd.arg("-l").arg(lib);
        }
        
        cmd.arg("-lSystem");
        cmd.arg("-arch").arg("x86_64");
        
        if self.options.strip_symbols {
            cmd.arg("-S");
        }
        
        if self.options.optimize_for_size {
            cmd.arg("-dead_strip");
        }
        
        let output = cmd.output().context("Failed to execute linker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Linking failed: {}", stderr);
        }
        
        Ok(())
    }
    
    fn link_windows(&self) -> Result<()> {
        let mut cmd = Command::new("link.exe");
        
        cmd.arg(format!("/OUT:{}", self.options.output_path.display()));
        
        for obj in &self.options.input_objects {
            cmd.arg(obj);
        }
        
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("/LIBPATH:{}", lib_path.display()));
        }
        
        for lib in &self.options.libraries {
            cmd.arg(format!("{}.lib", lib));
        }
        
        cmd.arg("kernel32.lib");
        cmd.arg("msvcrt.lib");
        
        if self.options.optimize_for_size {
            cmd.arg("/OPT:REF");
            cmd.arg("/OPT:ICF");
        }
        
        cmd.arg("/SUBSYSTEM:CONSOLE");
        cmd.arg("/MACHINE:X64");
        
        let output = cmd.output().context("Failed to execute linker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Linking failed: {}", stderr);
        }
        
        Ok(())
    }
    
    pub fn link_with_gcc(&self) -> Result<()> {
        let mut cmd = Command::new("gcc");
        
        cmd.arg("-o").arg(&self.options.output_path);
        
        for obj in &self.options.input_objects {
            cmd.arg(obj);
        }
        
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }
        
        for lib in &self.options.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        
        if self.options.static_link {
            cmd.arg("-static");
        }
        
        if self.options.strip_symbols {
            cmd.arg("-s");
        }
        
        if self.options.optimize_for_size {
            cmd.arg("-Os");
        }
        
        let output = cmd.output().context("Failed to execute gcc linker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("GCC linking failed: {}", stderr);
        }
        
        Ok(())
    }
    
    pub fn link_with_clang(&self) -> Result<()> {
        let mut cmd = Command::new("clang");
        
        cmd.arg("-o").arg(&self.options.output_path);
        
        for obj in &self.options.input_objects {
            cmd.arg(obj);
        }
        
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }
        
        for lib in &self.options.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        
        if self.options.static_link {
            cmd.arg("-static");
        }
        
        if self.options.strip_symbols {
            cmd.arg("-Wl,-s");
        }
        
        if self.options.optimize_for_size {
            cmd.arg("-Oz");
        }
        
        if let Some(ref triple) = self.options.target_triple {
            cmd.arg(format!("--target={}", triple));
        }
        
        let output = cmd.output().context("Failed to execute clang linker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Clang linking failed: {}", stderr);
        }
        
        Ok(())
    }
}

impl LinkerOptions {
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            input_objects: Vec::new(),
            libraries: Vec::new(),
            library_paths: Vec::new(),
            static_link: false,
            optimize_for_size: false,
            strip_symbols: false,
            target_triple: None,
        }
    }
    
    pub fn add_object(&mut self, path: PathBuf) -> &mut Self {
        self.input_objects.push(path);
        self
    }
    
    pub fn add_library(&mut self, name: String) -> &mut Self {
        self.libraries.push(name);
        self
    }
    
    pub fn add_library_path(&mut self, path: PathBuf) -> &mut Self {
        self.library_paths.push(path);
        self
    }
    
    pub fn with_static_link(mut self, enable: bool) -> Self {
        self.static_link = enable;
        self
    }
    
    pub fn with_size_optimization(mut self, enable: bool) -> Self {
        self.optimize_for_size = enable;
        self
    }
    
    pub fn with_stripped_symbols(mut self, enable: bool) -> Self {
        self.strip_symbols = enable;
        self
    }
    
    pub fn with_target_triple(mut self, triple: String) -> Self {
        self.target_triple = Some(triple);
        self
    }
}

impl Default for LinkerOptions {
    fn default() -> Self {
        Self::new(PathBuf::from("a.out"))
    }
}
