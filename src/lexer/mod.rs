mod scanner;
mod token;

pub use scanner::Scanner;
pub use token::{Token, TokenType};

use crate::error::Result;

pub fn lex(source: &str) -> Result<Vec<Token>> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()
}
