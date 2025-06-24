#![allow(dead_code)]

use crate::types::Ty;

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    Float(f64),
    Identifier(String),
    Binary { 
        op: String, 
        lhs: Box<Expression>, 
        rhs: Box<Expression>,
        ty: Option<Ty>, // Result type of the binary operation
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { 
        name: String, 
        value: Expression 
    },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Statement(Statement),
    Expression(Expression),
}

impl Expression {
    /// Get the inferred type of an expression (used for literals)
    pub fn get_literal_type(&self) -> Option<Ty> {
        match self {
            Expression::Number(_) => Some(Ty::Int),
            Expression::Float(_) => Some(Ty::Float),
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Identifier(_) => None, // Type must be looked up in symbol table
        }
    }
}


