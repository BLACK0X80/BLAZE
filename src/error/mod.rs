use std::fmt;

#[derive(Debug, Clone)]
pub enum CompileError {
    LexError {
        message: String,
        line: usize,
        column: usize,
    },
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    IoError {
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
            CompileError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
        }
    }
}

impl std::error::Error for CompileError {}

pub type Result<T> = std::result::Result<T, CompileError>;
