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
    Return(Value),             // value to return
    SIToFP(Value, Value),      // result_reg, int_value (signed integer to floating point)
    FPToSI(Value, Value),      // result_reg, float_value (floating point to signed integer)

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
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Inst>,
    pub next_reg: u32,
    pub next_ptr: u32, // New field for unique pointer IDs
}
