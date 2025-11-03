use blaze::{Compiler, CompileOptions};
use std::fs;
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            temp_dir: tempfile::tempdir().unwrap(),
        }
    }
    
    pub fn write_file(&self, name: &str, content: &str) -> String {
        let path = self.temp_dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_str().unwrap().to_string()
    }
    
    pub fn compile(&self, source: &str) -> Result<String, String> {
        let file_path = self.write_file("test.blz", source);
        
        let mut compiler = Compiler::new();
        let options = CompileOptions {
            input_file: file_path,
            output_file: None,
            optimization_level: 0,
            emit_ir: false,
            emit_asm: false,
            verbose: false,
        };
        
        match compiler.compile(&options) {
            Ok(_) => Ok("Compilation successful".to_string()),
            Err(e) => Err(format!("{}", e)),
        }
    }
    
    pub fn compile_and_run(&self, source: &str) -> Result<String, String> {
        let file_path = self.write_file("test.blz", source);
        let output_path = self.temp_dir.path().join("test_output");
        
        let mut compiler = Compiler::new();
        let options = CompileOptions {
            input_file: file_path,
            output_file: Some(output_path.to_str().unwrap().to_string()),
            optimization_level: 0,
            emit_ir: false,
            emit_asm: false,
            verbose: false,
        };
        
        compiler.compile(&options).map_err(|e| format!("{}", e))?;
        
        let output = std::process::Command::new(&output_path)
            .output()
            .map_err(|e| format!("Failed to run: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    pub fn expect_error(&self, source: &str, expected_error: &str) {
        match self.compile(source) {
            Ok(_) => panic!("Expected compilation to fail, but it succeeded"),
            Err(e) => assert!(
                e.contains(expected_error),
                "Expected error containing '{}', but got: {}",
                expected_error,
                e
            ),
        }
    }
    
    pub fn expect_success(&self, source: &str) {
        match self.compile(source) {
            Ok(_) => {},
            Err(e) => panic!("Expected compilation to succeed, but got error: {}", e),
        }
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! assert_compiles {
    ($source:expr) => {
        let ctx = TestContext::new();
        ctx.expect_success($source);
    };
}

#[macro_export]
macro_rules! assert_compile_error {
    ($source:expr, $error:expr) => {
        let ctx = TestContext::new();
        ctx.expect_error($source, $error);
    };
}

#[macro_export]
macro_rules! assert_output {
    ($source:expr, $expected:expr) => {
        let ctx = TestContext::new();
        match ctx.compile_and_run($source) {
            Ok(output) => assert_eq!(output.trim(), $expected.trim()),
            Err(e) => panic!("Compilation/execution failed: {}", e),
        }
    };
}
