use super::token::{Token, TokenType};
use crate::error::{CompileError, Result};

pub struct Scanner {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            let token = self.scan_token()?;
            tokens.push(token);
        }

        tokens.push(Token::new(TokenType::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Token> {
        let line = self.line;
        let column = self.column;
        let ch = self.advance();

        let token_type = match ch {
            '+' => TokenType::Plus,
            '-' => {
                if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Percent,
            '=' => {
                if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenType::And
                } else {
                    TokenType::Ampersand
                }
            }
            '|' => {
                if self.match_char('|') {
                    TokenType::Or
                } else {
                    TokenType::Pipe
                }
            }
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            ';' => TokenType::Semicolon,
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '"' => self.scan_string()?,
            '\'' => self.scan_char()?,
            _ if ch.is_ascii_digit() => self.scan_number()?,
            _ if ch.is_alphabetic() || ch == '_' => self.scan_identifier(),
            _ => {
                return Err(CompileError::LexError {
                    message: format!("Unexpected character: '{}'", ch),
                    line,
                    column,
                })
            }
        };

        Ok(Token::new(token_type, line, column))
    }

    fn scan_string(&mut self) -> Result<TokenType> {
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    break;
                }
                let escaped = self.advance();
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => escaped,
                });
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(CompileError::LexError {
                message: "Unterminated string".to_string(),
                line: start_line,
                column: start_column,
            });
        }

        self.advance();
        Ok(TokenType::StringLit(value))
    }

    fn scan_char(&mut self) -> Result<TokenType> {
        let start_line = self.line;
        let start_column = self.column;

        if self.is_at_end() {
            return Err(CompileError::LexError {
                message: "Unterminated character literal".to_string(),
                line: start_line,
                column: start_column,
            });
        }

        let ch = if self.peek() == '\\' {
            self.advance();
            if self.is_at_end() {
                return Err(CompileError::LexError {
                    message: "Unterminated character literal".to_string(),
                    line: start_line,
                    column: start_column,
                });
            }
            let escaped = self.advance();
            match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                _ => escaped,
            }
        } else {
            self.advance()
        };

        if self.is_at_end() || self.peek() != '\'' {
            return Err(CompileError::LexError {
                message: "Unterminated character literal".to_string(),
                line: start_line,
                column: start_column,
            });
        }

        self.advance();
        Ok(TokenType::CharLit(ch))
    }

    fn scan_number(&mut self) -> Result<TokenType> {
        let mut value = String::new();
        value.push(self.source[self.current - 1]);

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            value.push(self.advance());
        }

        if !self.is_at_end()
            && self.peek() == '.'
            && self.current + 1 < self.source.len()
            && self.source[self.current + 1].is_ascii_digit()
        {
            value.push(self.advance());

            while !self.is_at_end() && self.peek().is_ascii_digit() {
                value.push(self.advance());
            }

            Ok(TokenType::FloatLit(value.parse().unwrap()))
        } else {
            Ok(TokenType::IntLit(value.parse().unwrap()))
        }
    }

    fn scan_identifier(&mut self) -> TokenType {
        let mut value = String::new();
        value.push(self.source[self.current - 1]);

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            value.push(self.advance());
        }

        match value.as_str() {
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "fn" => TokenType::Fn,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "loop" => TokenType::Loop,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "struct" => TokenType::Struct,
            "impl" => TokenType::Impl,
            "match" => TokenType::Match,
            "enum" => TokenType::Enum,
            "pub" => TokenType::Pub,
            "mod" => TokenType::Mod,
            "use" => TokenType::Use,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "i32" => TokenType::I32,
            "i64" => TokenType::I64,
            "f32" => TokenType::F32,
            "f64" => TokenType::F64,
            "bool" => TokenType::Bool,
            "char" => TokenType::Char,
            "str" => TokenType::Str,
            "String" => TokenType::String,
            _ => TokenType::Ident(value),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }

            match self.peek() {
                ' ' | '\r' | '\t' | '\n' => {
                    self.advance();
                }
                '/' if self.current + 1 < self.source.len()
                    && self.source[self.current + 1] == '/' =>
                {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn peek(&self) -> char {
        self.source[self.current]
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
