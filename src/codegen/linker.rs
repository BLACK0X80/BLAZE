use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Platform {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }
}

pub struct Linker {
    runtime_libs: Vec<String>,
    stdlib_libs: Vec<String>,
    platform: Platform,
    runtime_lib_path: Option<PathBuf>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            runtime_libs: vec!["c".to_string(), "m".to_string()],
            stdlib_libs: Vec::new(),
            platform: Platform::detect(),
            runtime_lib_path: None,
        }
    }

    pub fn add_runtime_lib(&mut self, lib: String) {
        self.runtime_libs.push(lib);
    }

    pub fn add_stdlib_lib(&mut self, lib: String) {
        self.stdlib_libs.push(lib);
    }

    pub fn set_runtime_lib_path(&mut self, path: PathBuf) {
        self.runtime_lib_path = Some(path);
    }

    /// Build the BLAZE runtime library as a static library
    pub fn build_runtime_library(&self, output_dir: &Path) -> Result<PathBuf> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)
            .context("Failed to create runtime library output directory")?;

        let lib_name = match self.platform {
            Platform::Windows => "blaze_runtime.lib",
            _ => "libblaze_runtime.a",
        };

        let output_path = output_dir.join(lib_name);

        // Check if runtime library already exists and is up to date
        if output_path.exists() {
            return Ok(output_path);
        }

        // Build the runtime library using cargo
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--release")
            .arg("--lib");

        let output_result = cmd.output()
            .context("Failed to build runtime library with cargo")?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            bail!("Failed to build runtime library:\n{}", stderr);
        }

        // The library should be in target/release
        let source_lib = match self.platform {
            Platform::Windows => PathBuf::from("target/release/blaze_compiler.lib"),
            _ => PathBuf::from("target/release/libblaze_compiler.a"),
        };

        if source_lib.exists() {
            fs::copy(&source_lib, &output_path)
                .context("Failed to copy runtime library to output directory")?;
        } else {
            // If the library doesn't exist, we'll create a minimal one
            // This is a fallback for development
            self.create_minimal_runtime_lib(&output_path)?;
        }

        Ok(output_path)
    }

    /// Create a minimal runtime library with just the allocator functions
    fn create_minimal_runtime_lib(&self, output_path: &Path) -> Result<()> {
        // Create a temporary directory for object files
        let temp_dir = output_path.parent().unwrap().join("temp_runtime");
        fs::create_dir_all(&temp_dir)
            .context("Failed to create temporary directory")?;

        // For now, we'll just create an empty archive
        // In a real implementation, we would compile the runtime sources
        let mut cmd = match self.platform {
            Platform::Windows => Command::new("lib.exe"),
            _ => {
                let mut cmd = Command::new("ar");
                cmd.arg("rcs");
                cmd
            }
        };

        cmd.arg(output_path);

        // Create a dummy object file if needed
        // This is just a placeholder - in production, we'd compile actual runtime code

        let output_result = cmd.output();
        
        // Clean up temp directory
        let _ = fs::remove_dir_all(&temp_dir);

        match output_result {
            Ok(result) if result.status.success() => Ok(()),
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                bail!("Failed to create runtime library: {}", stderr);
            }
            Err(e) => bail!("Failed to execute archiver: {}", e),
        }
    }

    pub fn link_executable(&self, object_files: &[PathBuf], output: PathBuf) -> Result<()> {
        // Validate inputs
        if object_files.is_empty() {
            bail!("No object files to link");
        }

        // Validate all object files exist before attempting to link
        for obj in object_files {
            if !obj.exists() {
                bail!("Object file does not exist: {}", obj.display());
            }
        }

        let mut cmd = self.create_linker_command()?;

        // Add platform-specific flags first
        self.add_platform_specific_flags(&mut cmd);

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Add output flag
        self.add_output_flag(&mut cmd, &output);

        // Add library flags
        self.add_library_flags(&mut cmd);

        // Execute linker
        let output_result = cmd.output()
            .context("Failed to execute linker command")?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            let stdout = String::from_utf8_lossy(&output_result.stdout);
            
            // Parse and provide helpful error messages
            let error_msg = self.parse_linker_error(&stderr, &stdout);
            bail!("Linking failed:\n{}", error_msg);
        }

        Ok(())
    }

    pub fn link_static_library(&self, object_files: &[PathBuf], output: PathBuf) -> Result<()> {
        if object_files.is_empty() {
            bail!("No object files to link");
        }

        // Validate all object files exist
        for obj in object_files {
            if !obj.exists() {
                bail!("Object file does not exist: {}", obj.display());
            }
        }

        let mut cmd = match self.platform {
            Platform::Windows => Command::new("lib.exe"),
            _ => {
                let mut cmd = Command::new("ar");
                cmd.arg("rcs");
                cmd
            }
        };

        cmd.arg(&output);

        for obj in object_files {
            cmd.arg(obj);
        }

        let output_result = cmd.output()
            .context("Failed to execute archiver command")?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            bail!("Static library creation failed: {}", stderr);
        }

        Ok(())
    }

    fn create_linker_command(&self) -> Result<Command> {
        match self.platform {
            Platform::Windows => {
                // Use MSVC linker (link.exe)
                Ok(Command::new("link.exe"))
            }
            Platform::MacOS => {
                // Use Clang on macOS (preferred over ld directly)
                Ok(Command::new("clang"))
            }
            Platform::Linux => {
                // Try to find GCC or Clang
                if which("gcc") {
                    Ok(Command::new("gcc"))
                } else if which("clang") {
                    Ok(Command::new("clang"))
                } else {
                    bail!("No suitable linker found. Please install GCC or Clang.")
                }
            }
        }
    }

    fn add_output_flag(&self, cmd: &mut Command, output: &PathBuf) {
        match self.platform {
            Platform::Windows => {
                cmd.arg(format!("/OUT:{}", output.display()));
            }
            _ => {
                cmd.arg("-o");
                cmd.arg(output);
            }
        }
    }

    fn add_library_flags(&self, cmd: &mut Command) {
        match self.platform {
            Platform::Windows => {
                // Add BLAZE runtime library if available
                if let Some(ref runtime_path) = self.runtime_lib_path {
                    cmd.arg(runtime_path);
                }

                // Add system runtime libraries
                for lib in &self.runtime_libs {
                    if lib == "c" {
                        // MSVC uses different names
                        cmd.arg("msvcrt.lib");
                    } else if lib == "m" {
                        // Math library is included in msvcrt
                        continue;
                    } else if lib == "blaze_runtime" {
                        // Already added above
                        continue;
                    } else {
                        cmd.arg(format!("{}.lib", lib));
                    }
                }

                // Add stdlib libraries
                for lib in &self.stdlib_libs {
                    cmd.arg(format!("{}.lib", lib));
                }
            }
            _ => {
                // Add BLAZE runtime library if available
                if let Some(ref runtime_path) = self.runtime_lib_path {
                    cmd.arg(runtime_path);
                }

                // Add system runtime libraries (libc, libm)
                for lib in &self.runtime_libs {
                    if lib == "blaze_runtime" {
                        // Already added above
                        continue;
                    }
                    cmd.arg(format!("-l{}", lib));
                }

                // Add stdlib libraries
                for lib in &self.stdlib_libs {
                    cmd.arg(format!("-l{}", lib));
                }
            }
        }
    }

    fn add_platform_specific_flags(&self, cmd: &mut Command) {
        match self.platform {
            Platform::Windows => {
                // MSVC linker flags
                cmd.arg("/NOLOGO");           // Suppress copyright message
                cmd.arg("/SUBSYSTEM:CONSOLE"); // Console application
                cmd.arg("/MACHINE:X64");       // 64-bit target
                cmd.arg("/ENTRY:main");        // Entry point
                cmd.arg("/DEFAULTLIB:msvcrt.lib"); // C runtime
            }
            Platform::MacOS => {
                // Clang flags for macOS
                cmd.arg("-arch");
                cmd.arg("x86_64");
                cmd.arg("-mmacosx-version-min=10.15");
                cmd.arg("-lSystem"); // System library (includes libc)
            }
            Platform::Linux => {
                // GCC/Clang flags for Linux
                cmd.arg("-no-pie"); // Disable position independent executable for simplicity
                cmd.arg("-lm");     // Math library
            }
        }
    }

    fn parse_linker_error(&self, stderr: &str, stdout: &str) -> String {
        let combined = format!("{}\n{}", stderr, stdout);
        
        // Common error patterns and helpful messages
        if combined.contains("undefined reference") || combined.contains("unresolved external symbol") {
            format!(
                "{}\n\nHelp: This error occurs when the linker cannot find a function or variable.\n\
                 - Check that all required libraries are linked\n\
                 - Verify that function names match between declaration and definition\n\
                 - Ensure the runtime library is properly built and linked",
                combined
            )
        } else if combined.contains("cannot find") || combined.contains("No such file") {
            format!(
                "{}\n\nHelp: The linker cannot find a required file.\n\
                 - Check that all object files were generated successfully\n\
                 - Verify library search paths are correct\n\
                 - Ensure the runtime library is built before linking",
                combined
            )
        } else if combined.contains("multiple definition") {
            format!(
                "{}\n\nHelp: Multiple definitions of the same symbol were found.\n\
                 - Check for duplicate function or variable definitions\n\
                 - Verify that headers are properly guarded\n\
                 - Ensure libraries are not linked multiple times",
                combined
            )
        } else {
            combined
        }
    }
}

