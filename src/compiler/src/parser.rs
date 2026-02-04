use crate::ast::{AstNode, Block, Expression, Parameter, Statement, Type};
use crate::errors::{CompilerError, CompilerResult, SourceLocation};
use crate::lexer::{LocatedToken, Token};

pub struct Parser {
    tokens: Vec<LocatedToken>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<LocatedToken>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> CompilerResult<Vec<AstNode>> {
        let mut ast_nodes = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => ast_nodes.push(AstNode::Statement(stmt)),
                Err(err) => {
                    errors.push(err);
                    // Try to recover by advancing to the next statement
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(ast_nodes)
        } else {
            // For now, return the first error. Later we can implement multi-error reporting
            Err(errors.into_iter().next().unwrap())
        }
    }

    fn parse_statement(&mut self) -> CompilerResult<Statement> {
        match &self.peek().token {
            Token::Fn => self.parse_function_definition(),
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Loop => self.parse_loop_statement(),
            Token::Break => self.parse_break_statement(),
            Token::Continue => self.parse_continue_statement(),
            Token::LeftBrace => self.parse_block_statement(),
            _ => {
                // Try to parse as expression statement
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon, "Expected ';' after expression")?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_function_definition(&mut self) -> CompilerResult<Statement> {
        let fn_location = self.peek().location.clone();
        self.consume(Token::Fn, "Expected 'fn'")?;

        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "function name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };

        self.consume(Token::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => {
                        return Err(CompilerError::unexpected_token(
                            "parameter name",
                            &format!("{:?}", self.peek().token),
                            self.peek().location.clone(),
                        ));
                    }
                };

                self.consume(Token::Colon, "Expected ':' after parameter name")?;

                let param_type = self.parse_type()?;
                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(Token::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Statement::Function {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_let_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Let, "Expected 'let'")?;

        let mutable = self.match_token(&Token::Mut);

        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "variable name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };

        let type_annotation = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(Token::Semicolon, "Expected ';' after let statement")?;

        Ok(Statement::Let {
            name,
            mutable,
            type_annotation,
            value,
        })
    }

    fn parse_return_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Return, "Expected 'return'")?;

        let value = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(Token::Semicolon, "Expected ';' after return statement")?;
        Ok(Statement::Return(value))
    }

    fn parse_if_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::If, "Expected 'if'")?;

        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_token(&Token::Else) {
            if self.check(&Token::If) {
                // else if
                Some(Box::new(self.parse_if_statement()?))
            } else {
                // else block
                Some(Box::new(Statement::Block(self.parse_block()?)))
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::While, "Expected 'while'")?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::For, "Expected 'for'")?;

