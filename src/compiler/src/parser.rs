use crate::ast::{AstNode, Expression, Statement, Parameter, Block, Type};
use crate::lexer::{self, Token};

pub fn parse(tokens: Vec<Token>) -> Vec<AstNode> {
    let mut ast_nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Fn => {
                // Parse function definition
                match parse_function_definition(&tokens, &mut i) {
                    Ok(func_stmt) => ast_nodes.push(AstNode::Statement(func_stmt)),
                    Err(err) => {
                        eprintln!("Error parsing function: {}", err);
                        break;
                    }
                }
            }
            Token::Let => {
                // Expecting 'let identifier = expression;'
                i += 1; // consume 'let'
                
                let name = if let Token::Identifier(name) = &tokens[i] {
                    name.clone()
                } else {
                    eprintln!("Expected identifier after let");
                    break;
                };
                i += 1;
                
                if tokens[i] != Token::Assign {
                    eprintln!("Expected = after identifier in let statement");
                    break;
                }
                i += 1; // consume '='

                let mut expr_tokens = Vec::new();
                while i < tokens.len() && tokens[i] != Token::Semicolon {
                    expr_tokens.push(tokens[i].clone());
                    i += 1;
                }

                let expr = parse_expression(expr_tokens);

                if i >= tokens.len() || tokens[i] != Token::Semicolon {
                    eprintln!("Expected ; after expression in let statement");
                    break;
                }
                i += 1; // consume ';'

                ast_nodes.push(AstNode::Statement(Statement::Let { name, value: expr }));
            }
            Token::Return => {
                // Expecting 'return expression;'
                i += 1; // consume 'return'

                let mut expr_tokens = Vec::new();
                while i < tokens.len() && tokens[i] != Token::Semicolon {
                    expr_tokens.push(tokens[i].clone());
                    i += 1;
                }

                let expr = parse_expression(expr_tokens);

                if i >= tokens.len() || tokens[i] != Token::Semicolon {
                    eprintln!("Expected ; after expression in return statement");
                    break;
                }
                i += 1; // consume ';'

                ast_nodes.push(AstNode::Statement(Statement::Return(expr)));
            }
            Token::If => {
                // Parse if statement
                match parse_if_statement(&tokens, &mut i) {
                    Ok(if_stmt) => ast_nodes.push(AstNode::Statement(if_stmt)),
                    Err(err) => {
                        eprintln!("Error parsing if statement: {}", err);
                        break;
                    }
                }
            }
            Token::While => {
                // Parse while loop
                match parse_while_statement(&tokens, &mut i) {
                    Ok(while_stmt) => ast_nodes.push(AstNode::Statement(while_stmt)),
                    Err(err) => {
                        eprintln!("Error parsing while statement: {}", err);
                        break;
                    }
                }
            }
            Token::For => {
                // Parse for loop
                match parse_for_statement(&tokens, &mut i) {
                    Ok(for_stmt) => ast_nodes.push(AstNode::Statement(for_stmt)),
                    Err(err) => {
                        eprintln!("Error parsing for statement: {}", err);
                        break;
                    }
                }
            }
            Token::Loop => {
                // Parse infinite loop
                match parse_loop_statement(&tokens, &mut i) {
                    Ok(loop_stmt) => ast_nodes.push(AstNode::Statement(loop_stmt)),
                    Err(err) => {
                        eprintln!("Error parsing loop statement: {}", err);
                        break;
                    }
                }
            }
            Token::Break => {
                i += 1; // consume 'break'
                if i >= tokens.len() || tokens[i] != Token::Semicolon {
                    eprintln!("Expected ; after break statement");
                    break;
                }
                i += 1; // consume ';'
                ast_nodes.push(AstNode::Statement(Statement::Break));
            }
            Token::Continue => {
                i += 1; // consume 'continue'
                if i >= tokens.len() || tokens[i] != Token::Semicolon {
                    eprintln!("Expected ; after continue statement");
                    break;
                }
                i += 1; // consume ';'
                ast_nodes.push(AstNode::Statement(Statement::Continue));
            }
            Token::PrintMacro | Token::PrintlnMacro => {
                // Parse I/O macro as expression statement
                match parse_io_macro(&tokens, &mut i) {
                    Ok(expr) => {
                        // Expect semicolon after I/O macro
                        if i >= tokens.len() || tokens[i] != Token::Semicolon {
                            eprintln!("Expected ; after I/O macro");
                            break;
                        }
                        i += 1; // consume ';'
                        ast_nodes.push(AstNode::Expression(expr));
                    }
                    Err(err) => {
                        eprintln!("Error parsing I/O macro: {}", err);
                        break;
                    }
                }
            }
            Token::Eof => break,
            _ => {
                eprintln!("Unexpected token at top level: {:?}", tokens[i]);
                i += 1;
            }
        }
    }

    ast_nodes
}

