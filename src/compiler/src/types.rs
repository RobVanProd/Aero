// src/compiler/src/types.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
}

impl Ty {
    pub fn to_string(&self) -> String {
        match self {
            Ty::Int => "int".to_string(),
            Ty::Float => "float".to_string(),
            Ty::Bool => "bool".to_string(),
        }
    }

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
        "+" | "-" | "*" | "/" | "%" => {
            match (lhs, rhs) {
                (Ty::Int, Ty::Int) => Ok(Ty::Int),
                (Ty::Float, Ty::Float) => Ok(Ty::Float),
                (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float), // promote to float
                _ => Err(format!(
                    "Type mismatch in arithmetic operation `{}`: {} vs {}",
                    op,
                    lhs.to_string(),
                    rhs.to_string()
                )),
            }
        }
        // Comparison operations
        "==" | "!=" | "<" | ">" | "<=" | ">=" => {
            match (lhs, rhs) {
                (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) | (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
                (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Bool), // allow comparison with promotion
                _ => Err(format!(
                    "Type mismatch in comparison operation `{}`: {} vs {}",
                    op,
                    lhs.to_string(),
                    rhs.to_string()
                )),
            }
        }
        // Logical operations
        "&&" | "||" => match (lhs, rhs) {
            (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
            _ => Err(format!(
                "Logical operation `{}` requires boolean operands: {} vs {}",
                op,
                lhs.to_string(),
                rhs.to_string()
            )),
        },
        _ => Err(format!("Unknown binary operation: {}", op)),
    }
}

/// Check if a type promotion is needed from source to target
pub fn needs_promotion(from: &Ty, to: &Ty) -> bool {
    matches!((from, to), (Ty::Int, Ty::Float))
}
