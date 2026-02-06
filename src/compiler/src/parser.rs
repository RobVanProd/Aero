#![allow(clippy::result_large_err)]

use crate::ast::{
    AstNode, Block, Expression, FieldDecl, MatchArm, Parameter, Pattern, Statement, Type,
    VariantDecl, VariantDeclKind,
};
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
            Token::Struct => self.parse_struct_def(),
            Token::Enum => self.parse_enum_def(),
            Token::Impl => self.parse_impl_block(),
            _ => {
                // Try to parse as expression statement
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon, "Expected ';' after expression")?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_function_definition(&mut self) -> CompilerResult<Statement> {
        let _fn_location = self.peek().location.clone();
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

        loop {
            if self.match_token(&Token::LeftParen) {
                // Function call
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
                } else if let Expression::FieldAccess { object, field } = expr {
                    // method call: obj.method(args)
                    expr = Expression::MethodCall {
                        object,
                        method: field,
                        arguments,
                    };
                } else {
                    return Err(CompilerError::InvalidSyntax {
                        message: "Only identifiers can be called as functions".to_string(),
                        location: self.previous().location.clone(),
                    });
                }
            } else if self.match_token(&Token::LeftBracket) {
                // Index access: expr[index]
                let index = self.parse_expression()?;
                self.consume(Token::RightBracket, "Expected ']' after index")?;
                expr = Expression::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Dot) {
                // Field access or tuple index: expr.field or expr.0
                match &self.peek().token {
                    Token::IntegerLiteral(idx) => {
                        let idx = *idx as usize;
                        self.advance();
                        expr = Expression::TupleIndex {
                            object: Box::new(expr),
                            index: idx,
                        };
                    }
                    Token::Identifier(field) => {
                        let field = field.clone();
                        self.advance();
                        expr = Expression::FieldAccess {
                            object: Box::new(expr),
                            field,
                        };
                    }
                    _ => {
                        return Err(CompilerError::unexpected_token(
                            "field name or tuple index",
                            &format!("{:?}", self.peek().token),
                            self.peek().location.clone(),
                        ));
                    }
                }
            } else {
                break;
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
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                // Check for struct literal: Name { field: value, ... }
                if self.check(&Token::LeftBrace) {
                    // Peek ahead to see if this looks like a struct literal
                    // (identifier followed by colon means struct literal)
                    if self.is_struct_literal_start() {
                        return self.parse_struct_literal(name);
                    }
                }
                // Check for enum variant: Name::Variant
                if self.check(&Token::DoubleColon) {
                    return self.parse_enum_variant(name);
                }
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                // Check for unit tuple or tuple literal
                if self.check(&Token::RightParen) {
                    self.advance();
                    return Ok(Expression::TupleLiteral(vec![]));
                }
                let first = self.parse_expression()?;
                if self.match_token(&Token::Comma) {
                    // This is a tuple literal
                    let mut elements = vec![first];
                    if !self.check(&Token::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RightParen, "Expected ')' after tuple elements")?;
                    Ok(Expression::TupleLiteral(elements))
                } else {
                    // Parenthesized expression
                    self.consume(Token::RightParen, "Expected ')' after expression")?;
                    Ok(first)
                }
            }
            Token::LeftBracket => self.parse_array_literal(),
            Token::Match => self.parse_match_expression(),
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
            Token::StringLiteral(s) => {
                let s = format!("\"{}\"", s);
                self.advance();
                s
            }
            // Backward compat: old-style string-as-identifier
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
            Token::LeftBracket => {
                // Array type: [T; N]
                self.advance();
                let elem_type = self.parse_type()?;
                self.consume(Token::Semicolon, "Expected ';' in array type [T; N]")?;
                let size = match &self.peek().token {
                    Token::IntegerLiteral(n) => {
                        let n = *n as usize;
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(CompilerError::unexpected_token(
                            "array size",
                            &format!("{:?}", self.peek().token),
                            self.peek().location.clone(),
                        ));
                    }
                };
                self.consume(Token::RightBracket, "Expected ']' after array type")?;
                Ok(Type::Array(Box::new(elem_type), size))
            }
            Token::LeftParen => {
                // Tuple type: (T1, T2, ...)
                self.advance();
                let mut types = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightParen, "Expected ')' after tuple type")?;
                Ok(Type::Tuple(types))
            }
            _ => Err(CompilerError::unexpected_token(
                "type",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
        }
    }

    // --- Phase 4 parsing methods ---

    fn parse_array_literal(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::LeftBracket, "Expected '['")?;
        if self.check(&Token::RightBracket) {
            self.advance();
            return Ok(Expression::ArrayLiteral(vec![]));
        }
        let first = self.parse_expression()?;
        // Check for repeat syntax: [value; count]
        if self.match_token(&Token::Semicolon) {
            let count = match &self.peek().token {
                Token::IntegerLiteral(n) => {
                    let n = *n as usize;
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "array repeat count",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            self.consume(Token::RightBracket, "Expected ']' after array repeat")?;
            return Ok(Expression::ArrayRepeat {
                value: Box::new(first),
                count,
            });
        }
        // Comma-separated elements: [a, b, c]
        let mut elements = vec![first];
        while self.match_token(&Token::Comma) {
            if self.check(&Token::RightBracket) {
                break; // trailing comma
            }
            elements.push(self.parse_expression()?);
        }
        self.consume(Token::RightBracket, "Expected ']' after array elements")?;
        Ok(Expression::ArrayLiteral(elements))
    }

    fn parse_struct_def(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Struct, "Expected 'struct'")?;
        let name = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "struct name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };
        self.consume(Token::LeftBrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let field_name = match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "field name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            fields.push(FieldDecl {
                name: field_name,
                field_type,
            });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.consume(Token::RightBrace, "Expected '}' after struct fields")?;
        Ok(Statement::StructDef { name, fields })
    }

    fn parse_enum_def(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Enum, "Expected 'enum'")?;
        let name = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "enum name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };
        self.consume(Token::LeftBrace, "Expected '{' after enum name")?;
        let mut variants = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let variant_name = match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "variant name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            let kind = if self.match_token(&Token::LeftParen) {
                // Tuple variant: Variant(T1, T2)
                let mut types = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightParen, "Expected ')' after variant types")?;
                VariantDeclKind::Tuple(types)
            } else if self.match_token(&Token::LeftBrace) {
                // Struct variant: Variant { field: Type }
                let mut fields = Vec::new();
                while !self.check(&Token::RightBrace) && !self.is_at_end() {
                    let field_name = match &self.peek().token {
                        Token::Identifier(n) => {
                            let n = n.clone();
                            self.advance();
                            n
                        }
                        _ => break,
                    };
                    self.consume(Token::Colon, "Expected ':'")?;
                    let field_type = self.parse_type()?;
                    fields.push(FieldDecl {
                        name: field_name,
                        field_type,
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(Token::RightBrace, "Expected '}'")?;
                VariantDeclKind::Struct(fields)
            } else {
                VariantDeclKind::Unit
            };
            variants.push(VariantDecl {
                name: variant_name,
                kind,
            });
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.consume(Token::RightBrace, "Expected '}' after enum variants")?;
        Ok(Statement::EnumDef { name, variants })
    }

    fn parse_impl_block(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Impl, "Expected 'impl'")?;
        let type_name = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "type name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };
        self.consume(Token::LeftBrace, "Expected '{' after impl type")?;
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_function_definition()?);
        }
        self.consume(Token::RightBrace, "Expected '}' after impl block")?;
        Ok(Statement::ImplBlock {
            type_name,
            methods,
        })
    }

    fn parse_struct_literal(&mut self, name: String) -> CompilerResult<Expression> {
        self.consume(Token::LeftBrace, "Expected '{'")?;
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let field_name = match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "field name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let value = self.parse_expression()?;
            fields.push((field_name, value));
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.consume(Token::RightBrace, "Expected '}' after struct literal")?;
        Ok(Expression::StructLiteral { name, fields })
    }

    fn parse_enum_variant(&mut self, enum_name: String) -> CompilerResult<Expression> {
        self.consume(Token::DoubleColon, "Expected '::'")?;
        let variant = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "variant name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };
        // Check for variant data: Variant(expr)
        let data = if self.match_token(&Token::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(Token::RightParen, "Expected ')' after variant data")?;
            Some(Box::new(expr))
        } else {
            None
        };
        Ok(Expression::EnumVariant {
            enum_name,
            variant,
            data,
        })
    }

    fn parse_match_expression(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::Match, "Expected 'match'")?;
        let expr = self.parse_expression()?;
        self.consume(Token::LeftBrace, "Expected '{' after match expression")?;
        let mut arms = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(Token::FatArrow, "Expected '=>' after pattern")?;
            let body = self.parse_expression()?;
            arms.push(MatchArm { pattern, body });
            // Comma is optional between arms
            self.match_token(&Token::Comma);
        }
        self.consume(Token::RightBrace, "Expected '}' after match arms")?;
        Ok(Expression::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> CompilerResult<Pattern> {
        match &self.peek().token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::IntegerLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Expression::IntegerLiteral(n)))
            }
            Token::FloatLiteral(f) => {
                let f = *f;
                self.advance();
                Ok(Pattern::Literal(Expression::FloatLiteral(f)))
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(Expression::StringLiteral(s)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.check(&Token::DoubleColon) {
                    // Enum pattern: EnumName::Variant or EnumName::Variant(pattern)
                    self.advance();
                    let variant = match &self.peek().token {
                        Token::Identifier(v) => {
                            let v = v.clone();
                            self.advance();
                            v
                        }
                        _ => {
                            return Err(CompilerError::unexpected_token(
                                "variant name",
                                &format!("{:?}", self.peek().token),
                                self.peek().location.clone(),
                            ));
                        }
                    };
                    let data = if self.match_token(&Token::LeftParen) {
                        let inner = self.parse_pattern()?;
                        self.consume(Token::RightParen, "Expected ')'")?;
                        Some(Box::new(inner))
                    } else {
                        None
                    };
                    Ok(Pattern::Enum {
                        enum_name: name,
                        variant,
                        data,
                    })
                } else {
                    // Variable binding pattern
                    Ok(Pattern::Identifier(name))
                }
            }
            Token::LeftParen => {
                // Tuple pattern
                self.advance();
                let mut patterns = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightParen, "Expected ')'")?;
                Ok(Pattern::Tuple(patterns))
            }
            _ => Err(CompilerError::unexpected_token(
                "pattern",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
        }
    }

    /// Check if the next tokens look like a struct literal (Name { field: ... })
    /// as opposed to a block statement after an identifier
    fn is_struct_literal_start(&self) -> bool {
        // Look for pattern: { identifier : ... }
        if self.current + 2 < self.tokens.len() {
            let after_brace = &self.tokens[self.current + 1];
            let after_ident = &self.tokens[self.current + 2];
            matches!(after_brace.token, Token::Identifier(_))
                && matches!(after_ident.token, Token::Colon)
        } else {
            false
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

    fn consume(&mut self, token: Token, _message: &str) -> CompilerResult<&LocatedToken> {
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
                | Token::StringLiteral(_)
                | Token::Identifier(_)
                | Token::LeftParen
                | Token::LeftBracket
                | Token::LogicalNot
                | Token::Minus
                | Token::Match
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
                | Token::Return
                | Token::Struct
                | Token::Enum
                | Token::Impl => return,
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