fn parse_expression(expr_tokens: Vec<Token>) -> Expression {
    // Simple expression parsing for now (only numbers, identifiers, and binary ops)
    // This needs to be replaced with a proper Pratt parser or similar for operator precedence
    let mut i = 0;
    let mut lhs = match &expr_tokens[i] {
        Token::IntegerLiteral(val) => Expression::Number(*val),
        Token::FloatLiteral(val) => Expression::Float(*val),
        Token::LogicalNot => {
            // Parse unary not operator
            i += 1; // consume '!'
            let operand = match &expr_tokens[i] {
                Token::IntegerLiteral(val) => Expression::Number(*val),
                Token::FloatLiteral(val) => Expression::Float(*val),
                Token::Identifier(name) => Expression::Identifier(name.clone()),
                _ => panic!("Unsupported operand for unary not: {:?}", expr_tokens[i]),
            };
            Expression::Unary {
                op: crate::ast::UnaryOp::Not,
                operand: Box::new(operand),
            }
        }
        Token::Minus => {
            // Parse unary minus operator
            i += 1; // consume '-'
            let operand = match &expr_tokens[i] {
                Token::IntegerLiteral(val) => Expression::Number(*val),
                Token::FloatLiteral(val) => Expression::Float(*val),
                Token::Identifier(name) => Expression::Identifier(name.clone()),
                _ => panic!("Unsupported operand for unary minus: {:?}", expr_tokens[i]),
            };
            Expression::Unary {
                op: crate::ast::UnaryOp::Minus,
                operand: Box::new(operand),
            }
        }
        Token::Identifier(name) => {
            // Check if this is a function call
            if i + 1 < expr_tokens.len() && expr_tokens[i + 1] == Token::LeftParen {
                // Parse function call
                let func_name = name.clone();
                i += 2; // skip identifier and '('
                
                let mut arguments = Vec::new();
                let mut arg_tokens = Vec::new();
                let mut paren_count = 0;
                
                while i < expr_tokens.len() {
                    match &expr_tokens[i] {
                        Token::LeftParen => {
                            paren_count += 1;
                            arg_tokens.push(expr_tokens[i].clone());
                        }
                        Token::RightParen if paren_count > 0 => {
                            paren_count -= 1;
                            arg_tokens.push(expr_tokens[i].clone());
                        }
                        Token::RightParen if paren_count == 0 => {
                            // End of function call
                            if !arg_tokens.is_empty() {
                                arguments.push(parse_expression(arg_tokens));
                            }
                            i += 1; // consume ')'
                            break;
                        }
                        Token::Comma if paren_count == 0 => {
                            // End of current argument
                            if !arg_tokens.is_empty() {
                                arguments.push(parse_expression(arg_tokens));
                                arg_tokens = Vec::new();
                            }
                        }
                        _ => {
                            arg_tokens.push(expr_tokens[i].clone());
                        }
                    }
                    i += 1;
                }
                
                Expression::FunctionCall {
                    name: func_name,
                    arguments,
                }
            } else {
                Expression::Identifier(name.clone())
            }
        }
        _ => panic!("Unsupported expression token: {:?}", expr_tokens[i]),
    };
    i += 1;

    while i < expr_tokens.len() {
        let op = match &expr_tokens[i] {
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Multiply => "*".to_string(),
            Token::Divide => "/".to_string(),
            Token::Modulo => "%".to_string(),
            Token::Equal => "==".to_string(),
            Token::NotEqual => "!=".to_string(),
            Token::LessThan => "<".to_string(),
            Token::GreaterThan => ">".to_string(),
            Token::LessEqual => "<=".to_string(),
            Token::GreaterEqual => ">=".to_string(),
            Token::LogicalAnd => "&&".to_string(),
            Token::LogicalOr => "||".to_string(),
            Token::Dot => {
                // Check if this is a range operator (..)
                if i + 1 < expr_tokens.len() && expr_tokens[i + 1] == Token::Dot {
                    i += 1; // consume the second dot
                    "..".to_string()
                } else {
                    panic!("Single dot not supported as operator");
                }
            },
            _ => panic!("Unsupported operator token: {:?}", expr_tokens[i]),
        };
        i += 1;
        let rhs = match &expr_tokens[i] {
            Token::IntegerLiteral(val) => Expression::Number(*val),
            Token::FloatLiteral(val) => Expression::Float(*val),
            Token::Identifier(name) => Expression::Identifier(name.clone()),
            _ => panic!("Unsupported expression token: {:?}", expr_tokens[i]),
        };
        i += 1;
        lhs = Expression::Binary { 
            op, 
            lhs: Box::new(lhs), 
            rhs: Box::new(rhs),
            ty: None, // Will be filled in by semantic analysis
        };
    }
    lhs
}

