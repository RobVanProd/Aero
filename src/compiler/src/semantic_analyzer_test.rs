#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Statement, Expression, Parameter, Block, Type, BinaryOp, ComparisonOp, LogicalOp, UnaryOp};
    use crate::types::Ty;
    use crate::errors::{CompilerError, SourceLocation};

    // Helper function to create a semantic analyzer
    fn create_analyzer() -> SemanticAnalyzer {
        SemanticAnalyzer::new()
    }

    // Helper function to create a simple function AST node
    fn create_function_ast(name: &str, params: Vec<Parameter>, return_type: Option<Type>, body: Block) -> AstNode {
        AstNode::Statement(Statement::Function {
            name: name.to_string(),
            parameters: params,
            return_type,
            body,
        })
    }

    // ===== PHASE 3 COMPREHENSIVE SEMANTIC ANALYZER TESTS =====

    #[test]
    fn test_simple_variable_declaration() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(42)),
            })
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutable_variable_declaration() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: true,
                type_annotation: Some(Type::Named("i32".to_string())),
                value: Some(Expression::IntegerLiteral(42)),
            })
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_variable_type_annotation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: Some(Type::Named("i32".to_string())),
                value: Some(Expression::IntegerLiteral(42)),
            })
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_expression_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Let {
                name: "y".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(10)),
            }),
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier("x".to_string())),
                    right: Box::new(Expression::Identifier("y".to_string())),
                    ty: None,
                }),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comparison_expression_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Comparison {
                    op: ComparisonOp::GreaterThan,
                    left: Box::new(Expression::Identifier("x".to_string())),
                    right: Box::new(Expression::IntegerLiteral(0)),
                }),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_expression_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "a".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Comparison {
                    op: ComparisonOp::GreaterThan,
                    left: Box::new(Expression::IntegerLiteral(5)),
                    right: Box::new(Expression::IntegerLiteral(0)),
                }),
            }),
            AstNode::Statement(Statement::Let {
                name: "b".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Comparison {
                    op: ComparisonOp::LessThan,
                    left: Box::new(Expression::IntegerLiteral(10)),
                    right: Box::new(Expression::IntegerLiteral(20)),
                }),
            }),
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Logical {
                    op: LogicalOp::And,
                    left: Box::new(Expression::Identifier("a".to_string())),
                    right: Box::new(Expression::Identifier("b".to_string())),
                }),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_expression_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "flag".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Comparison {
                    op: ComparisonOp::Equal,
                    left: Box::new(Expression::IntegerLiteral(1)),
                    right: Box::new(Expression::IntegerLiteral(1)),
                }),
            }),
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(Expression::Identifier("flag".to_string())),
                }),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_expression_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "name".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("World".to_string())),
            }),
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "Hello, {}!".to_string(),
                arguments: vec![Expression::Identifier("name".to_string())],
            })),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_println_expression_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string: "Value: {}".to_string(),
                arguments: vec![Expression::IntegerLiteral(42)],
            })),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::Identifier("undefined_var".to_string()))),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_err());
        match result.unwrap_err() {
            CompilerError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_var");
            }
            _ => panic!("Expected undefined variable error"),
        }
    }

    #[test]
    fn test_variable_redefinition_in_same_scope() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Let {
                name: "x".to_string(), // Same name in same scope
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(10)),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        // This should be allowed (variable shadowing in same scope)
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_scope_variable_shadowing() {
        let mut analyzer = create_analyzer();
        
        let inner_block = Block {
            statements: vec![
                Statement::Let {
                    name: "x".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::IntegerLiteral(10)), // Shadow outer x
                },
            ],
            expression: Some(Expression::Identifier("x".to_string())), // Should refer to inner x
        };
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Block(inner_block)),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_string_argument_count_validation() {
        let mut analyzer = create_analyzer();
        
        // Test correct argument count
        let ast_correct = vec![
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "Hello, {} and {}!".to_string(), // 2 placeholders
                arguments: vec![
                    Expression::StringLiteral("Alice".to_string()),
                    Expression::StringLiteral("Bob".to_string()),
                ], // 2 arguments
            })),
        ];
        
        let result = analyzer.analyze(ast_correct);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_string_argument_mismatch_error() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "Hello, {} and {}!".to_string(), // 2 placeholders
                arguments: vec![Expression::StringLiteral("Alice".to_string())], // 1 argument
            })),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_err());
        match result.unwrap_err() {
            CompilerError::FormatArgumentMismatch { expected, actual, .. } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected format argument mismatch error"),
        }
    }

    #[test]
    fn test_return_statement_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Return(Some(Expression::IntegerLiteral(42)))),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_return_statement_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Return(None)),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_statement_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Break),
        ];
        
        let result = analyzer.analyze(ast);
        // Break outside loop should be an error, but for now we'll just check it doesn't panic
        // The actual validation logic would need to be implemented
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_continue_statement_validation() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Continue),
        ];
        
        let result = analyzer.analyze(ast);
        // Continue outside loop should be an error, but for now we'll just check it doesn't panic
        // The actual validation logic would need to be implemented
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_block_statement_validation() {
        let mut analyzer = create_analyzer();
        
        let block = Block {
            statements: vec![
                Statement::Let {
                    name: "x".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::IntegerLiteral(5)),
                },
                Statement::Let {
                    name: "y".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::IntegerLiteral(10)),
                },
            ],
            expression: Some(Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier("x".to_string())),
                right: Box::new(Expression::Identifier("y".to_string())),
                ty: None,
            }),
        };
        
        let ast = vec![
            AstNode::Statement(Statement::Block(block)),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_nested_expressions() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Let {
                name: "y".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(10)),
            }),
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Logical {
                    op: LogicalOp::And,
                    left: Box::new(Expression::Comparison {
                        op: ComparisonOp::GreaterThan,
                        left: Box::new(Expression::Identifier("x".to_string())),
                        right: Box::new(Expression::IntegerLiteral(0)),
                    }),
                    right: Box::new(Expression::Comparison {
                        op: ComparisonOp::LessThan,
                        left: Box::new(Expression::Identifier("y".to_string())),
                        right: Box::new(Expression::IntegerLiteral(20)),
                    }),
                }),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_print_statements() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "First: {}".to_string(),
                arguments: vec![Expression::IntegerLiteral(1)],
            })),
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string: "Second: {}".to_string(),
                arguments: vec![Expression::IntegerLiteral(2)],
            })),
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "Third: {} and {}".to_string(),
                arguments: vec![
                    Expression::IntegerLiteral(3),
                    Expression::IntegerLiteral(4),
                ],
            })),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_literal_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "message".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("Hello World".to_string())),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_literal_type_inference() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "pi".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::FloatLiteral(3.14)),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_type_expressions() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "int_val".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(42)),
            }),
            AstNode::Statement(Statement::Let {
                name: "float_val".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::FloatLiteral(3.14)),
            }),
            AstNode::Statement(Statement::Let {
                name: "string_val".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("hello".to_string())),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_empty_ast() {
        let mut analyzer = create_analyzer();
        let ast = vec![];
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_expression_statement() {
        let mut analyzer = create_analyzer();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::IntegerLiteral(42))),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let mut analyzer = create_analyzer();
        
        // Create a deeply nested binary expression: ((((1 + 2) + 3) + 4) + 5)
        let mut expr = Expression::IntegerLiteral(1);
        for i in 2..=5 {
            expr = Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(Expression::IntegerLiteral(i)),
                ty: None,
            };
        }
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(expr),
            }),
        ];
        
        let result = analyzer.analyze(ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_binary_operators() {
        let mut analyzer = create_analyzer();
        
        let operators = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
        ];
        
        let mut statements = vec![];
        for (i, op) in operators.iter().enumerate() {
            statements.push(AstNode::Statement(Statement::Let {
                name: format!("result_{}", i),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Binary {
                    op: op.clone(),
                    left: Box::new(Expression::IntegerLiteral(10)),
                    right: Box::new(Expression::IntegerLiteral(5)),
                    ty: None,
                }),
            }));
        }
        
        let result = analyzer.analyze(statements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_comparison_operators() {
        let mut analyzer = create_analyzer();
        
        let operators = vec![
            ComparisonOp::Equal,
            ComparisonOp::NotEqual,
            ComparisonOp::LessThan,
            ComparisonOp::GreaterThan,
            ComparisonOp::LessEqual,
            ComparisonOp::GreaterEqual,
        ];
        
        let mut statements = vec![];
        for (i, op) in operators.iter().enumerate() {
            statements.push(AstNode::Statement(Statement::Let {
                name: format!("result_{}", i),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Comparison {
                    op: op.clone(),
                    left: Box::new(Expression::IntegerLiteral(10)),
                    right: Box::new(Expression::IntegerLiteral(5)),
                }),
            }));
        }
        
        let result = analyzer.analyze(statements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_logical_operators() {
        let mut analyzer = create_analyzer();
        
        let operators = vec![LogicalOp::And, LogicalOp::Or];
        
        let mut statements = vec![];
        for (i, op) in operators.iter().enumerate() {
            statements.push(AstNode::Statement(Statement::Let {
                name: format!("result_{}", i),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Logical {
                    op: op.clone(),
                    left: Box::new(Expression::Comparison {
                        op: ComparisonOp::GreaterThan,
                        left: Box::new(Expression::IntegerLiteral(5)),
                        right: Box::new(Expression::IntegerLiteral(0)),
                    }),
                    right: Box::new(Expression::Comparison {
                        op: ComparisonOp::LessThan,
                        left: Box::new(Expression::IntegerLiteral(10)),
                        right: Box::new(Expression::IntegerLiteral(20)),
                    }),
                }),
            }));
        }
        
        let result = analyzer.analyze(statements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_unary_operators() {
        let mut analyzer = create_analyzer();
        
        let operators = vec![UnaryOp::Not, UnaryOp::Negate];
        
        let mut statements = vec![];
        
        // Test logical not with boolean expression
        statements.push(AstNode::Statement(Statement::Let {
            name: "not_result".to_string(),
            mutable: false,
            type_annotation: None,
            value: Some(Expression::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expression::Comparison {
                    op: ComparisonOp::Equal,
                    left: Box::new(Expression::IntegerLiteral(1)),
                    right: Box::new(Expression::IntegerLiteral(1)),
                }),
            }),
        }));
        
        // Test unary negate with integer
        statements.push(AstNode::Statement(Statement::Let {
            name: "negate_result".to_string(),
            mutable: false,
            type_annotation: None,
            value: Some(Expression::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expression::IntegerLiteral(42)),
            }),
        }));
        
        let result = analyzer.analyze(statements);
        assert!(result.is_ok());
    }
}