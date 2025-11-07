//! Semantic analysis module for the BLAZE compiler.
//!
//! This module performs comprehensive semantic analysis on the AST, including:
//!
//! - **Symbol resolution**: Building symbol tables and resolving identifiers
//! - **Scope resolution**: Managing lexical scopes and variable visibility
//! - **Type checking**: Validating type correctness and performing type inference
//! - **Borrow checking**: Enforcing memory safety rules (ownership, borrowing)
//! - **Lifetime analysis**: Inferring and validating reference lifetimes
//!
//! # Examples
//!
//! ```rust,no_run
//! use blaze::semantic::SemanticAnalyzer;
//! use blaze::parser::Program;
//!
//! let program = Program::new();  // Assume we have a parsed program
//! let mut analyzer = SemanticAnalyzer::new();
//!
//! // Perform all semantic checks
//! analyzer.analyze(&program).expect("Semantic analysis failed");
//! ```

pub mod type_checker;
pub mod borrow_checker;
pub mod symbol_table;
pub mod lifetime_analyzer;
pub mod scope_resolver;
pub mod type_inference;

pub use type_checker::TypeChecker;
pub use borrow_checker::BorrowChecker;
pub use symbol_table::SymbolTable;
pub use lifetime_analyzer::LifetimeAnalyzer;
pub use scope_resolver::ScopeResolver;
pub use type_inference::TypeInference;

use crate::parser::Program;
use anyhow::Result;

/// Semantic analyzer that coordinates all semantic analysis phases.
///
/// The `SemanticAnalyzer` performs comprehensive validation of the AST,
/// ensuring type safety, memory safety, and semantic correctness before
/// code generation.
///
/// # Analysis Phases
///
/// 1. **Symbol Table Construction**: Builds symbol tables for all scopes
/// 2. **Scope Resolution**: Resolves identifiers to their declarations
/// 3. **Lifetime Analysis**: Infers and validates reference lifetimes
/// 4. **Type Checking**: Validates type correctness and performs inference
/// 5. **Borrow Checking**: Enforces ownership and borrowing rules
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_checker: TypeChecker,
    borrow_checker: BorrowChecker,
    lifetime_analyzer: LifetimeAnalyzer,
    scope_resolver: ScopeResolver,
    type_inference: TypeInference,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use blaze::semantic::SemanticAnalyzer;
    ///
    /// let analyzer = SemanticAnalyzer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            type_checker: TypeChecker::new(),
            borrow_checker: BorrowChecker::new(),
            lifetime_analyzer: LifetimeAnalyzer::new(),
            scope_resolver: ScopeResolver::new(),
            type_inference: TypeInference::new(),
        }
    }

    /// Performs complete semantic analysis on a program.
    ///
    /// This method runs all semantic analysis phases in the correct order:
    /// 1. Symbol table construction
    /// 2. Scope resolution
    /// 3. Lifetime analysis
    /// 4. Type checking
    /// 5. Borrow checking
    ///
    /// # Arguments
    ///
    /// * `program` - The parsed AST to analyze
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all semantic checks pass, or an error describing
    /// the first semantic error encountered.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Undefined variables or functions are referenced
    /// - Type mismatches are detected
    /// - Borrow checking rules are violated
    /// - Lifetime constraints cannot be satisfied
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blaze::semantic::SemanticAnalyzer;
    /// use blaze::{compile};
    ///
    /// let source = r#"
    ///     fn main() {
    ///         let x: i32 = 42;
    ///         let y = x + 10;  // Type inference
    ///     }
    /// "#;
    ///
    /// let program = compile(source).expect("Parse failed");
    /// let mut analyzer = SemanticAnalyzer::new();
    ///
    /// analyzer.analyze(&program).expect("Semantic analysis failed");
    /// ```
    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        self.symbol_table.analyze(program)?;
        self.scope_resolver.resolve(program)?;
        self.type_inference.infer(program)?;
        self.lifetime_analyzer.analyze(program)?;
        self.type_checker.check(program, &self.symbol_table)?;
        self.borrow_checker.check(program, &self.symbol_table)?;
        Ok(())
    }
}
