pub mod error;
pub mod lexer;
pub mod parser;

pub use error::{CompileError, Result};
pub use lexer::{lex, Token, TokenType};
pub use parser::{parse, Program};

use std::path::Path;

pub fn compile_file(path: &Path) -> Result<Program> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::IoError { 
            message: format!("Failed to read file: {}", e) 
        })?;
    
    compile(&source)
}

pub fn compile(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    Ok(program)
}

pub fn check(source: &str) -> Result<()> {
    let _ = compile(source)?;
    Ok(())
}