fn parse_function_definition(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    // Expecting: fn function_name(param1: type1, param2: type2) -> return_type { body }
    *i += 1; // consume 'fn'
    
    // Parse function name
    let name = match &tokens[*i] {
        Token::Identifier(name) => name.clone(),
        _ => return Err("Expected function name after 'fn'".to_string()),
    };
    *i += 1;
    
    // Parse parameter list
    if tokens[*i] != Token::LeftParen {
        return Err("Expected '(' after function name".to_string());
    }
    *i += 1; // consume '('
    
    let parameters = parse_parameter_list(tokens, i)?;
    
    if tokens[*i] != Token::RightParen {
        return Err("Expected ')' after parameter list".to_string());
    }
    *i += 1; // consume ')'
    
    // Parse optional return type
    let return_type = if tokens[*i] == Token::Arrow {
        *i += 1; // consume '->'
        Some(parse_type(tokens, i)?)
    } else {
        None
    };
    
    // Parse function body
    let body = parse_block(tokens, i)?;
    
    Ok(Statement::Function {
        name,
        parameters,
        return_type,
        body,
    })
}

fn parse_parameter_list(tokens: &[Token], i: &mut usize) -> Result<Vec<Parameter>, String> {
    let mut parameters = Vec::new();
    
    // Handle empty parameter list
    if tokens[*i] == Token::RightParen {
        return Ok(parameters);
    }
    
    loop {
        // Parse parameter name
        let name = match &tokens[*i] {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected parameter name".to_string()),
        };
        *i += 1;
        
        // Expect ':'
        if tokens[*i] != Token::Colon {
            return Err("Expected ':' after parameter name".to_string());
        }
        *i += 1; // consume ':'
        
        // Parse parameter type
        let param_type = parse_type(tokens, i)?;
        
        parameters.push(Parameter { name, param_type });
        
        // Check for more parameters
        if tokens[*i] == Token::Comma {
            *i += 1; // consume ','
            continue;
        } else {
            break;
        }
    }
    
    Ok(parameters)
}

fn parse_type(tokens: &[Token], i: &mut usize) -> Result<Type, String> {
    match &tokens[*i] {
        Token::Identifier(type_name) => {
            *i += 1;
            Ok(Type { name: type_name.clone() })
        }
        _ => Err("Expected type name".to_string()),
    }
}

