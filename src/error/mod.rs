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

/// Beautiful error messages with colors and suggestions
pub mod beautiful;

use std::fmt;
pub use diagnostics::{Diagnostic, DiagnosticBuilder, DiagnosticEmitter, DiagnosticCollector, Severity};
pub use suggestions::{did_you_mean, suggest_type_conversion, suggest_borrow_fix, suggest_syntax_fix, example_for_pattern};

/// Information about a borrow (immutable or mutable)
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub borrow_type: BorrowType,
    pub location: String,
    pub line: usize,
    pub column: usize,
}

/// Type of borrow
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

/// Information about a lifetime
#[derive(Debug, Clone)]
pub struct LifetimeInfo {
    pub variable: String,
    pub lifetime: String,
    pub location: String,
}

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
        /// Source code snippet showing the error location
        source_snippet: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
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
        /// Expected token or construct
        expected: Option<String>,
        /// Found token or construct
        found: Option<String>,
        /// Source code snippet showing the error location
        source_snippet: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
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
        /// Source code snippet showing the error location
        source_snippet: Option<String>,
        /// Helpful suggestion for fixing the error
        suggestion: Option<String>,
        /// Additional help message
        help: Option<String>,
    },
    
    /// I/O error (file operations).
    ///
    /// Occurs when file reading, writing, or other I/O operations fail.
    IoError {
        /// Error message describing what went wrong
        message: String,
        /// File path involved in the error
        path: Option<String>,
    },
    
    /// Semantic analysis error (borrow checking, lifetime analysis).
    ///
    /// Occurs during semantic analysis when code violates language rules.
    SemanticError {
        /// Error message describing what went wrong
        message: String,
        /// Line number where the error occurred
        line: Option<usize>,
        /// Column number where the error occurred
        column: Option<usize>,
        /// Source code snippet showing the error location
        source_snippet: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
        /// Related information (e.g., conflicting definitions)
        related_info: Vec<String>,
    },
    
    /// Code generation error (LLVM backend, linking).
    ///
    /// Occurs during code generation or linking phases.
    CodegenError {
        /// Error message describing what went wrong
        message: String,
        /// Compilation phase where error occurred
        phase: String,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },
    
    /// Borrow checking error (conflicting borrows)
    BorrowError {
        /// Error message describing the borrow conflict
        message: String,
        /// Information about conflicting borrows
        conflicting_borrows: Vec<BorrowInfo>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },
    
    /// Lifetime analysis error
    LifetimeError {
        /// Error message describing the lifetime issue
        message: String,
        /// Information about lifetimes involved
        lifetime_info: Vec<LifetimeInfo>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::LexError { message, line, column, source_snippet, suggestion } => {
                write!(f, "❌ Lexer Error at {}:{}\n   {}\n", line, column, message)?;
                if let Some(snippet) = source_snippet {
                    write!(f, "\n   {}\n   {}^\n", snippet, " ".repeat(*column - 1))?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::ParseError { message, line, column, expected, found, source_snippet, suggestion } => {
                write!(f, "❌ Parse Error at {}:{}\n   {}\n", line, column, message)?;
                if let (Some(exp), Some(fnd)) = (expected, found) {
                    write!(f, "   Expected: {}\n   Found: {}\n", exp, fnd)?;
                }
                if let Some(snippet) = source_snippet {
                    write!(f, "\n   {}\n   {}^\n", snippet, " ".repeat(*column - 1))?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::TypeError { message, expected, found, line, column, source_snippet, suggestion, help } => {
                if let (Some(l), Some(c)) = (line, column) {
                    write!(f, "❌ Type Error at {}:{}\n", l, c)?;
                } else {
                    write!(f, "❌ Type Error\n")?;
                }
                write!(f, "   {}\n", message)?;
                if let (Some(exp), Some(fnd)) = (expected, found) {
                    write!(f, "\n   Expected type: {}\n   Found type: {}\n", exp, fnd)?;
                }
                if let Some(snippet) = source_snippet {
                    write!(f, "\n   {}\n", snippet)?;
                    if let Some(c) = column {
                        write!(f, "   {}^\n", " ".repeat(*c - 1))?;
                    }
                }
                if let Some(h) = help {
                    write!(f, "\n❓ Help: {}\n", h)?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::BorrowError { message, conflicting_borrows, suggestion } => {
                write!(f, "❌ Borrow Checker Error\n   {}\n", message)?;
                write!(f, "\n   Conflicting borrows:\n")?;
                for (i, borrow) in conflicting_borrows.iter().enumerate() {
                    write!(f, "   {}. {:?} borrow at {}:{}:{}\n", 
                           i + 1, borrow.borrow_type, borrow.location, borrow.line, borrow.column)?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::LifetimeError { message, lifetime_info, suggestion } => {
                write!(f, "❌ Lifetime Error\n   {}\n", message)?;
                if !lifetime_info.is_empty() {
                    write!(f, "\n   Lifetime information:\n")?;
                    for info in lifetime_info {
                        write!(f, "   • Variable '{}' with lifetime '{}' at {}\n",
                               info.variable, info.lifetime, info.location)?;
                    }
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::SemanticError { message, line, column, source_snippet, suggestion, related_info } => {
                if let (Some(l), Some(c)) = (line, column) {
                    write!(f, "❌ Semantic Error at {}:{}\n", l, c)?;
                } else {
                    write!(f, "❌ Semantic Error\n")?;
                }
                write!(f, "   {}\n", message)?;
                if let Some(snippet) = source_snippet {
                    write!(f, "\n   {}\n", snippet)?;
                    if let Some(c) = column {
                        write!(f, "   {}^\n", " ".repeat(*c - 1))?;
                    }
                }
                if !related_info.is_empty() {
                    write!(f, "\n   Related information:\n")?;
                    for info in related_info {
                        write!(f, "   • {}\n", info)?;
                    }
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
            }
            
            CompileError::IoError { message, path } => {
                write!(f, "❌ IO Error\n   {}\n", message)?;
                if let Some(p) = path {
                    write!(f, "   Path: {}\n", p)?;
                }
                Ok(())
            }
            
            CompileError::CodegenError { message, phase, suggestion } => {
                write!(f, "❌ Code Generation Error ({})\n   {}\n", phase, message)?;
                if let Some(sug) = suggestion {
                    write!(f, "\n💡 Suggestion: {}\n", sug)?;
                }
                Ok(())
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