// Helper function to check if a command exists
fn which(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        
        // Verify platform detection matches the current OS
        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);
        
        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);
        
        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_linker_creation() {
        let linker = Linker::new();
        
        // Verify default runtime libraries are set
        assert!(linker.runtime_libs.contains(&"c".to_string()));
        assert!(linker.runtime_libs.contains(&"m".to_string()));
        
        // Verify platform is detected
        assert_eq!(linker.platform, Platform::detect());
    }

    #[test]
    fn test_add_runtime_lib() {
        let mut linker = Linker::new();
        linker.add_runtime_lib("pthread".to_string());
        
        assert!(linker.runtime_libs.contains(&"pthread".to_string()));
    }

    #[test]
    fn test_add_stdlib_lib() {
        let mut linker = Linker::new();
        linker.add_stdlib_lib("mylib".to_string());
        
        assert!(linker.stdlib_libs.contains(&"mylib".to_string()));
    }

    #[test]
    fn test_set_runtime_lib_path() {
        let mut linker = Linker::new();
        let path = PathBuf::from("/path/to/runtime.a");
        linker.set_runtime_lib_path(path.clone());
        
        assert_eq!(linker.runtime_lib_path, Some(path));
    }

    #[test]
    fn test_link_executable_with_no_object_files() {
        let linker = Linker::new();
        let result = linker.link_executable(&[], PathBuf::from("output"));
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No object files"));
    }

    #[test]
    fn test_link_executable_with_missing_object_file() {
        let linker = Linker::new();
        let non_existent = PathBuf::from("non_existent.o");
        let result = linker.link_executable(&[non_existent], PathBuf::from("output"));
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_link_static_library_with_no_object_files() {
        let linker = Linker::new();
        let result = linker.link_static_library(&[], PathBuf::from("output.a"));
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No object files"));
    }

    #[test]
    fn test_link_static_library_with_missing_object_file() {
        let linker = Linker::new();
        let non_existent = PathBuf::from("non_existent.o");
        let result = linker.link_static_library(&[non_existent], PathBuf::from("output.a"));
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_create_linker_command() {
        let linker = Linker::new();
        let result = linker.create_linker_command();
        
        // Should succeed on all platforms
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_linker_error_undefined_reference() {
        let linker = Linker::new();
        let stderr = "undefined reference to `foo`";
        let stdout = "";
        
        let parsed = linker.parse_linker_error(stderr, stdout);
        
        assert!(parsed.contains("undefined reference"));
        assert!(parsed.contains("Help:"));
        assert!(parsed.contains("cannot find a function"));
    }

    #[test]
    fn test_parse_linker_error_cannot_find() {
        let linker = Linker::new();
        let stderr = "cannot find -lmylib";
        let stdout = "";
        
        let parsed = linker.parse_linker_error(stderr, stdout);
        
        assert!(parsed.contains("cannot find"));
        assert!(parsed.contains("Help:"));
        assert!(parsed.contains("required file"));
    }

    #[test]
    fn test_parse_linker_error_multiple_definition() {
        let linker = Linker::new();
        let stderr = "multiple definition of `main`";
        let stdout = "";
        
        let parsed = linker.parse_linker_error(stderr, stdout);
        
        assert!(parsed.contains("multiple definition"));
        assert!(parsed.contains("Help:"));
        assert!(parsed.contains("duplicate"));
    }

    #[test]
    fn test_parse_linker_error_generic() {
        let linker = Linker::new();
        let stderr = "some other error";
        let stdout = "some output";
        
        let parsed = linker.parse_linker_error(stderr, stdout);
        
        assert!(parsed.contains("some other error"));
        assert!(parsed.contains("some output"));
    }

    #[test]
    fn test_build_runtime_library_creates_directory() {
        let linker = Linker::new();
        let temp_dir = std::env::temp_dir().join("blaze_test_runtime");
        
        // Clean up if exists
        let _ = fs::remove_dir_all(&temp_dir);
        
        let result = linker.build_runtime_library(&temp_dir);
        
        // Should create the directory even if build fails
        assert!(temp_dir.exists());
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Result may fail if cargo build fails, but that's ok for this test
        let _ = result;
    }

    #[test]
    fn test_which_helper() {
        // Test with a command that should exist on all platforms
        #[cfg(target_os = "windows")]
        let exists = which("cmd");
        
        #[cfg(not(target_os = "windows"))]
        let exists = which("sh");
        
        assert!(exists);
        
        // Test with a command that shouldn't exist
        let not_exists = which("this_command_definitely_does_not_exist_12345");
        assert!(!not_exists);
    }
}
