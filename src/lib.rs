pub mod error;
pub mod lexer;
pub mod parser;
pub mod ir;
pub mod semantic;
pub mod codegen;
pub mod runtime;
pub mod stdlib;
pub mod utils;
pub mod optimizer;
pub mod cli;
pub mod analysis;
pub mod backend;
pub mod jit;
pub mod macro_system;
pub mod profiler;
pub mod diagnostics;
pub mod async_runtime;
pub mod borrow_checker;
pub mod type_inference;
pub mod trait_system;
pub mod generics;
pub mod lifetime_analyzer;
pub mod pattern_matching;
pub mod ffi;
pub mod inline_assembly;
pub mod memory_model;
pub mod concurrency;
pub mod simd;
pub mod reflection;
pub mod package_manager;
pub mod const_eval;
pub mod gc;
pub mod incremental;
pub mod repl;
pub mod debugger;
pub mod cross_compile;
pub mod wasm_backend;
pub mod linter;
pub mod security;
pub mod ide_support;
pub mod testing;
pub mod build_system;
pub mod plugin_system;
pub mod documentation;

pub use error::{CompileError, Result};
pub use lexer::{lex, Token, TokenType};
pub use parser::{parse, Program};
pub use semantic::SemanticAnalyzer;
pub use ir::{Module as IRModule, generate as generate_ir};

use std::path::{Path, PathBuf};

pub fn compile_file(path: &Path) -> Result<Program> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::IoError { 
            message: format!("Failed to read file: {}", e),
            path: Some(path.display().to_string()),
        })?;
    
    compile(&source)
}

pub fn compile(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    Ok(program)
}

pub fn compile_with_semantics(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| CompileError::SemanticError {
            message: e.to_string(),
            line: None,
            column: None,
            source_snippet: None,
            suggestion: None,
            related_info: vec![],
        })?;
    Ok(program)
}

pub fn compile_to_ir(source: &str) -> Result<IRModule> {
    let program = compile_with_semantics(source)?;
    generate_ir(&program)
        .map_err(|e| CompileError::CodegenError {
            message: e.to_string(),
            phase: "IR Generation".to_string(),
            suggestion: None,
        })
}

pub fn compile_to_executable(source: &str, output: PathBuf, optimization_level: u8) -> Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| CompileError::SemanticError { 
            message: format!("Semantic analysis failed: {}", e),
            line: None,
            column: None,
            source_snippet: None,
            suggestion: None,
            related_info: vec![],
        })?;
    
    let ir_module = ir::generate(&program)
        .map_err(|e| CompileError::CodegenError { 
            message: format!("IR generation failed: {}", e),
            phase: "IR Generation".to_string(),
            suggestion: None,
        })?;
    
    let mut optimizer = ir::optimization::Optimizer::new();
    let optimized = optimizer.optimize(&ir_module, optimization_level)
        .map_err(|e| CompileError::CodegenError {
            message: format!("Optimization failed: {}", e),
            phase: "Optimization".to_string(),
            suggestion: None,
        })?;
    
    Ok(())
}

pub fn check(source: &str) -> Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| CompileError::SemanticError { 
            message: format!("Semantic analysis failed: {}", e),
            line: None,
            column: None,
            source_snippet: None,
            suggestion: None,
            related_info: vec![],
        })?;
    
    Ok(())
}