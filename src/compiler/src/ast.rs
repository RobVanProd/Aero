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
    // Struct-related expressions
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
        base: Option<Box<Expression>>, // For struct update syntax
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    // Pattern matching expression
    Match {
        expression: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    // Method call expression
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    // Array and collection expressions
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    VecMacro {
        elements: Vec<Expression>,
    },
    // String formatting
    FormatMacro {
        format_string: String,
        arguments: Vec<Expression>,
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
    // Struct definition
    Struct {
        name: String,
        generics: Vec<String>,
        fields: Vec<StructField>,
        is_tuple: bool,
    },
    // Enum definition
    Enum {
        name: String,
        generics: Vec<String>,
        variants: Vec<EnumVariant>,
    },
    // Implementation block
    Impl {
        generics: Vec<String>,
        type_name: String,
        trait_name: Option<String>,
        methods: Vec<Function>,
    },
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
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),
    // Generic types
    Generic {
        name: String,
        type_args: Vec<Type>,
    },
    // Array types
    Array {
        element_type: Box<Type>,
        size: Option<usize>,
    },
    // Slice types
    Slice {
        element_type: Box<Type>,
    },
    // Vec types
    Vec {
        element_type: Box<Type>,
    },
    // HashMap types
    HashMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    // Reference types
    Reference {
        mutable: bool,
        inner_type: Box<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<EnumVariantData>,
}

#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Expression), // Using Expression for literals to reuse existing literal types
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
        rest: bool, // For .. syntax
    },
    Enum {
        variant: String,
        data: Option<Box<Pattern>>,
    },
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
    Binding {
        name: String,
        pattern: Box<Pattern>,
    },
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
            Expression::StructLiteral { .. } => None, // Type must be looked up from struct definition
            Expression::FieldAccess { .. } => None, // Type must be looked up from field definition
            Expression::Match { .. } => None, // Type must be inferred from match arms
            Expression::MethodCall { .. } => None, // Type must be looked up from method signature
            Expression::ArrayLiteral { .. } => None, // Type must be inferred from elements
            Expression::ArrayAccess { .. } => None, // Type must be looked up from array element type
            Expression::VecMacro { .. } => None, // Type must be inferred from elements
            Expression::FormatMacro { .. } => None, // Returns String type
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Enum and Pattern Matching Tests
    
    #[test]
    fn test_enum_definition() {
        let enum_def = Statement::Enum {
            name: "Color".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "Red".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Green".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Blue".to_string(),
                    data: None,
                },
            ],
        };

        match enum_def {
            Statement::Enum { name, generics, variants } => {
                assert_eq!(name, "Color");
                assert_eq!(generics.len(), 0);
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Red");
                assert_eq!(variants[1].name, "Green");
                assert_eq!(variants[2].name, "Blue");
                assert!(variants[0].data.is_none());
                assert!(variants[1].data.is_none());
                assert!(variants[2].data.is_none());
            }
            _ => panic!("Expected Enum statement"),
        }
    }

    #[test]
    fn test_enum_with_tuple_data() {
        let enum_def = Statement::Enum {
            name: "Option".to_string(),
            generics: vec!["T".to_string()],
            variants: vec![
                EnumVariant {
                    name: "Some".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
                },
                EnumVariant {
                    name: "None".to_string(),
                    data: None,
                },
            ],
        };

        match enum_def {
            Statement::Enum { name, generics, variants } => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Some");
                assert!(variants[0].data.is_some());
                match &variants[0].data {
                    Some(EnumVariantData::Tuple(types)) => {
                        assert_eq!(types.len(), 1);
                        assert_eq!(types[0], Type::Named("T".to_string()));
                    }
                    _ => panic!("Expected tuple variant data"),
                }
                assert_eq!(variants[1].name, "None");
                assert!(variants[1].data.is_none());
            }
            _ => panic!("Expected Enum statement"),
        }
    }

    #[test]
    fn test_enum_with_struct_data() {
        let enum_def = Statement::Enum {
            name: "Shape".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "Circle".to_string(),
                    data: Some(EnumVariantData::Struct(vec![
                        StructField {
                            name: "radius".to_string(),
                            field_type: Type::Named("f64".to_string()),
                            visibility: Visibility::Public,
                        },
                    ])),
                },
                EnumVariant {
                    name: "Rectangle".to_string(),
                    data: Some(EnumVariantData::Struct(vec![
                        StructField {
                            name: "width".to_string(),
                            field_type: Type::Named("f64".to_string()),
                            visibility: Visibility::Public,
                        },
                        StructField {
                            name: "height".to_string(),
                            field_type: Type::Named("f64".to_string()),
                            visibility: Visibility::Public,
                        },
                    ])),
                },
            ],
        };

        match enum_def {
            Statement::Enum { name, generics, variants } => {
                assert_eq!(name, "Shape");
                assert_eq!(generics.len(), 0);
                assert_eq!(variants.len(), 2);
                
                // Check Circle variant
                assert_eq!(variants[0].name, "Circle");
                match &variants[0].data {
                    Some(EnumVariantData::Struct(fields)) => {
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].name, "radius");
                        assert_eq!(fields[0].field_type, Type::Named("f64".to_string()));
                    }
                    _ => panic!("Expected struct variant data"),
                }
                
                // Check Rectangle variant
                assert_eq!(variants[1].name, "Rectangle");
                match &variants[1].data {
                    Some(EnumVariantData::Struct(fields)) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].name, "width");
                        assert_eq!(fields[1].name, "height");
                    }
                    _ => panic!("Expected struct variant data"),
                }
            }
            _ => panic!("Expected Enum statement"),
        }
    }

    #[test]
    fn test_match_expression() {
        let match_expr = Expression::Match {
            expression: Box::new(Expression::Identifier("color".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Enum {
                        variant: "Red".to_string(),
                        data: None,
                    },
                    guard: None,
                    body: Expression::IntegerLiteral(1),
                },
                MatchArm {
                    pattern: Pattern::Enum {
                        variant: "Green".to_string(),
                        data: None,
                    },
                    guard: None,
                    body: Expression::IntegerLiteral(2),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::IntegerLiteral(0),
                },
            ],
        };

        match match_expr {
            Expression::Match { expression, arms } => {
                assert!(matches!(*expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 3);
                
                // Check first arm
                assert!(matches!(arms[0].pattern, Pattern::Enum { .. }));
                assert!(arms[0].guard.is_none());
                assert!(matches!(arms[0].body, Expression::IntegerLiteral(1)));
                
                // Check wildcard arm
                assert!(matches!(arms[2].pattern, Pattern::Wildcard));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_pattern_wildcard() {
        let pattern = Pattern::Wildcard;
        assert!(matches!(pattern, Pattern::Wildcard));
    }

    #[test]
    fn test_pattern_identifier() {
        let pattern = Pattern::Identifier("x".to_string());
        match pattern {
            Pattern::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected Identifier pattern"),
        }
    }

    #[test]
    fn test_pattern_literal() {
        let pattern = Pattern::Literal(Expression::IntegerLiteral(42));
        match pattern {
            Pattern::Literal(expr) => {
                assert!(matches!(expr, Expression::IntegerLiteral(42)));
            }
            _ => panic!("Expected Literal pattern"),
        }
    }

    #[test]
    fn test_pattern_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
            Pattern::Wildcard,
        ]);

        match pattern {
            Pattern::Tuple(patterns) => {
                assert_eq!(patterns.len(), 3);
                assert!(matches!(patterns[0], Pattern::Identifier(_)));
                assert!(matches!(patterns[1], Pattern::Identifier(_)));
                assert!(matches!(patterns[2], Pattern::Wildcard));
            }
            _ => panic!("Expected Tuple pattern"),
        }
    }

    #[test]
    fn test_pattern_struct() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Identifier("px".to_string())),
                ("y".to_string(), Pattern::Identifier("py".to_string())),
            ],
            rest: false,
        };

        match pattern {
            Pattern::Struct { name, fields, rest } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert!(matches!(fields[0].1, Pattern::Identifier(_)));
                assert_eq!(fields[1].0, "y");
                assert!(matches!(fields[1].1, Pattern::Identifier(_)));
                assert!(!rest);
            }
            _ => panic!("Expected Struct pattern"),
        }
    }

    #[test]
    fn test_pattern_struct_with_rest() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Identifier("px".to_string())),
            ],
            rest: true,
        };

        match pattern {
            Pattern::Struct { name, fields, rest } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 1);
                assert!(rest);
            }
            _ => panic!("Expected Struct pattern"),
        }
    }

    #[test]
    fn test_pattern_enum() {
        let pattern = Pattern::Enum {
            variant: "Some".to_string(),
            data: Some(Box::new(Pattern::Identifier("value".to_string()))),
        };

        match pattern {
            Pattern::Enum { variant, data } => {
                assert_eq!(variant, "Some");
                assert!(data.is_some());
                match *data.unwrap() {
                    Pattern::Identifier(name) => assert_eq!(name, "value"),
                    _ => panic!("Expected Identifier pattern in enum data"),
                }
            }
            _ => panic!("Expected Enum pattern"),
        }
    }

    #[test]
    fn test_pattern_range_inclusive() {
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Expression::IntegerLiteral(1))),
            end: Box::new(Pattern::Literal(Expression::IntegerLiteral(10))),
            inclusive: true,
        };

        match pattern {
            Pattern::Range { start, end, inclusive } => {
                assert!(matches!(**start, Pattern::Literal(_)));
                assert!(matches!(**end, Pattern::Literal(_)));
                assert!(inclusive);
            }
            _ => panic!("Expected Range pattern"),
        }
    }

    #[test]
    fn test_pattern_range_exclusive() {
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Expression::IntegerLiteral(1))),
            end: Box::new(Pattern::Literal(Expression::IntegerLiteral(10))),
            inclusive: false,
        };

        match pattern {
            Pattern::Range { start, end, inclusive } => {
                assert!(matches!(**start, Pattern::Literal(_)));
                assert!(matches!(**end, Pattern::Literal(_)));
                assert!(!inclusive);
            }
            _ => panic!("Expected Range pattern"),
        }
    }

    #[test]
    fn test_pattern_or() {
        let pattern = Pattern::Or(vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Enum { variant: "Green".to_string(), data: None },
            Pattern::Enum { variant: "Blue".to_string(), data: None },
        ]);

        match pattern {
            Pattern::Or(patterns) => {
                assert_eq!(patterns.len(), 3);
                for p in patterns {
                    assert!(matches!(p, Pattern::Enum { .. }));
                }
            }
            _ => panic!("Expected Or pattern"),
        }
    }

    #[test]
    fn test_pattern_binding() {
        let pattern = Pattern::Binding {
            name: "color".to_string(),
            pattern: Box::new(Pattern::Enum {
                variant: "Red".to_string(),
                data: None,
            }),
        };

        match pattern {
            Pattern::Binding { name, pattern } => {
                assert_eq!(name, "color");
                assert!(matches!(**pattern, Pattern::Enum { .. }));
            }
            _ => panic!("Expected Binding pattern"),
        }
    }

    #[test]
    fn test_match_with_guard() {
        let match_expr = Expression::Match {
            expression: Box::new(Expression::Identifier("x".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("n".to_string()),
                    guard: Some(Expression::Comparison {
                        op: ComparisonOp::GreaterThan,
                        left: Box::new(Expression::Identifier("n".to_string())),
                        right: Box::new(Expression::IntegerLiteral(0)),
                    }),
                    body: Expression::Identifier("n".to_string()),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::IntegerLiteral(0),
                },
            ],
        };

        match match_expr {
            Expression::Match { expression, arms } => {
                assert!(matches!(*expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 2);
                
                // Check first arm with guard
                assert!(matches!(arms[0].pattern, Pattern::Identifier(_)));
                assert!(arms[0].guard.is_some());
                match &arms[0].guard {
                    Some(Expression::Comparison { op, .. }) => {
                        assert!(matches!(op, ComparisonOp::GreaterThan));
                    }
                    _ => panic!("Expected comparison guard"),
                }
                
                // Check wildcard arm without guard
                assert!(matches!(arms[1].pattern, Pattern::Wildcard));
                assert!(arms[1].guard.is_none());
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_nested_patterns() {
        let pattern = Pattern::Enum {
            variant: "Some".to_string(),
            data: Some(Box::new(Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Pattern::Identifier("px".to_string())),
                    ("y".to_string(), Pattern::Wildcard),
                ],
                rest: true,
            })),
        };

        match pattern {
            Pattern::Enum { variant, data } => {
                assert_eq!(variant, "Some");
                assert!(data.is_some());
                match *data.unwrap() {
                    Pattern::Struct { name, fields, rest } => {
                        assert_eq!(name, "Point");
                        assert_eq!(fields.len(), 2);
                        assert!(rest);
                    }
                    _ => panic!("Expected Struct pattern in enum data"),
                }
            }
            _ => panic!("Expected Enum pattern"),
        }
    }

    #[test]
    fn test_complex_match_expression() {
        let match_expr = Expression::Match {
            expression: Box::new(Expression::Identifier("result".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Enum {
                        variant: "Ok".to_string(),
                        data: Some(Box::new(Pattern::Binding {
                            name: "value".to_string(),
                            pattern: Box::new(Pattern::Range {
                                start: Box::new(Pattern::Literal(Expression::IntegerLiteral(1))),
                                end: Box::new(Pattern::Literal(Expression::IntegerLiteral(100))),
                                inclusive: true,
                            }),
                        })),
                    },
                    guard: None,
                    body: Expression::Identifier("value".to_string()),
                },
                MatchArm {
                    pattern: Pattern::Enum {
                        variant: "Ok".to_string(),
                        data: Some(Box::new(Pattern::Identifier("other".to_string()))),
                    },
                    guard: None,
                    body: Expression::IntegerLiteral(-1),
                },
                MatchArm {
                    pattern: Pattern::Enum {
                        variant: "Err".to_string(),
                        data: Some(Box::new(Pattern::Wildcard)),
                    },
                    guard: None,
                    body: Expression::IntegerLiteral(0),
                },
            ],
        };

        match match_expr {
            Expression::Match { expression, arms } => {
                assert!(matches!(*expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 3);
                
                // Check complex first arm
                match &arms[0].pattern {
                    Pattern::Enum { variant, data } => {
                        assert_eq!(variant, "Ok");
                        assert!(data.is_some());
                        match data.as_ref().unwrap().as_ref() {
                            Pattern::Binding { name, pattern } => {
                                assert_eq!(name, "value");
                                assert!(matches!(pattern.as_ref(), Pattern::Range { .. }));
                            }
                            _ => panic!("Expected Binding pattern"),
                        }
                    }
                    _ => panic!("Expected Enum pattern"),
                }
            }
            _ => panic!("Expected Match expression"),
        }
    }

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
            param_type: Type::Named("i32".to_string()),
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type::Named("i32".to_string()),
        };

        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "sum".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier("a".to_string())),
                        right: Box::new(Expression::Identifier("b".to_string())),
                        ty: None,
                    }),
                },
            ],
            expression: Some(Expression::Identifier("sum".to_string())),
        };

        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Some(Type::Named("i32".to_string())),
            body,
        };

        match func_stmt {
            Statement::Function { name, parameters, return_type, body } => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name, "a");
                assert_eq!(parameters[0].param_type, Type::Named("i32".to_string()));
                assert_eq!(parameters[1].name, "b");
                assert_eq!(parameters[1].param_type, Type::Named("i32".to_string()));
                assert!(return_type.is_some());
                assert_eq!(return_type.unwrap(), Type::Named("i32".to_string()));
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

        // Test that all comparison operators can be created
        for op in ops {
            let expr = Expression::Comparison {
                op: op.clone(),
                left: Box::new(Expression::IntegerLiteral(1)),
                right: Box::new(Expression::IntegerLiteral(2)),
            };
            assert_eq!(expr.get_literal_type(), Some(Ty::Bool));
        }
    }

    // Tests for struct-related AST nodes
    #[test]
    fn test_struct_definition() {
        let struct_stmt = Statement::Struct {
            name: "Point".to_string(),
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("i32".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("i32".to_string()),
                    visibility: Visibility::Private,
                },
            ],
            is_tuple: false,
        };

        match struct_stmt {
            Statement::Struct { name, fields, is_tuple } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert!(matches!(fields[0].field_type, Type::Named(ref s) if s == "i32"));
                assert!(matches!(fields[0].visibility, Visibility::Public));
                assert_eq!(fields[1].name, "y");
                assert!(matches!(fields[1].field_type, Type::Named(ref s) if s == "i32"));
                assert!(matches!(fields[1].visibility, Visibility::Private));
                assert!(!is_tuple);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_tuple_struct_definition() {
        let tuple_struct = Statement::Struct {
            name: "Color".to_string(),
            fields: vec![
                StructField {
                    name: "0".to_string(),
                    field_type: Type::Named("u8".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "1".to_string(),
                    field_type: Type::Named("u8".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "2".to_string(),
                    field_type: Type::Named("u8".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: true,
        };

        match tuple_struct {
            Statement::Struct { name, fields, is_tuple } => {
                assert_eq!(name, "Color");
                assert_eq!(fields.len(), 3);
                assert!(is_tuple);
                // All fields should be public for tuple structs
                for field in fields {
                    assert!(matches!(field.visibility, Visibility::Public));
                    assert!(matches!(field.field_type, Type::Named(ref s) if s == "u8"));
                }
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_struct_literal_expression() {
        let struct_literal = Expression::StructLiteral {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expression::IntegerLiteral(10)),
                ("y".to_string(), Expression::IntegerLiteral(20)),
            ],
            base: None,
        };

        match struct_literal {
            Expression::StructLiteral { name, fields, base } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert!(matches!(fields[0].1, Expression::IntegerLiteral(10)));
                assert_eq!(fields[1].0, "y");
                assert!(matches!(fields[1].1, Expression::IntegerLiteral(20)));
                assert!(base.is_none());
            }
            _ => panic!("Expected StructLiteral expression"),
        }
    }

    #[test]
    fn test_struct_literal_with_base() {
        let struct_literal = Expression::StructLiteral {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expression::IntegerLiteral(5)),
            ],
            base: Some(Box::new(Expression::Identifier("old_point".to_string()))),
        };

        match struct_literal {
            Expression::StructLiteral { name, fields, base } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "x");
                assert!(matches!(fields[0].1, Expression::IntegerLiteral(5)));
                assert!(base.is_some());
                assert!(matches!(*base.unwrap(), Expression::Identifier(ref s) if s == "old_point"));
            }
            _ => panic!("Expected StructLiteral expression"),
        }
    }

    #[test]
    fn test_field_access_expression() {
        let field_access = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("point".to_string())),
            field: "x".to_string(),
        };

        match field_access {
            Expression::FieldAccess { object, field } => {
                assert!(matches!(*object, Expression::Identifier(ref s) if s == "point"));
                assert_eq!(field, "x");
            }
            _ => panic!("Expected FieldAccess expression"),
        }
    }

    #[test]
    fn test_chained_field_access() {
        let chained_access = Expression::FieldAccess {
            object: Box::new(Expression::FieldAccess {
                object: Box::new(Expression::Identifier("rect".to_string())),
                field: "top_left".to_string(),
            }),
            field: "x".to_string(),
        };

        match chained_access {
            Expression::FieldAccess { object, field } => {
                assert_eq!(field, "x");
                assert!(matches!(*object, Expression::FieldAccess { .. }));
            }
            _ => panic!("Expected FieldAccess expression"),
        }
    }

    #[test]
    fn test_struct_field_construction() {
        let field = StructField {
            name: "width".to_string(),
            field_type: Type::Named("f64".to_string()),
            visibility: Visibility::Public,
        };

        assert_eq!(field.name, "width");
        assert!(matches!(field.field_type, Type::Named(ref s) if s == "f64"));
        assert!(matches!(field.visibility, Visibility::Public));
    }

    #[test]
    fn test_visibility_enum() {
        let public_vis = Visibility::Public;
        let private_vis = Visibility::Private;

        assert!(matches!(public_vis, Visibility::Public));
        assert!(matches!(private_vis, Visibility::Private));
    }

    #[test]
    fn test_struct_literal_type_inference() {
        let struct_literal = Expression::StructLiteral {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expression::IntegerLiteral(1)),
                ("y".to_string(), Expression::IntegerLiteral(2)),
            ],
            base: None,
        };

        // Struct literals should return None for literal type since it needs to be looked up
        assert_eq!(struct_literal.get_literal_type(), None);
    }

    #[test]
    fn test_field_access_type_inference() {
        let field_access = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("point".to_string())),
            field: "x".to_string(),
        };

        // Field access should return None for literal type since it needs to be looked up
        assert_eq!(field_access.get_literal_type(), None);
    }

    #[test]
    fn test_empty_struct_definition() {
        let empty_struct = Statement::Struct {
            name: "Empty".to_string(),
            fields: vec![],
            is_tuple: false,
        };

        match empty_struct {
            Statement::Struct { name, fields, is_tuple } => {
                assert_eq!(name, "Empty");
                assert_eq!(fields.len(), 0);
                assert!(!is_tuple);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_struct_with_complex_field_types() {
        let complex_struct = Statement::Struct {
            name: "ComplexStruct".to_string(),
            fields: vec![
                StructField {
                    name: "id".to_string(),
                    field_type: Type::Named("String".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "values".to_string(),
                    field_type: Type::Named("Vec<i32>".to_string()),
                    visibility: Visibility::Private,
                },
            ],
            is_tuple: false,
        };

        match complex_struct {
            Statement::Struct { name, fields, .. } => {
                assert_eq!(name, "ComplexStruct");
                assert_eq!(fields.len(), 2);
                assert!(matches!(fields[0].field_type, Type::Named(ref s) if s == "String"));
                assert!(matches!(fields[1].field_type, Type::Named(ref s) if s == "Vec<i32>"));
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_struct_literal_with_expressions() {
        let struct_literal = Expression::StructLiteral {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::IntegerLiteral(5)),
                    right: Box::new(Expression::IntegerLiteral(3)),
                    ty: None,
                }),
                ("y".to_string(), Expression::FunctionCall {
                    name: "get_y".to_string(),
                    arguments: vec![],
                }),
            ],
            base: None,
        };

        match struct_literal {
            Expression::StructLiteral { name, fields, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert!(matches!(fields[0].1, Expression::Binary { .. }));
                assert!(matches!(fields[1].1, Expression::FunctionCall { .. }));
            }
            _ => panic!("Expected StructLiteral expression"),
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

    // Generic and Collection AST Tests

    #[test]
    fn test_generic_struct_definition() {
        let generic_struct = Statement::Struct {
            name: "Container".to_string(),
            generics: vec!["T".to_string(), "U".to_string()],
            fields: vec![
                StructField {
                    name: "value".to_string(),
                    field_type: Type::Named("T".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "metadata".to_string(),
                    field_type: Type::Named("U".to_string()),
                    visibility: Visibility::Private,
                },
            ],
            is_tuple: false,
        };

        match generic_struct {
            Statement::Struct { name, generics, fields, is_tuple } => {
                assert_eq!(name, "Container");
                assert_eq!(generics.len(), 2);
                assert_eq!(generics[0], "T");
                assert_eq!(generics[1], "U");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "value");
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
                assert_eq!(fields[1].name, "metadata");
                assert_eq!(fields[1].field_type, Type::Named("U".to_string()));
                assert!(!is_tuple);
            }
            _ => panic!("Expected generic Struct statement"),
        }
    }

    #[test]
    fn test_generic_enum_definition() {
        let generic_enum = Statement::Enum {
            name: "Result".to_string(),
            generics: vec!["T".to_string(), "E".to_string()],
            variants: vec![
                EnumVariant {
                    name: "Ok".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
                },
                EnumVariant {
                    name: "Err".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("E".to_string())])),
                },
            ],
        };

        match generic_enum {
            Statement::Enum { name, generics, variants } => {
                assert_eq!(name, "Result");
                assert_eq!(generics.len(), 2);
                assert_eq!(generics[0], "T");
                assert_eq!(generics[1], "E");
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Ok");
                assert_eq!(variants[1].name, "Err");
                
                // Check Ok variant data
                match &variants[0].data {
                    Some(EnumVariantData::Tuple(types)) => {
                        assert_eq!(types.len(), 1);
                        assert_eq!(types[0], Type::Named("T".to_string()));
                    }
                    _ => panic!("Expected tuple variant data for Ok"),
                }
                
                // Check Err variant data
                match &variants[1].data {
                    Some(EnumVariantData::Tuple(types)) => {
                        assert_eq!(types.len(), 1);
                        assert_eq!(types[0], Type::Named("E".to_string()));
                    }
                    _ => panic!("Expected tuple variant data for Err"),
                }
            }
            _ => panic!("Expected generic Enum statement"),
        }
    }

    #[test]
    fn test_impl_block() {
        let impl_block = Statement::Impl {
            generics: vec!["T".to_string()],
            type_name: "Container".to_string(),
            trait_name: None,
            methods: vec![
                Function {
                    name: "new".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "value".to_string(),
                            param_type: Type::Named("T".to_string()),
                        },
                    ],
                    return_type: Some(Type::Generic {
                        name: "Container".to_string(),
                        type_args: vec![Type::Named("T".to_string())],
                    }),
                    body: Block {
                        statements: vec![],
                        expression: Some(Expression::StructLiteral {
                            name: "Container".to_string(),
                            fields: vec![("value".to_string(), Expression::Identifier("value".to_string()))],
                            base: None,
                        }),
                    },
                },
            ],
        };

        match impl_block {
            Statement::Impl { generics, type_name, trait_name, methods } => {
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(type_name, "Container");
                assert!(trait_name.is_none());
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "new");
                assert_eq!(methods[0].parameters.len(), 1);
                assert_eq!(methods[0].parameters[0].name, "value");
            }
            _ => panic!("Expected Impl statement"),
        }
    }

    #[test]
    fn test_impl_block_with_trait() {
        let impl_block = Statement::Impl {
            generics: vec!["T".to_string()],
            type_name: "Container".to_string(),
            trait_name: Some("Display".to_string()),
            methods: vec![
                Function {
                    name: "fmt".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "self".to_string(),
                            param_type: Type::Reference {
                                mutable: false,
                                inner_type: Box::new(Type::Named("Self".to_string())),
                            },
                        },
                    ],
                    return_type: Some(Type::Named("String".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: Some(Expression::FormatMacro {
                            format_string: "Container({})".to_string(),
                            arguments: vec![Expression::FieldAccess {
                                object: Box::new(Expression::Identifier("self".to_string())),
                                field: "value".to_string(),
                            }],
                        }),
                    },
                },
            ],
        };

        match impl_block {
            Statement::Impl { generics, type_name, trait_name, methods } => {
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(type_name, "Container");
                assert_eq!(trait_name, Some("Display".to_string()));
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "fmt");
            }
            _ => panic!("Expected Impl statement with trait"),
        }
    }

    #[test]
    fn test_array_literal() {
        let array_literal = Expression::ArrayLiteral {
            elements: vec![
                Expression::IntegerLiteral(1),
                Expression::IntegerLiteral(2),
                Expression::IntegerLiteral(3),
                Expression::IntegerLiteral(4),
                Expression::IntegerLiteral(5),
            ],
        };

        match array_literal {
            Expression::ArrayLiteral { elements } => {
                assert_eq!(elements.len(), 5);
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        Expression::IntegerLiteral(value) => {
                            assert_eq!(*value, (i + 1) as i64);
                        }
                        _ => panic!("Expected IntegerLiteral in array"),
                    }
                }
            }
            _ => panic!("Expected ArrayLiteral expression"),
        }
    }

    #[test]
    fn test_array_access() {
        let array_access = Expression::ArrayAccess {
            array: Box::new(Expression::Identifier("numbers".to_string())),
            index: Box::new(Expression::IntegerLiteral(0)),
        };

        match array_access {
            Expression::ArrayAccess { array, index } => {
                match array.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "numbers"),
                    _ => panic!("Expected Identifier for array"),
                }
                match index.as_ref() {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 0),
                    _ => panic!("Expected IntegerLiteral for index"),
                }
            }
            _ => panic!("Expected ArrayAccess expression"),
        }
    }

    #[test]
    fn test_vec_macro() {
        let vec_macro = Expression::VecMacro {
            elements: vec![
                Expression::IntegerLiteral(10),
                Expression::IntegerLiteral(20),
                Expression::IntegerLiteral(30),
            ],
        };

        match vec_macro {
            Expression::VecMacro { elements } => {
                assert_eq!(elements.len(), 3);
                let expected_values = [10, 20, 30];
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        Expression::IntegerLiteral(value) => {
                            assert_eq!(*value, expected_values[i]);
                        }
                        _ => panic!("Expected IntegerLiteral in vec macro"),
                    }
                }
            }
            _ => panic!("Expected VecMacro expression"),
        }
    }

    #[test]
    fn test_format_macro() {
        let format_macro = Expression::FormatMacro {
            format_string: "Hello, {}! You have {} messages.".to_string(),
            arguments: vec![
                Expression::Identifier("name".to_string()),
                Expression::IntegerLiteral(5),
            ],
        };

        match format_macro {
            Expression::FormatMacro { format_string, arguments } => {
                assert_eq!(format_string, "Hello, {}! You have {} messages.");
                assert_eq!(arguments.len(), 2);
                match &arguments[0] {
                    Expression::Identifier(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected Identifier for first argument"),
                }
                match &arguments[1] {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 5),
                    _ => panic!("Expected IntegerLiteral for second argument"),
                }
            }
            _ => panic!("Expected FormatMacro expression"),
        }
    }

    #[test]
    fn test_method_call() {
        let method_call = Expression::MethodCall {
            object: Box::new(Expression::Identifier("vec".to_string())),
            method: "push".to_string(),
            arguments: vec![Expression::IntegerLiteral(42)],
        };

        match method_call {
            Expression::MethodCall { object, method, arguments } => {
                match object.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "vec"),
                    _ => panic!("Expected Identifier for object"),
                }
                assert_eq!(method, "push");
                assert_eq!(arguments.len(), 1);
                match &arguments[0] {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 42),
                    _ => panic!("Expected IntegerLiteral for argument"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_generic_types() {
        // Test Generic type
        let generic_type = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::Named("i32".to_string())],
        };

        match generic_type {
            Type::Generic { name, type_args } => {
                assert_eq!(name, "Vec");
                assert_eq!(type_args.len(), 1);
                assert_eq!(type_args[0], Type::Named("i32".to_string()));
            }
            _ => panic!("Expected Generic type"),
        }

        // Test Array type
        let array_type = Type::Array {
            element_type: Box::new(Type::Named("f64".to_string())),
            size: Some(10),
        };

        match array_type {
            Type::Array { element_type, size } => {
                assert_eq!(*element_type, Type::Named("f64".to_string()));
                assert_eq!(size, Some(10));
            }
            _ => panic!("Expected Array type"),
        }

        // Test Slice type
        let slice_type = Type::Slice {
            element_type: Box::new(Type::Named("u8".to_string())),
        };

        match slice_type {
            Type::Slice { element_type } => {
                assert_eq!(*element_type, Type::Named("u8".to_string()));
            }
            _ => panic!("Expected Slice type"),
        }

        // Test Vec type
        let vec_type = Type::Vec {
            element_type: Box::new(Type::Named("String".to_string())),
        };

        match vec_type {
            Type::Vec { element_type } => {
                assert_eq!(*element_type, Type::Named("String".to_string()));
            }
            _ => panic!("Expected Vec type"),
        }

        // Test HashMap type
        let hashmap_type = Type::HashMap {
            key_type: Box::new(Type::Named("String".to_string())),
            value_type: Box::new(Type::Named("i32".to_string())),
        };

        match hashmap_type {
            Type::HashMap { key_type, value_type } => {
                assert_eq!(*key_type, Type::Named("String".to_string()));
                assert_eq!(*value_type, Type::Named("i32".to_string()));
            }
            _ => panic!("Expected HashMap type"),
        }
    }

    #[test]
    fn test_complex_generic_expressions() {
        // Test nested array access with method call
        let complex_expr = Expression::MethodCall {
            object: Box::new(Expression::ArrayAccess {
                array: Box::new(Expression::Identifier("containers".to_string())),
                index: Box::new(Expression::IntegerLiteral(0)),
            }),
            method: "get_value".to_string(),
            arguments: vec![],
        };

        match complex_expr {
            Expression::MethodCall { object, method, arguments } => {
                assert_eq!(method, "get_value");
                assert_eq!(arguments.len(), 0);
                match object.as_ref() {
                    Expression::ArrayAccess { array, index } => {
                        match array.as_ref() {
                            Expression::Identifier(name) => assert_eq!(name, "containers"),
                            _ => panic!("Expected Identifier for array"),
                        }
                        match index.as_ref() {
                            Expression::IntegerLiteral(value) => assert_eq!(*value, 0),
                            _ => panic!("Expected IntegerLiteral for index"),
                        }
                    }
                    _ => panic!("Expected ArrayAccess for object"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_nested_generic_types() {
        // Test Vec<Vec<i32>>
        let nested_vec_type = Type::Vec {
            element_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match nested_vec_type {
            Type::Vec { element_type } => {
                match element_type.as_ref() {
                    Type::Vec { element_type: inner_element_type } => {
                        assert_eq!(**inner_element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected nested Vec type"),
                }
            }
            _ => panic!("Expected Vec type"),
        }

        // Test HashMap<String, Vec<i32>>
        let complex_hashmap_type = Type::HashMap {
            key_type: Box::new(Type::Named("String".to_string())),
            value_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match complex_hashmap_type {
            Type::HashMap { key_type, value_type } => {
                assert_eq!(*key_type, Type::Named("String".to_string()));
                match value_type.as_ref() {
                    Type::Vec { element_type } => {
                        assert_eq!(**element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected Vec type for HashMap value"),
                }
            }
            _ => panic!("Expected HashMap type"),
        }
    }

    #[test]
    fn test_reference_types() {
        // Test mutable reference
        let mut_ref_type = Type::Reference {
            mutable: true,
            inner_type: Box::new(Type::Named("String".to_string())),
        };

        match mut_ref_type {
            Type::Reference { mutable, inner_type } => {
                assert!(mutable);
                assert_eq!(*inner_type, Type::Named("String".to_string()));
            }
            _ => panic!("Expected Reference type"),
        }

        // Test immutable reference
        let ref_type = Type::Reference {
            mutable: false,
            inner_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match ref_type {
            Type::Reference { mutable, inner_type } => {
                assert!(!mutable);
                match inner_type.as_ref() {
                    Type::Vec { element_type } => {
                        assert_eq!(**element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected Vec type for reference"),
                }
            }
            _ => panic!("Expected Reference type"),
        }
    }
}


