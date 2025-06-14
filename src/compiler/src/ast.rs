#![allow(dead_code)]

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Statement {
    Let { 
        name: String, 
        value: Expression 
    },
}

#[derive(Debug)]
pub enum AstNode {
    Statement(Statement),
    Expression(Expression),
}


