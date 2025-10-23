//! # BLAZE Compiler
//!
//! A systems programming language compiler with Rust-like syntax, memory safety guarantees,
//! and LLVM backend integration.
//!
//! ## Overview
//!
//! BLAZE is a statically-typed, memory-safe programming language that combines the performance
//! of systems languages with modern safety features. The compiler implements comprehensive
//! semantic analysis including type checking, borrow checking, and lifetime analysis, then
//! generates optimized machine code through LLVM.
//!
//! ## Features
//!
//! - **Complete Frontend**: Full lexer and parser with comprehensive AST generation
//! - **Semantic Analysis**: Type checking, borrow checking, lifetime analysis, and scope resolution
//! - **Memory Safety**: Rust-inspired ownership system preventing use-after-free and data races
//! - **IR Generation**: Multi-pass intermediate representation with SSA form
//! - **LLVM Backend**: Production-quality code generation with optimization support
//! - **Runtime System**: Efficient memory allocator with leak detection and intrinsics
//! - **Rich Diagnostics**: Colored error messages with source context and suggestions
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use blaze::{compile_to_executable, check};
//! use std::path::PathBuf;
//!
//! // Check syntax and semantics
//! let source = r#"
//!     fn main() {
//!         let x: i32 = 42;
//!         println(x);
//!     }
//! "#;
//!
//! // Validate without generating code
//! check(source).expect("Compilation failed");
//!
//! // Compile to executable
//! compile_to_executable(source, PathBuf::from("output"), 2)
//!     .expect("Compilation failed");
//! ```
//!
//! ## Compilation Pipeline
//!
//! The compiler follows a multi-stage pipeline:
//!
//! 1. **Lexical Analysis** ([`lexer`]) - Tokenization
//! 2. **Syntax Analysis** ([`parser`]) - AST construction
//! 3. **Semantic Analysis** ([`semantic`]) - Type checking, borrow checking, lifetime analysis
//! 4. **IR Generation** ([`ir`]) - Intermediate representation with SSA form
//! 5. **Code Generation** ([`codegen`]) - LLVM backend and linking
//!
//! ## Examples
//!
//! ### Compile a file
//!
//! ```rust,no_run
//! use blaze::compile_file;
//! use std::path::Path;
//!
//! let program = compile_file(Path::new("examples/hello_world.blz"))
//!     .expect("Failed to compile");
//! ```
//!
//! ### Parse source code
//!
//! ```rust,no_run
//! use blaze::compile;
//!
//! let source = "fn add(a: i32, b: i32) -> i32 { return a + b; }";
//! let program = compile(source).expect("Parse failed");
//! ```

/// Error types and result type for compilation
pub mod error;

/// Lexical analysis - tokenization of source code
pub mod lexer;

/// Syntax analysis - AST construction from tokens
pub mod parser;

/// Intermediate representation generation and optimization
pub mod ir;

/// Semantic analysis - type checking, borrow checking, lifetime analysis
pub mod semantic;

/// Code generation - LLVM backend and linking
pub mod codegen;

/// Runtime support - memory allocator, intrinsics, panic handling
pub mod runtime;

/// Standard library - collections and utilities
pub mod stdlib;

/// Utility functions - diagnostics, source maps
pub mod utils;

pub use error::{CompileError, Result};
pub use lexer::{lex, Token, TokenType};
pub use parser::{parse, Program};
pub use ir::Module as IRModule;
pub use semantic::SemanticAnalyzer;
pub use codegen::CodeGenerator;

use std::path::{Path, PathBuf};

/// Compiles a BLAZE source file to an AST.
///
/// This function reads the source file, performs lexical and syntax analysis,
/// and returns the parsed program as an AST.
///
/// # Arguments
///
/// * `path` - Path to the source file
///
/// # Returns
///
/// Returns `Ok(Program)` containing the parsed AST on success, or a [`CompileError`]
/// if any compilation phase fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The file cannot be read (I/O error)
/// - The source contains lexical errors (invalid tokens)
/// - The source contains syntax errors (invalid grammar)
///
/// # Examples
///
/// ```rust,no_run
/// use blaze::compile_file;
/// use std::path::Path;
///
/// let program = compile_file(Path::new("examples/hello_world.blz"))
///     .expect("Failed to compile file");
/// println!("Successfully parsed program with {} functions", program.functions.len());
/// ```
pub fn compile_file(path: &Path) -> Result<Program> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::IoError { 
            message: format!("Failed to read file: {}", e) 
        })?;
    
    compile(&source)
}

