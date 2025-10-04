mod ast;

pub use ast::*;

use crate::lexer::{Token, TokenType};
use crate::error::{CompileError, Result};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        Ok(Program { items })
    }
    
    fn parse_item(&mut self) -> Result<Item> {
        match &self.peek().token_type {
            TokenType::Fn => Ok(Item::Function(self.parse_function()?)),
            TokenType::Struct => Ok(Item::Struct(self.parse_struct()?)),
            _ => Err(self.error("Expected function or struct")),
        }
    }
    
    fn parse_function(&mut self) -> Result<Function> {
        self.consume(TokenType::Fn)?;
        
        let name = self.consume_ident()?;
        
        self.consume(TokenType::LeftParen)?;
        let mut params = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume_ident()?;
                self.consume(TokenType::Colon)?;
                let param_type = self.parse_type()?;
                params.push(Param { name: param_name, ty: param_type });
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen)?;
        
        let return_type = if self.match_token(TokenType::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(TokenType::LeftBrace)?;
        let mut body = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::RightBrace)?;
        
        Ok(Function { name, params, return_type, body })
    }
    
    fn parse_struct(&mut self) -> Result<Struct> {
        self.consume(TokenType::Struct)?;
        let name = self.consume_ident()?;
        self.consume(TokenType::LeftBrace)?;
        
        let mut fields = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let field_name = self.consume_ident()?;
            self.consume(TokenType::Colon)?;
            let field_type = self.parse_type()?;
            fields.push(Field { name: field_name, ty: field_type });
            
            self.match_token(TokenType::Comma);
        }
        
        self.consume(TokenType::RightBrace)?;
        
        Ok(Struct { name, fields })
    }
    
    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.peek().token_type {
            TokenType::Let => self.parse_let(),
            TokenType::Return => self.parse_return(),
            TokenType::While => self.parse_while(),
            TokenType::If => self.parse_if(),
            _ => {
                let expr = self.parse_expression()?;
                self.match_token(TokenType::Semicolon);
                Ok(Statement::Expression(expr))
            }
        }
    }
    
    fn parse_let(&mut self) -> Result<Statement> {
        self.consume(TokenType::Let)?;
        
        let mutable = self.match_token(TokenType::Mut);
        let name = self.consume_ident()?;
        
        let ty = if self.match_token(TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(TokenType::Equal)?;
        let value = self.parse_expression()?;
        self.match_token(TokenType::Semicolon);
        
        Ok(Statement::Let { name, mutable, ty, value })
    }
    
    fn parse_return(&mut self) -> Result<Statement> {
        self.consume(TokenType::Return)?;
        
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.match_token(TokenType::Semicolon);
        Ok(Statement::Return(value))
    }
    
    fn parse_while(&mut self) -> Result<Statement> {
        self.consume(TokenType::While)?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::LeftBrace)?;
        
        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::RightBrace)?;
        Ok(Statement::While { condition, body })
    }
    
    fn parse_if(&mut self) -> Result<Statement> {
        self.consume(TokenType::If)?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::LeftBrace)?;
        
        let mut then_body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            then_body.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::RightBrace)?;
        
        let else_body = if self.match_token(TokenType::Else) {
            self.consume(TokenType::LeftBrace)?;
            let mut body = Vec::new();
            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                body.push(self.parse_statement()?);
            }
            self.consume(TokenType::RightBrace)?;
            Some(body)
        } else {
            None
        };
        
        Ok(Statement::If { condition, then_body, else_body })
    }
    
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_and()?;
        
        while self.match_token(TokenType::Or) {
            let right = self.parse_and()?;
            left = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(TokenType::And) {
            let right = self.parse_equality()?;
            left = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;
        
        while let Some(op) = self.match_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let binary_op = match op {
                TokenType::EqualEqual => BinaryOp::Eq,
                TokenType::BangEqual => BinaryOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                op: binary_op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_term()?;
        
        while let Some(op) = self.match_tokens(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let binary_op = match op {
                TokenType::Less => BinaryOp::Lt,
                TokenType::LessEqual => BinaryOp::Le,
                TokenType::Greater => BinaryOp::Gt,
                TokenType::GreaterEqual => BinaryOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            left = Expression::Binary {
                op: binary_op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_term(&mut self) -> Result<Expression> {
        let mut left = self.parse_factor()?;
        
        while let Some(op) = self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let binary_op = match op {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            left = Expression::Binary {
                op: binary_op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary()?;
        
        while let Some(op) = self.match_tokens(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let binary_op = match op {
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                TokenType::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expression::Binary {
                op: binary_op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(op) = self.match_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let unary_op = match op {
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: unary_op,
                expr: Box::new(expr),
            });
        }
        
        self.parse_call()
    }
    
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        while self.match_token(TokenType::LeftParen) {
            let mut args = Vec::new();
            
            if !self.check(TokenType::RightParen) {
                loop {
                    args.push(self.parse_expression()?);
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(TokenType::RightParen)?;
            expr = Expression::Call {
                func: Box::new(expr),
                args,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expression> {
        let token = self.advance().clone();
        
        match &token.token_type {
            TokenType::IntLit(n) => Ok(Expression::IntLit(*n)),
            TokenType::FloatLit(f) => Ok(Expression::FloatLit(*f)),
            TokenType::StringLit(s) => Ok(Expression::StringLit(s.clone())),
            TokenType::CharLit(c) => Ok(Expression::CharLit(*c)),
            TokenType::True => Ok(Expression::BoolLit(true)),
            TokenType::False => Ok(Expression::BoolLit(false)),
            TokenType::Ident(name) => Ok(Expression::Ident(name.clone())),
            TokenType::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen)?;
                Ok(expr)
            }
            _ => Err(self.error("Expected expression")),
        }
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        let token = self.advance().clone();
        
        match &token.token_type {
            TokenType::I32 => Ok(Type::I32),
            TokenType::I64 => Ok(Type::I64),
            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),
            TokenType::Bool => Ok(Type::Bool),
            TokenType::Char => Ok(Type::Char),
            TokenType::String => Ok(Type::String),
            TokenType::Ident(name) => Ok(Type::Custom(name.clone())),
            _ => Err(self.error("Expected type")),
        }
    }
    
    fn consume(&mut self, token_type: TokenType) -> Result<()> {
        if self.check(token_type.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!("Expected {:?}", token_type)))
        }
    }
    
    fn consume_ident(&mut self) -> Result<String> {
        let token = self.advance().clone();
        match &token.token_type {
            TokenType::Ident(name) => Ok(name.clone()),
            _ => Err(self.error("Expected identifier")),
        }
    }
    
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn match_tokens(&mut self, types: &[TokenType]) -> Option<TokenType> {
        for ty in types {
            if self.check(ty.clone()) {
                let token = self.advance().clone();
                return Some(token.token_type);
            }
        }
        None
    }
    
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(&token_type)
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }
    
    fn error(&self, message: &str) -> CompileError {
        let token = self.peek();
        CompileError::ParseError {
            message: message.to_string(),
            line: token.line,
            column: token.column,
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}