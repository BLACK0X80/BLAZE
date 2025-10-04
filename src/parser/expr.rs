use super::ast::*;
use super::Parser;
use crate::error::ParseError;
use crate::lexer::TokenType;
use anyhow::Result;

impl Parser<'_> {
    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let expr = self.parse_or()?;

        if self.match_token(&TokenType::Equal) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                value: Box::new(value),
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::PipePipe], |parser| parser.parse_and())
    }

    fn parse_and(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::AmpersandAmpersand], |parser| {
            parser.parse_equality()
        })
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(
            &[TokenType::EqualEqual, TokenType::BangEqual],
            |parser| parser.parse_comparison(),
        )
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            |parser| parser.parse_bitwise_or(),
        )
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::Pipe], |parser| {
            parser.parse_bitwise_xor()
        })
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::Caret], |parser| {
            parser.parse_bitwise_and()
        })
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::Ampersand], |parser| {
            parser.parse_shift()
        })
    }

    fn parse_shift(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(
            &[TokenType::LeftShift, TokenType::RightShift],
            |parser| parser.parse_term(),
        )
    }

    fn parse_term(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(&[TokenType::Minus, TokenType::Plus], |parser| {
            parser.parse_factor()
        })
    }

    fn parse_factor(&mut self) -> Result<Expression> {
        self.parse_left_associative_binary_op(
            &[TokenType::Slash, TokenType::Star, TokenType::Percent],
            |parser| parser.parse_unary(),
        )
    }

    fn parse_left_associative_binary_op<F>(
        &mut self,
        operators: &[TokenType],
        parse_operand: F,
    ) -> Result<Expression>
    where
        F: Fn(&mut Self) -> Result<Expression>,
    {
        let mut expr = parse_operand(self)?;

        while let Some(operator) = self.get_matching_operator(operators) {
            let right = parse_operand(self)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: self.token_to_binary_operator(operator),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn get_matching_operator(&mut self, operators: &[TokenType]) -> Option<TokenType> {
        for &op in operators {
            if self.match_token(&op) {
                return Some(op);
            }
        }
        None
    }

    fn token_to_binary_operator(&self, token: TokenType) -> BinaryOperator {
        match token {
            TokenType::Plus => BinaryOperator::Add,
            TokenType::Minus => BinaryOperator::Subtract,
            TokenType::Star => BinaryOperator::Multiply,
            TokenType::Slash => BinaryOperator::Divide,
            TokenType::Percent => BinaryOperator::Modulo,
            TokenType::EqualEqual => BinaryOperator::Equal,
            TokenType::BangEqual => BinaryOperator::NotEqual,
            TokenType::Less => BinaryOperator::Less,
            TokenType::LessEqual => BinaryOperator::LessEqual,
            TokenType::Greater => BinaryOperator::Greater,
            TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
            TokenType::AmpersandAmpersand => BinaryOperator::LogicalAnd,
            TokenType::PipePipe => BinaryOperator::LogicalOr,
            TokenType::Ampersand => BinaryOperator::BitwiseAnd,
            TokenType::Pipe => BinaryOperator::BitwiseOr,
            TokenType::Caret => BinaryOperator::BitwiseXor,
            TokenType::LeftShift => BinaryOperator::LeftShift,
            TokenType::RightShift => BinaryOperator::RightShift,
            _ => unreachable!(),
        }
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        if self.match_token(&TokenType::Bang)
            || self.match_token(&TokenType::Minus)
            || self.match_token(&TokenType::Ampersand)
            || self.match_token(&TokenType::AmpersandMut)
            || self.match_token(&TokenType::Star)
        {
            let operator = match self.previous().token_type {
                TokenType::Bang => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                TokenType::Ampersand => UnaryOperator::Reference,
                TokenType::AmpersandMut => UnaryOperator::MutableReference,
                TokenType::Star => UnaryOperator::Dereference,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(right),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenType::LeftBracket) {
                let index = self.parse_expression()?;
                self.consume(TokenType::RightBracket, "Expected ']'")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&TokenType::Dot) {
                if let TokenType::Identifier(name) = &self.advance()?.token_type {
                    if self.check(&TokenType::LeftParen) {
                        self.advance()?;
                        let mut args = Vec::new();
                        if !self.check(&TokenType::RightParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.match_token(&TokenType::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume(TokenType::RightParen, "Expected ')'")?;
                        expr = Expression::MethodCall {
                            object: Box::new(expr),
                            method: name.clone(),
                            args,
                        };
                    } else {
                        expr = Expression::FieldAccess {
                            object: Box::new(expr),
                            field: name.clone(),
                        };
                    }
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: "field name".to_string(),
                        found: "other".to_string(),
                        line: self.peek().line,
                        column: self.peek().column,
                    }
                    .into());
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')'")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match &self.peek().token_type {
            TokenType::True => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenType::IntLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Expression::Literal(Literal::Integer(val)))
            }
            TokenType::FloatLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Expression::Literal(Literal::Float(val)))
            }
            TokenType::StringLiteral(value) => {
                let val = value.clone();
                self.advance()?;
                Ok(Expression::Literal(Literal::String(val)))
            }
            TokenType::CharLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Expression::Literal(Literal::Char(val)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(Expression::Identifier(name))
            }
            TokenType::LeftParen => {
                self.advance()?;
                if self.check(&TokenType::RightParen) {
                    self.advance()?;
                    return Ok(Expression::Literal(Literal::Unit));
                }

                let expr = self.parse_expression()?;

                if self.match_token(&TokenType::Comma) {
                    let mut elements = vec![expr];
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    Ok(Expression::TupleLiteral(elements))
                } else {
                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    Ok(expr)
                }
            }
            TokenType::LeftBracket => {
                self.advance()?;
                let mut elements = Vec::new();

                if !self.check(&TokenType::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RightBracket, "Expected ']'")?;
                Ok(Expression::ArrayLiteral(elements))
            }
            TokenType::LeftBrace => self.parse_block_expression(),
            TokenType::If => self.parse_if_expression(),
            TokenType::Match => self.parse_match_expression(),
            TokenType::Pipe => self.parse_closure(),
            _ => {
                if let TokenType::Identifier(_) = &self.peek().token_type {
                    if self.peek_next().map(|t| &t.token_type) == Some(&TokenType::LeftBrace) {
                        return self.parse_struct_literal();
                    }
                }

                Err(ParseError::UnexpectedToken {
                    line: self.peek().line,
                    column: self.peek().column,
                }
                .into())
            }
        }
    }

    fn parse_block_expression(&mut self) -> Result<Expression> {
        let statements = self.parse_block()?;
        Ok(Expression::Block(statements))
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        self.consume(TokenType::If, "Expected 'if'")?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block_expression()?;

        let else_branch = if self.match_token(&TokenType::Else) {
            if self.check(&TokenType::If) {
                Some(Box::new(self.parse_if_expression()?))
            } else {
                Some(Box::new(self.parse_block_expression()?))
            }
        } else {
            None
        };

        Ok(Expression::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_match_expression(&mut self) -> Result<Expression> {
        self.consume(TokenType::Match, "Expected 'match'")?;
        let expr = self.parse_expression()?;
        self.consume(TokenType::LeftBrace, "Expected '{'")?;

        let mut arms = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(&TokenType::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(TokenType::FatArrow, "Expected '=>'")?;
            let body = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}'")?;

        Ok(Expression::Match {
            expression: Box::new(expr),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.peek().token_type {
            TokenType::IntLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Pattern::Literal(Literal::Integer(val)))
            }
            TokenType::FloatLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Pattern::Literal(Literal::Float(val)))
            }
            TokenType::StringLiteral(value) => {
                let val = value.clone();
                self.advance()?;
                Ok(Pattern::Literal(Literal::String(val)))
            }
            TokenType::CharLiteral(value) => {
                let val = *value;
                self.advance()?;
                Ok(Pattern::Literal(Literal::Char(val)))
            }
            TokenType::True => {
                self.advance()?;
                Ok(Pattern::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance()?;
                Ok(Pattern::Literal(Literal::Boolean(false)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(Pattern::Identifier(name))
            }
            _ => Err(ParseError::UnexpectedToken {
                line: self.peek().line,
                column: self.peek().column,
            }
            .into()),
        }
    }

    fn parse_closure(&mut self) -> Result<Expression> {
        self.consume(TokenType::Pipe, "Expected '|'")?;
        let mut params = Vec::new();

        if !self.check(&TokenType::Pipe) {
            loop {
                if let TokenType::Identifier(name) = &self.advance()?.token_type {
                    let ty = if self.match_token(&TokenType::Colon) {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };

                    params.push(ClosureParam {
                        name: name.clone(),
                        ty,
                    });
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: "parameter name".to_string(),
                        found: "other".to_string(),
                        line: self.peek().line,
                        column: self.peek().column,
                    }
                    .into());
                }

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::Pipe, "Expected '|'")?;
        let body = self.parse_expression()?;

        Ok(Expression::Closure {
            params,
            body: Box::new(body),
        })
    }

    fn parse_struct_literal(&mut self) -> Result<Expression> {
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(ParseError::ExpectedToken {
                expected: "struct name".to_string(),
                found: "other".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            }
            .into());
        };

        self.consume(TokenType::LeftBrace, "Expected '{'")?;
        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let field_name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                name.clone()
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: "field name".to_string(),
                    found: "other".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                }
                .into());
            };

            self.consume(TokenType::Colon, "Expected ':'")?;
            let value = self.parse_expression()?;

            fields.push(FieldInit {
                name: field_name,
                value,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}'")?;

        Ok(Expression::StructLiteral { name, fields })
    }
}
