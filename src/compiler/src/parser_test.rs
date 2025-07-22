#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{tokenize_with_locations, Token, LocatedToken};
    use crate::ast::{AstNode, Statement, Expression, Parameter, Block, Type, BinaryOp, ComparisonOp, LogicalOp, UnaryOp};
    use crate::errors::SourceLocation;

    // Helper function to create a parser from source code
    fn create_parser(source: &str) -> Parser {
        let tokens = tokenize_with_locations(source, None);
        Parser::new(tokens)
    }

    // ===== PHASE 3 COMPREHENSIVE PARSER TESTS =====

    #[test]
    fn test_simple_let_statement() {
        let source = "let x = 5;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Let { name, mutable, type_annotation, value }) => {
                assert_eq!(name, "x");
                assert_eq!(*mutable, false);
                assert!(type_annotation.is_none());
                assert!(matches!(value.as_ref().unwrap(), Expression::IntegerLiteral(5)));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_mutable_variable_declaration() {
        let source = "let mut x: i32 = 5;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { name, mutable, type_annotation, value }) => {
                assert_eq!(name, "x");
                assert_eq!(*mutable, true);
                assert!(type_annotation.is_some());
                match type_annotation.as_ref().unwrap() {
                    Type::Named(type_name) => assert_eq!(type_name, "i32"),
                }
                assert!(matches!(value.as_ref().unwrap(), Expression::IntegerLiteral(5)));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_function_call_parsing() {
        let source = "let result = add(5, 3);";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { name, value, .. }) => {
                assert_eq!(name, "result");
                match value.as_ref().unwrap() {
                    Expression::FunctionCall { name, arguments } => {
                        assert_eq!(name, "add");
                        assert_eq!(arguments.len(), 2);
                        assert!(matches!(arguments[0], Expression::IntegerLiteral(5)));
                        assert!(matches!(arguments[1], Expression::IntegerLiteral(3)));
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_binary_expression_parsing() {
        let source = "let result = x + y;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::Binary { op, left, right, .. } => {
                        assert!(matches!(op, BinaryOp::Add));
                        assert!(matches!(**left, Expression::Identifier(ref s) if s == "x"));
                        assert!(matches!(**right, Expression::Identifier(ref s) if s == "y"));
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_comparison_expression_parsing() {
        let source = "let result = x > 5;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::Comparison { op, left, right } => {
                        assert!(matches!(op, ComparisonOp::GreaterThan));
                        assert!(matches!(**left, Expression::Identifier(ref s) if s == "x"));
                        assert!(matches!(**right, Expression::IntegerLiteral(5)));
                    }
                    _ => panic!("Expected comparison expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_logical_expression_parsing() {
        let source = "let result = x && y;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::Logical { op, left, right } => {
                        assert!(matches!(op, LogicalOp::And));
                        assert!(matches!(**left, Expression::Identifier(ref s) if s == "x"));
                        assert!(matches!(**right, Expression::Identifier(ref s) if s == "y"));
                    }
                    _ => panic!("Expected logical expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_unary_expression_parsing() {
        let source = "let result = !flag;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::Unary { op, operand } => {
                        assert!(matches!(op, UnaryOp::Not));
                        assert!(matches!(**operand, Expression::Identifier(ref s) if s == "flag"));
                    }
                    _ => panic!("Expected unary expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_print_macro_parsing() {
        let source = r#"print!("Hello, {}!", name);"#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Expression(Expression::Print { format_string, arguments })) => {
                assert_eq!(format_string, "Hello, {}!");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0], Expression::Identifier(ref s) if s == "name"));
            }
            _ => panic!("Expected print expression"),
        }
    }

    #[test]
    fn test_println_macro_parsing() {
        let source = r#"println!("Value: {}", x);"#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Expression(Expression::Println { format_string, arguments })) => {
                assert_eq!(format_string, "Value: {}");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0], Expression::Identifier(ref s) if s == "x"));
            }
            _ => panic!("Expected println expression"),
        }
    }

    #[test]
    fn test_return_statement_parsing() {
        let source = "return x;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Return(Some(expression))) => {
                assert!(matches!(expression, Expression::Identifier(ref s) if s == "x"));
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_empty_return_statement() {
        let source = "return;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Return(None)) => {
                // Expected empty return
            }
            _ => panic!("Expected empty return statement"),
        }
    }

    #[test]
    fn test_break_statement() {
        let source = "break;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Break) => {
                // Expected break statement
            }
            _ => panic!("Expected break statement"),
        }
    }

    #[test]
    fn test_continue_statement() {
        let source = "continue;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Continue) => {
                // Expected continue statement
            }
            _ => panic!("Expected continue statement"),
        }
    }

    #[test]
    fn test_block_statement() {
        let source = "{ let x = 5; }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Block(block)) => {
                assert_eq!(block.statements.len(), 1);
                assert!(matches!(block.statements[0], Statement::Let { .. }));
                assert!(block.expression.is_none());
            }
            _ => panic!("Expected block statement"),
        }
    }

    #[test]
    fn test_multiple_statements() {
        let source = "let x = 5; let y = 10;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 2);
        assert!(matches!(ast[0], AstNode::Statement(Statement::Let { .. })));
        assert!(matches!(ast[1], AstNode::Statement(Statement::Let { .. })));
    }

    #[test]
    fn test_nested_expressions() {
        let source = "let result = (x + y) * z;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::Binary { op, left, right, .. } => {
                        assert!(matches!(op, BinaryOp::Multiply));
                        // Left should be (x + y)
                        match left.as_ref() {
                            Expression::Binary { op, .. } => {
                                assert!(matches!(op, BinaryOp::Add));
                            }
                            _ => panic!("Expected binary expression on left"),
                        }
                        // Right should be z
                        assert!(matches!(**right, Expression::Identifier(ref s) if s == "z"));
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_empty_input() {
        let source = "";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let source = "   \n\t  \r\n  ";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 0);
    }

    #[test]
    fn test_single_expression() {
        let source = "42;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Expression(Expression::IntegerLiteral(42))) => {
                // Expected
            }
            _ => panic!("Expected integer literal expression"),
        }
    }

    #[test]
    fn test_error_recovery() {
        // Test parser error recovery with invalid syntax
        let source = "let x = ; let y = 5;"; // Missing value in first let
        let mut parser = create_parser(source);
        let result = parser.parse();
        
        // Should return an error but not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_string_literal_parsing() {
        let source = r#"let message = "Hello World";"#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                assert!(matches!(value.as_ref().unwrap(), Expression::StringLiteral(ref s) if s == "Hello World"));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_float_literal_parsing() {
        let source = "let pi = 3.14;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                assert!(matches!(value.as_ref().unwrap(), Expression::FloatLiteral(f) if (*f - 3.14).abs() < f64::EPSILON));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_identifier_parsing() {
        let source = "let result = variable_name;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                assert!(matches!(value.as_ref().unwrap(), Expression::Identifier(ref s) if s == "variable_name"));
            }
            _ => panic!("Expected let statement"),
        }
    }
}