        let variable = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "loop variable",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };

        self.consume(Token::In, "Expected 'in' after for loop variable")?;

        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_loop_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Loop, "Expected 'loop'")?;
        let body = self.parse_block()?;
        Ok(Statement::Loop { body })
    }

    fn parse_break_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Break, "Expected 'break'")?;
        self.consume(Token::Semicolon, "Expected ';' after break")?;
        Ok(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Continue, "Expected 'continue'")?;
        self.consume(Token::Semicolon, "Expected ';' after continue")?;
        Ok(Statement::Continue)
    }

    fn parse_block_statement(&mut self) -> CompilerResult<Statement> {
        let block = self.parse_block()?;
        Ok(Statement::Block(block))
    }

    fn parse_block(&mut self) -> CompilerResult<Block> {
        self.consume(Token::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();
        let mut expression = None;

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Check if this is the last statement and it's an expression without semicolon
            if self.is_expression_start() {
                let checkpoint = self.current;
                match self.parse_expression() {
                    Ok(expr) => {
                        if self.check(&Token::RightBrace) {
                            // This is a block expression (no semicolon)
                            expression = Some(expr);
                            break;
                        } else if self.match_token(&Token::Semicolon) {
                            // This is an expression statement
                            statements.push(Statement::Expression(expr));
                        } else {
                            // Reset and try parsing as statement
                            self.current = checkpoint;
                            statements.push(self.parse_statement()?);
                        }
                    }
                    Err(_) => {
                        // Reset and try parsing as statement
                        self.current = checkpoint;
                        statements.push(self.parse_statement()?);
                    }
                }
            } else {
                statements.push(self.parse_statement()?);
            }
        }

        self.consume(Token::RightBrace, "Expected '}'")?;

        Ok(Block {
            statements,
            expression,
        })
    }

    fn parse_expression(&mut self) -> CompilerResult<Expression> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&Token::LogicalOr) {
            let right = self.parse_logical_and()?;
            expr = Expression::Logical {
                op: crate::ast::LogicalOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::LogicalAnd) {
            let right = self.parse_equality()?;
            expr = Expression::Logical {
                op: crate::ast::LogicalOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while let Some(op) = self.match_equality_operator() {
            let right = self.parse_comparison()?;
            expr = Expression::Comparison {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_term()?;

        while let Some(op) = self.match_comparison_operator() {
            let right = self.parse_term()?;
            expr = Expression::Comparison {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let op = match self.previous().token {
                Token::Plus => crate::ast::BinaryOp::Add,
                Token::Minus => crate::ast::BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                ty: None,
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_token(&Token::Multiply)
            || self.match_token(&Token::Divide)
            || self.match_token(&Token::Modulo)
        {
            let op = match self.previous().token {
                Token::Multiply => crate::ast::BinaryOp::Multiply,
                Token::Divide => crate::ast::BinaryOp::Divide,
                Token::Modulo => crate::ast::BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                ty: None,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> CompilerResult<Expression> {
        if self.match_token(&Token::LogicalNot) || self.match_token(&Token::Minus) {
            let op = match self.previous().token {
                Token::LogicalNot => crate::ast::UnaryOp::Not,
                Token::Minus => crate::ast::UnaryOp::Negate,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> CompilerResult<Expression> {
        let mut expr = self.parse_primary()?;

        while self.match_token(&Token::LeftParen) {
            let mut arguments = Vec::new();
            if !self.check(&Token::RightParen) {
                loop {
                    arguments.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::RightParen, "Expected ')' after arguments")?;

            if let Expression::Identifier(name) = expr {
                expr = Expression::FunctionCall { name, arguments };
            } else {
                return Err(CompilerError::InvalidSyntax {
                    message: "Only identifiers can be called as functions".to_string(),
                    location: self.previous().location.clone(),
                });
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> CompilerResult<Expression> {
        match &self.peek().token {
            Token::IntegerLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expression::IntegerLiteral(value))
            }
            Token::FloatLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expression::FloatLiteral(value))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::PrintMacro => self.parse_print_macro(false),
            Token::PrintlnMacro => self.parse_print_macro(true),
            _ => Err(CompilerError::unexpected_token(
                "expression",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
        }
    }

    fn parse_print_macro(&mut self, is_println: bool) -> CompilerResult<Expression> {
        if is_println {
            self.consume(Token::PrintlnMacro, "Expected 'println!'")?;
        } else {
            self.consume(Token::PrintMacro, "Expected 'print!'")?;
        }

        self.consume(Token::LeftParen, "Expected '(' after print macro")?;

        let format_string = match &self.peek().token {
            Token::Identifier(s) if s.starts_with('"') && s.ends_with('"') => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "format string",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };

        let mut arguments = Vec::new();
        while self.match_token(&Token::Comma) {
            arguments.push(self.parse_expression()?);
        }

        self.consume(Token::RightParen, "Expected ')' after print arguments")?;

        if is_println {
            Ok(Expression::Println {
                format_string,
                arguments,
            })
        } else {
            Ok(Expression::Print {
                format_string,
                arguments,
            })
        }
    }

    fn parse_type(&mut self) -> CompilerResult<Type> {
        match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Type::Named(name))
            }
            _ => Err(CompilerError::unexpected_token(
                "type",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
        }
    }

    // Helper methods
    fn match_equality_operator(&mut self) -> Option<crate::ast::ComparisonOp> {
        if self.match_token(&Token::Equal) {
            Some(crate::ast::ComparisonOp::Equal)
        } else if self.match_token(&Token::NotEqual) {
            Some(crate::ast::ComparisonOp::NotEqual)
        } else {
            None
        }
    }

    fn match_comparison_operator(&mut self) -> Option<crate::ast::ComparisonOp> {
        if self.match_token(&Token::LessThan) {
            Some(crate::ast::ComparisonOp::LessThan)
        } else if self.match_token(&Token::GreaterThan) {
            Some(crate::ast::ComparisonOp::GreaterThan)
        } else if self.match_token(&Token::LessEqual) {
            Some(crate::ast::ComparisonOp::LessEqual)
        } else if self.match_token(&Token::GreaterEqual) {
            Some(crate::ast::ComparisonOp::GreaterEqual)
        } else {
            None
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token) == std::mem::discriminant(token)
        }
    }

    fn advance(&mut self) -> &LocatedToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().token, Token::Eof)
    }

    fn peek(&self) -> &LocatedToken {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &LocatedToken {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token: Token, message: &str) -> CompilerResult<&LocatedToken> {
        if self.check(&token) {
            Ok(self.advance())
        } else {
            Err(CompilerError::unexpected_token(
                &format!("{:?}", token),
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            ))
        }
    }

    fn is_expression_start(&self) -> bool {
        matches!(
            self.peek().token,
            Token::IntegerLiteral(_)
                | Token::FloatLiteral(_)
                | Token::Identifier(_)
                | Token::LeftParen
                | Token::LogicalNot
                | Token::Minus
                | Token::PrintMacro
                | Token::PrintlnMacro
        )
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if matches!(self.previous().token, Token::Semicolon) {
                return;
            }

            match self.peek().token {
                Token::Fn
                | Token::Let
                | Token::If
                | Token::While
                | Token::For
                | Token::Loop
                | Token::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}

// Convenience function for backward compatibility
pub fn parse(tokens: Vec<Token>) -> Vec<AstNode> {
    // Convert tokens to LocatedTokens with unknown locations for backward compatibility
    let located_tokens: Vec<LocatedToken> = tokens
        .into_iter()
        .map(|token| LocatedToken::new(token, SourceLocation::unknown()))
        .collect();

    let mut parser = Parser::new(located_tokens);
    match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            Vec::new()
        }
    }
}

pub fn parse_with_locations(tokens: Vec<LocatedToken>) -> CompilerResult<Vec<AstNode>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
