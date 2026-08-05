// src/compiler/src/ir.rs

use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Reg(u32),
    ImmInt(i64),
    ImmFloat(f64),
    ImmString(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Reg(r) => write!(f, "{}", r),
            Value::ImmInt(n) => write!(f, "{}", n),
            Value::ImmFloat(fl) => write!(f, "{}", fl),
            Value::ImmString(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct EnumVariantSchema {
    pub name: String,
    pub payload: Option<LogicalType>,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct EnumSchema {
    pub name: String,
    pub variants: Vec<EnumVariantSchema>,
}

impl EnumSchema {
    pub fn is_unit(&self) -> bool {
        self.variants
            .iter()
            .all(|variant| variant.payload.is_none())
    }

    pub fn logical_type(&self) -> LogicalType {
        LogicalType::Enum {
            name: self.name.clone(),
            variants: self.variants.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
    Add(Value, Value, Value),  // result, lhs, rhs (integer)
    FAdd(Value, Value, Value), // result, lhs, rhs (float)
    Sub(Value, Value, Value),  // result, lhs, rhs (integer)
    FSub(Value, Value, Value), // result, lhs, rhs (float)
    Mul(Value, Value, Value),  // result, lhs, rhs (integer)
    FMul(Value, Value, Value), // result, lhs, rhs (float)
    Div(Value, Value, Value),  // result, lhs, rhs (integer)
    FDiv(Value, Value, Value), // result, lhs, rhs (float)
    Alloca(Value, String),     // pointer_reg, variable_name
    Store(Value, Value),       // pointer_reg, value_to_store
    Load(Value, Value),        // result_reg, pointer_reg
    /// Verified mutable stack place for an admitted local Copy-data binding.
    CheckedMutableCopyPlaceAlloca {
        result: Value,
        name: String,
        ty: LogicalType,
    },
    /// Verified whole-value reassignment of an existing admitted mutable Copy-data place.
    CheckedCopyPlaceAssignment {
        target: Value,
        value: Value,
        ty: LogicalType,
    },
    /// Verified read-only alias of an existing admitted Copy-data place.
    CheckedImmutableBorrow {
        result: Value,
        source: Value,
        pointee: LogicalType,
    },
    /// Verified exclusive non-escaping alias of a mutable Copy-data owner or reference place.
    CheckedMutableBorrow {
        result: Value,
        source: Value,
        pointee: LogicalType,
    },
    /// Verified whole-value write through a checked mutable-reference alias.
    CheckedMutableDereferenceAssignment {
        target: Value,
        value: Value,
        pointee: LogicalType,
    },
    /// Verified end of a non-escaping mutable Copy-data borrow or reborrow.
    CheckedMutableBorrowEnd {
        reference: Value,
        source: Value,
        pointee: LogicalType,
    },
    /// Verified read-only place binding for an immutable Copy-data-reference parameter.
    CheckedImmutableReferenceParameter {
        result: Value,
        parameter: String,
        pointee: LogicalType,
    },
    CheckedMutableReferenceParameter {
        result: Value,
        parameter: String,
        pointee: LogicalType,
    },
    /// Verified direct SSA binding of an owned enum function parameter.
    CheckedEnumParameter {
        result: Value,
        parameter: String,
        schema: EnumSchema,
    },
    /// Verified construction of one unit or unary recursive CopyData variant.
    CheckedEnumVariant {
        result: Value,
        schema: EnumSchema,
        variant_index: usize,
        payload: Option<Value>,
    },
    /// Verified extraction of the selected unary recursive CopyData payload.
    CheckedEnumPayload {
        result: Value,
        value: Value,
        schema: EnumSchema,
        variant_index: usize,
    },
    /// Verified exhaustive dispatch with one target per declaration-ordered variant.
    CheckedEnumDispatch {
        value: Value,
        schema: EnumSchema,
        targets: Vec<String>,
    },
    Return(Value),        // value to return
    SIToFP(Value, Value), // result_reg, int_value (signed integer to floating point)
    FPToSI(Value, Value), // result_reg, float_value (floating point to signed integer)

    // Function operations
    FunctionDef {
        name: String,
        parameters: Vec<(String, String)>, // (name, type)
        return_type: Option<String>,
        body: Vec<Inst>,
    },
    /// Source-admitted function definition whose logical signature may contain
    /// verified all-component-`Copy` struct values.
    CheckedFunctionDef {
        name: String,
        parameters: Vec<(String, LogicalType)>,
        result: LogicalType,
        body: Vec<Inst>,
    },
    Call {
        function: String,
        arguments: Vec<Value>,
        result: Option<Value>,
    },

    // Control flow operations
    Branch {
        condition: Value,
        true_label: String,
        false_label: String,
    },
    Jump(String),  // Unconditional jump to label
    Label(String), // Label for jumps and branches

    // Comparison operations
    ICmp {
        op: String, // "eq", "ne", "slt", "sgt", "sle", "sge"
        result: Value,
        left: Value,
        right: Value,
    },
    FCmp {
        op: String, // "oeq", "one", "olt", "ogt", "ole", "oge"
        result: Value,
        left: Value,
        right: Value,
    },

    // I/O operations
    Print {
        format_string: String,
        arguments: Vec<Value>,
    },
    Println {
        format_string: String,
        arguments: Vec<Value>,
    },

    // Logical operations
    And {
        result: Value,
        left: Value,
        right: Value,
    },
    Or {
        result: Value,
        left: Value,
        right: Value,
    },
    Not {
        result: Value,
        operand: Value,
    },

    // Unary operations
    Neg {
        result: Value,
        operand: Value,
    },

    // Aggregate operations (Phase 4)
    AllocaArray {
        result: Value,     // pointer to array
        elem_type: String, // LLVM element type
        count: usize,      // number of elements
    },
    GetElementPtr {
        result: Value,     // pointer to element
        base: Value,       // base pointer
        index: Value,      // element index
        elem_type: String, // LLVM element type
    },
    /// Verified storage for a fixed array whose element is recursively admitted
    /// Copy data. The legacy variant name is retained for checked-IR compatibility;
    /// validity is defined by the recursive schema, not by a struct-only guard.
    CheckedCopyStructArrayAlloca {
        result: Value,
        element: LogicalType,
        count: usize,
    },
    /// Verified pointer to one element in a recursively checked fixed-array place.
    CheckedCopyStructArrayElementPtr {
        result: Value,
        base: Value,
        index: Value,
        element: LogicalType,
        count: usize,
    },
    AllocaStruct {
        result: Value,       // pointer to struct
        struct_type: String, // LLVM struct type name
    },
    GetFieldPtr {
        result: Value,       // pointer to field
        base: Value,         // struct pointer
        field_index: u32,    // field index
        struct_type: String, // LLVM struct type name
    },
    /// Verified storage for a source-admitted finite recursive Copy struct.
    CheckedStructAlloca {
        result: Value,
        struct_name: String,
        field_types: Vec<LogicalType>,
    },
    /// Verified pointer to one field in a `CheckedStructAlloca` place.
    CheckedStructFieldPtr {
        result: Value,
        base: Value,
        struct_name: String,
        field_index: u32,
        field_type: LogicalType,
    },
    /// Verified storage for one admitted recursive heterogeneous Copy tuple.
    CheckedTupleAlloca {
        result: Value,
        element_types: Vec<LogicalType>,
    },
    /// Verified pointer to one ordered field in a `CheckedTupleAlloca` place.
    CheckedTupleFieldPtr {
        result: Value,
        base: Value,
        element_types: Vec<LogicalType>,
        field_index: usize,
        field_type: LogicalType,
    },

    // Phase 6: Vec/Collection IR operations
    VecAlloca {
        result: Value,
        element_type: String,
    },
    VecPush {
        vec_ptr: Value,
        value: Value,
    },
    VecPop {
        result: Value,
        vec_ptr: Value,
    },
    VecLength {
        result: Value,
        vec_ptr: Value,
    },
    VecCapacity {
        result: Value,
        vec_ptr: Value,
    },
    VecAccess {
        result: Value,
        vec_ptr: Value,
        index: Value,
    },
    VecInit {
        result: Value,
        element_type: String,
        elements: Vec<Value>,
    },

    // Array aliases (used by stdlib ArrayOps)
    ArrayLength {
        result: Value,
        array_ptr: Value,
    },
    ArrayAccess {
        result: Value,
        array_ptr: Value,
        index: Value,
    },

    // Phase 6: Enum/ADT IR operations (for Result<T,E> and Option<T>)
    EnumDiscriminant {
        result: Value,
        enum_ptr: Value,
    },
    EnumVariantData {
        result: Value,
        enum_ptr: Value,
        variant_index: usize,
    },
    EnumConstruct {
        result: Value,
        enum_name: String,
        variant_name: String,
        variant_index: usize,
        data: Vec<Value>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Inst>,
    pub next_reg: u32,
    pub next_ptr: u32, // New field for unique pointer IDs
}

/// The source-level meaning carried by a value or storage location in checked IR.
///
/// Numeric locals deliberately retain their historical physical LLVM representation;
/// this type describes their logical contract and must not be used to infer a new
/// overflow, division, or aggregate policy.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum LogicalType {
    Int,
    Float,
    Bool,
    Void,
    String,
    ImmutableReference {
        pointee: Box<LogicalType>,
    },
    MutableReference {
        pointee: Box<LogicalType>,
    },
    Array {
        element: Box<LogicalType>,
        count: usize,
    },
    Struct {
        name: String,
        fields: Vec<LogicalType>,
    },
    Tuple {
        elements: Vec<LogicalType>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariantSchema>,
    },
}

impl LogicalType {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }
}

impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::Bool => write!(f, "Bool"),
            Self::Void => write!(f, "Void"),
            Self::String => write!(f, "String"),
            Self::ImmutableReference { pointee } => write!(f, "&{pointee}"),
            Self::MutableReference { pointee } => write!(f, "&mut {pointee}"),
            Self::Array { element, count } => write!(f, "Array<{element}; {count}>"),
            Self::Struct { name, fields } => {
                write!(f, "Struct<{name}; [")?;
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, "]>")
            }
            Self::Tuple { elements } => {
                write!(f, "Tuple<[")?;
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{element}")?;
                }
                write!(f, "]>")
            }
            Self::Enum { name, variants } => {
                write!(f, "Enum<{name}; [")?;
                for (index, variant) in variants.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", variant.name)?;
                    if let Some(payload) = &variant.payload {
                        write!(f, "({payload})")?;
                    }
                }
                write!(f, "]>")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct ResultId(pub u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct PlaceId(pub u32);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionSignature {
    pub parameters: Vec<(String, LogicalType)>,
    pub result: LogicalType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlaceMetadata {
    pub id: PlaceId,
    pub name: Option<String>,
    pub pointee: LogicalType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockMetadata {
    pub label: String,
    pub reachable: bool,
    pub successors: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionMetadata {
    pub signature: FunctionSignature,
    pub results: BTreeMap<ResultId, LogicalType>,
    pub places: BTreeMap<PlaceId, PlaceMetadata>,
    pub blocks: Vec<BlockMetadata>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct IrMetadata {
    pub functions: BTreeMap<String, FunctionMetadata>,
}

pub(crate) type RawIr = HashMap<String, Function>;

/// Checked IR keeps the legacy raw instruction map intact while attaching the
/// verifier-derived logical contract. This avoids changing the shape or behavior of
/// the deprecated unchecked APIs.
#[derive(Debug, PartialEq, Clone)]
pub struct CheckedIr {
    raw: RawIr,
    metadata: IrMetadata,
}

impl CheckedIr {
    pub(crate) fn new(raw: RawIr, metadata: IrMetadata) -> Self {
        Self { raw, metadata }
    }

    pub(crate) fn raw(&self) -> &RawIr {
        &self.raw
    }

    pub(crate) fn into_raw(self) -> RawIr {
        self.raw
    }

    pub fn metadata(&self) -> &IrMetadata {
        &self.metadata
    }
}

impl From<CheckedIr> for RawIr {
    fn from(checked: CheckedIr) -> Self {
        checked.into_raw()
    }
}

/// Raw private IR is accepted at the checked codegen boundary, but it carries no
/// trusted metadata. The mandatory verifier derives that metadata before emission.
impl From<RawIr> for CheckedIr {
    fn from(raw: RawIr) -> Self {
        Self {
            raw,
            metadata: IrMetadata::default(),
        }
    }
}
