// src/compiler/src/types.rs

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    String,
    Array(Box<Ty>, usize), // element type, size (fixed-size array)
    Tuple(Vec<Ty>),        // product type
    Struct(String),        // struct name (fields resolved via StructRegistry)
    Enum(String),          // enum name (variants resolved via EnumRegistry)
    Void,                  // unit / no value
    // Phase 5: Ownership & borrowing
    Reference(Box<Ty>, bool), // &T (false=immutable) or &mut T (true=mutable)
    TypeParam(String),        // generic type parameter (e.g., T)
    // Phase 6: Standard library types
    Option(Box<Ty>),                  // Option<T> - Some(T) or None
    Result(Box<Ty>, Box<Ty>),         // Result<T, E> - Ok(T) or Err(E)
    Vec(Box<Ty>),                     // Vec<T> - dynamic/growable array
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
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Ty::Struct(name) => write!(f, "{}", name),
            Ty::Enum(name) => write!(f, "{}", name),
            Ty::Void => f.write_str("()"),
            Ty::Reference(inner, mutable) => {
                if *mutable {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Ty::TypeParam(name) => write!(f, "{}", name),
            // Phase 6: Standard library types
            Ty::Option(inner) => write!(f, "Option<{}>", inner),
            Ty::Result(ok_ty, err_ty) => write!(f, "Result<{}, {}>", ok_ty, err_ty),
            Ty::Vec(elem) => write!(f, "Vec<{}>", elem),
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

/// Ownership state of a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipState {
    Owned,                  // Variable owns its value
    Moved,                  // Value has been moved out
    ImmutablyBorrowed(u32), // Number of active immutable borrows
    MutablyBorrowed,        // Active mutable borrow
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

    /// Returns true if this type is a Copy type (cheap stack copy, no move semantics).
    /// Copy types: integers, floats, booleans, references, and tuples/arrays of Copy types.
    pub fn is_copy_type(&self) -> bool {
        match self {
            Ty::Int | Ty::Float | Ty::Bool | Ty::Void => true,
            Ty::Reference(_, _) => true, // references are always Copy
            Ty::Tuple(elems) => elems.iter().all(|t| t.is_copy_type()),
            Ty::Array(elem, _) => elem.is_copy_type(),
            // Move types: String, Struct, Enum, Option, Result, Vec
            Ty::String | Ty::Struct(_) | Ty::Enum(_) => false,
            Ty::Option(_) | Ty::Result(_, _) | Ty::Vec(_) => false, // heap types are move
            Ty::TypeParam(_) => false, // conservative: generics are not Copy by default
        }
    }

    /// Returns the inner type if this is a reference, otherwise None.
    pub fn deref_type(&self) -> Option<&Ty> {
        match self {
            Ty::Reference(inner, _) => Some(inner),
            _ => None,
        }
    }

    /// Returns true if this is a mutable reference.
    pub fn is_mut_ref(&self) -> bool {
        matches!(self, Ty::Reference(_, true))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_type_display() {
        let opt_int = Ty::Option(Box::new(Ty::Int));
        assert_eq!(format!("{}", opt_int), "Option<int>");
        
        let opt_string = Ty::Option(Box::new(Ty::String));
        assert_eq!(format!("{}", opt_string), "Option<String>");
    }

    #[test]
    fn test_result_type_display() {
        let result_int_string = Ty::Result(Box::new(Ty::Int), Box::new(Ty::String));
        assert_eq!(format!("{}", result_int_string), "Result<int, String>");
    }

    #[test]
    fn test_option_not_copy() {
        let opt = Ty::Option(Box::new(Ty::Int));
        assert!(!opt.is_copy_type(), "Option should not be Copy");
    }

    #[test]
    fn test_result_not_copy() {
        let res = Ty::Result(Box::new(Ty::Int), Box::new(Ty::String));
        assert!(!res.is_copy_type(), "Result should not be Copy");
    }

    #[test]
    fn test_option_equality() {
        let opt1 = Ty::Option(Box::new(Ty::Int));
        let opt2 = Ty::Option(Box::new(Ty::Int));
        let opt3 = Ty::Option(Box::new(Ty::Float));
        
        assert_eq!(opt1, opt2);
        assert_ne!(opt1, opt3);
    }
}