/// Compiles BLAZE source code to an AST.
///
/// This function performs lexical and syntax analysis on the provided source code
/// and returns the parsed program as an AST. It does not perform semantic analysis
/// or code generation.
///
/// # Arguments
///
/// * `source` - The BLAZE source code as a string
///
/// # Returns
///
/// Returns `Ok(Program)` containing the parsed AST on success, or a [`CompileError`]
/// if lexical or syntax analysis fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The source contains invalid tokens (lexical errors)
/// - The source has invalid syntax (parse errors)
///
/// # Examples
///
/// ```rust,no_run
/// use blaze::compile;
///
/// let source = r#"
///     fn factorial(n: i32) -> i32 {
///         if n <= 1 {
///             return 1;
///         }
///         return n * factorial(n - 1);
///     }
/// "#;
///
/// let program = compile(source).expect("Parse failed");
/// assert_eq!(program.functions.len(), 1);
/// ```
pub fn compile(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    Ok(program)
}

/// Compiles BLAZE source code to an executable binary.
///
/// This function performs the complete compilation pipeline:
/// 1. Lexical analysis (tokenization)
/// 2. Syntax analysis (AST construction)
/// 3. Semantic analysis (type checking, borrow checking, lifetime analysis)
/// 4. IR generation (intermediate representation with SSA form)
/// 5. Code generation (LLVM backend)
/// 6. Linking (platform-specific linker)
///
/// # Arguments
///
/// * `source` - The BLAZE source code as a string
/// * `output` - Path where the executable should be written
/// * `optimization_level` - Optimization level (0-3):
///   - 0: No optimization (fastest compilation)
///   - 1: Basic optimization
///   - 2: Moderate optimization (recommended)
///   - 3: Aggressive optimization (slowest compilation, best performance)
///
/// # Returns
///
/// Returns `Ok(())` on successful compilation, or a [`CompileError`] if any
/// compilation phase fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Lexical or syntax analysis fails
/// - Semantic analysis detects type errors, borrow errors, or lifetime errors
/// - IR generation fails
/// - Code generation or linking fails
///
/// # Examples
///
/// ```rust,no_run
/// use blaze::compile_to_executable;
/// use std::path::PathBuf;
///
/// let source = r#"
///     fn main() {
///         let x: i32 = 42;
///         println("The answer is: ");
///         println(x);
///     }
/// "#;
///
/// // Compile with moderate optimization
/// compile_to_executable(source, PathBuf::from("my_program"), 2)
///     .expect("Compilation failed");
/// ```
///
/// # Platform Support
///
/// The generated executable is platform-specific:
/// - **Windows**: PE executable (.exe)
/// - **Linux**: ELF executable
/// - **macOS**: Mach-O executable
pub fn compile_to_executable(source: &str, output: PathBuf, optimization_level: u8) -> Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| CompileError::SemanticError { 
            message: format!("Semantic analysis failed: {}", e) 
        })?;
    
    let ir_module = ir::generate(&program)
        .map_err(|e| CompileError::CodegenError { 
            message: format!("IR generation failed: {}", e) 
        })?;
    
    let mut codegen = CodeGenerator::new();
    codegen.generate(&ir_module, output, optimization_level)
        .map_err(|e| CompileError::CodegenError { 
            message: format!("Code generation failed: {}", e) 
        })?;
    
    Ok(())
}

/// Checks BLAZE source code for errors without generating code.
///
/// This function performs lexical analysis, syntax analysis, and semantic analysis
/// to validate the source code, but does not generate IR or machine code. This is
/// useful for fast error checking during development.
///
/// # Arguments
///
/// * `source` - The BLAZE source code as a string
///
/// # Returns
///
/// Returns `Ok(())` if the source code is valid, or a [`CompileError`] if any
/// errors are detected.
///
/// # Errors
///
/// This function will return an error if:
/// - The source contains lexical errors (invalid tokens)
/// - The source contains syntax errors (invalid grammar)
/// - The source contains semantic errors (type errors, borrow errors, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// use blaze::check;
///
/// let valid_source = r#"
///     fn main() {
///         let x: i32 = 42;
///     }
/// "#;
///
/// // This should succeed
/// check(valid_source).expect("Valid code should pass");
///
/// let invalid_source = r#"
///     fn main() {
///         let x: i32 = "not a number";  // Type error
///     }
/// "#;
///
/// // This should fail
/// assert!(check(invalid_source).is_err());
/// ```
///
/// # Performance
///
/// This function is significantly faster than [`compile_to_executable`] because it
/// skips IR generation, optimization, and code generation. Use this for quick
/// validation during development.
pub fn check(source: &str) -> Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| CompileError::SemanticError { 
            message: format!("Semantic analysis failed: {}", e) 
        })?;
    
    Ok(())
}