fn parse_block(tokens: &[Token], i: &mut usize) -> Result<Block, String> {
    if tokens[*i] != Token::LeftBrace {
        return Err("Expected '{' to start block".to_string());
    }
    *i += 1; // consume '{'
    
    let mut statements = Vec::new();
    let mut expression = None;
    
    while *i < tokens.len() && tokens[*i] != Token::RightBrace {
        match &tokens[*i] {
            Token::Let => {
                statements.push(parse_let_statement(tokens, i)?);
            }
            Token::Return => {
                statements.push(parse_return_statement(tokens, i)?);
            }
            Token::If => {
                statements.push(parse_if_statement(tokens, i)?);
            }
            Token::While => {
                statements.push(parse_while_statement(tokens, i)?);
            }
            Token::For => {
                statements.push(parse_for_statement(tokens, i)?);
            }
            Token::Loop => {
                statements.push(parse_loop_statement(tokens, i)?);
            }
            Token::Break => {
                *i += 1; // consume 'break'
                if *i >= tokens.len() || tokens[*i] != Token::Semicolon {
                    return Err("Expected ';' after break statement".to_string());
                }
                *i += 1; // consume ';'
                statements.push(Statement::Break);
            }
            Token::Continue => {
                *i += 1; // consume 'continue'
                if *i >= tokens.len() || tokens[*i] != Token::Semicolon {
                    return Err("Expected ';' after continue statement".to_string());
                }
                *i += 1; // consume ';'
                statements.push(Statement::Continue);
            }
            Token::PrintMacro | Token::PrintlnMacro => {
                // Parse I/O macro as expression statement
                match parse_io_macro(tokens, i) {
                    Ok(expr) => {
                        // Expect semicolon after I/O macro
                        if *i >= tokens.len() || tokens[*i] != Token::Semicolon {
                            return Err("Expected ';' after I/O macro".to_string());
                        }
                        *i += 1; // consume ';'
                        // For now, treat I/O macros as return statements to add them to the AST
                        statements.push(Statement::Return(expr));
                    }
                    Err(err) => {
                        return Err(format!("Error parsing I/O macro: {}", err));
                    }
                }
            }
            Token::Eof => {
                return Err("Unexpected end of file in block".to_string());
            }
            _ => {
                // Try to parse as expression statement
                let expr_start = *i;
                let mut expr_tokens = Vec::new();
                let mut brace_count = 0;
                
                while *i < tokens.len() {
                    match &tokens[*i] {
                        Token::LeftBrace => brace_count += 1,
                        Token::RightBrace if brace_count > 0 => brace_count -= 1,
                        Token::RightBrace if brace_count == 0 => break,
                        Token::Semicolon if brace_count == 0 => {
                            *i += 1; // consume ';'
                            break;
                        }
                        _ => {}
                    }
                    expr_tokens.push(tokens[*i].clone());
                    *i += 1;
                }
                
                if !expr_tokens.is_empty() {
                    let expr = parse_expression(expr_tokens);
                    // If this is the last statement without semicolon, it's the block expression
                    if *i < tokens.len() && tokens[*i] == Token::RightBrace && expression.is_none() {
                        expression = Some(expr);
                    } else {
                        // It's an expression statement (with semicolon)
                        statements.push(Statement::Return(expr)); // Treat as expression statement for now
                    }
                }
            }
        }
    }
    
    if *i >= tokens.len() || tokens[*i] != Token::RightBrace {
        return Err("Expected '}' to close block".to_string());
    }
    *i += 1; // consume '}'
    
    Ok(Block { statements, expression })
}

fn parse_let_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'let'
    
    let name = match &tokens[*i] {
        Token::Identifier(name) => name.clone(),
        _ => return Err("Expected identifier after 'let'".to_string()),
    };
    *i += 1;
    
    if tokens[*i] != Token::Assign {
        return Err("Expected '=' after identifier in let statement".to_string());
    }
    *i += 1; // consume '='
    
    let mut expr_tokens = Vec::new();
    while *i < tokens.len() && tokens[*i] != Token::Semicolon {
        expr_tokens.push(tokens[*i].clone());
        *i += 1;
    }
    
    let expr = parse_expression(expr_tokens);
    
    if *i >= tokens.len() || tokens[*i] != Token::Semicolon {
        return Err("Expected ';' after expression in let statement".to_string());
    }
    *i += 1; // consume ';'
    
    Ok(Statement::Let { name, value: expr })
}

fn parse_return_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'return'
    
    let mut expr_tokens = Vec::new();
    while *i < tokens.len() && tokens[*i] != Token::Semicolon {
        expr_tokens.push(tokens[*i].clone());
        *i += 1;
    }
    
    let expr = parse_expression(expr_tokens);
    
    if *i >= tokens.len() || tokens[*i] != Token::Semicolon {
        return Err("Expected ';' after expression in return statement".to_string());
    }
    *i += 1; // consume ';'
    
    Ok(Statement::Return(expr))
}

