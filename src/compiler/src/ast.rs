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
        ty: Option<Ty>,
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
    Loop { body: Block },
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
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Identifier(_) => None,
            Expression::FunctionCall { .. } => None,
            Expression::Print { .. } => None,
            Expression::Println { .. } => None,
            Expression::Comparison { .. } => Some(Ty::Bool),
            Expression::Logical { .. } => Some(Ty::Bool),
            Expression::Unary { op, .. } => match op {
                UnaryOp::Not => Some(Ty::Bool),
                UnaryOp::Negate => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Expression::IntegerLiteral(1).get_literal_type(), Some(Ty::Int));
        assert_eq!(Expression::FloatLiteral(1.0).get_literal_type(), Some(Ty::Float));
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
}
