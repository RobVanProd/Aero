use crate::ast::{AstNode, Expression, Statement, Parameter, Block, Type, StructField, Visibility, EnumVariant, EnumVariantData, MatchArm, Pattern, Function};
use crate::lexer::{Token, LocatedToken};
use crate::errors::{CompilerError, CompilerResult, SourceLocation};

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
            Token::Struct => self.parse_struct_definition(),
            Token::Enum => self.parse_enum_definition(),
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
        let fn_location = self.peek().location.clone();
        self.consume(Token::Fn, "Expected 'fn'")?;
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(CompilerError::unexpected_token("function name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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
                    _ => return Err(CompilerError::unexpected_token("parameter name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                };

                self.consume(Token::Colon, "Expected ':' after parameter name")?;
                
                let param_type = self.parse_type()?;
                parameters.push(Parameter { name: param_name, param_type });

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
            _ => return Err(CompilerError::unexpected_token("variable name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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
            _ => return Err(CompilerError::unexpected_token("loop variable", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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

    fn parse_struct_definition(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Struct, "Expected 'struct'")?;
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(CompilerError::unexpected_token("struct name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        };

        // Parse generic parameters if present
        let generics = if self.match_token(&Token::LessThan) {
            let mut generics = Vec::new();
            if !self.check(&Token::GreaterThan) {
                loop {
                    match &self.peek().token {
                        Token::Identifier(generic_name) => {
                            generics.push(generic_name.clone());
                            self.advance();
                        }
                        _ => return Err(CompilerError::unexpected_token("generic parameter name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::GreaterThan, "Expected '>' after generic parameters")?;
            generics
        } else {
            Vec::new()
        };

        // Check if this is a tuple struct
        let is_tuple = self.check(&Token::LeftParen);
        
        let fields = if is_tuple {
            // Parse tuple struct: struct Point(f64, f64);
            self.parse_tuple_struct_fields()?
        } else {
            // Parse named struct: struct Point { x: f64, y: f64 }
            self.parse_named_struct_fields()?
        };

        Ok(Statement::Struct {
            name,
            generics,
            fields,
            is_tuple,
        })
    }

    fn parse_tuple_struct_fields(&mut self) -> CompilerResult<Vec<StructField>> {
        self.consume(Token::LeftParen, "Expected '(' for tuple struct")?;
        let mut fields = Vec::new();
        let mut field_index = 0;
        
        if !self.check(&Token::RightParen) {
            loop {
                // Parse optional visibility for tuple struct fields
                let visibility = if let Token::Identifier(name) = &self.peek().token {
                    if name == "pub" {
                        self.advance(); // consume 'pub'
                        Visibility::Public
                    } else {
                        Visibility::Public // Tuple struct fields are public by default
                    }
                } else {
                    Visibility::Public
                };
                
                let field_type = self.parse_type()?;
                fields.push(StructField {
                    name: field_index.to_string(), // Use index as field name for tuple structs
                    field_type,
                    visibility,
                });
                field_index += 1;
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after tuple struct fields")?;
        self.consume(Token::Semicolon, "Expected ';' after tuple struct definition")?;
        Ok(fields)
    }

    fn parse_named_struct_fields(&mut self) -> CompilerResult<Vec<StructField>> {
        self.consume(Token::LeftBrace, "Expected '{' for struct definition")?;
        let mut fields = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Parse visibility (pub or private)
            let visibility = if let Token::Identifier(name) = &self.peek().token {
                if name == "pub" {
                    self.advance(); // consume 'pub'
                    Visibility::Public
                } else {
                    Visibility::Private
                }
            } else {
                Visibility::Private
            };
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(CompilerError::unexpected_token("field name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
            };
            
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            
            fields.push(StructField {
                name: field_name,
                field_type,
                visibility,
            });
            
            // Optional comma after field
            if !self.match_token(&Token::Comma) {
                // Allow trailing comma or no comma before closing brace
                if !self.check(&Token::RightBrace) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct fields")?;
        Ok(fields)
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

        while self.match_token(&Token::Multiply) || self.match_token(&Token::Divide) || self.match_token(&Token::Modulo) {
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
                } else {
                    return Err(CompilerError::InvalidSyntax {
                        message: "Only identifiers can be called as functions".to_string(),
                        location: self.previous().location.clone(),
                    });
                }
            } else if self.match_token(&Token::Dot) {
                // Field access or method call
                let field_name = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => return Err(CompilerError::unexpected_token("field name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                };
                
                // Check if this is a method call
                if self.check(&Token::LeftParen) {
                    self.advance(); // consume '('
                    let mut arguments = Vec::new();
                    if !self.check(&Token::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RightParen, "Expected ')' after method arguments")?;
                    
                    expr = Expression::MethodCall {
                        object: Box::new(expr),
                        method: field_name,
                        arguments,
                    };
                } else {
                    // Field access
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field: field_name,
                    };
                }
            } else if self.check(&Token::LeftBracket) && matches!(expr, Expression::Identifier(_)) {
                // Array access: expr[index]
                self.advance(); // consume '['
                let index = self.parse_expression()?;
                self.consume(Token::RightBracket, "Expected ']' after array index")?;
                
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
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
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check if this is a struct literal
                if self.check(&Token::LeftBrace) {
                    self.parse_struct_literal(name)
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::PrintMacro => self.parse_print_macro(false),
            Token::PrintlnMacro => self.parse_print_macro(true),
            Token::Match => self.parse_match_expression(),
            Token::Vec => self.parse_vec_macro(),
            Token::Format => self.parse_format_macro(),
            Token::LeftBracket => self.parse_array_literal(),
            _ => Err(CompilerError::unexpected_token("expression", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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
            _ => return Err(CompilerError::unexpected_token("format string", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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

    fn parse_struct_literal(&mut self, name: String) -> CompilerResult<Expression> {
        self.consume(Token::LeftBrace, "Expected '{' for struct literal")?;
        
        let mut fields = Vec::new();
        let mut base = None;
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Check for struct update syntax (..)
            if self.match_token(&Token::DotDot) {
                base = Some(Box::new(self.parse_expression()?));
                break;
            }
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(CompilerError::unexpected_token("field name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
            };
            
            self.consume(Token::Colon, "Expected ':' after field name in struct literal")?;
            let field_value = self.parse_expression()?;
            
            fields.push((field_name, field_value));
            
            if !self.match_token(&Token::Comma) {
                // Allow trailing comma or no comma before closing brace
                if !self.check(&Token::RightBrace) && !self.check(&Token::DotDot) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct literal")?;
        
        Ok(Expression::StructLiteral {
            name,
            fields,
            base,
        })
    }

    fn parse_enum_definition(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Enum, "Expected 'enum'")?;
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(CompilerError::unexpected_token("enum name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        };

        // Parse generic parameters if present
        let generics = if self.match_token(&Token::LessThan) {
            let mut generics = Vec::new();
            if !self.check(&Token::GreaterThan) {
                loop {
                    match &self.peek().token {
                        Token::Identifier(generic_name) => {
                            generics.push(generic_name.clone());
                            self.advance();
                        }
                        _ => return Err(CompilerError::unexpected_token("generic parameter name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::GreaterThan, "Expected '>' after generic parameters")?;
            generics
        } else {
            Vec::new()
        };

        self.consume(Token::LeftBrace, "Expected '{' after enum name")?;
        
        let mut variants = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let variant_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(CompilerError::unexpected_token("enum variant name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
            };

            let data = if self.check(&Token::LeftParen) {
                // Tuple variant: Some(T)
                Some(self.parse_enum_tuple_variant()?)
            } else if self.check(&Token::LeftBrace) {
                // Struct variant: Point { x: i32, y: i32 }
                Some(self.parse_enum_struct_variant()?)
            } else {
                // Unit variant: None
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                data,
            });

            if !self.match_token(&Token::Comma) {
                // Allow trailing comma or no comma before closing brace
                if !self.check(&Token::RightBrace) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }

        self.consume(Token::RightBrace, "Expected '}' after enum variants")?;

        Ok(Statement::Enum {
            name,
            generics,
            variants,
        })
    }

    fn parse_enum_tuple_variant(&mut self) -> CompilerResult<EnumVariantData> {
        self.consume(Token::LeftParen, "Expected '(' for tuple variant")?;
        
        let mut types = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                types.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after tuple variant types")?;
        Ok(EnumVariantData::Tuple(types))
    }

    fn parse_enum_struct_variant(&mut self) -> CompilerResult<EnumVariantData> {
        self.consume(Token::LeftBrace, "Expected '{' for struct variant")?;
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Parse visibility (pub or private)
            let visibility = if let Token::Identifier(name) = &self.peek().token {
                if name == "pub" {
                    self.advance(); // consume 'pub'
                    Visibility::Public
                } else {
                    Visibility::Private
                }
            } else {
                Visibility::Private
            };
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(CompilerError::unexpected_token("field name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
            };
            
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            
            fields.push(StructField {
                name: field_name,
                field_type,
                visibility,
            });
            
            if !self.match_token(&Token::Comma) {
                if !self.check(&Token::RightBrace) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct variant fields")?;
        Ok(EnumVariantData::Struct(fields))
    }

    fn parse_impl_block(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Impl, "Expected 'impl'")?;
        
        // Parse generic parameters if present
        let generics = if self.match_token(&Token::LessThan) {
            let mut generics = Vec::new();
            if !self.check(&Token::GreaterThan) {
                loop {
                    match &self.peek().token {
                        Token::Identifier(generic_name) => {
                            generics.push(generic_name.clone());
                            self.advance();
                        }
                        _ => return Err(CompilerError::unexpected_token("generic parameter name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::GreaterThan, "Expected '>' after generic parameters")?;
            generics
        } else {
            Vec::new()
        };

        let type_name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(CompilerError::unexpected_token("type name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        };

        // Check for trait implementation (impl Trait for Type)
        let trait_name = if let Token::Identifier(name) = &self.peek().token {
            if name == "for" {
                // This was actually a trait name, not type name
                let trait_name = type_name;
                self.advance(); // consume 'for'
                
                let actual_type_name = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => return Err(CompilerError::unexpected_token("type name after 'for'", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                };
                
                (actual_type_name, Some(trait_name))
            } else {
                (type_name, None)
            }
        } else {
            (type_name, None)
        };

        self.consume(Token::LeftBrace, "Expected '{' after impl declaration")?;
        
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Fn) {
                let method = self.parse_method_definition()?;
                methods.push(method);
            } else {
                return Err(CompilerError::unexpected_token("method definition", &format!("{:?}", self.peek().token), self.peek().location.clone()));
            }
        }

        self.consume(Token::RightBrace, "Expected '}' after impl block")?;

        Ok(Statement::Impl {
            generics,
            type_name: trait_name.0,
            trait_name: trait_name.1,
            methods,
        })
    }

    fn parse_method_definition(&mut self) -> CompilerResult<Function> {
        self.consume(Token::Fn, "Expected 'fn'")?;
        
        let name = match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(CompilerError::unexpected_token("method name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        };

        self.consume(Token::LeftParen, "Expected '(' after method name")?;
        
        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match &self.peek().token {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => return Err(CompilerError::unexpected_token("parameter name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                };

                self.consume(Token::Colon, "Expected ':' after parameter name")?;
                
                let param_type = self.parse_type()?;
                parameters.push(Parameter { name: param_name, param_type });

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

        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_match_expression(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::Match, "Expected 'match'")?;
        
        let expression = Box::new(self.parse_expression()?);
        
        self.consume(Token::LeftBrace, "Expected '{' after match expression")?;
        
        let mut arms = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            
            // Parse optional guard condition
            let guard = if let Token::Identifier(name) = &self.peek().token {
                if name == "if" {
                    self.advance(); // consume 'if'
                    Some(self.parse_expression()?)
                } else {
                    None
                }
            } else {
                None
            };
            
            self.consume(Token::FatArrow, "Expected '=>' after match pattern")?;
            
            let body = self.parse_expression()?;
            
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
            
            // Optional comma after match arm
            if !self.match_token(&Token::Comma) {
                // Allow trailing comma or no comma before closing brace
                if !self.check(&Token::RightBrace) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after match arms")?;
        
        Ok(Expression::Match {
            expression,
            arms,
        })
    }

    fn parse_pattern(&mut self) -> CompilerResult<Pattern> {
        self.parse_or_pattern()
    }

    fn parse_or_pattern(&mut self) -> CompilerResult<Pattern> {
        let mut patterns = vec![self.parse_binding_pattern()?];
        
        while self.match_token(&Token::Pipe) {
            patterns.push(self.parse_binding_pattern()?);
        }
        
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or(patterns))
        }
    }

    fn parse_binding_pattern(&mut self) -> CompilerResult<Pattern> {
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            if self.peek_ahead(1).map(|t| &t.token) == Some(&Token::At) {
                self.advance(); // consume identifier
                self.consume(Token::At, "Expected '@' for binding pattern")?;
                let pattern = Box::new(self.parse_primary_pattern()?);
                return Ok(Pattern::Binding { name, pattern });
            }
        }
        
        self.parse_primary_pattern()
    }

    fn parse_primary_pattern(&mut self) -> CompilerResult<Pattern> {
        match &self.peek().token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::IntegerLiteral(value) => {
                let value = *value;
                self.advance();
                
                // Check for range pattern
                if self.check(&Token::DotDot) || self.check(&Token::DotDotEqual) {
                    let inclusive = self.match_token(&Token::DotDotEqual);
                    if !inclusive {
                        self.consume(Token::DotDot, "Expected '..' for range pattern")?;
                    }
                    
                    let end_pattern = Box::new(self.parse_primary_pattern()?);
                    Ok(Pattern::Range {
                        start: Box::new(Pattern::Literal(Expression::IntegerLiteral(value))),
                        end: end_pattern,
                        inclusive,
                    })
                } else {
                    Ok(Pattern::Literal(Expression::IntegerLiteral(value)))
                }
            }
            Token::FloatLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Pattern::Literal(Expression::FloatLiteral(value)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                if self.check(&Token::LeftParen) {
                    // Enum tuple variant pattern: Some(x)
                    self.consume(Token::LeftParen, "Expected '(' for enum tuple pattern")?;
                    
                    let data = if self.check(&Token::RightParen) {
                        None
                    } else {
                        Some(Box::new(self.parse_pattern()?))
                    };
                    
                    self.consume(Token::RightParen, "Expected ')' after enum tuple pattern")?;
                    
                    Ok(Pattern::Enum {
                        variant: name,
                        data,
                    })
                } else if self.check(&Token::LeftBrace) {
                    // Struct pattern: Point { x, y } or enum struct variant
                    self.parse_struct_pattern(name)
                } else {
                    // Simple identifier pattern
                    Ok(Pattern::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                
                if self.check(&Token::RightParen) {
                    // Unit tuple pattern: ()
                    self.advance();
                    Ok(Pattern::Tuple(vec![]))
                } else {
                    // Tuple pattern: (x, y, z)
                    let mut patterns = vec![self.parse_pattern()?];
                    
                    while self.match_token(&Token::Comma) {
                        if self.check(&Token::RightParen) {
                            break; // Allow trailing comma
                        }
                        patterns.push(self.parse_pattern()?);
                    }
                    
                    self.consume(Token::RightParen, "Expected ')' after tuple pattern")?;
                    
                    if patterns.len() == 1 {
                        // Single element in parentheses, not a tuple
                        Ok(patterns.into_iter().next().unwrap())
                    } else {
                        Ok(Pattern::Tuple(patterns))
                    }
                }
            }
            _ => Err(CompilerError::unexpected_token("pattern", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        }
    }

    fn parse_struct_pattern(&mut self, name: String) -> CompilerResult<Pattern> {
        self.consume(Token::LeftBrace, "Expected '{' for struct pattern")?;
        
        let mut fields = Vec::new();
        let mut rest = false;
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Check for rest pattern (..)
            if self.match_token(&Token::DotDot) {
                rest = true;
                break;
            }
            
            let field_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(CompilerError::unexpected_token("field name", &format!("{:?}", self.peek().token), self.peek().location.clone())),
            };
            
            let field_pattern = if self.match_token(&Token::Colon) {
                // Explicit pattern: { x: pattern }
                self.parse_pattern()?
            } else {
                // Shorthand: { x } is equivalent to { x: x }
                Pattern::Identifier(field_name.clone())
            };
            
            fields.push((field_name, field_pattern));
            
            if !self.match_token(&Token::Comma) {
                // Allow trailing comma or no comma before closing brace or rest
                if !self.check(&Token::RightBrace) && !self.check(&Token::DotDot) {
                    return Err(CompilerError::unexpected_token("',' or '}'", &format!("{:?}", self.peek().token), self.peek().location.clone()));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct pattern")?;
        
        Ok(Pattern::Struct {
            name,
            fields,
            rest,
        })
    }

    fn peek_ahead(&self, offset: usize) -> Option<&LocatedToken> {
        let index = self.current + offset;
        if index < self.tokens.len() {
            Some(&self.tokens[index])
        } else {
            None
        }
    }

    fn parse_type(&mut self) -> CompilerResult<Type> {
        match &self.peek().token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check for generic type parameters
                if self.match_token(&Token::LessThan) {
                    let mut type_args = Vec::new();
                    if !self.check(&Token::GreaterThan) {
                        loop {
                            type_args.push(self.parse_type()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::GreaterThan, "Expected '>' after generic type arguments")?;
                    
                    // Handle specific generic types
                    match name.as_str() {
                        "Vec" => {
                            if type_args.len() != 1 {
                                return Err(CompilerError::InvalidSyntax {
                                    message: "Vec requires exactly one type argument".to_string(),
                                    location: self.previous().location.clone(),
                                });
                            }
                            Ok(Type::Vec {
                                element_type: Box::new(type_args.into_iter().next().unwrap()),
                            })
                        }
                        "HashMap" => {
                            if type_args.len() != 2 {
                                return Err(CompilerError::InvalidSyntax {
                                    message: "HashMap requires exactly two type arguments".to_string(),
                                    location: self.previous().location.clone(),
                                });
                            }
                            let mut args = type_args.into_iter();
                            Ok(Type::HashMap {
                                key_type: Box::new(args.next().unwrap()),
                                value_type: Box::new(args.next().unwrap()),
                            })
                        }
                        _ => Ok(Type::Generic {
                            name,
                            type_args,
                        })
                    }
                } else {
                    // Handle special built-in types
                    match name.as_str() {
                        "Vec" => {
                            // Vec without type parameters - error
                            return Err(CompilerError::InvalidSyntax {
                                message: "Vec requires type parameters".to_string(),
                                location: self.previous().location.clone(),
                            });
                        }
                        "HashMap" => {
                            // HashMap without type parameters - error
                            return Err(CompilerError::InvalidSyntax {
                                message: "HashMap requires type parameters".to_string(),
                                location: self.previous().location.clone(),
                            });
                        }
                        _ => Ok(Type::Named(name))
                    }
                }
            }
            Token::LeftBrace => {
                // Array type: [T; N] or [T]
                self.advance(); // consume '['
                let element_type = Box::new(self.parse_type()?);
                
                if self.match_token(&Token::Semicolon) {
                    // Fixed-size array: [T; N]
                    let size = match &self.peek().token {
                        Token::IntegerLiteral(n) => {
                            let size = *n as usize;
                            self.advance();
                            Some(size)
                        }
                        _ => return Err(CompilerError::unexpected_token("array size", &format!("{:?}", self.peek().token), self.peek().location.clone())),
                    };
                    self.consume(Token::RightBrace, "Expected ']' after array size")?;
                    Ok(Type::Array { element_type, size })
                } else {
                    // Slice type: [T]
                    self.consume(Token::RightBrace, "Expected ']' after array element type")?;
                    Ok(Type::Slice { element_type })
                }
            }
            _ => Err(CompilerError::unexpected_token("type", &format!("{:?}", self.peek().token), self.peek().location.clone())),
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
                | Token::Match
        )
    }



    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if matches!(self.previous().token, Token::Semicolon) {
                return;
            }

            match self.peek().token {
                Token::Fn | Token::Let | Token::If | Token::While | Token::For | Token::Loop | Token::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    // Generic and collection parsing methods
    fn parse_vec_macro(&mut self) -> CompilerResult<Expression> {
        // Note: Vec! macro parsing - for now we'll handle it as a special identifier
        // The lexer should tokenize "vec!" as Token::Vec followed by Token::LogicalNot
        // But since we don't have proper macro support yet, we'll parse it as a function-like call
        self.consume(Token::Vec, "Expected 'vec'")?;
        
        // Check if this is followed by ! for macro syntax
        if self.match_token(&Token::LogicalNot) {
            // This is vec! macro
            self.consume(Token::LeftBracket, "Expected '[' after 'vec!'")?;
            
            let mut elements = Vec::new();
            if !self.check(&Token::RightBracket) {
                loop {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(Token::RightBracket, "Expected ']' after vec elements")?;
            
            Ok(Expression::VecMacro { elements })
        } else {
            // This is just Vec as a type identifier, treat as regular identifier
            Ok(Expression::Identifier("Vec".to_string()))
        }
    }

    fn parse_format_macro(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::Format, "Expected 'format!'")?;
        self.consume(Token::LeftParen, "Expected '(' after 'format!'")?;

        let format_string = match &self.peek().token {
            Token::Identifier(s) if s.starts_with('"') && s.ends_with('"') => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(CompilerError::unexpected_token("format string", &format!("{:?}", self.peek().token), self.peek().location.clone())),
        };

        let mut arguments = Vec::new();
        while self.match_token(&Token::Comma) {
            arguments.push(self.parse_expression()?);
        }

        self.consume(Token::RightParen, "Expected ')' after format arguments")?;

        Ok(Expression::FormatMacro {
            format_string,
            arguments,
        })
    }

    fn parse_array_literal(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::LeftBracket, "Expected '[' for array literal")?;
        
        let mut elements = Vec::new();
        if !self.check(&Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightBracket, "Expected ']' after array elements")?;
        
        Ok(Expression::ArrayLiteral { elements })
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

#[cfg(test)]
mod parser_struct_test;

#[cfg(test)]
mod parser_struct_simple_test;

#[cfg(test)]
mod parser_enum_pattern_tests {
    include!("parser_enum_pattern_test.rs");
}

#[cfg(test)]
mod parser_generic_collection_tests {
    include!("parser_generic_collection_test.rs");
}