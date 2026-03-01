#![allow(clippy::result_large_err)]

use crate::ast::{
    AstNode, Block, Expression, FieldDecl, MatchArm, Parameter, Pattern, Statement, TraitMethod,
    Type, VariantDecl, VariantDeclKind,
};
use crate::errors::{CompilerError, CompilerResult, SourceLocation};
use crate::lexer::{LocatedToken, Token, tokenize_with_locations};

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
        } else if errors.len() == 1 {
            Err(errors.into_iter().next().unwrap())
        } else {
            Err(CompilerError::MultiError { errors })
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
            Token::Trait => self.parse_trait_def(),
            // Phase 7: Module system
            Token::Mod => self.parse_mod_declaration(),
            Token::Use => self.parse_use_import(),
            Token::Pub => self.parse_pub_item(),
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

        // Parse optional generic type parameters: fn name<T: Bound, U>(...)
        let (type_params, trait_bounds) = self.parse_optional_type_params()?;

        self.consume(Token::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                // Phase 5: Handle &self, &mut self, and self parameters
                if self.check(&Token::Ampersand) {
                    self.advance(); // consume &
                    let mutable = self.match_token(&Token::Mut);
                    if self.check(&Token::Self_) {
                        self.advance(); // consume self
                        parameters.push(Parameter {
                            name: "self".to_string(),
                            param_type: Type::Reference(
                                Box::new(Type::Named("Self".to_string())),
                                mutable,
                            ),
                        });
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        continue;
                    } else {
                        // Not &self, backtrack: this was a reference type parameter
                        self.current -= if mutable { 2 } else { 1 };
                    }
                } else if self.check(&Token::Self_) {
                    self.advance();
                    parameters.push(Parameter {
                        name: "self".to_string(),
                        param_type: Type::Named("Self".to_string()),
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    continue;
                }

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

        // Phase 5: Parse optional where clause: where T: Bound, U: Bound
        let mut all_bounds = trait_bounds;
        if self.match_token(&Token::Where) {
            loop {
                // Consume type param name
                let where_param = match &self.peek().token {
                    Token::Identifier(n) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => break,
                };
                // Consume : Bound1 + Bound2
                if self.match_token(&Token::Colon) {
                    let mut param_bounds = Vec::new();
                    if let Token::Identifier(bound_name) = &self.peek().token {
                        param_bounds.push(bound_name.clone());
                        self.advance();
                    }
                    while self.check(&Token::Plus) {
                        self.advance();
                        if let Token::Identifier(bound_name) = &self.peek().token {
                            param_bounds.push(bound_name.clone());
                            self.advance();
                        }
                    }
                    if !param_bounds.is_empty() {
                        all_bounds.push((where_param, param_bounds));
                    }
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        let body = self.parse_block()?;

        Ok(Statement::Function {
            name,
            parameters,
            return_type,
            body,
            type_params,
            trait_bounds: all_bounds,
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

        // Phase 5: Borrow expressions &x and &mut x
        if self.match_token(&Token::Ampersand) {
            let mutable = self.match_token(&Token::Mut);
            let expr = self.parse_unary()?;
            return Ok(Expression::Borrow {
                expr: Box::new(expr),
                mutable,
            });
        }

        // Phase 5: Dereference expression *x
        if self.match_token(&Token::Multiply) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Deref(Box::new(expr)));
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
            Token::FStringLiteral(s) => {
                // Outside print!/println!, keep f-strings as raw string literals for now.
                let s = s.clone();
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::VecMacro => self.parse_vec_macro_literal(),
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Phase 6: Standard library constructors Some, None, Ok, Err
                match name.as_str() {
                    "Some" => {
                        // Some(value) - Option::Some
                        self.consume(Token::LeftParen, "Expected '(' after 'Some'")?;
                        let value = self.parse_expression()?;
                        self.consume(Token::RightParen, "Expected ')' after Some value")?;
                        return Ok(Expression::EnumVariant {
                            enum_name: "Option".to_string(),
                            variant: "Some".to_string(),
                            data: Some(Box::new(value)),
                        });
                    }
                    "None" => {
                        // None - Option::None (no data)
                        return Ok(Expression::EnumVariant {
                            enum_name: "Option".to_string(),
                            variant: "None".to_string(),
                            data: None,
                        });
                    }
                    "Ok" => {
                        // Ok(value) - Result::Ok
                        self.consume(Token::LeftParen, "Expected '(' after 'Ok'")?;
                        let value = self.parse_expression()?;
                        self.consume(Token::RightParen, "Expected ')' after Ok value")?;
                        return Ok(Expression::EnumVariant {
                            enum_name: "Result".to_string(),
                            variant: "Ok".to_string(),
                            data: Some(Box::new(value)),
                        });
                    }
                    "Err" => {
                        // Err(error) - Result::Err
                        self.consume(Token::LeftParen, "Expected '(' after 'Err'")?;
                        let value = self.parse_expression()?;
                        self.consume(Token::RightParen, "Expected ')' after Err value")?;
                        return Ok(Expression::EnumVariant {
                            enum_name: "Result".to_string(),
                            variant: "Err".to_string(),
                            data: Some(Box::new(value)),
                        });
                    }
                    _ => {}
                }

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
            // Phase 7: Closure expressions |params| body
            Token::Pipe => self.parse_closure(),
            _ => Err(CompilerError::unexpected_token(
                "expression",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
        }
    }

    /// Parse closure expression: `|x: i32, y: i32| x + y` or `|x| { ... }`
    fn parse_closure(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::Pipe, "Expected '|' to start closure")?;

        let mut params = Vec::new();

        // Parse parameters (may be empty: || { ... })
        if !self.check(&Token::Pipe) {
            loop {
                let param_name = match &self.peek().token {
                    Token::Identifier(n) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(CompilerError::unexpected_token(
                            "closure parameter name",
                            &format!("{:?}", self.peek().token),
                            self.peek().location.clone(),
                        ));
                    }
                };

                // Optional type annotation: |x: i32|
                let param_type = if self.match_token(&Token::Colon) {
                    self.parse_type()?
                } else {
                    // Infer type later (default to i32 for now)
                    Type::Named("i32".to_string())
                };

                params.push(Parameter {
                    name: param_name,
                    param_type,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(Token::Pipe, "Expected '|' after closure parameters")?;

        // Parse body: either a block { ... } or a single expression
        let body = if self.check(&Token::LeftBrace) {
            let block = self.parse_block()?;
            if let Some(expr) = block.expression {
                expr
            } else if let Some(Statement::Expression(expr)) = block.statements.last() {
                expr.clone()
            } else {
                Expression::IntegerLiteral(0) // Unit closure
            }
        } else {
            self.parse_expression()?
        };

        Ok(Expression::Closure {
            params,
            body: Box::new(body),
        })
    }

    fn parse_print_macro(&mut self, is_println: bool) -> CompilerResult<Expression> {
        if is_println {
            self.consume(Token::PrintlnMacro, "Expected 'println!'")?;
        } else {
            self.consume(Token::PrintMacro, "Expected 'print!'")?;
        }

        self.consume(Token::LeftParen, "Expected '(' after print macro")?;

        let mut arguments = Vec::new();
        let format_string = match &self.peek().token {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            Token::FStringLiteral(s) => {
                let template = s.clone();
                let location = self.peek().location.clone();
                self.advance();
                let (format_string, mut interpolated_args) =
                    self.parse_interpolated_format_string(&template, location)?;
                arguments.append(&mut interpolated_args);
                format_string
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

    fn parse_interpolated_format_string(
        &self,
        template: &str,
        location: SourceLocation,
    ) -> CompilerResult<(String, Vec<Expression>)> {
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0usize;
        let mut format_string = String::new();
        let mut arguments = Vec::new();

        while i < chars.len() {
            match chars[i] {
                '{' => {
                    // Escaped opening brace: `{{`
                    if i + 1 < chars.len() && chars[i + 1] == '{' {
                        format_string.push('{');
                        i += 2;
                        continue;
                    }

                    let start = i + 1;
                    let mut depth = 1usize;
                    i += 1;

                    while i < chars.len() {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }

                    if depth != 0 {
                        return Err(CompilerError::InvalidSyntax {
                            message: "Unclosed interpolation in format string".to_string(),
                            location,
                        });
                    }

                    let expr_source: String = chars[start..i].iter().collect();
                    let expr =
                        self.parse_interpolation_expression(&expr_source, location.clone())?;
                    format_string.push_str("{}");
                    arguments.push(expr);
                    i += 1; // consume closing `}`
                }
                '}' => {
                    // Escaped closing brace: `}}`
                    if i + 1 < chars.len() && chars[i + 1] == '}' {
                        format_string.push('}');
                        i += 2;
                    } else {
                        return Err(CompilerError::InvalidSyntax {
                            message: "Unmatched `}` in format string".to_string(),
                            location,
                        });
                    }
                }
                ch => {
                    format_string.push(ch);
                    i += 1;
                }
            }
        }

        Ok((format_string, arguments))
    }

    fn parse_interpolation_expression(
        &self,
        expr_source: &str,
        location: SourceLocation,
    ) -> CompilerResult<Expression> {
        let trimmed = expr_source.trim();
        if trimmed.is_empty() {
            return Err(CompilerError::InvalidSyntax {
                message: "Empty interpolation expression".to_string(),
                location,
            });
        }

        let tokens = tokenize_with_locations(trimmed, None);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression()?;
        if !parser.is_at_end() {
            return Err(CompilerError::InvalidSyntax {
                message: format!("Invalid interpolation expression: `{}`", trimmed),
                location,
            });
        }
        Ok(expr)
    }

    fn parse_type(&mut self) -> CompilerResult<Type> {
        match &self.peek().token {
            // Phase 5: Reference types &T and &mut T
            Token::Ampersand => {
                self.advance();
                let mutable = self.match_token(&Token::Mut);
                let inner = self.parse_type()?;
                Ok(Type::Reference(Box::new(inner), mutable))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                // Check for generic type: Name<T1, T2>
                if self.check(&Token::LessThan) {
                    let checkpoint = self.current;
                    self.advance(); // consume '<'
                    let mut type_args = Vec::new();
                    if !self.check(&Token::GreaterThan) {
                        loop {
                            match self.parse_type() {
                                Ok(t) => type_args.push(t),
                                Err(_) => {
                                    // Not a generic type, backtrack
                                    self.current = checkpoint;
                                    return Ok(Type::Named(name));
                                }
                            }
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    if self.match_token(&Token::GreaterThan) {
                        return Ok(Type::Generic(name, type_args));
                    } else {
                        // Backtrack - not a generic type
                        self.current = checkpoint;
                    }
                }
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

    fn parse_vec_macro_literal(&mut self) -> CompilerResult<Expression> {
        self.consume(Token::VecMacro, "Expected 'vec!'")?;
        // For now vec! lowers to the same IR/semantics as array literals.
        self.parse_array_literal()
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
        let (type_params, _bounds) = self.parse_optional_type_params()?;
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
        Ok(Statement::StructDef {
            name,
            fields,
            type_params,
        })
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
        let (type_params, _bounds) = self.parse_optional_type_params()?;
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
        Ok(Statement::EnumDef {
            name,
            variants,
            type_params,
        })
    }

    fn parse_impl_block(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Impl, "Expected 'impl'")?;

        // Parse optional generic type parameters: impl<T>
        let (type_params, _bounds) = self.parse_optional_type_params()?;

        let first_name = match &self.peek().token {
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
        // Skip generic type args on the first name: Container<T> or Container<T, U>
        self.skip_generic_type_args();

        // Check for "impl Trait for Type" syntax
        let (trait_name, type_name) = if self.match_token(&Token::For) {
            let type_name = match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "type name after 'for'",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            // Skip generic type args on the type name after 'for': Container<T>
            self.skip_generic_type_args();
            (Some(first_name), type_name)
        } else {
            (None, first_name)
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
            type_params,
            trait_name,
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

                // Phase 6: Standard library pattern shortcuts Some, None, Ok, Err
                match name.as_str() {
                    "Some" => {
                        // Some(pattern) - matches Option::Some
                        self.consume(Token::LeftParen, "Expected '(' after 'Some' in pattern")?;
                        let inner = self.parse_pattern()?;
                        self.consume(Token::RightParen, "Expected ')' after Some pattern")?;
                        return Ok(Pattern::Enum {
                            enum_name: "Option".to_string(),
                            variant: "Some".to_string(),
                            data: Some(Box::new(inner)),
                        });
                    }
                    "None" => {
                        // None - matches Option::None
                        return Ok(Pattern::Enum {
                            enum_name: "Option".to_string(),
                            variant: "None".to_string(),
                            data: None,
                        });
                    }
                    "Ok" => {
                        // Ok(pattern) - matches Result::Ok
                        self.consume(Token::LeftParen, "Expected '(' after 'Ok' in pattern")?;
                        let inner = self.parse_pattern()?;
                        self.consume(Token::RightParen, "Expected ')' after Ok pattern")?;
                        return Ok(Pattern::Enum {
                            enum_name: "Result".to_string(),
                            variant: "Ok".to_string(),
                            data: Some(Box::new(inner)),
                        });
                    }
                    "Err" => {
                        // Err(pattern) - matches Result::Err
                        self.consume(Token::LeftParen, "Expected '(' after 'Err' in pattern")?;
                        let inner = self.parse_pattern()?;
                        self.consume(Token::RightParen, "Expected ')' after Err pattern")?;
                        return Ok(Pattern::Enum {
                            enum_name: "Result".to_string(),
                            variant: "Err".to_string(),
                            data: Some(Box::new(inner)),
                        });
                    }
                    _ => {}
                }

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

    // --- Phase 5 parsing methods ---

    /// Parse optional generic type parameters: <T, U, V>
    fn parse_optional_type_params(
        &mut self,
    ) -> CompilerResult<(Vec<String>, Vec<(String, Vec<String>)>)> {
        if !self.match_token(&Token::LessThan) {
            return Ok((vec![], vec![]));
        }
        let mut params = Vec::new();
        let mut bounds = Vec::new();
        loop {
            let param_name = match &self.peek().token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "type parameter name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            params.push(param_name.clone());
            // Phase 5: Parse optional trait bounds  T: Bound1 + Bound2
            if self.match_token(&Token::Colon) {
                let mut param_bounds = Vec::new();
                // Consume the first bound name
                if let Token::Identifier(bound_name) = &self.peek().token {
                    param_bounds.push(bound_name.clone());
                    self.advance();
                }
                // Consume additional bounds: + Bound2 + Bound3 ...
                while self.check(&Token::Plus) {
                    self.advance(); // consume +
                    if let Token::Identifier(bound_name) = &self.peek().token {
                        param_bounds.push(bound_name.clone());
                        self.advance();
                    }
                }
                if !param_bounds.is_empty() {
                    bounds.push((param_name, param_bounds));
                }
            }
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.consume(Token::GreaterThan, "Expected '>' after type parameters")?;
        Ok((params, bounds))
    }

    /// Skip generic type arguments like `<T>` or `<T, U>` if present.
    /// Used after type names in impl blocks where we don't need to store the args.
    fn skip_generic_type_args(&mut self) {
        if self.check(&Token::LessThan) {
            self.advance(); // consume <
            let mut depth = 1;
            while depth > 0 && !self.is_at_end() {
                if self.check(&Token::LessThan) {
                    depth += 1;
                } else if self.check(&Token::GreaterThan) {
                    depth -= 1;
                }
                self.advance();
            }
        }
    }

    fn parse_trait_def(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Trait, "Expected 'trait'")?;
        let name = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "trait name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };
        let (type_params, _bounds) = self.parse_optional_type_params()?;
        self.consume(Token::LeftBrace, "Expected '{' after trait name")?;
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            self.consume(Token::Fn, "Expected 'fn' in trait body")?;
            let method_name = match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "method name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            };
            self.consume(Token::LeftParen, "Expected '(' after method name")?;
            let mut parameters = Vec::new();
            if !self.check(&Token::RightParen) {
                loop {
                    // Handle &self and &mut self
                    if self.check(&Token::Ampersand) {
                        self.advance(); // consume &
                        let mutable = self.match_token(&Token::Mut);
                        if self.check(&Token::Self_) {
                            self.advance(); // consume self
                            parameters.push(Parameter {
                                name: "self".to_string(),
                                param_type: Type::Reference(
                                    Box::new(Type::Named("Self".to_string())),
                                    mutable,
                                ),
                            });
                        }
                    } else if self.check(&Token::Self_) {
                        self.advance(); // consume self
                        parameters.push(Parameter {
                            name: "self".to_string(),
                            param_type: Type::Named("Self".to_string()),
                        });
                    } else {
                        let param_name = match &self.peek().token {
                            Token::Identifier(n) => {
                                let n = n.clone();
                                self.advance();
                                n
                            }
                            _ => break,
                        };
                        self.consume(Token::Colon, "Expected ':' after parameter name")?;
                        let param_type = self.parse_type()?;
                        parameters.push(Parameter {
                            name: param_name,
                            param_type,
                        });
                    }
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::RightParen, "Expected ')'")?;
            let return_type = if self.match_token(&Token::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };
            // Check for default body or just a semicolon
            let body = if self.check(&Token::LeftBrace) {
                Some(self.parse_block()?)
            } else {
                self.consume(
                    Token::Semicolon,
                    "Expected ';' or '{' after trait method signature",
                )?;
                None
            };
            methods.push(TraitMethod {
                name: method_name,
                parameters,
                return_type,
                body,
            });
        }
        self.consume(Token::RightBrace, "Expected '}' after trait body")?;
        Ok(Statement::TraitDef {
            name,
            type_params,
            methods,
        })
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
                | Token::FStringLiteral(_)
                | Token::Identifier(_)
                | Token::LeftParen
                | Token::LeftBracket
                | Token::LogicalNot
                | Token::Minus
                | Token::Match
                | Token::Pipe // closures
            | Token::PrintMacro
                | Token::PrintlnMacro
                | Token::VecMacro
                | Token::Ampersand
                | Token::Multiply
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
                | Token::Impl
                | Token::Trait
                | Token::Mod
                | Token::Use
                | Token::Pub => return,
                _ => {}
            }

            self.advance();
        }
    }

    // --- Phase 7: Module system parsing ---

    /// Parse `mod <name>;`
    fn parse_mod_declaration(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Mod, "Expected 'mod'")?;

        let name = match &self.peek().token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "module name",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        };

        self.consume(Token::Semicolon, "Expected ';' after mod declaration")?;

        Ok(Statement::ModDecl {
            name,
            is_public: false,
        })
    }

    /// Parse `use <path>::<item>;` or `use <path>::<item> as <alias>;`
    fn parse_use_import(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Use, "Expected 'use'")?;

        let mut path = Vec::new();

        // Parse first segment
        match &self.peek().token {
            Token::Identifier(n) => {
                path.push(n.clone());
                self.advance();
            }
            _ => {
                return Err(CompilerError::unexpected_token(
                    "module path",
                    &format!("{:?}", self.peek().token),
                    self.peek().location.clone(),
                ));
            }
        }

        // Parse remaining `::segment` parts
        while self.match_token(&Token::DoubleColon) {
            match &self.peek().token {
                Token::Identifier(n) => {
                    path.push(n.clone());
                    self.advance();
                }
                Token::Multiply => {
                    // Glob import: use foo::*
                    path.push("*".to_string());
                    self.advance();
                    break;
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "path segment or '*'",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            }
        }

        // Optional `as <alias>`
        let alias = if self.match_token(&Token::As) {
            match &self.peek().token {
                Token::Identifier(n) => {
                    let n = n.clone();
                    self.advance();
                    Some(n)
                }
                _ => {
                    return Err(CompilerError::unexpected_token(
                        "alias name",
                        &format!("{:?}", self.peek().token),
                        self.peek().location.clone(),
                    ));
                }
            }
        } else {
            None
        };

        self.consume(Token::Semicolon, "Expected ';' after use statement")?;

        Ok(Statement::UseImport { path, alias })
    }

    /// Parse `pub fn ...` / `pub struct ...` / `pub enum ...` / `pub mod ...`
    fn parse_pub_item(&mut self) -> CompilerResult<Statement> {
        self.consume(Token::Pub, "Expected 'pub'")?;

        match &self.peek().token {
            Token::Fn => self.parse_function_definition(),
            Token::Struct => self.parse_struct_def(),
            Token::Enum => self.parse_enum_def(),
            Token::Trait => self.parse_trait_def(),
            Token::Mod => {
                // pub mod foo;
                self.consume(Token::Mod, "Expected 'mod'")?;
                let name = match &self.peek().token {
                    Token::Identifier(n) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => {
                        return Err(CompilerError::unexpected_token(
                            "module name",
                            &format!("{:?}", self.peek().token),
                            self.peek().location.clone(),
                        ));
                    }
                };
                self.consume(Token::Semicolon, "Expected ';' after pub mod")?;
                Ok(Statement::ModDecl {
                    name,
                    is_public: true,
                })
            }
            _ => Err(CompilerError::unexpected_token(
                "fn, struct, enum, trait, or mod after 'pub'",
                &format!("{:?}", self.peek().token),
                self.peek().location.clone(),
            )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Statement};
    use crate::lexer::tokenize_with_locations;

    #[test]
    fn println_f_string_desugars_to_format_and_arguments() {
        let source = r#"println!(f"hello {name}, {count + 1}");"#;
        let tokens = tokenize_with_locations(source, None);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().expect("parser should succeed");

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string,
                arguments,
            })) => {
                assert_eq!(format_string, "hello {}, {}");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Expression::Identifier(ref s) if s == "name"));
                assert!(matches!(
                    arguments[1],
                    Expression::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
            }
            _ => panic!("expected println expression"),
        }
    }

    #[test]
    fn vec_macro_parses_as_array_literal() {
        let source = "let xs = vec![1, 2, 3];";
        let tokens = tokenize_with_locations(source, None);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().expect("parser should succeed");

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Let {
                value: Some(Expression::ArrayLiteral(elements)),
                ..
            }) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], Expression::IntegerLiteral(1)));
                assert!(matches!(elements[1], Expression::IntegerLiteral(2)));
                assert!(matches!(elements[2], Expression::IntegerLiteral(3)));
            }
            _ => panic!("expected vec![] to parse as array literal"),
        }
    }
}