fn parse_if_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'if'
    
    // Parse condition expression
    let mut condition_tokens = Vec::new();
    while *i < tokens.len() && tokens[*i] != Token::LeftBrace {
        condition_tokens.push(tokens[*i].clone());
        *i += 1;
    }
    
    if condition_tokens.is_empty() {
        return Err("Expected condition after 'if'".to_string());
    }
    
    let condition = parse_expression(condition_tokens);
    
    // Parse then block
    let then_block = parse_block(tokens, i)?;
    
    // Check for else clause
    let else_block = if *i < tokens.len() && tokens[*i] == Token::Else {
        *i += 1; // consume 'else'
        
        if tokens[*i] == Token::If {
            // else if - parse as nested if statement
            Some(Box::new(parse_if_statement(tokens, i)?))
        } else {
            // else block - we need to create a wrapper for the else block
            // For now, we'll represent it as a nested if with a true condition
            let else_body = parse_block(tokens, i)?;
            // We need to handle this differently - let's create a simple wrapper
            // This is a temporary solution until we improve the AST structure
            Some(Box::new(Statement::If {
                condition: Expression::Number(1), // Always true for else block
                then_block: else_body,
                else_block: None,
            }))
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

fn parse_while_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'while'
    
    // Parse condition expression
    let mut condition_tokens = Vec::new();
    while *i < tokens.len() && tokens[*i] != Token::LeftBrace {
        condition_tokens.push(tokens[*i].clone());
        *i += 1;
    }
    
    if condition_tokens.is_empty() {
        return Err("Expected condition after 'while'".to_string());
    }
    
    let condition = parse_expression(condition_tokens);
    
    // Parse body block
    let body = parse_block(tokens, i)?;
    
    Ok(Statement::While { condition, body })
}

fn parse_for_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'for'
    
    // Parse variable name
    let variable = match &tokens[*i] {
        Token::Identifier(name) => name.clone(),
        _ => return Err("Expected variable name after 'for'".to_string()),
    };
    *i += 1;
    
    // Expect 'in'
    if tokens[*i] != Token::In {
        return Err("Expected 'in' after for variable".to_string());
    }
    *i += 1; // consume 'in'
    
    // Parse iterable expression
    let mut iterable_tokens = Vec::new();
    while *i < tokens.len() && tokens[*i] != Token::LeftBrace {
        iterable_tokens.push(tokens[*i].clone());
        *i += 1;
    }
    
    if iterable_tokens.is_empty() {
        return Err("Expected iterable expression after 'in'".to_string());
    }
    
    let iterable = parse_expression(iterable_tokens);
    
    // Parse body block
    let body = parse_block(tokens, i)?;
    
    Ok(Statement::For {
        variable,
        iterable,
        body,
    })
}

fn parse_loop_statement(tokens: &[Token], i: &mut usize) -> Result<Statement, String> {
    *i += 1; // consume 'loop'
    
    // Parse body block
    let body = parse_block(tokens, i)?;
    
    Ok(Statement::Loop { body })
}

