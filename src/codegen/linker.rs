use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;

pub struct Linker {
    target_triple: String,
    library_paths: Vec<PathBuf>,
    libraries: Vec<String>,
    link_time_optimization: bool,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            library_paths: Vec::new(),
            libraries: Vec::new(),
            link_time_optimization: false,
        }
    }

    pub fn set_target_triple(&mut self, triple: String) {
        self.target_triple = triple;
    }

    pub fn add_library_path<P: AsRef<Path>>(&mut self, path: P) {
        self.library_paths.push(path.as_ref().to_path_buf());
    }

    pub fn add_library(&mut self, lib: String) {
        self.libraries.push(lib);
    }

    pub fn enable_lto(&mut self, enable: bool) {
        self.link_time_optimization = enable;
    }

    pub fn link_executable<P: AsRef<Path>>(
        &self,
        object_files: &[P],
        output: P,
    ) -> Result<()> {
        let mut cmd = self.get_linker_command();
        
        cmd.arg("-o").arg(output.as_ref());
        
        for obj_file in object_files {
            cmd.arg(obj_file.as_ref());
        }
        
        for lib_path in &self.library_paths {
            cmd.arg("-L").arg(lib_path);
        }
        
        for lib in &self.libraries {
            cmd.arg("-l").arg(lib);
        }
        
        if self.link_time_optimization {
            cmd.arg("-flto");
        }
        
        self.add_default_libraries(&mut cmd);
        self.add_runtime_libraries(&mut cmd);
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Linking failed: {}", stderr));
        }
        
        Ok(())
    }

    pub fn link_shared_library<P: AsRef<Path>>(
        &self,
        object_files: &[P],
        output: P,
    ) -> Result<()> {
        let mut cmd = self.get_linker_command();
        
        cmd.arg("-shared");
        cmd.arg("-o").arg(output.as_ref());
        
        for obj_file in object_files {
            cmd.arg(obj_file.as_ref());
        }
        
        for lib_path in &self.library_paths {
            cmd.arg("-L").arg(lib_path);
        }
        
        for lib in &self.libraries {
            cmd.arg("-l").arg(lib);
        }
        
        if self.link_time_optimization {
            cmd.arg("-flto");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Linking failed: {}", stderr));
        }
        
        Ok(())
    }

    pub fn link_static_library<P: AsRef<Path>>(
        &self,
        object_files: &[P],
        output: P,
    ) -> Result<()> {
        let mut cmd = Command::new("ar");
        
        cmd.arg("rcs");
        cmd.arg(output.as_ref());
        
        for obj_file in object_files {
            cmd.arg(obj_file.as_ref());
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Static library creation failed: {}", stderr));
        }
        
        Ok(())
    }

    fn get_linker_command(&self) -> Command {
        if self.target_triple.contains("windows") {
            Command::new("link.exe")
        } else if self.target_triple.contains("darwin") {
            Command::new("ld")
        } else {
            Command::new("ld")
        }
    }

    fn add_default_libraries(&self, cmd: &mut Command) {
        if self.target_triple.contains("linux") {
            cmd.arg("-lc");
            cmd.arg("-lm");
            cmd.arg("-lpthread");
            cmd.arg("-ldl");
        } else if self.target_triple.contains("darwin") {
            cmd.arg("-lSystem");
            cmd.arg("-lm");
        } else if self.target_triple.contains("windows") {
            cmd.arg("kernel32.lib");
            cmd.arg("msvcrt.lib");
        }
    }

    fn add_runtime_libraries(&self, cmd: &mut Command) {
        cmd.arg("-L/usr/lib");
        cmd.arg("-L/lib");
        
        if self.target_triple.contains("x86_64") {
            cmd.arg("-L/usr/lib/x86_64-linux-gnu");
            cmd.arg("-L/lib/x86_64-linux-gnu");
        }
    }

    pub fn create_object_from_assembly<P: AsRef<Path>>(
        &self,
        assembly_file: P,
        output: P,
    ) -> Result<()> {
        let mut cmd = Command::new("as");
        
        if self.target_triple.contains("x86_64") {
            cmd.arg("--64");
        } else if self.target_triple.contains("i686") {
            cmd.arg("--32");
        }
        
        cmd.arg("-o").arg(output.as_ref());
        cmd.arg(assembly_file.as_ref());
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Assembly failed: {}", stderr));
        }
        
        Ok(())
    }

    pub fn get_system_library_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if self.target_triple.contains("linux") {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/lib"));
            
            if self.target_triple.contains("x86_64") {
                paths.push(PathBuf::from("/usr/lib/x86_64-linux-gnu"));
                paths.push(PathBuf::from("/lib/x86_64-linux-gnu"));
            }
        } else if self.target_triple.contains("darwin") {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/System/Library/Frameworks"));
        } else if self.target_triple.contains("windows") {
            if let Ok(windows_kits) = std::env::var("WindowsSdkDir") {
                paths.push(PathBuf::from(windows_kits).join("Lib"));
            }
            
            if let Ok(vs_install) = std::env::var("VCToolsInstallDir") {
                paths.push(PathBuf::from(vs_install).join("lib"));
            }
        }
        
        paths
    }

    pub fn find_library(&self, name: &str) -> Option<PathBuf> {
        let library_name = if self.target_triple.contains("windows") {
            format!("{}.lib", name)
        } else {
            format!("lib{}.a", name)
        };
        
        for path in &self.library_paths {
            let lib_path = path.join(&library_name);
            if lib_path.exists() {
                return Some(lib_path);
            }
        }
        
        for path in self.get_system_library_paths() {
            let lib_path = path.join(&library_name);
            if lib_path.exists() {
                return Some(lib_path);
            }
        }
        
        None
    }

    pub fn strip_executable<P: AsRef<Path>>(&self, executable: P) -> Result<()> {
        let mut cmd = Command::new("strip");
        cmd.arg(executable.as_ref());
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Strip failed: {}", stderr));
        }
        
        Ok(())
    }
}

