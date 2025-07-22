#![allow(dead_code)]

use crate::types::Ty;

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),
    Binary { 
        op: BinaryOp, 
        left: Box<Expression>, 
        right: Box<Expression>,
        ty: Option<Ty>, // Result type of the binary operation
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Print {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Println {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Comparison {
        op: ComparisonOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { 
        name: String, 
        mutable: bool,
        type_annotation: Option<Type>,
        value: Option<Expression>
    },
    Return(Option<Expression>),
    Expression(Expression),
    Block(Block),
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,  // &&
    Or,   // ||
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,     // !
    Negate,  // - (unary minus)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}



impl Expression {
    /// Get the inferred type of an expression (used for literals)
    pub fn get_literal_type(&self) -> Option<Ty> {
        match self {
            Expression::IntegerLiteral(_) => Some(Ty::Int),
            Expression::FloatLiteral(_) => Some(Ty::Float),
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Identifier(_) => None, // Type must be looked up in symbol table
            Expression::FunctionCall { .. } => None, // Type must be looked up from function signature
            Expression::Print { .. } => None, // Print operations don't return values (unit type)
            Expression::Println { .. } => None, // Println operations don't return values (unit type)
            Expression::Comparison { .. } => Some(Ty::Bool), // Comparisons return boolean
            Expression::Logical { .. } => Some(Ty::Bool), // Logical operations return boolean
            Expression::Unary { op, .. } => {
                match op {
                    UnaryOp::Not => Some(Ty::Bool), // Logical not returns boolean
                    UnaryOp::Negate => None, // Unary minus type depends on operand type
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call_expression() {
        let func_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(5),
                Expression::IntegerLiteral(3),
            ],
        };

        match func_call {
            Expression::FunctionCall { name, arguments } => {
                assert_eq!(name, "add");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Expression::IntegerLiteral(5)));
                assert!(matches!(arguments[1], Expression::IntegerLiteral(3)));
            }
            _ => panic!("Expected FunctionCall expression"),
        }
    }

    #[test]
    fn test_function_statement() {
        let param1 = Parameter {
            name: "a".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type { name: "i32".to_string() },
        };

        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "sum".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("a".to_string())),
                        rhs: Box::new(Expression::Identifier("b".to_string())),
                        ty: None,
                    },
                },
            ],
            expression: Some(Expression::Identifier("sum".to_string())),
        };

        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };

        match func_stmt {
            Statement::Function { name, parameters, return_type, body } => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name, "a");
                assert_eq!(parameters[0].param_type, "i32");
                assert_eq!(parameters[1].name, "b");
                assert_eq!(parameters[1].param_type, "i32");
                assert!(return_type.is_some());
                assert_eq!(return_type.unwrap().name, "i32");
                assert_eq!(body.statements.len(), 1);
                assert!(body.expression.is_some());
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_parameter_construction() {
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "f64".to_string() },
        };

        assert_eq!(param.name, "x");
        assert_eq!(param.param_type, "f64");
    }

    #[test]
    fn test_block_construction() {
        let block = Block {
            statements: vec![
                Statement::Let {
                    name: "x".to_string(),
                    value: Expression::IntegerLiteral(42),
                },
                Statement::Return(Expression::Identifier("x".to_string())),
            ],
            expression: None,
        };

        assert_eq!(block.statements.len(), 2);
        assert!(matches!(block.statements[0], Statement::Let { .. }));
        assert!(matches!(block.statements[1], Statement::Return(_)));
        assert!(block.expression.is_none());
    }

    #[test]
    fn test_function_call_with_no_arguments() {
        let func_call = Expression::FunctionCall {
            name: "get_value".to_string(),
            arguments: vec![],
        };

        match func_call {
            Expression::FunctionCall { name, arguments } => {
                assert_eq!(name, "get_value");
                assert_eq!(arguments.len(), 0);
            }
            _ => panic!("Expected FunctionCall expression"),
        }
    }

    #[test]
    fn test_function_with_no_parameters() {
        let func_stmt = Statement::Function {
            name: "main".to_string(),
            parameters: vec![],
            return_type: None,
            body: Block {
                statements: vec![],
                expression: None,
            },
        };

        match func_stmt {
            Statement::Function { name, parameters, return_type, body } => {
                assert_eq!(name, "main");
                assert_eq!(parameters.len(), 0);
                assert!(return_type.is_none());
                assert_eq!(body.statements.len(), 0);
                assert!(body.expression.is_none());
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let inner_call = Expression::FunctionCall {
            name: "multiply".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(2),
                Expression::IntegerLiteral(3),
            ],
        };

        let outer_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(1),
                inner_call,
            ],
        };

        match outer_call {
            Expression::FunctionCall { name, arguments } => {
                assert_eq!(name, "add");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Expression::IntegerLiteral(1)));
                assert!(matches!(arguments[1], Expression::FunctionCall { .. }));
            }
            _ => panic!("Expected FunctionCall expression"),
        }
    }

    #[test]
    fn test_function_call_literal_type() {
        let func_call = Expression::FunctionCall {
            name: "test".to_string(),
            arguments: vec![],
        };

        // Function calls should return None for literal type since it needs to be looked up
        assert_eq!(func_call.get_literal_type(), None);
    }

    #[test]
    fn test_if_statement() {
        let if_stmt = Statement::If {
            condition: Expression::Binary {
                op: ">".to_string(),
                lhs: Box::new(Expression::Identifier("x".to_string())),
                rhs: Box::new(Expression::IntegerLiteral(5)),
                ty: None,
            },
            then_block: Block {
                statements: vec![
                    Statement::Let {
                        name: "result".to_string(),
                        value: Expression::IntegerLiteral(1),
                    },
                ],
                expression: None,
            },
            else_block: None,
        };

        match if_stmt {
            Statement::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expression::Binary { .. }));
                assert_eq!(then_block.statements.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_if_else_statement() {
        let if_else_stmt = Statement::If {
            condition: Expression::Identifier("flag".to_string()),
            then_block: Block {
                statements: vec![Statement::Break],
                expression: None,
            },
            else_block: Some(Box::new(Statement::Continue)),
        };

        match if_else_stmt {
            Statement::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expression::Identifier(_)));
                assert_eq!(then_block.statements.len(), 1);
                assert!(matches!(then_block.statements[0], Statement::Break));
                assert!(else_block.is_some());
                assert!(matches!(*else_block.unwrap(), Statement::Continue));
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_while_statement() {
        let while_stmt = Statement::While {
            condition: Expression::Binary {
                op: "<".to_string(),
                lhs: Box::new(Expression::Identifier("i".to_string())),
                rhs: Box::new(Expression::IntegerLiteral(10)),
                ty: None,
            },
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "i".to_string(),
                        value: Expression::Binary {
                            op: "+".to_string(),
                            lhs: Box::new(Expression::Identifier("i".to_string())),
                            rhs: Box::new(Expression::IntegerLiteral(1)),
                            ty: None,
                        },
                    },
                ],
                expression: None,
            },
        };

        match while_stmt {
            Statement::While { condition, body } => {
                assert!(matches!(condition, Expression::Binary { .. }));
                assert_eq!(body.statements.len(), 1);
                assert!(matches!(body.statements[0], Statement::Let { .. }));
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_for_statement() {
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Binary {
                op: "..".to_string(),
                lhs: Box::new(Expression::IntegerLiteral(0)),
                rhs: Box::new(Expression::IntegerLiteral(10)),
                ty: None,
            },
            body: Block {
                statements: vec![
                    Statement::Let {
                        name: "temp".to_string(),
                        value: Expression::FunctionCall {
                            name: "println".to_string(),
                            arguments: vec![Expression::Identifier("i".to_string())],
                        },
                    },
                ],
                expression: None,
            },
        };

        match for_stmt {
            Statement::For { variable, iterable, body } => {
                assert_eq!(variable, "i");
                assert!(matches!(iterable, Expression::Binary { .. }));
                assert_eq!(body.statements.len(), 1);
                assert!(matches!(body.statements[0], Statement::Let { .. }));
            }
            _ => panic!("Expected For statement"),
        }
    }

    #[test]
    fn test_loop_statement() {
        let loop_stmt = Statement::Loop {
            body: Block {
                statements: vec![
                    Statement::If {
                        condition: Expression::Identifier("should_exit".to_string()),
                        then_block: Block {
                            statements: vec![Statement::Break],
                            expression: None,
                        },
                        else_block: None,
                    },
                ],
                expression: None,
            },
        };

        match loop_stmt {
            Statement::Loop { body } => {
                assert_eq!(body.statements.len(), 1);
                assert!(matches!(body.statements[0], Statement::If { .. }));
            }
            _ => panic!("Expected Loop statement"),
        }
    }

    #[test]
    fn test_break_statement() {
        let break_stmt = Statement::Break;
        assert!(matches!(break_stmt, Statement::Break));
    }

    #[test]
    fn test_continue_statement() {
        let continue_stmt = Statement::Continue;
        assert!(matches!(continue_stmt, Statement::Continue));
    }

    #[test]
    fn test_nested_control_flow() {
        let nested_stmt = Statement::While {
            condition: Expression::Identifier("running".to_string()),
            body: Block {
                statements: vec![
                    Statement::For {
                        variable: "j".to_string(),
                        iterable: Expression::Binary {
                            op: "..".to_string(),
                            lhs: Box::new(Expression::IntegerLiteral(0)),
                            rhs: Box::new(Expression::IntegerLiteral(5)),
                            ty: None,
                        },
                        body: Block {
                            statements: vec![
                                Statement::If {
                                    condition: Expression::Binary {
                                        op: "==".to_string(),
                                        lhs: Box::new(Expression::Identifier("j".to_string())),
                                        rhs: Box::new(Expression::IntegerLiteral(3)),
                                        ty: None,
                                    },
                                    then_block: Block {
                                        statements: vec![Statement::Break],
                                        expression: None,
                                    },
                                    else_block: None,
                                },
                            ],
                            expression: None,
                        },
                    },
                ],
                expression: None,
            },
        };

        match nested_stmt {
            Statement::While { condition, body } => {
                assert!(matches!(condition, Expression::Identifier(_)));
                assert_eq!(body.statements.len(), 1);
                assert!(matches!(body.statements[0], Statement::For { .. }));
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_control_flow_with_complex_conditions() {
        let complex_if = Statement::If {
            condition: Expression::Binary {
                op: "&&".to_string(),
                lhs: Box::new(Expression::Binary {
                    op: ">".to_string(),
                    lhs: Box::new(Expression::Identifier("x".to_string())),
                    rhs: Box::new(Expression::IntegerLiteral(0)),
                    ty: None,
                }),
                rhs: Box::new(Expression::Binary {
                    op: "<".to_string(),
                    lhs: Box::new(Expression::Identifier("x".to_string())),
                    rhs: Box::new(Expression::IntegerLiteral(100)),
                    ty: None,
                }),
                ty: None,
            },
            then_block: Block {
                statements: vec![Statement::Continue],
                expression: None,
            },
            else_block: Some(Box::new(Statement::Break)),
        };

        match complex_if {
            Statement::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expression::Binary { op, .. } if op == "&&"));
                assert_eq!(then_block.statements.len(), 1);
                assert!(matches!(then_block.statements[0], Statement::Continue));
                assert!(else_block.is_some());
                assert!(matches!(*else_block.unwrap(), Statement::Break));
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_print_expression() {
        let print_expr = Expression::Print {
            format_string: "Hello, {}!".to_string(),
            arguments: vec![Expression::Identifier("name".to_string())],
        };

        match print_expr {
            Expression::Print { format_string, arguments } => {
                assert_eq!(format_string, "Hello, {}!");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0], Expression::Identifier(_)));
            }
            _ => panic!("Expected Print expression"),
        }
    }

    #[test]
    fn test_println_expression() {
        let println_expr = Expression::Println {
            format_string: "Value: {}".to_string(),
            arguments: vec![
                Expression::Binary {
                    op: "+".to_string(),
                    lhs: Box::new(Expression::IntegerLiteral(5)),
                    rhs: Box::new(Expression::IntegerLiteral(3)),
                    ty: None,
                },
            ],
        };

        match println_expr {
            Expression::Println { format_string, arguments } => {
                assert_eq!(format_string, "Value: {}");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0], Expression::Binary { .. }));
            }
            _ => panic!("Expected Println expression"),
        }
    }

    #[test]
    fn test_comparison_expressions() {
        let equal_expr = Expression::Comparison {
            op: ComparisonOp::Equal,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::IntegerLiteral(5)),
        };

        let not_equal_expr = Expression::Comparison {
            op: ComparisonOp::NotEqual,
            left: Box::new(Expression::Identifier("y".to_string())),
            right: Box::new(Expression::IntegerLiteral(10)),
        };

        let less_than_expr = Expression::Comparison {
            op: ComparisonOp::LessThan,
            left: Box::new(Expression::IntegerLiteral(3)),
            right: Box::new(Expression::IntegerLiteral(7)),
        };

        match equal_expr {
            Expression::Comparison { op, left, right } => {
                assert!(matches!(op, ComparisonOp::Equal));
                assert!(matches!(*left, Expression::Identifier(_)));
                assert!(matches!(*right, Expression::IntegerLiteral(5)));
            }
            _ => panic!("Expected Comparison expression"),
        }

        match not_equal_expr {
            Expression::Comparison { op, .. } => {
                assert!(matches!(op, ComparisonOp::NotEqual));
            }
            _ => panic!("Expected Comparison expression"),
        }

        match less_than_expr {
            Expression::Comparison { op, .. } => {
                assert!(matches!(op, ComparisonOp::LessThan));
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_logical_expressions() {
        let and_expr = Expression::Logical {
            op: LogicalOp::And,
            left: Box::new(Expression::Comparison {
                op: ComparisonOp::GreaterThan,
                left: Box::new(Expression::Identifier("x".to_string())),
                right: Box::new(Expression::IntegerLiteral(0)),
            }),
            right: Box::new(Expression::Comparison {
                op: ComparisonOp::LessThan,
                left: Box::new(Expression::Identifier("x".to_string())),
                right: Box::new(Expression::IntegerLiteral(100)),
            }),
        };

        let or_expr = Expression::Logical {
            op: LogicalOp::Or,
            left: Box::new(Expression::Identifier("flag1".to_string())),
            right: Box::new(Expression::Identifier("flag2".to_string())),
        };

        match and_expr {
            Expression::Logical { op, left, right } => {
                assert!(matches!(op, LogicalOp::And));
                assert!(matches!(*left, Expression::Comparison { .. }));
                assert!(matches!(*right, Expression::Comparison { .. }));
            }
            _ => panic!("Expected Logical expression"),
        }

        match or_expr {
            Expression::Logical { op, .. } => {
                assert!(matches!(op, LogicalOp::Or));
            }
            _ => panic!("Expected Logical expression"),
        }
    }

    #[test]
    fn test_unary_expressions() {
        let not_expr = Expression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expression::Identifier("flag".to_string())),
        };

        let minus_expr = Expression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expression::IntegerLiteral(42)),
        };

        match not_expr {
            Expression::Unary { op, operand } => {
                assert!(matches!(op, UnaryOp::Not));
                assert!(matches!(*operand, Expression::Identifier(_)));
            }
            _ => panic!("Expected Unary expression"),
        }

        match minus_expr {
            Expression::Unary { op, operand } => {
                assert!(matches!(op, UnaryOp::Negate));
                assert!(matches!(*operand, Expression::IntegerLiteral(42)));
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_complex_expression_combinations() {
        // Test a complex expression: !((x > 5) && (y < 10))
        let complex_expr = Expression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expression::Logical {
                op: LogicalOp::And,
                left: Box::new(Expression::Comparison {
                    op: ComparisonOp::GreaterThan,
                    left: Box::new(Expression::Identifier("x".to_string())),
                    right: Box::new(Expression::IntegerLiteral(5)),
                }),
                right: Box::new(Expression::Comparison {
                    op: ComparisonOp::LessThan,
                    left: Box::new(Expression::Identifier("y".to_string())),
                    right: Box::new(Expression::IntegerLiteral(10)),
                }),
            }),
        };

        match complex_expr {
            Expression::Unary { op, operand } => {
                assert!(matches!(op, UnaryOp::Not));
                assert!(matches!(*operand, Expression::Logical { .. }));
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_io_expressions_with_multiple_arguments() {
        let multi_arg_print = Expression::Print {
            format_string: "{} + {} = {}".to_string(),
            arguments: vec![
                Expression::Identifier("a".to_string()),
                Expression::Identifier("b".to_string()),
                Expression::Binary {
                    op: "+".to_string(),
                    lhs: Box::new(Expression::Identifier("a".to_string())),
                    rhs: Box::new(Expression::Identifier("b".to_string())),
                    ty: None,
                },
            ],
        };

        match multi_arg_print {
            Expression::Print { format_string, arguments } => {
                assert_eq!(format_string, "{} + {} = {}");
                assert_eq!(arguments.len(), 3);
                assert!(matches!(arguments[0], Expression::Identifier(_)));
                assert!(matches!(arguments[1], Expression::Identifier(_)));
                assert!(matches!(arguments[2], Expression::Binary { .. }));
            }
            _ => panic!("Expected Print expression"),
        }
    }

    #[test]
    fn test_expression_literal_types() {
        // Test that comparison expressions return Bool type
        let comparison = Expression::Comparison {
            op: ComparisonOp::Equal,
            left: Box::new(Expression::IntegerLiteral(1)),
            right: Box::new(Expression::IntegerLiteral(2)),
        };
        assert_eq!(comparison.get_literal_type(), Some(Ty::Bool));

        // Test that logical expressions return Bool type
        let logical = Expression::Logical {
            op: LogicalOp::And,
            left: Box::new(Expression::Identifier("a".to_string())),
            right: Box::new(Expression::Identifier("b".to_string())),
        };
        assert_eq!(logical.get_literal_type(), Some(Ty::Bool));

        // Test that logical not returns Bool type
        let logical_not = Expression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expression::Identifier("flag".to_string())),
        };
        assert_eq!(logical_not.get_literal_type(), Some(Ty::Bool));

        // Test that unary minus returns None (depends on operand)
        let unary_minus = Expression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expression::IntegerLiteral(5)),
        };
        assert_eq!(unary_minus.get_literal_type(), None);

        // Test that I/O expressions return None (unit type)
        let print_expr = Expression::Print {
            format_string: "test".to_string(),
            arguments: vec![],
        };
        assert_eq!(print_expr.get_literal_type(), None);

        let println_expr = Expression::Println {
            format_string: "test".to_string(),
            arguments: vec![],
        };
        assert_eq!(println_expr.get_literal_type(), None);
    }

    #[test]
    fn test_comparison_operators() {
        let ops = vec![
            ComparisonOp::Equal,
            ComparisonOp::NotEqual,
            ComparisonOp::LessThan,
            ComparisonOp::GreaterThan,
            ComparisonOp::LessEqual,
            ComparisonOp::GreaterEqual,
        ];

        for op in ops {
            let expr = Expression::Comparison {
                op: op.clone(),
                left: Box::new(Expression::IntegerLiteral(1)),
                right: Box::new(Expression::IntegerLiteral(2)),
            };

            match expr {
                Expression::Comparison { op: actual_op, .. } => {
                    // Just verify the operator matches what we set
                    match (&op, &actual_op) {
                        (ComparisonOp::Equal, ComparisonOp::Equal) => (),
                        (ComparisonOp::NotEqual, ComparisonOp::NotEqual) => (),
                        (ComparisonOp::LessThan, ComparisonOp::LessThan) => (),
                        (ComparisonOp::GreaterThan, ComparisonOp::GreaterThan) => (),
                        (ComparisonOp::LessEqual, ComparisonOp::LessEqual) => (),
                        (ComparisonOp::GreaterEqual, ComparisonOp::GreaterEqual) => (),
                        _ => panic!("Operator mismatch"),
                    }
                }
                _ => panic!("Expected Comparison expression"),
            }
        }
    }

    #[test]
    fn test_logical_operators() {
        let and_expr = Expression::Logical {
            op: LogicalOp::And,
            left: Box::new(Expression::Identifier("a".to_string())),
            right: Box::new(Expression::Identifier("b".to_string())),
        };

        let or_expr = Expression::Logical {
            op: LogicalOp::Or,
            left: Box::new(Expression::Identifier("c".to_string())),
            right: Box::new(Expression::Identifier("d".to_string())),
        };

        match and_expr {
            Expression::Logical { op: LogicalOp::And, .. } => (),
            _ => panic!("Expected And logical expression"),
        }

        match or_expr {
            Expression::Logical { op: LogicalOp::Or, .. } => (),
            _ => panic!("Expected Or logical expression"),
        }
    }

    #[test]
    fn test_unary_operators() {
        let not_expr = Expression::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expression::Identifier("flag".to_string())),
        };

        let minus_expr = Expression::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expression::IntegerLiteral(42)),
        };

        match not_expr {
            Expression::Unary { op: UnaryOp::Not, .. } => (),
            _ => panic!("Expected Not unary expression"),
        }

        match minus_expr {
            Expression::Unary { op: UnaryOp::Negate, .. } => (),
            _ => panic!("Expected Minus unary expression"),
        }
    }
}


