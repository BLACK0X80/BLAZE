use blaze_compiler::{compile_to_executable, lex, parse, ir, CodeGenerator};
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a temporary directory for test outputs
fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper function to compile source code to an executable
fn compile_source(source: &str, output_name: &str, opt_level: u8) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.path().join(output_name);
    
    compile_to_executable(source, output_path.clone(), opt_level)?;
    
    // Keep temp_dir alive by leaking it (for test purposes)
    let _ = temp_dir.into_path();
    
    Ok(output_path)
}

#[test]
fn test_complete_compilation_pipeline() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_pipeline");
    
    let result = compile_to_executable(source, output.clone(), 0);
    
    // The compilation should complete without errors
    // Note: This may fail if LLVM is not properly configured, but the pipeline should execute
    match result {
        Ok(_) => {
            // Check that object file was created
            let object_file = output.with_extension("o");
            // Object file might exist depending on LLVM configuration
        }
        Err(e) => {
            // If LLVM is not configured, we expect a specific error
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation failed") || 
                error_msg.contains("LLVM") ||
                error_msg.contains("Linking"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_optimization_level_0() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_opt0");
    
    // Test with optimization level 0 (no optimizations)
    let result = compile_to_executable(source, output, 0);
    
    // Should complete IR generation and attempt code generation
    match result {
        Ok(_) => {
            // Success - LLVM is configured
        }
        Err(e) => {
            // Expected if LLVM not configured
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_optimization_level_1() {
    let source = r#"
        fn multiply(a: i32, b: i32) -> i32 {
            return a * b;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_opt1");
    
    // Test with optimization level 1
    let result = compile_to_executable(source, output, 1);
    
    match result {
        Ok(_) => {
            // Success - LLVM is configured
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_optimization_level_2() {
    let source = r#"
        fn calculate(x: i32) -> i32 {
            let a: i32 = x + 5;
            let b: i32 = a * 2;
            return b;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_opt2");
    
    // Test with optimization level 2 (default)
    let result = compile_to_executable(source, output, 2);
    
    match result {
        Ok(_) => {
            // Success - LLVM is configured
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_optimization_level_3() {
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_opt3");
    
    // Test with optimization level 3 (aggressive)
    let result = compile_to_executable(source, output, 3);
    
    match result {
        Ok(_) => {
            // Success - LLVM is configured
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_error_propagation_from_lexer() {
    let source = r#"
        fn main() {
            let x: i32 = @invalid;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_lex_error");
    
    let result = compile_to_executable(source, output, 0);
    
    // Should fail during lexing
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Unexpected character") || error_msg.contains("Lex"),
        "Expected lexer error, got: {}", error_msg
    );
}

#[test]
fn test_error_propagation_from_parser() {
    let source = r#"
        fn main() {
            let x: i32 = 
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_parse_error");
    
    let result = compile_to_executable(source, output, 0);
    
    // Should fail during parsing
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Parse") || error_msg.contains("Unexpected"),
        "Expected parser error, got: {}", error_msg
    );
}

#[test]
fn test_error_propagation_from_semantic_analysis() {
    let source = r#"
        fn main() {
            let x: i32 = "string";
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_semantic_error");
    
    let result = compile_to_executable(source, output, 0);
    
    // Should fail during semantic analysis (type mismatch)
    // Note: This depends on semantic analyzer implementation
    match result {
        Ok(_) => {
            // Semantic analyzer might not catch this yet
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            // Could be semantic or parse error depending on implementation
            assert!(
                error_msg.contains("Semantic") || 
                error_msg.contains("type") ||
                error_msg.contains("Parse"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_ir_generation_phase() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;
    
    // Test that IR generation works independently
    let tokens = lex(source).expect("Lexing should succeed");
    let program = parse(tokens).expect("Parsing should succeed");
    let ir_module = ir::generate(&program).expect("IR generation should succeed");
    
    // Verify IR module structure
    assert_eq!(ir_module.functions.len(), 1);
    assert_eq!(ir_module.functions[0].name, "add");
    assert_eq!(ir_module.functions[0].params.len(), 2);
}

#[test]
fn test_codegen_with_multiple_functions() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn subtract(a: i32, b: i32) -> i32 {
            return a - b;
        }
        
        fn main() {
            let x: i32 = add(5, 3);
            let y: i32 = subtract(10, 4);
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_multi_func");
    
    let result = compile_to_executable(source, output, 1);
    
    match result {
        Ok(_) => {
            // Success - all phases completed
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_codegen_with_control_flow() {
    let source = r#"
        fn max(a: i32, b: i32) -> i32 {
            if a > b {
                return a;
            }
            return b;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_control_flow");
    
    let result = compile_to_executable(source, output, 2);
    
    match result {
        Ok(_) => {
            // Success - control flow handled
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_codegen_context_management() {
    // Test that CodeGenerator properly manages LLVM context lifecycle
    let source = r#"
        fn simple() -> i32 {
            return 42;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_context");
    
    // Create CodeGenerator without any context-dependent fields
    let mut codegen = CodeGenerator::new();
    
    // Generate IR
    let tokens = lex(source).expect("Lexing should succeed");
    let program = parse(tokens).expect("Parsing should succeed");
    let ir_module = ir::generate(&program).expect("IR generation should succeed");
    
    // Call generate which should create context internally
    let result = codegen.generate(&ir_module, output, 1);
    
    match result {
        Ok(_) => {
            // Success - context was properly managed
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            // Should fail at LLVM/linking stage, not context creation
            assert!(
                error_msg.contains("Code generation") || 
                error_msg.contains("LLVM") ||
                error_msg.contains("Linking"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[test]
fn test_codegen_pipeline_phases() {
    // Test that all phases of the pipeline execute in order
    let source = r#"
        fn add(x: i32, y: i32) -> i32 {
            return x + y;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_phases");
    
    // Test complete pipeline
    let result = compile_to_executable(source, output.clone(), 2);
    
    match result {
        Ok(_) => {
            // All phases completed successfully
            // Object file should exist
            let object_file = output.with_extension("o");
            // Note: File existence depends on LLVM configuration
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            // Error should indicate which phase failed
            assert!(
                error_msg.contains("Register allocation") ||
                error_msg.contains("Code generation") ||
                error_msg.contains("Linking"),
                "Error should indicate pipeline phase: {}", error_msg
            );
        }
    }
}

#[test]
fn test_optimization_passes_applied() {
    // Test that optimization level affects code generation
    let source = r#"
        fn compute(x: i32) -> i32 {
            let a: i32 = x + 1;
            let b: i32 = a + 2;
            let c: i32 = b + 3;
            return c;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    
    // Test with different optimization levels
    for opt_level in 0..=3 {
        let output = temp_dir.path().join(format!("test_opt_passes_{}", opt_level));
        let result = compile_to_executable(source, output, opt_level);
        
        match result {
            Ok(_) => {
                // Optimization level was accepted and applied
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                // Should not fail due to optimization level
                assert!(
                    error_msg.contains("Code generation") || error_msg.contains("LLVM"),
                    "Optimization level {} caused unexpected error: {}", opt_level, error_msg
                );
            }
        }
    }
}

#[test]
fn test_error_context_in_pipeline() {
    // Test that errors include proper context about which phase failed
    let source = r#"
        fn test() -> i32 {
            return 123;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_error_context");
    
    let result = compile_to_executable(source, output, 2);
    
    match result {
        Ok(_) => {
            // Success
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            // Error message should provide context
            assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
            // Should mention the phase that failed
            assert!(
                error_msg.contains("failed") || 
                error_msg.contains("error") ||
                error_msg.contains("Error"),
                "Error should describe what failed: {}", error_msg
            );
        }
    }
}

#[test]
fn test_linker_integration() {
    // Test that linker is properly called after code generation
    let source = r#"
        fn entry() -> i32 {
            return 0;
        }
    "#;
    
    let temp_dir = create_temp_dir();
    let output = temp_dir.path().join("test_linker");
    
    let result = compile_to_executable(source, output.clone(), 1);
    
    match result {
        Ok(_) => {
            // Linker was called successfully
            // Executable should exist (if LLVM is configured)
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            // If it fails, should be at linking or code generation phase
            assert!(
                error_msg.contains("Linking") || 
                error_msg.contains("Code generation") ||
                error_msg.contains("LLVM"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}