fn parse_io_macro(tokens: &[Token], i: &mut usize) -> Result<Expression, String> {
    let is_println = matches!(tokens[*i], Token::PrintlnMacro);
    *i += 1; // consume 'print!' or 'println!'
    
    // Expect '('
    if tokens[*i] != Token::LeftParen {
        return Err("Expected '(' after I/O macro".to_string());
    }
    *i += 1; // consume '('
    
    // Parse format string
    let format_string = match &tokens[*i] {
        Token::Identifier(s) if s.starts_with('"') && s.ends_with('"') => {
            // Remove quotes from the string
            s[1..s.len()-1].to_string()
        }
        _ => return Err("Expected format string after '('".to_string()),
    };
    *i += 1;
    
    // Parse arguments
    let mut arguments = Vec::new();
    
    while *i < tokens.len() && tokens[*i] != Token::RightParen {
        if tokens[*i] == Token::Comma {
            *i += 1; // consume ','
            continue;
        }
        
        // Parse argument expression
        let mut arg_tokens = Vec::new();
        let mut paren_count = 0;
        
        while *i < tokens.len() {
            match &tokens[*i] {
                Token::LeftParen => {
                    paren_count += 1;
                    arg_tokens.push(tokens[*i].clone());
                }
                Token::RightParen if paren_count > 0 => {
                    paren_count -= 1;
                    arg_tokens.push(tokens[*i].clone());
                }
                Token::RightParen if paren_count == 0 => {
                    // End of macro call
                    break;
                }
                Token::Comma if paren_count == 0 => {
                    // End of current argument
                    break;
                }
                _ => {
                    arg_tokens.push(tokens[*i].clone());
                }
            }
            *i += 1;
        }
        
        if !arg_tokens.is_empty() {
            arguments.push(parse_expression(arg_tokens));
        }
    }
    
    if tokens[*i] != Token::RightParen {
        return Err("Expected ')' to close I/O macro".to_string());
    }
    *i += 1; // consume ')'
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple_function() {
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, parameters, return_type, body }) => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name, "a");
                assert_eq!(parameters[0].param_type.name, "i32");
                assert_eq!(parameters[1].name, "b");
                assert_eq!(parameters[1].param_type.name, "i32");
                assert!(return_type.is_some());
                assert_eq!(return_type.as_ref().unwrap().name, "i32");
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_no_params() {
        let source = "fn main() { let x = 5; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, parameters, return_type, body }) => {
                assert_eq!(name, "main");
                assert_eq!(parameters.len(), 0);
                assert!(return_type.is_none());
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_no_return_type() {
        let source = "fn greet(name: String) { let msg = name; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, parameters, return_type, body }) => {
                assert_eq!(name, "greet");
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].name, "name");
                assert_eq!(parameters[0].param_type.name, "String");
                assert!(return_type.is_none());
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_with_expression_body() {
        let source = "fn double(x: i32) -> i32 { x * 2 }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, parameters, return_type, body }) => {
                assert_eq!(name, "double");
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].name, "x");
                assert_eq!(parameters[0].param_type.name, "i32");
                assert!(return_type.is_some());
                assert_eq!(return_type.as_ref().unwrap().name, "i32");
                // Should have expression in body
                assert!(body.expression.is_some());
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b; } fn main() { let x = 5; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 2);
        
        // First function
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, .. }) => {
                assert_eq!(name, "add");
            }
            _ => panic!("Expected function statement"),
        }
        
        // Second function
        match &ast_nodes[1] {
            AstNode::Statement(Statement::Function { name, .. }) => {
                assert_eq!(name, "main");
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let source = "fn main() { let result = add(5, 3); }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, body, .. }) => {
                assert_eq!(name, "main");
                assert_eq!(body.statements.len(), 1);
                
                // Check the let statement contains a function call
                match &body.statements[0] {
                    Statement::Let { name, value } => {
                        assert_eq!(name, "result");
                        match value {
                            Expression::FunctionCall { name, arguments } => {
                                assert_eq!(name, "add");
                                assert_eq!(arguments.len(), 2);
                                assert!(matches!(arguments[0], Expression::Number(5)));
                                assert!(matches!(arguments[1], Expression::Number(3)));
                            }
                            _ => panic!("Expected function call expression"),
                        }
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let source = "fn main() { let x = getValue(); }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { body, .. }) => {
                match &body.statements[0] {
                    Statement::Let { value, .. } => {
                        match value {
                            Expression::FunctionCall { name, arguments } => {
                                assert_eq!(name, "getValue");
                                assert_eq!(arguments.len(), 0);
                            }
                            _ => panic!("Expected function call expression"),
                        }
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if x > 5 { let y = 10; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::If { condition, then_block, else_block }) => {
                // Check condition is a binary expression
                assert!(matches!(condition, Expression::Binary { .. }));
                assert_eq!(then_block.statements.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let source = "if x > 5 { let y = 10; } else { let y = 0; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::If { condition, then_block, else_block }) => {
                assert!(matches!(condition, Expression::Binary { .. }));
                assert_eq!(then_block.statements.len(), 1);
                assert!(else_block.is_some());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let source = "while i < 10 { let i = i + 1; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::While { condition, body }) => {
                assert!(matches!(condition, Expression::Binary { .. }));
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_parse_for_statement() {
        let source = "for i in 0..10 { let x = i; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::For { variable, iterable, body }) => {
                assert_eq!(variable, "i");
                assert!(matches!(iterable, Expression::Binary { .. })); // Range expression
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_parse_loop_statement() {
        let source = "loop { break; }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Loop { body }) => {
                assert_eq!(body.statements.len(), 1);
                assert!(matches!(body.statements[0], Statement::Break));
            }
            _ => panic!("Expected loop statement"),
        }
    }

    #[test]
    fn test_parse_break_continue() {
        let source = "break; continue;";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 2);
        assert!(matches!(ast_nodes[0], AstNode::Statement(Statement::Break)));
        assert!(matches!(ast_nodes[1], AstNode::Statement(Statement::Continue)));
    }

    #[test]
    fn test_parse_nested_control_flow() {
        let source = "while running { for i in 0..5 { if i == 3 { break; } } }";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::While { body, .. }) => {
                assert_eq!(body.statements.len(), 1);
                match &body.statements[0] {
                    Statement::For { body: for_body, .. } => {
                        assert_eq!(for_body.statements.len(), 1);
                        assert!(matches!(for_body.statements[0], Statement::If { .. }));
                    }
                    _ => panic!("Expected for statement inside while"),
                }
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_parse_print_macro() {
        let source = r#"print!("Hello, World!");"#;
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Expression(Expression::Print { format_string, arguments }) => {
                assert_eq!(format_string, "Hello, World!");
                assert_eq!(arguments.len(), 0);
            }
            _ => panic!("Expected print expression"),
        }
    }

    #[test]
    fn test_parse_println_macro() {
        let source = r#"println!("Hello, World!");"#;
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Expression(Expression::Println { format_string, arguments }) => {
                assert_eq!(format_string, "Hello, World!");
                assert_eq!(arguments.len(), 0);
            }
            _ => panic!("Expected println expression"),
        }
    }

    #[test]
    fn test_parse_print_with_arguments() {
        let source = r#"println!("Value: {}", x + 5);"#;
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Expression(Expression::Println { format_string, arguments }) => {
                assert_eq!(format_string, "Value: {}");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0], Expression::Binary { .. }));
            }
            _ => panic!("Expected println expression"),
        }
    }

    #[test]
    fn test_parse_print_multiple_arguments() {
        let source = r#"println!("{} + {} = {}", a, b, a + b);"#;
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Expression(Expression::Println { format_string, arguments }) => {
                assert_eq!(format_string, "{} + {} = {}");
                assert_eq!(arguments.len(), 3);
                assert!(matches!(arguments[0], Expression::Identifier(_)));
                assert!(matches!(arguments[1], Expression::Identifier(_)));
                assert!(matches!(arguments[2], Expression::Binary { .. }));
            }
            _ => panic!("Expected println expression"),
        }
    }

    #[test]
    fn test_parse_function_with_io() {
        let source = r#"fn main() { println!("Hello from function!"); }"#;
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Function { name, body, .. }) => {
                assert_eq!(name, "main");
                assert_eq!(body.statements.len(), 1);
                // I/O macro is treated as a return statement for now
                match &body.statements[0] {
                    Statement::Return(Expression::Println { format_string, .. }) => {
                        assert_eq!(format_string, "Hello from function!");
                    }
                    _ => panic!("Expected println in function body"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let source = "let x = !true;";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Let { name, value }) => {
                assert_eq!(name, "x");
                match value {
                    Expression::Unary { op, operand } => {
                        assert!(matches!(op, crate::ast::UnaryOp::Not));
                        assert!(matches!(**operand, Expression::Identifier(_)));
                    }
                    _ => panic!("Expected unary not expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_unary_minus() {
        let source = "let x = -42;";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Let { name, value }) => {
                assert_eq!(name, "x");
                match value {
                    Expression::Unary { op, operand } => {
                        assert!(matches!(op, crate::ast::UnaryOp::Minus));
                        assert!(matches!(**operand, Expression::Number(42)));
                    }
                    _ => panic!("Expected unary minus expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_complex_expression_with_unary() {
        let source = "let result = !flag && x > -5;";
        let tokens = tokenize(source);
        let ast_nodes = parse(tokens);
        
        assert_eq!(ast_nodes.len(), 1);
        match &ast_nodes[0] {
            AstNode::Statement(Statement::Let { name, value }) => {
                assert_eq!(name, "result");
                // Should parse as a complex binary expression with unary operators
                assert!(matches!(value, Expression::Binary { .. }));
            }
            _ => panic!("Expected let statement"),
        }
    }
}
