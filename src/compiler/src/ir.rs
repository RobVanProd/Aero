// src/compiler/src/ir.rs

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Reg(u32),
    ImmInt(i64),
    ImmFloat(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Reg(r) => write!(f, "{}", r),
            Value::ImmInt(n) => write!(f, "{}", n),
            Value::ImmFloat(fl) => write!(f, "{}", fl),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
    Add(Value, Value, Value), // result, lhs, rhs (integer)
    FAdd(Value, Value, Value), // result, lhs, rhs (float)
    Sub(Value, Value, Value), // result, lhs, rhs (integer)
    FSub(Value, Value, Value), // result, lhs, rhs (float)
    Mul(Value, Value, Value), // result, lhs, rhs (integer)
    FMul(Value, Value, Value), // result, lhs, rhs (float)
    Div(Value, Value, Value), // result, lhs, rhs (integer)
    FDiv(Value, Value, Value), // result, lhs, rhs (float)
    Alloca(Value, String), // pointer_reg, variable_name
    Store(Value, Value), // pointer_reg, value_to_store
    Load(Value, Value), // result_reg, pointer_reg
    Return(Value), // value to return
    SIToFP(Value, Value), // result_reg, int_value (signed integer to floating point)
    FPToSI(Value, Value), // result_reg, float_value (floating point to signed integer)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Inst>,
    pub next_reg: u32,
    pub next_ptr: u32, // New field for unique pointer IDs
}


