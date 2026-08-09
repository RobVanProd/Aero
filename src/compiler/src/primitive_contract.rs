use crate::ir::{LogicalType, Value};
use crate::types::Ty;

/// One source-to-backend authority for primitive identity. Physical lowering is
/// deliberately private: logical `Char` remains distinct from logical `Int` even
/// though both use an `i32` lane where character values are scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimitiveKind {
    Int,
    Float,
    Bool,
    Char,
}

impl PrimitiveKind {
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        match name {
            "int" | "i32" => Some(Self::Int),
            "float" | "f64" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "char" => Some(Self::Char),
            _ => None,
        }
    }

    pub(crate) fn from_ty(ty: &Ty) -> Option<Self> {
        match ty {
            Ty::Int => Some(Self::Int),
            Ty::Float => Some(Self::Float),
            Ty::Bool => Some(Self::Bool),
            Ty::Char => Some(Self::Char),
            _ => None,
        }
    }

    pub(crate) fn from_logical_type(ty: &LogicalType) -> Option<Self> {
        match ty {
            LogicalType::Int => Some(Self::Int),
            LogicalType::Float => Some(Self::Float),
            LogicalType::Bool => Some(Self::Bool),
            LogicalType::Char => Some(Self::Char),
            _ => None,
        }
    }

    pub(crate) fn ty(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::Float => Ty::Float,
            Self::Bool => Ty::Bool,
            Self::Char => Ty::Char,
        }
    }

    pub(crate) fn logical_type(self) -> LogicalType {
        match self {
            Self::Int => LogicalType::Int,
            Self::Float => LogicalType::Float,
            Self::Bool => LogicalType::Bool,
            Self::Char => LogicalType::Char,
        }
    }

    pub(crate) fn admits_integer_predicate(self, predicate: &str) -> bool {
        match self {
            Self::Int => matches!(predicate, "eq" | "ne" | "slt" | "sgt" | "sle" | "sge"),
            Self::Bool | Self::Char => matches!(predicate, "eq" | "ne"),
            Self::Float => false,
        }
    }

    pub(crate) fn scalar_llvm_type(self) -> &'static str {
        match self {
            Self::Int | Self::Char => "i32",
            Self::Float => "double",
            Self::Bool => "i1",
        }
    }

    pub(crate) fn copy_data_llvm_type(self) -> &'static str {
        match self {
            // Preserve the accepted aggregate/reference numeric representation.
            Self::Int | Self::Float => "double",
            Self::Bool => "i1",
            Self::Char => "i32",
        }
    }

    pub(crate) fn copy_data_zero(self) -> &'static str {
        match self {
            Self::Int | Self::Float => "0x0000000000000000",
            Self::Bool => "false",
            Self::Char => "0",
        }
    }

    pub(crate) fn alignment(self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Char => 4,
            Self::Int | Self::Float => 8,
        }
    }

    pub(crate) fn raw_zero_value(self) -> Value {
        match self {
            Self::Int | Self::Bool => Value::ImmInt(0),
            Self::Float => Value::ImmFloat(0.0),
            Self::Char => Value::ImmChar('\0'),
        }
    }
}
