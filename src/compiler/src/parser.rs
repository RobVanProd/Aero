use crate::ast::{AstNode, Expression, Statement};
use crate::lexer::{self, Token};

pub fn parse(tokens: Vec<Token>) -> Vec<AstNode> {
    let mut ast_nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
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
        Token::Identifier(name) => Expression::Identifier(name.clone()),
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


