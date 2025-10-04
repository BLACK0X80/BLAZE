use super::ast::*;
use super::Parser;
use crate::lexer::TokenType;
use crate::error::ParseError;
use anyhow::Result;

impl Parser<'_> {
    pub fn parse_type(&mut self) -> Result<Type> {
        self.parse_reference_type()
    }

    fn parse_reference_type(&mut self) -> Result<Type> {
        if self.match_token(&TokenType::Ampersand) {
            let mutable = self.match_token(&TokenType::Mut);
            let inner = self.parse_primary_type()?;
            return Ok(Type::Reference {
                mutable,
                inner: Box::new(inner),
            });
        }

        if self.match_token(&TokenType::Star) {
            let mutable = self.match_token(&TokenType::Mut);
            let inner = self.parse_primary_type()?;
            return Ok(Type::Pointer {
                mutable,
                inner: Box::new(inner),
            });
        }

        self.parse_primary_type()
    }

    fn parse_primary_type(&mut self) -> Result<Type> {
        match &self.peek().token_type {
            TokenType::I8 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I8))
            }
            TokenType::I16 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I16))
            }
            TokenType::I32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I32))
            }
            TokenType::I64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I64))
            }
            TokenType::I128 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I128))
            }
            TokenType::U8 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U8))
            }
            TokenType::U16 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U16))
            }
            TokenType::U32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U32))
            }
            TokenType::U64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U64))
            }
            TokenType::U128 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U128))
            }
            TokenType::F32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::F32))
            }
            TokenType::F64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::F64))
            }
            TokenType::Bool => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
            TokenType::Char => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Char))
            }
            TokenType::Str => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Str))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                
                if self.match_token(&TokenType::Less) {
                    let mut args = Vec::new();
                    if !self.check(&TokenType::Greater) {
                        loop {
                            args.push(self.parse_type()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenType::Greater, "Expected '>'")?;
                    Ok(Type::Generic { name, args })
                } else {
                    Ok(Type::Identifier(name))
                }
            }
            TokenType::LeftParen => {
                self.advance()?;
                
                if self.check(&TokenType::RightParen) {
                    self.advance()?;
                    return Ok(Type::Unit);
                }
                
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
                
                self.consume(TokenType::RightParen, "Expected ')'")?;
                
                if types.len() == 1 {
                    Ok(types.into_iter().next().unwrap())
                } else {
                    Ok(Type::Tuple(types))
                }
            }
            TokenType::LeftBracket => {
                self.advance()?;
                let element_type = self.parse_type()?;
                
                if self.match_token(&TokenType::Semicolon) {
                    let size = self.parse_expression()?;
                    self.consume(TokenType::RightBracket, "Expected ']'")?;
                    Ok(Type::Array {
                        element_type: Box::new(element_type),
                        size: Some(size),
                    })
                } else {
                    self.consume(TokenType::RightBracket, "Expected ']'")?;
                    Ok(Type::Slice(Box::new(element_type)))
                }
            }
            TokenType::Fn => {
                self.advance()?;
                self.consume(TokenType::LeftParen, "Expected '('")?;
                
                let mut params = Vec::new();
                if !self.check(&TokenType::RightParen) {
                    loop {
                        params.push(self.parse_type()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(TokenType::RightParen, "Expected ')'")?;
                self.consume(TokenType::Arrow, "Expected '->'")?;
                let return_type = self.parse_type()?;
                
                Ok(Type::Function {
                    params,
                    return_type: Box::new(return_type),
                })
            }
            _ => {
                Err(ParseError::ExpectedToken {
                    expected: "type".to_string(),
                    found: format!("{:?}", self.peek().token_type),
                    line: self.peek().line,
                    column: self.peek().column,
                }.into())
            }
        }
    }
}