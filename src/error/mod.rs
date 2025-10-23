//! Error types and diagnostic system for the BLAZE compiler.
//!
//! This module provides comprehensive error handling with rich diagnostics,
//! including source code context, colored output, and helpful suggestions.
//!
//! # Examples
//!
//! ```rust,no_run
//! use blaze::error::{CompileError, Diagnostic, Severity};
//!
//! // Create a type error with suggestion
//! let error = CompileError::TypeError {
//!     message: "mismatched types".to_string(),
//!     expected: Some("i32".to_string()),
//!     found: Some("bool".to_string()),
//!     line: Some(5),
//!     column: Some(10),
//!     suggestion: Some("try converting the boolean to an integer".to_string()),
//! };
//!
//! println!("{}", error);
//! ```

/// Diagnostic system for rich error reporting
pub mod diagnostics;

/// Suggestion generation for common errors
pub mod suggestions;

use std::fmt;
pub use diagnostics::{Diagnostic, DiagnosticBuilder, DiagnosticEmitter, DiagnosticCollector, Severity};
pub use suggestions::{did_you_mean, suggest_type_conversion, suggest_borrow_fix, suggest_syntax_fix, example_for_pattern};

/// Compilation error types.
///
/// This enum represents all possible errors that can occur during compilation,
/// from lexical analysis through code generation.
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Lexical analysis error (invalid token).
    ///
    /// Occurs when the lexer encounters invalid characters or malformed tokens.
    LexError {
        /// Error message describing what went wrong
        message: String,
        /// Line number where the error occurred (1-indexed)
        line: usize,
        /// Column number where the error occurred (1-indexed)
        column: usize,
    },
    
    /// Syntax analysis error (invalid grammar).
    ///
    /// Occurs when the parser encounters unexpected tokens or invalid syntax.
    ParseError {
        /// Error message describing what went wrong
        message: String,
        /// Line number where the error occurred (1-indexed)
        line: usize,
        /// Column number where the error occurred (1-indexed)
        column: usize,
    },
    
    /// Type checking error (type mismatch).
    ///
    /// Occurs when types don't match in expressions, assignments, or function calls.
    TypeError {
        /// Error message describing the type mismatch
        message: String,
        /// Expected type (if known)
        expected: Option<String>,
        /// Found type (if known)
        found: Option<String>,
        /// Line number where the error occurred (1-indexed)
        line: Option<usize>,
        /// Column number where the error occurred (1-indexed)
        column: Option<usize>,
        /// Helpful suggestion for fixing the error
        suggestion: Option<String>,
    },
    
    /// I/O error (file operations).
    ///
    /// Occurs when file reading, writing, or other I/O operations fail.
    IoError {
        /// Error message describing what went wrong
        message: String,
    },
    
    /// Semantic analysis error (borrow checking, lifetime analysis).
    ///
    /// Occurs during semantic analysis when code violates language rules.
    SemanticError {
        /// Error message describing what went wrong
        message: String,
    },
    
    /// Code generation error (LLVM backend, linking).
    ///
    /// Occurs during code generation or linking phases.
    CodegenError {
        /// Error message describing what went wrong
        message: String,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::LexError {
                message,
                line,
                column,
            } => {
                write!(f, "Lexer error at {}:{}: {}", line, column, message)
            }
            CompileError::ParseError {
                message,
                line,
                column,
            } => {
                write!(f, "Parser error at {}:{}: {}", line, column, message)
            }
            CompileError::TypeError {
                message,
                expected,
                found,
                line,
                column,
                suggestion,
            } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "Type error at {}:{}: {}", line, column, message)?;
                } else {
                    write!(f, "Type error: {}", message)?;
                }
                
                if let Some(exp) = expected {
                    write!(f, "\n  expected: {}", exp)?;
                }
                
                if let Some(fnd) = found {
                    write!(f, "\n  found: {}", fnd)?;
                }
                
                if let Some(sug) = suggestion {
                    write!(f, "\n  help: {}", sug)?;
                }
                
                Ok(())
            }
            CompileError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
            CompileError::SemanticError { message } => {
                write!(f, "Semantic error: {}", message)
            }
            CompileError::CodegenError { message } => {
                write!(f, "Code generation error: {}", message)
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Result type for compilation operations.
///
/// This is a convenience type alias for `Result<T, CompileError>`.
///
/// # Examples
///
/// ```rust,no_run
/// use blaze::error::Result;
///
/// fn parse_number(s: &str) -> Result<i32> {
///     s.parse().map_err(|_| blaze::error::CompileError::TypeError {
///         message: "invalid number".to_string(),
///         expected: Some("i32".to_string()),
///         found: Some(format!("'{}'", s)),
///         line: None,
///         column: None,
///         suggestion: Some("ensure the value is a valid integer".to_string()),
///     })
/// }
/// ```
pub type Result<T> = std::result::Result<T, CompileError>;
