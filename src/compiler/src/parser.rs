use crate::ast::{AstNode, Expression, Statement};
use crate::lexer;
use crate::semantic_analyzer;
use crate::ir_generator;
use crate::code_generator;

pub fn parse(tokens: Vec<String>) -> Vec<AstNode> {
    let mut ast_nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "keyword:let" => {
                // Expecting 'let identifier = expression;'
                i += 1; // consume 'let'
                let name_token = tokens[i].clone(); // identifier name token
                let name = name_token.split(':').nth(1).unwrap().to_string(); // Extract just the name
                i += 1;
                if tokens[i] != "=" {
                    eprintln!("Expected = after identifier in let statement");
                    break; // Use break instead of return to continue parsing other statements
                }
                i += 1; // consume '='

                let mut expr_tokens = Vec::new();
                while i < tokens.len() && tokens[i] != ";" {
                    expr_tokens.push(tokens[i].clone());
                    i += 1;
                }

                let expr = parse_expression(expr_tokens);

                if i >= tokens.len() || tokens[i] != ";" {
                    eprintln!("Expected ; after expression in let statement");
                    break; // Use break instead of return
                }
                i += 1; // consume ';'

                ast_nodes.push(AstNode::Statement(Statement::Let { name, value: expr }));
            }
            _ => {
                eprintln!("Unexpected token at top level: {}", tokens[i]);
                i += 1;
            }
        }
    }

    ast_nodes
}

fn parse_expression(expr_tokens: Vec<String>) -> Expression {
    // Simple expression parsing for now (only numbers, identifiers, and binary ops)
    // This needs to be replaced with a proper Pratt parser or similar for operator precedence
    let mut i = 0;
    let mut lhs = match expr_tokens[i].split(':').next().unwrap() {
        "integer_literal" => Expression::Number(expr_tokens[i].split(':').nth(1).unwrap().parse().unwrap()),
        "float_literal" => Expression::Float(expr_tokens[i].split(':').nth(1).unwrap().parse().unwrap()),
        "identifier" => Expression::Identifier(expr_tokens[i].split(':').nth(1).unwrap().to_string()),
        _ => panic!("Unsupported expression token: {}", expr_tokens[i]),
    };
    i += 1;

    while i < expr_tokens.len() {
        let op = expr_tokens[i].clone();
        i += 1;
        let rhs = match expr_tokens[i].split(':').next().unwrap() {
            "integer_literal" => Expression::Number(expr_tokens[i].split(':').nth(1).unwrap().parse().unwrap()),
            "float_literal" => Expression::Float(expr_tokens[i].split(':').nth(1).unwrap().parse().unwrap()),
            "identifier" => Expression::Identifier(expr_tokens[i].split(':').nth(1).unwrap().to_string()),
            _ => panic!("Unsupported expression token: {}", expr_tokens[i]),
        };
        i += 1;
        lhs = Expression::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    lhs
}


