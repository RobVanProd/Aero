// src/compiler/src/ir.rs

use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Reg(u32),
    ImmInt(i64),
    ImmFloat(f64),
    ImmChar(char),
    ImmString(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Reg(r) => write!(f, "{}", r),
            Value::ImmInt(n) => write!(f, "{}", n),
            Value::ImmFloat(fl) => write!(f, "{}", fl),
            Value::ImmChar(character) => write!(f, "U+{:04X}", u32::from(*character)),
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
    /// Verified mutable stack place for an admitted local CopyData or owned-enum binding.
    CheckedMutableOwnedPlaceAlloca {
        result: Value,
        name: String,
        ty: LogicalType,
    },
    /// Verified immutable owner storage for an admitted enum that may be observed
    /// through a non-escaping immutable reference.
    CheckedImmutableEnumOwnerPlaceAlloca {
        result: Value,
        name: String,
        schema: EnumSchema,
    },
    /// Verified compiler-owned storage for one exhaustive admitted Match result.
    /// The place begins uninitialized and every reachable dispatch arm must write one
    /// exact-type value before the single merged load.
    CheckedMatchResultPlaceAlloca {
        result: Value,
        result_type: LogicalType,
        dispatch_schema: EnumSchema,
    },
    /// Verified whole-value reassignment of an existing admitted mutable owned place.
    CheckedOwnedPlaceAssignment {
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
    /// Verified lexical end of a read-only enum alias whose source is a mutable owner.
    CheckedMutableOwnerImmutableEnumBorrowEnd {
        reference: Value,
        source: Value,
        schema: EnumSchema,
    },
    /// Verified non-owning enum observation used only by its adjacent exhaustive Match.
    CheckedImmutableEnumMatchRead {
        result: Value,
        reference: Value,
        schema: EnumSchema,
    },
    /// Verified exclusive enum observation used only by its adjacent exhaustive Match.
    CheckedMutableEnumMatchRead {
        result: Value,
        reference: Value,
        schema: EnumSchema,
    },
    /// Verified exclusive non-escaping alias of a mutable Copy-data owner or reference place.
    CheckedMutableBorrow {
        result: Value,
        source: Value,
        pointee: LogicalType,
    },
    /// Verified immediate call-only alias of a typed field/tuple/fixed-array place.
    /// `root` is the complete local owner conservatively loaned for the call;
    /// `source` is the independently typed projected place passed to the callee.
    CheckedProjectedBorrow {
        result: Value,
        root: Value,
        source: Value,
        root_type: LogicalType,
        pointee: LogicalType,
        mutable: bool,
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
    /// Verified end of one immediate projected call loan.
    CheckedProjectedBorrowEnd {
        reference: Value,
        root: Value,
        source: Value,
        root_type: LogicalType,
        pointee: LogicalType,
        mutable: bool,
    },
    /// Verified creation of one allocation-free local byte-buffer owner.
    CheckedByteBufferNew {
        result: Value,
        name: String,
    },
    /// Verified transfer of one byte-buffer resource to a new local owner place.
    CheckedByteBufferMove {
        result: Value,
        source: Value,
        name: String,
    },
    /// Verified non-escaping shared loan of one live byte-buffer owner.
    CheckedByteBufferImmutableBorrow {
        result: Value,
        source: Value,
    },
    /// Verified exact end of one shared byte-buffer loan.
    CheckedByteBufferImmutableBorrowEnd {
        reference: Value,
        source: Value,
    },
    /// Verified non-escaping exclusive loan of one live byte-buffer owner.
    CheckedByteBufferMutableBorrow {
        result: Value,
        source: Value,
    },
    /// Verified exact end of one exclusive byte-buffer loan.
    CheckedByteBufferMutableBorrowEnd {
        reference: Value,
        source: Value,
    },
    /// Push one checked i32 byte through an active exclusive loan. The result is
    /// the new positive length or the private R1B failure sentinel -1, -2, or -3.
    CheckedByteBufferPush {
        result: Value,
        reference: Value,
        byte: Value,
    },
    /// Read the nonnegative exact length through an active shared loan.
    CheckedByteBufferLength {
        result: Value,
        reference: Value,
    },
    /// Read the nonnegative exact capacity through an active shared loan.
    CheckedByteBufferCapacity {
        result: Value,
        reference: Value,
    },
    /// Read one initialized byte through an active shared loan. The result is
    /// 0..=255 or the private R1B out-of-bounds sentinel -4.
    CheckedByteBufferGet {
        result: Value,
        reference: Value,
        index: Value,
    },
    /// Destroy one live byte-buffer owner exactly once.
    CheckedByteBufferDrop {
        owner: Value,
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
    /// Verified construction of one positional variant with two or more ordered
    /// recursive CopyData fields. Unary construction retains `CheckedEnumVariant`
    /// so its accepted checked-IR and LLVM identity does not drift.
    CheckedEnumVariantFields {
        result: Value,
        schema: EnumSchema,
        variant_index: usize,
        fields: Vec<Value>,
    },
    /// Verified extraction of the selected unary recursive CopyData payload.
    CheckedEnumPayload {
        result: Value,
        value: Value,
        schema: EnumSchema,
        variant_index: usize,
    },
    /// Verified extraction of one declaration-ordered field from a selected
    /// positional multi-field variant.
    CheckedEnumField {
        result: Value,
        value: Value,
        schema: EnumSchema,
        variant_index: usize,
        field_index: usize,
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
    Char,
    Void,
    String,
    /// Private checked resource used by the R1 byte-storage substrate. It is not
    /// CopyData and is unavailable to source syntax until a later R1C checkpoint.
    ByteBuffer,
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
    /// Private product lane for two-or-more positional enum fields. This is not a
    /// source tuple type: it preserves the distinction between `V((A, B))` and
    /// `V(A, B)` in checked schema identity.
    EnumFields {
        fields: Vec<LogicalType>,
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
            Self::Char => write!(f, "Char"),
            Self::Void => write!(f, "Void"),
            Self::String => write!(f, "String"),
            Self::ByteBuffer => write!(f, "ByteBuffer"),
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
            Self::EnumFields { fields } => {
                write!(f, "EnumFields<[")?;
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, "]>")
            }
            Self::Enum { name, variants } => {
                let display_name =
                    crate::builtin_carrier_contract::private_carrier_source_name(name)
                        .or_else(|| {
                            crate::generic_enum_contract::private_generic_enum_source_name(name)
                        })
                        .unwrap_or_else(|| name.clone());
                write!(f, "Enum<{display_name}; [")?;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct ByteBufferId(pub u32);

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum ByteBufferPlaceRole {
    Owner { moved_from: Option<PlaceId> },
    ImmutableLoan { owner: PlaceId },
    MutableLoan { owner: PlaceId },
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct ByteBufferPlaceMetadata {
    pub place: PlaceId,
    pub identity: ByteBufferId,
    pub role: ByteBufferPlaceRole,
}

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

#[derive(PartialEq, Eq, Clone)]
pub struct FunctionMetadata {
    pub signature: FunctionSignature,
    pub results: BTreeMap<ResultId, LogicalType>,
    pub places: BTreeMap<PlaceId, PlaceMetadata>,
    pub blocks: Vec<BlockMetadata>,
    pub byte_buffers: BTreeMap<PlaceId, ByteBufferPlaceMetadata>,
}

impl fmt::Debug for FunctionMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("FunctionMetadata");
        debug
            .field("signature", &self.signature)
            .field("results", &self.results)
            .field("places", &self.places)
            .field("blocks", &self.blocks);
        if !self.byte_buffers.is_empty() {
            debug.field("byte_buffers", &self.byte_buffers);
        }
        debug.finish()
    }
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
