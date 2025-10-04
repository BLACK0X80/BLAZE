use super::ast::*;
use super::Parser;
use crate::lexer::TokenType;
use crate::error::ParseError;
use anyhow::Result;

impl Parser<'_> {
    pub fn parse_statement(&mut self) -> Result<Statement> {
        match &self.peek().token_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Loop => self.parse_loop_statement(),
            TokenType::LeftBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Let, "Expected 'let'")?;
        
        let mutable = self.match_token(&TokenType::Mut);
        
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(ParseError::ExpectedToken {
                expected: "variable name".to_string(),
                found: "other".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            }.into());
        };

        let ty = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.match_token(&TokenType::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';'")?;

        Ok(Statement::Let {
            name,
            ty,
            value,
            mutable,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Return, "Expected 'return'")?;
        
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(TokenType::Semicolon, "Expected ';'")?;
        Ok(Statement::Return(value))
    }

    fn parse_break_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Break, "Expected 'break'")?;
        
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(TokenType::Semicolon, "Expected ';'")?;
        Ok(Statement::Break(value))
    }

    fn parse_continue_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Continue, "Expected 'continue'")?;
        self.consume(TokenType::Semicolon, "Expected ';'")?;
        Ok(Statement::Continue)
    }

    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::While, "Expected 'while'")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::For, "Expected 'for'")?;
        
        let variable = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(ParseError::ExpectedToken {
                expected: "variable name".to_string(),
                found: "other".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            }.into());
        };

        self.consume(TokenType::In, "Expected 'in'")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_loop_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Loop, "Expected 'loop'")?;
        let body = self.parse_block()?;
        Ok(Statement::Loop { body })
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        let statements = self.parse_block()?;
        Ok(Statement::Block(statements))
    }

    pub fn parse_block(&mut self) -> Result<Vec<Statement>> {
        self.consume(TokenType::LeftBrace, "Expected '{'")?;
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}'")?;
        Ok(statements)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        
        if !self.check(&TokenType::RightBrace) {
            self.consume(TokenType::Semicolon, "Expected ';'")?;
        }
        
        Ok(Statement::Expression(expr))
    }
}