#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Let, Mut, Fn, Return, If, Else, While, For, Loop, Break, Continue,
    Struct, Impl, Match, Enum, Pub, Mod, Use, True, False,
    In, Const, Static, Ref, SelfType, SelfValue,
    Trait, Async, Await, Unsafe, Where, Type, As, Dyn, Move,
    Crate, Super, Extern, Box, Underscore,
    
    I32, I64, F32, F64, Bool, Char, Str, String,
    
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    CharLit(char),
    
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    And, Or, Bang,
    Ampersand, Pipe, Caret,
    LeftShift, RightShift,
    
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Semicolon, Colon, Comma, Dot,
    Arrow, FatArrow,
    
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token { token_type, line, column }
    }
}