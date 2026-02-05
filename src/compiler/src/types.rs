// src/compiler/src/types.rs

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Ty::Int => "int",
            Ty::Float => "float",
            Ty::Bool => "bool",
        };
        f.write_str(s)
    }
}

impl Ty {
    pub fn from_string(s: &str) -> Option<Ty> {
        match s {
            "int" => Some(Ty::Int),
            "float" => Some(Ty::Float),
            "bool" => Some(Ty::Bool),
            _ => None,
        }
    }
}

/// Type inference and promotion rules for binary operations
pub fn infer_binary_type(op: &str, lhs: &Ty, rhs: &Ty) -> Result<Ty, String> {
    match op {
        // Arithmetic operations
        "+" | "-" | "*" | "/" | "%" => match (lhs, rhs) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Float, Ty::Float) => Ok(Ty::Float),
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float), // promote to float
            _ => Err(format!(
                "Type mismatch in arithmetic operation `{}`: {} vs {}",
                op, lhs, rhs
            )),
        },
        // Comparison operations
        "==" | "!=" | "<" | ">" | "<=" | ">=" => match (lhs, rhs) {
            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) | (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Bool), // allow comparison with promotion
            _ => Err(format!(
                "Type mismatch in comparison operation `{}`: {} vs {}",
                op, lhs, rhs
            )),
        },
        // Logical operations
        "&&" | "||" => match (lhs, rhs) {
            (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
            _ => Err(format!(
                "Logical operation `{}` requires boolean operands: {} vs {}",
                op, lhs, rhs
            )),
        },
        _ => Err(format!("Unknown binary operation: {}", op)),
    }
}

/// Check if a type promotion is needed from source to target
pub fn needs_promotion(from: &Ty, to: &Ty) -> bool {
    matches!((from, to), (Ty::Int, Ty::Float))
}
