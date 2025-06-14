#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    Float(f64),
    Identifier(String),
    Binary { 
        op: String, 
        lhs: Box<Expression>, 
        rhs: Box<Expression> 
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


