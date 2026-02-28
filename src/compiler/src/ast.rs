#![allow(dead_code)]

use crate::types::Ty;

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    Identifier(String),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        ty: Option<Ty>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
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
    // Phase 4: Data structures
    ArrayLiteral(Vec<Expression>),
    ArrayRepeat {
        value: Box<Expression>,
        count: usize,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    TupleLiteral(Vec<Expression>),
    TupleIndex {
        object: Box<Expression>,
        index: usize,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        data: Option<Box<Expression>>,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    // Phase 5: Ownership & borrowing
    Borrow {
        expr: Box<Expression>,
        mutable: bool,
    },
    Deref(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        mutable: bool,
        type_annotation: Option<Type>,
        value: Option<Expression>,
    },
    Return(Option<Expression>),
    Expression(Expression),
    Block(Block),
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
        type_params: Vec<String>, // Phase 5: generic type parameters <T, U>
        trait_bounds: Vec<(String, Vec<String>)>, // Phase 5: T: Display + Clone -> [("T", ["Display", "Clone"])]
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
    // Phase 4: Data structures
    StructDef {
        name: String,
        fields: Vec<FieldDecl>,
        type_params: Vec<String>, // Phase 5: generic type parameters
    },
    EnumDef {
        name: String,
        variants: Vec<VariantDecl>,
        type_params: Vec<String>, // Phase 5: generic type parameters
    },
    ImplBlock {
        type_name: String,
        methods: Vec<Statement>,
        type_params: Vec<String>,   // Phase 5: generic type parameters
        trait_name: Option<String>, // Phase 5: impl Trait for Type
    },
    // Phase 5: Traits
    TraitDef {
        name: String,
        type_params: Vec<String>,
        methods: Vec<TraitMethod>,
    },
}

/// Match arm: pattern => expression/block
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression,
}

/// Patterns for match expressions and destructuring
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,            // _
    Literal(Expression), // 42, "hello", true
    Identifier(String),  // x (binds value)
    Tuple(Vec<Pattern>), // (a, b, c)
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Enum {
        enum_name: String,
        variant: String,
        data: Option<Box<Pattern>>,
    },
}

/// Field declaration in struct definition
#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub field_type: Type,
}

/// Variant declaration in enum definition
#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub name: String,
    pub kind: VariantDeclKind,
}

/// Trait method signature (may have default body)
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Option<Block>, // None = required, Some = default impl
}

#[derive(Debug, Clone)]
pub enum VariantDeclKind {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<FieldDecl>),
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
    Array(Box<Type>, usize), // [T; N]
    Tuple(Vec<Type>),        // (T1, T2, ...)
    // Phase 5
    Reference(Box<Type>, bool), // &T (false) or &mut T (true)
    Generic(String, Vec<Type>), // Name<T1, T2> e.g., Vec<i32>
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
            Expression::StringLiteral(_) => Some(Ty::String),
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Identifier(_) => None,
            Expression::FunctionCall { .. } => None,
            Expression::MethodCall { .. } => None,
            Expression::Print { .. } => None,
            Expression::Println { .. } => None,
            Expression::Comparison { .. } => Some(Ty::Bool),
            Expression::Logical { .. } => Some(Ty::Bool),
            Expression::Unary { op, .. } => match op {
                UnaryOp::Not => Some(Ty::Bool),
                UnaryOp::Negate => None,
            },
            Expression::ArrayLiteral(_) | Expression::ArrayRepeat { .. } => None,
            Expression::IndexAccess { .. } => None,
            Expression::FieldAccess { .. } => None,
            Expression::TupleLiteral(_) => None,
            Expression::TupleIndex { .. } => None,
            Expression::StructLiteral { .. } => None,
            Expression::EnumVariant { .. } => None,
            Expression::Match { .. } => None,
            Expression::Borrow { .. } => None,
            Expression::Deref(_) => None,
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
    fn binary_op_display_is_stable() {
        assert_eq!(BinaryOp::Add.to_string(), "+");
        assert_eq!(BinaryOp::Multiply.to_string(), "*");
    }

    #[test]
    fn let_statement_shape() {
        let stmt = Statement::Let {
            name: "x".to_string(),
            mutable: false,
            type_annotation: Some(Type::Named("i32".to_string())),
            value: Some(Expression::IntegerLiteral(10)),
        };

        match stmt {
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                assert_eq!(name, "x");
                assert!(!mutable);
                assert!(matches!(type_annotation, Some(Type::Named(t)) if t == "i32"));
                assert!(matches!(value, Some(Expression::IntegerLiteral(10))));
            }
            _ => panic!("expected let"),
        }
    }

    #[test]
    fn expression_literal_types() {
        assert_eq!(
            Expression::IntegerLiteral(1).get_literal_type(),
            Some(Ty::Int)
        );
        assert_eq!(
            Expression::FloatLiteral(1.0).get_literal_type(),
            Some(Ty::Float)
        );
        assert_eq!(
            Expression::Comparison {
                op: ComparisonOp::Equal,
                left: Box::new(Expression::IntegerLiteral(1)),
                right: Box::new(Expression::IntegerLiteral(1)),
            }
            .get_literal_type(),
            Some(Ty::Bool)
        );
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
