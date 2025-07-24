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
    
    // Function operations
    FunctionDef {
        name: String,
        parameters: Vec<(String, String)>, // (name, type)
        return_type: Option<String>,
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
    Jump(String), // Unconditional jump to label
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
    
    // Struct operations
    StructDef {
        name: String,
        fields: Vec<(String, String)>, // (field_name, field_type)
        is_tuple: bool,
    },
    StructAlloca {
        result: Value,
        struct_name: String,
    },
    StructInit {
        result: Value,
        struct_name: String,
        field_values: Vec<(String, Value)>, // (field_name, value)
    },
    FieldAccess {
        result: Value,
        struct_ptr: Value,
        field_name: String,
        field_index: usize,
    },
    FieldStore {
        struct_ptr: Value,
        field_name: String,
        field_index: usize,
        value: Value,
    },
    StructCopy {
        result: Value,
        source: Value,
        struct_name: String,
    },
    
    // Enum operations
    EnumDef {
        name: String,
        variants: Vec<(String, Option<Vec<String>>)>, // (variant_name, optional_data_types)
        discriminant_type: String,
    },
    EnumAlloca {
        result: Value,
        enum_name: String,
    },
    EnumConstruct {
        result: Value,
        enum_name: String,
        variant_name: String,
        variant_index: usize,
        data_values: Vec<Value>, // Values for variant data
    },
    EnumDiscriminant {
        result: Value,
        enum_ptr: Value,
    },
    EnumExtract {
        result: Value,
        enum_ptr: Value,
        variant_index: usize,
        data_index: usize, // Index of data within variant
    },
    
    // Pattern matching operations
    Match {
        discriminant: Value,
        arms: Vec<MatchArm>,
        default_label: Option<String>,
    },
    PatternCheck {
        result: Value,
        discriminant: Value,
        expected_variant: usize,
    },
    
    // Switch-like instruction for efficient pattern matching
    Switch {
        discriminant: Value,
        cases: Vec<(i64, String)>, // (variant_index, label)
        default_label: String,
    },
    
    // Array and collection operations
    ArrayAlloca {
        result: Value,
        element_type: String,
        size: Value, // Size can be dynamic
    },
    ArrayInit {
        result: Value,
        element_type: String,
        elements: Vec<Value>,
    },
    ArrayAccess {
        result: Value,
        array_ptr: Value,
        index: Value,
    },
    ArrayStore {
        array_ptr: Value,
        index: Value,
        value: Value,
    },
    ArrayLength {
        result: Value,
        array_ptr: Value,
    },
    BoundsCheck {
        array_ptr: Value,
        index: Value,
        success_label: String,
        failure_label: String,
    },
    
    // Vec operations
    VecAlloca {
        result: Value,
        element_type: String,
    },
    VecInit {
        result: Value,
        element_type: String,
        elements: Vec<Value>,
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
    
    // Generic type operations
    GenericInstantiate {
        result: Value,
        base_type: String,
        type_args: Vec<String>,
        instantiated_name: String,
    },
    GenericMethodCall {
        result: Option<Value>,
        object: Value,
        method: String,
        type_args: Vec<String>,
        arguments: Vec<Value>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern_checks: Vec<PatternCheck>,
    pub bindings: Vec<(String, Value)>, // Variable bindings from pattern
    pub guard: Option<Value>, // Optional guard condition
    pub body_label: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PatternCheck {
    pub check_type: PatternCheckType,
    pub target: Value,
    pub expected: PatternValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternCheckType {
    VariantMatch,
    LiteralMatch,
    RangeMatch,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternValue {
    Variant(usize),
    Literal(Value),
    Range(Value, Value, bool), // start, end, inclusive
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Inst>,
    pub next_reg: u32,
    pub next_ptr: u32, // New field for unique pointer IDs
}


