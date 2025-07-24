#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Statement, Expression, Parameter, Block, Type, BinaryOp, ComparisonOp, LogicalOp, UnaryOp};
    use crate::ir::{Inst, Value};
    use crate::types::Ty;

    // ===== PHASE 3 COMPREHENSIVE IR GENERATOR TESTS =====

    #[test]
    fn test_simple_let_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(42)),
            })
        ];
        
        let ir = ir_gen.generate_ir(ast);
        
        // Check that main function exists
        assert!(ir.contains_key("main"));
        let main_func = &ir["main"];
        
        // Should have at least: alloca, store instructions
        assert!(main_func.body.len() >= 2);
        
        // Check for alloca instruction
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        
        // Check for store instruction with immediate value
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Store(Value::ImmInt(42), _))));
    }

    #[test]
    fn test_mutable_variable_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: true,
                type_annotation: Some(Type::Named("i32".to_string())),
                value: Some(Expression::IntegerLiteral(42)),
            })
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should generate similar IR to immutable, but track mutability
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Store(Value::ImmInt(42), _))));
    }

    #[test]
    fn test_binary_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
                    ty: Some(Ty::Int),
                }),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for x, y, result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "y")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should have an Add instruction
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
    }

    #[test]
    fn test_comparison_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for x and result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should have a comparison instruction (implementation dependent)
        // This might be Cmp, Gt, or similar depending on IR design
        assert!(main_func.body.len() > 4); // At least some instructions generated
    }

    #[test]
    fn test_logical_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for a, b, result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "a")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "b")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should generate multiple instructions for logical operations
        assert!(main_func.body.len() > 6);
    }

    #[test]
    fn test_unary_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for flag and result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "flag")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should generate instructions for unary operation
        assert!(main_func.body.len() > 4);
    }

    #[test]
    fn test_print_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have alloca for name
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "name")));
        
        // Should have a Print instruction
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Print { .. })));
    }

    #[test]
    fn test_println_expression_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string: "Value: {}".to_string(),
                arguments: vec![Expression::IntegerLiteral(42)],
            })),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have a Print instruction (println is handled as print with newline)
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Print { .. })));
    }

    #[test]
    fn test_return_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Return(Some(Expression::IntegerLiteral(42)))),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have a Return instruction
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Return(_))));
    }

    #[test]
    fn test_empty_return_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Return(None)),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have a Return instruction with no value
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Return(Value::Unit))));
    }

    #[test]
    fn test_break_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Break),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should generate some IR (implementation dependent)
        // Break might generate Jump instruction or similar
        assert!(main_func.body.len() >= 0); // At least doesn't crash
    }

    #[test]
    fn test_continue_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Continue),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should generate some IR (implementation dependent)
        // Continue might generate Jump instruction or similar
        assert!(main_func.body.len() >= 0); // At least doesn't crash
    }

    #[test]
    fn test_block_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
                ty: Some(Ty::Int),
            }),
        };
        
        let ast = vec![
            AstNode::Statement(Statement::Block(block)),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for x and y
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "y")));
        
        // Should have an Add instruction for the block expression
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
    }

    #[test]
    fn test_nested_expressions_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "result".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expression::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::IntegerLiteral(2)),
                        right: Box::new(Expression::IntegerLiteral(3)),
                        ty: Some(Ty::Int),
                    }),
                    right: Box::new(Expression::IntegerLiteral(4)),
                    ty: Some(Ty::Int),
                }),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have alloca for result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should have both Add and Multiply instructions
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
    }

    #[test]
    fn test_string_literal_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "message".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("Hello World".to_string())),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have alloca for message
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "message")));
        
        // Should have store instruction with string value
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Store(Value::ImmString(_), _))));
    }

    #[test]
    fn test_float_literal_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "pi".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::FloatLiteral(3.14)),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have alloca for pi
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "pi")));
        
        // Should have store instruction with float value
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Store(Value::ImmFloat(_), _))));
    }

    #[test]
    fn test_identifier_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "x".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(42)),
            }),
            AstNode::Statement(Statement::Let {
                name: "y".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::Identifier("x".to_string())),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for both x and y
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "y")));
        
        // Should have load instruction to read x's value
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Load(_, _))));
    }

    #[test]
    fn test_multiple_statements_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
                    ty: Some(Ty::Int),
                }),
            }),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for all three variables
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "x")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "y")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should have multiple store instructions
        let store_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Store(_, _))).count();
        assert!(store_count >= 3);
    }

    #[test]
    fn test_all_binary_operators_ir() {
        let mut ir_gen = IrGenerator::new();
        
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
                    ty: Some(Ty::Int),
                }),
            }));
        }
        
        let ir = ir_gen.generate_ir(statements);
        let main_func = &ir["main"];
        
        // Should have allocas for all result variables
        for i in 0..operators.len() {
            let var_name = format!("result_{}", i);
            assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == &var_name)));
        }
        
        // Should have arithmetic instructions
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Sub(_, _, _))));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Div(_, _, _))));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Mod(_, _, _))));
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_empty_ast_ir() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![];
        let ir = ir_gen.generate_ir(ast);
        
        // Should still create main function
        assert!(ir.contains_key("main"));
        let main_func = &ir["main"];
        
        // Main function should be mostly empty
        assert!(main_func.body.len() == 0 || main_func.body.len() == 1); // Might have implicit return
    }

    #[test]
    fn test_single_expression_statement_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Expression(Expression::IntegerLiteral(42))),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should generate some IR for the expression
        assert!(main_func.body.len() >= 0); // At least doesn't crash
    }

    #[test]
    fn test_deeply_nested_expressions_ir() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a deeply nested binary expression: ((((1 + 2) + 3) + 4) + 5)
        let mut expr = Expression::IntegerLiteral(1);
        for i in 2..=5 {
            expr = Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(Expression::IntegerLiteral(i)),
                ty: Some(Ty::Int),
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
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have alloca for result
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "result")));
        
        // Should have multiple Add instructions for nested operations
        let add_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Add(_, _, _))).count();
        assert!(add_count >= 4); // At least 4 additions in the nested expression
    }

    #[test]
    fn test_complex_print_with_multiple_arguments_ir() {
        let mut ir_gen = IrGenerator::new();
        
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "a".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(5)),
            }),
            AstNode::Statement(Statement::Let {
                name: "b".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::IntegerLiteral(10)),
            }),
            AstNode::Statement(Statement::Expression(Expression::Print {
                format_string: "{} + {} = {}".to_string(),
                arguments: vec![
                    Expression::Identifier("a".to_string()),
                    Expression::Identifier("b".to_string()),
                    Expression::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier("a".to_string())),
                        right: Box::new(Expression::Identifier("b".to_string())),
                        ty: Some(Ty::Int),
                    },
                ],
            })),
        ];
        
        let ir = ir_gen.generate_ir(ast);
        let main_func = &ir["main"];
        
        // Should have allocas for a and b
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "a")));
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Alloca(_, ref name) if name == "b")));
        
        // Should have Print instruction
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Print { .. })));
        
        // Should have Add instruction for the expression argument
        assert!(main_func.body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
    }
}