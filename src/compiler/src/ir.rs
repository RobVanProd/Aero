// src/compiler/src/ir.rs

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Reg(u32),
    ImmInt(i64),
    ImmFloat(f64),
    Var(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Reg(r) => write!(f, "{}", r),
            Value::ImmInt(n) => write!(f, "{}", n),
            Value::ImmFloat(fl) => write!(f, "{}", fl),
            Value::Var(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
    Add(Value, Value, Value), // result, lhs, rhs
    Sub(Value, Value, Value), // result, lhs, rhs
    Mul(Value, Value, Value), // result, lhs, rhs
    Div(Value, Value, Value), // result, lhs, rhs
    Store(String, Value), // variable_name, value
    Load(Value, String), // result_reg, variable_name
    Assign(Value, Value), // target_reg, source_value
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Inst>,
    pub next_reg: u32,
}


