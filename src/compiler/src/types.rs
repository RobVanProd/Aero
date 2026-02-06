// src/compiler/src/types.rs

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    String,
    Array(Box<Ty>, usize),        // element type, size
    Tuple(Vec<Ty>),               // product type
    Struct(String),               // struct name (fields resolved via StructRegistry)
    Enum(String),                 // enum name (variants resolved via EnumRegistry)
    Void,                         // unit / no value
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => f.write_str("int"),
            Ty::Float => f.write_str("float"),
            Ty::Bool => f.write_str("bool"),
            Ty::String => f.write_str("String"),
            Ty::Array(elem, size) => write!(f, "[{}; {}]", elem, size),
            Ty::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Ty::Struct(name) => write!(f, "{}", name),
            Ty::Enum(name) => write!(f, "{}", name),
            Ty::Void => f.write_str("()"),
        }
    }
}

/// Field definition for structs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldDef {
    pub name: String,
    pub ty: Ty,
}

/// Struct definition stored in registry
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

impl StructDef {
    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    pub fn field_type(&self, name: &str) -> Option<&Ty> {
        self.fields.iter().find(|f| f.name == name).map(|f| &f.ty)
    }
}

/// Enum variant kinds
#[derive(Debug, Clone)]
pub enum VariantKind {
    Unit,
    Tuple(Vec<Ty>),
    Struct(Vec<FieldDef>),
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub kind: VariantKind,
}

/// Enum definition stored in registry
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

impl EnumDef {
    pub fn variant_index(&self, name: &str) -> Option<usize> {
        self.variants.iter().position(|v| v.name == name)
    }
}

impl Ty {
    pub fn from_string(s: &str) -> Option<Ty> {
        match s {
            "int" | "i32" => Some(Ty::Int),
            "float" | "f64" => Some(Ty::Float),
            "bool" => Some(Ty::Bool),
            "String" => Some(Ty::String),
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
