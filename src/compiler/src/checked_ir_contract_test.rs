use crate::ir::{Function, Inst, Value};
use crate::{CodeGenerationError, CodeGenerator, IrVerificationError, try_generate_code};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

type RawIr = HashMap<String, Function>;

fn function(name: &str, body: Vec<Inst>) -> Function {
    Function {
        name: name.to_string(),
        body,
        next_reg: 32,
        next_ptr: 16,
    }
}

fn single_function(name: &str, body: Vec<Inst>) -> RawIr {
    HashMap::from([(name.to_string(), function(name, body))])
}

fn program_with_definition(
    callee_name: &str,
    parameters: Vec<(String, String)>,
    return_type: Option<&str>,
    callee_body: Vec<Inst>,
    mut main_body: Vec<Inst>,
) -> RawIr {
    main_body.insert(
        0,
        Inst::FunctionDef {
            name: callee_name.to_string(),
            parameters,
            return_type: return_type.map(str::to_string),
            body: callee_body,
        },
    );
    HashMap::from([
        ("main".to_string(), function("main", main_body)),
        (callee_name.to_string(), function(callee_name, Vec::new())),
    ])
}

fn int_call_ir(arguments: Vec<Value>, result: Option<Value>) -> RawIr {
    program_with_definition(
        "takes_int",
        vec![("value".to_string(), "i32".to_string())],
        Some("i32"),
        vec![Inst::Return(Value::ImmInt(0))],
        vec![
            Inst::Call {
                function: "takes_int".to_string(),
                arguments,
                result,
            },
            Inst::Return(Value::ImmInt(0)),
        ],
    )
}

fn void_call_with_result_ir() -> RawIr {
    program_with_definition(
        "notify",
        Vec::new(),
        None,
        // This is the legacy raw-IR representation emitted for a source-level void
        // function. The checked representation may adapt it, but the call itself must
        // remain the sole invalid instruction in this probe.
        vec![Inst::Return(Value::ImmInt(0))],
        vec![
            Inst::Call {
                function: "notify".to_string(),
                arguments: Vec::new(),
                result: Some(Value::Reg(0)),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
    )
}

fn void_operand_ir() -> RawIr {
    program_with_definition(
        "bad_void_return",
        Vec::new(),
        None,
        vec![Inst::Return(Value::ImmInt(7))],
        vec![Inst::Return(Value::ImmInt(0))],
    )
}

struct InvalidIrCase {
    name: &'static str,
    expected_fragments: &'static [&'static str],
    ir: RawIr,
}

fn invalid_ir_cases() -> Vec<InvalidIrCase> {
    vec![
        InvalidIrCase {
            name: "empty module has no process entry",
            expected_fragments: &["process", "entry", "main", "missing"],
            ir: HashMap::new(),
        },
        InvalidIrCase {
            name: "helper-only module has no process entry",
            expected_fragments: &["process", "entry", "main", "missing"],
            ir: single_function("helper", vec![Inst::Return(Value::ImmInt(0))]),
        },
        InvalidIrCase {
            name: "printf is reserved by the runtime ABI",
            expected_fragments: &["printf", "reserved", "runtime", "abi"],
            ir: HashMap::from([
                (
                    "main".to_string(),
                    function("main", vec![Inst::Return(Value::ImmInt(0))]),
                ),
                (
                    "printf".to_string(),
                    function("printf", vec![Inst::Return(Value::ImmInt(0))]),
                ),
            ]),
        },
        InvalidIrCase {
            name: "function name is not an LLVM-safe admitted symbol",
            expected_fragments: &["function", "symbol", "bad name", "not", "admitted"],
            ir: HashMap::from([
                (
                    "main".to_string(),
                    function("main", vec![Inst::Return(Value::ImmInt(0))]),
                ),
                (
                    "bad name".to_string(),
                    function("bad name", vec![Inst::Return(Value::ImmInt(0))]),
                ),
            ]),
        },
        InvalidIrCase {
            name: "parameter name is not an LLVM-safe admitted symbol",
            expected_fragments: &["parameter", "symbol", "bad name", "not", "admitted"],
            ir: program_with_definition(
                "bad_parameter",
                vec![("bad name".to_string(), "i32".to_string())],
                Some("i32"),
                vec![Inst::Return(Value::ImmInt(0))],
                vec![Inst::Return(Value::ImmInt(0))],
            ),
        },
        InvalidIrCase {
            name: "block label collides with generated result namespace",
            expected_fragments: &["block", "label", "symbol", "reg0", "not", "admitted"],
            ir: single_function(
                "main",
                vec![
                    Inst::Jump("reg0".to_string()),
                    Inst::Label("reg0".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "raw constant integer division by zero",
            expected_fragments: &["constant", "integer", "division", "zero", "not", "admitted"],
            ir: single_function(
                "main",
                vec![
                    Inst::Div(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(0)),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "function map key disagrees with body name",
            expected_fragments: &["map", "key", "disagrees", "body", "name"],
            ir: HashMap::from([(
                "alias".to_string(),
                function("main", vec![Inst::Return(Value::ImmInt(0))]),
            )]),
        },
        InvalidIrCase {
            name: "empty emitted function body",
            expected_fragments: &["reachable", "terminator", "missing"],
            ir: single_function("main", Vec::new()),
        },
        InvalidIrCase {
            name: "function definition has no module entry",
            expected_fragments: &["definition", "no", "matching", "module", "entry"],
            ir: single_function(
                "main",
                vec![
                    Inst::FunctionDef {
                        name: "orphan".to_string(),
                        parameters: Vec::new(),
                        return_type: Some("i32".to_string()),
                        body: vec![Inst::Return(Value::ImmInt(0))],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "emitted definition entry mixes ignored runtime instructions",
            expected_fragments: &["module", "entry", "mixes", "runtime", "instructions"],
            ir: HashMap::from([
                (
                    "main".to_string(),
                    function(
                        "main",
                        vec![
                            Inst::FunctionDef {
                                name: "helper".to_string(),
                                parameters: Vec::new(),
                                return_type: Some("i32".to_string()),
                                body: vec![Inst::Return(Value::ImmInt(0))],
                            },
                            Inst::Return(Value::ImmInt(0)),
                        ],
                    ),
                ),
                (
                    "helper".to_string(),
                    function("helper", vec![Inst::Return(Value::ImmInt(0))]),
                ),
            ]),
        },
        InvalidIrCase {
            name: "string parameter signature",
            expected_fragments: &["unsupported", "string", "parameter"],
            ir: program_with_definition(
                "takes_string",
                vec![("value".to_string(), "string".to_string())],
                Some("i32"),
                vec![Inst::Return(Value::ImmInt(0))],
                vec![Inst::Return(Value::ImmInt(0))],
            ),
        },
        InvalidIrCase {
            name: "duplicate parameter names",
            expected_fragments: &["duplicate", "parameter", "value"],
            ir: program_with_definition(
                "duplicates",
                vec![
                    ("value".to_string(), "i32".to_string()),
                    ("value".to_string(), "i32".to_string()),
                ],
                Some("i32"),
                vec![Inst::Return(Value::ImmInt(0))],
                vec![Inst::Return(Value::ImmInt(0))],
            ),
        },
        InvalidIrCase {
            name: "invalid process entry signature",
            expected_fragments: &["process", "entry", "exact", "i32", "main"],
            ir: HashMap::from([(
                "main".to_string(),
                function(
                    "main",
                    vec![Inst::FunctionDef {
                        name: "main".to_string(),
                        parameters: vec![("argument".to_string(), "i32".to_string())],
                        return_type: Some("i32".to_string()),
                        body: vec![Inst::Return(Value::ImmInt(0))],
                    }],
                ),
            )]),
        },
        InvalidIrCase {
            name: "integer immediate outside i32",
            expected_fragments: &["integer", "outside", "i32", "range"],
            ir: single_function("main", vec![Inst::Return(Value::ImmInt(i64::MAX))]),
        },
        InvalidIrCase {
            name: "duplicate result definition",
            expected_fragments: &["duplicate", "result", "definition"],
            ir: single_function(
                "main",
                vec![
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Sub(Value::Reg(0), Value::ImmInt(4), Value::ImmInt(3)),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "undefined value use",
            expected_fragments: &["undefined", "result", "use"],
            ir: single_function("main", vec![Inst::Return(Value::Reg(99))]),
        },
        InvalidIrCase {
            name: "use before later definition",
            expected_fragments: &["use", "before", "definition"],
            ir: single_function(
                "main",
                vec![
                    Inst::Add(Value::Reg(0), Value::Reg(1), Value::ImmInt(2)),
                    Inst::Add(Value::Reg(1), Value::ImmInt(3), Value::ImmInt(4)),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "cross-block result does not dominate merge use",
            expected_fragments: &["result", "dominat"],
            ir: single_function(
                "main",
                vec![
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Branch {
                        condition: Value::Reg(0),
                        true_label: "defines".to_string(),
                        false_label: "skips".to_string(),
                    },
                    Inst::Label("defines".to_string()),
                    Inst::Add(Value::Reg(1), Value::ImmInt(2), Value::ImmInt(3)),
                    Inst::Jump("merge".to_string()),
                    Inst::Label("skips".to_string()),
                    Inst::Jump("merge".to_string()),
                    Inst::Label("merge".to_string()),
                    Inst::Return(Value::Reg(1)),
                ],
            ),
        },
        InvalidIrCase {
            name: "result and place identifier collision",
            expected_fragments: &["identifier", "place", "result"],
            ir: single_function(
                "main",
                vec![
                    Inst::Alloca(Value::Reg(0), "slot".to_string()),
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "unused scalar place has no inferred pointee",
            expected_fragments: &["place", "logical", "pointee"],
            ir: single_function(
                "main",
                vec![
                    Inst::Alloca(Value::Reg(0), "unused".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "load from non-place",
            expected_fragments: &["load", "place"],
            ir: single_function(
                "main",
                vec![
                    Inst::Load(Value::Reg(0), Value::ImmInt(7)),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "place use before later definition",
            expected_fragments: &["place", "use", "before", "definition"],
            ir: single_function(
                "main",
                vec![
                    Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                    Inst::Alloca(Value::Reg(0), "late".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "cross-block place does not dominate merge use",
            expected_fragments: &["place", "definition", "dominat", "use"],
            ir: single_function(
                "main",
                vec![
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Branch {
                        condition: Value::Reg(0),
                        true_label: "defines".to_string(),
                        false_label: "skips".to_string(),
                    },
                    Inst::Label("defines".to_string()),
                    Inst::Alloca(Value::Reg(1), "branch_local".to_string()),
                    Inst::Store(Value::Reg(1), Value::ImmInt(7)),
                    Inst::Jump("merge".to_string()),
                    Inst::Label("skips".to_string()),
                    Inst::Jump("merge".to_string()),
                    Inst::Label("merge".to_string()),
                    Inst::Load(Value::Reg(2), Value::Reg(1)),
                    Inst::Return(Value::Reg(2)),
                ],
            ),
        },
        InvalidIrCase {
            name: "string stored into numeric place",
            expected_fragments: &["store", "string", "numeric"],
            ir: single_function(
                "main",
                vec![
                    Inst::Alloca(Value::Reg(0), "number".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmString("wrong".to_string())),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "boolean result stored into numeric place",
            expected_fragments: &["store", "bool", "numeric"],
            ir: single_function(
                "main",
                vec![
                    Inst::Alloca(Value::Reg(0), "number".to_string()),
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(1),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Store(Value::Reg(0), Value::Reg(1)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "operator operand type mismatch",
            expected_fragments: &["add", "operand", "type"],
            ir: single_function(
                "main",
                vec![
                    Inst::Add(
                        Value::Reg(0),
                        Value::ImmString("not-a-number".to_string()),
                        Value::ImmInt(1),
                    ),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "call argument type mismatch",
            expected_fragments: &["call", "argument", "type"],
            ir: int_call_ir(
                vec![Value::ImmString("not-an-int".to_string())],
                Some(Value::Reg(0)),
            ),
        },
        InvalidIrCase {
            name: "non-void call is missing its result",
            expected_fragments: &["call", "result", "missing"],
            ir: int_call_ir(vec![Value::ImmInt(1)], None),
        },
        InvalidIrCase {
            name: "call result is not a result identifier",
            expected_fragments: &["call", "result", "identifier"],
            ir: int_call_ir(vec![Value::ImmInt(1)], Some(Value::ImmInt(7))),
        },
        InvalidIrCase {
            name: "void call has a result",
            expected_fragments: &["void", "call", "result"],
            ir: void_call_with_result_ir(),
        },
        InvalidIrCase {
            name: "void value used as an operand",
            expected_fragments: &["void", "operand"],
            ir: void_operand_ir(),
        },
        InvalidIrCase {
            name: "return type mismatch",
            expected_fragments: &["return", "string", "i32"],
            ir: single_function(
                "main",
                vec![Inst::Return(Value::ImmString("not-i32".to_string()))],
            ),
        },
        InvalidIrCase {
            name: "non-boolean branch condition",
            expected_fragments: &["branch", "condition", "bool"],
            ir: single_function(
                "main",
                vec![
                    Inst::Branch {
                        condition: Value::ImmInt(1),
                        true_label: "yes".to_string(),
                        false_label: "no".to_string(),
                    },
                    Inst::Label("yes".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("no".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "invalid integer comparison predicate",
            expected_fragments: &["icmp", "predicate", "unordered", "not", "admitted"],
            ir: single_function(
                "main",
                vec![
                    Inst::ICmp {
                        op: "unordered".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "invalid floating comparison predicate",
            expected_fragments: &["fcmp", "predicate", "eq", "not", "admitted"],
            ir: single_function(
                "main",
                vec![
                    Inst::FCmp {
                        op: "eq".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmFloat(1.0),
                        right: Value::ImmFloat(1.0),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "undefined jump target",
            expected_fragments: &["jump", "target", "missing"],
            ir: single_function("main", vec![Inst::Jump("missing".to_string())]),
        },
        InvalidIrCase {
            name: "undefined branch target",
            expected_fragments: &["branch", "target", "missing"],
            ir: single_function(
                "main",
                vec![
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Branch {
                        condition: Value::Reg(0),
                        true_label: "present".to_string(),
                        false_label: "missing".to_string(),
                    },
                    Inst::Label("present".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "duplicate label definition",
            expected_fragments: &["duplicate", "label", "same"],
            ir: single_function(
                "main",
                vec![
                    Inst::Jump("same".to_string()),
                    Inst::Label("same".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("same".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "getelementptr base is not a place",
            expected_fragments: &["getelementptr", "base", "place"],
            ir: single_function(
                "main",
                vec![
                    Inst::GetElementPtr {
                        result: Value::Reg(0),
                        base: Value::ImmInt(5),
                        index: Value::ImmInt(0),
                        elem_type: "double".to_string(),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "getelementptr index is not an integer",
            expected_fragments: &["getelementptr", "index", "integer"],
            ir: single_function(
                "main",
                vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "double".to_string(),
                        count: 1,
                    },
                    Inst::GetElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmString("bad-index".to_string()),
                        elem_type: "[1 x double]".to_string(),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "array allocation uses backend-incompatible physical element",
            expected_fragments: &["physical", "array", "element", "i32", "double"],
            ir: single_function(
                "main",
                vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "i32".to_string(),
                        count: 1,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "boolean value cannot refine a physical double array element",
            expected_fragments: &["store", "Bool", "numeric", "place"],
            ir: single_function(
                "main",
                vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "double".to_string(),
                        count: 1,
                    },
                    Inst::GetElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmInt(0),
                        elem_type: "[1 x double]".to_string(),
                    },
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(2),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Store(Value::Reg(1), Value::Reg(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "getelementptr uses scalar rather than aggregate descriptor",
            expected_fragments: &["getelementptr", "element", "type", "double"],
            ir: single_function(
                "main",
                vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "double".to_string(),
                        count: 1,
                    },
                    Inst::GetElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmInt(0),
                        elem_type: "double".to_string(),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "getelementptr element type disagrees with base",
            expected_fragments: &["getelementptr", "element", "type"],
            ir: single_function(
                "main",
                vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "double".to_string(),
                        count: 1,
                    },
                    Inst::GetElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmInt(0),
                        elem_type: "i1".to_string(),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "reachable block lacks terminator",
            expected_fragments: &["reachable", "terminator", "missing"],
            ir: single_function(
                "main",
                vec![Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2))],
            ),
        },
        InvalidIrCase {
            name: "reachable block terminator is not final",
            expected_fragments: &["reachable", "terminator", "final"],
            ir: single_function(
                "main",
                vec![
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                ],
            ),
        },
        InvalidIrCase {
            name: "unreachable block terminator is not final",
            expected_fragments: &["unreachable", "terminator", "final"],
            ir: single_function(
                "main",
                vec![
                    Inst::Jump("exit".to_string()),
                    Inst::Label("dead".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Label("exit".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
        InvalidIrCase {
            name: "unreachable block lacks terminator",
            expected_fragments: &["unreachable", "terminator", "missing"],
            ir: single_function(
                "main",
                vec![
                    Inst::Jump("exit".to_string()),
                    Inst::Label("dead".to_string()),
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Label("exit".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        },
    ]
}

fn unsupported_instruction_cases() -> Vec<InvalidIrCase> {
    let unsupported = [
        (
            "AllocaStruct",
            &["unsupported", "alloca", "struct"] as &'static [&'static str],
            Inst::AllocaStruct {
                result: Value::Reg(0),
                struct_type: "Point".to_string(),
            },
        ),
        (
            "GetFieldPtr",
            &["unsupported", "field", "pointer"],
            Inst::GetFieldPtr {
                result: Value::Reg(0),
                base: Value::ImmInt(0),
                field_index: 0,
                struct_type: "Point".to_string(),
            },
        ),
        (
            "VecAlloca",
            &["unsupported", "vec", "alloca"],
            Inst::VecAlloca {
                result: Value::Reg(0),
                element_type: "double".to_string(),
            },
        ),
        (
            "VecPush",
            &["unsupported", "vec", "push"],
            Inst::VecPush {
                vec_ptr: Value::ImmInt(0),
                value: Value::ImmInt(1),
            },
        ),
        (
            "VecPop",
            &["unsupported", "vec", "pop"],
            Inst::VecPop {
                result: Value::Reg(0),
                vec_ptr: Value::ImmInt(0),
            },
        ),
        (
            "VecLength",
            &["unsupported", "vec", "length"],
            Inst::VecLength {
                result: Value::Reg(0),
                vec_ptr: Value::ImmInt(0),
            },
        ),
        (
            "VecCapacity",
            &["unsupported", "vec", "capacity"],
            Inst::VecCapacity {
                result: Value::Reg(0),
                vec_ptr: Value::ImmInt(0),
            },
        ),
        (
            "VecAccess",
            &["unsupported", "vec", "access"],
            Inst::VecAccess {
                result: Value::Reg(0),
                vec_ptr: Value::ImmInt(0),
                index: Value::ImmInt(0),
            },
        ),
        (
            "VecInit",
            &["unsupported", "vec", "init"],
            Inst::VecInit {
                result: Value::Reg(0),
                element_type: "double".to_string(),
                elements: vec![Value::ImmInt(1)],
            },
        ),
        (
            "ArrayLength",
            &["unsupported", "array", "length"],
            Inst::ArrayLength {
                result: Value::Reg(0),
                array_ptr: Value::ImmInt(0),
            },
        ),
        (
            "ArrayAccess",
            &["unsupported", "array", "access"],
            Inst::ArrayAccess {
                result: Value::Reg(0),
                array_ptr: Value::ImmInt(0),
                index: Value::ImmInt(0),
            },
        ),
        (
            "EnumDiscriminant",
            &["unsupported", "enum", "discriminant"],
            Inst::EnumDiscriminant {
                result: Value::Reg(0),
                enum_ptr: Value::ImmInt(0),
            },
        ),
        (
            "EnumVariantData",
            &["unsupported", "enum", "variant", "data"],
            Inst::EnumVariantData {
                result: Value::Reg(0),
                enum_ptr: Value::ImmInt(0),
                variant_index: 0,
            },
        ),
        (
            "EnumConstruct",
            &["unsupported", "enum", "construct"],
            Inst::EnumConstruct {
                result: Value::Reg(0),
                enum_name: "Ghost".to_string(),
                variant_name: "Missing".to_string(),
                variant_index: 0,
                data: Vec::new(),
            },
        ),
    ];

    unsupported
        .into_iter()
        .map(|(name, expected_fragments, instruction)| InvalidIrCase {
            name,
            expected_fragments,
            ir: single_function("main", vec![instruction, Inst::Return(Value::ImmInt(0))]),
        })
        .collect()
}

fn assert_ir_verification_error(
    label: &str,
    expected_fragments: &[&str],
    error: CodeGenerationError,
) {
    let verification_error: IrVerificationError = match error {
        CodeGenerationError::IrVerification(error) => error,
        other => panic!("{label}: expected IR verification identity, received {other:?}"),
    };
    let diagnostic = verification_error.to_string().to_ascii_lowercase();
    assert!(
        expected_fragments
            .iter()
            .all(|fragment| diagnostic.contains(&fragment.to_ascii_lowercase())),
        "{label}: diagnostic `{diagnostic}` did not contain every required fragment {expected_fragments:?}"
    );
}

fn assert_checked_codegen_rejects(
    label: &str,
    expected_fragments: &[&str],
    result: Result<String, CodeGenerationError>,
) {
    match result {
        Ok(llvm) => panic!(
            "{label}: checked codegen returned partial/error-text IR instead of an error:\n{llvm}"
        ),
        Err(error) => assert_ir_verification_error(label, expected_fragments, error),
    }
}

fn assert_both_checked_codegen_entrypoints_reject(case: InvalidIrCase) {
    let free = catch_unwind(AssertUnwindSafe(|| try_generate_code(case.ir.clone())))
        .unwrap_or_else(|_| panic!("{}: free checked codegen unwound", case.name));
    assert_checked_codegen_rejects(case.name, case.expected_fragments, free);

    let method = catch_unwind(AssertUnwindSafe(|| {
        CodeGenerator::new().try_generate_code(case.ir)
    }))
    .unwrap_or_else(|_| panic!("{}: method checked codegen unwound", case.name));
    assert_checked_codegen_rejects(case.name, case.expected_fragments, method);
}

#[test]
fn checked_codegen_reverifies_each_malformed_private_raw_ir_invariant() {
    for case in invalid_ir_cases() {
        assert_both_checked_codegen_entrypoints_reject(case);
    }
}

#[test]
fn checked_codegen_rejects_every_unadmitted_instruction_variant_explicitly() {
    for case in unsupported_instruction_cases() {
        assert_both_checked_codegen_entrypoints_reject(case);
    }
}

#[test]
fn checked_codegen_emits_admitted_bool_comparison_and_i32_float_conversion_forms() {
    let boolean_ir = single_function(
        "main",
        vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmInt(1),
                right: Value::ImmInt(1),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(1),
                left: Value::Reg(0),
                right: Value::Reg(0),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
    );
    let boolean_llvm = try_generate_code(boolean_ir).expect("Bool equality must be admitted");
    assert!(
        boolean_llvm.contains("icmp eq i1"),
        "Bool equality was not emitted with i1 operands:\n{boolean_llvm}"
    );

    let conversion_ir = single_function(
        "main",
        vec![
            Inst::FPToSI(Value::Reg(0), Value::ImmFloat(3.5)),
            Inst::Return(Value::Reg(0)),
        ],
    );
    let conversion_llvm =
        try_generate_code(conversion_ir).expect("Float-to-Int conversion must be admitted");
    for marker in ["fptosi double", "to i32", "sitofp i32"] {
        assert!(
            conversion_llvm.contains(marker),
            "checked Float-to-Int conversion missed `{marker}`:\n{conversion_llvm}"
        );
    }
    assert!(
        !conversion_llvm.contains("to i64"),
        "checked Float-to-Int conversion leaked the legacy i64 form:\n{conversion_llvm}"
    );
}

#[test]
fn checked_codegen_does_not_append_a_return_after_an_infinite_loop_terminator() {
    let ir = single_function(
        "main",
        vec![
            Inst::Jump("loop".to_string()),
            Inst::Label("loop".to_string()),
            Inst::Jump("loop".to_string()),
        ],
    );
    let llvm = try_generate_code(ir).expect("well-formed infinite loop must be admitted");
    assert!(
        !llvm.contains("ret i32"),
        "checked codegen appended a second terminator after an infinite loop:\n{llvm}"
    );
}

#[test]
fn checked_codegen_uses_verified_raw_function_abi_for_calls() {
    let ir = HashMap::from([
        (
            "main".to_string(),
            function(
                "main",
                vec![
                    Inst::Call {
                        function: "helper".to_string(),
                        arguments: Vec::new(),
                        result: Some(Value::Reg(0)),
                    },
                    Inst::Return(Value::Reg(0)),
                ],
            ),
        ),
        (
            "helper".to_string(),
            function("helper", vec![Inst::Return(Value::ImmInt(7))]),
        ),
    ]);
    let llvm = try_generate_code(ir).expect("verified raw helper call must emit");
    assert!(llvm.contains("define i32 @helper()"), "{llvm}");
    assert!(llvm.contains("call i32 @helper()"), "{llvm}");
    assert!(!llvm.contains("call double @helper()"), "{llvm}");
}

#[test]
fn checked_codegen_widens_temporary_names_beyond_extreme_raw_ids() {
    let mut main = function(
        "main",
        vec![
            Inst::FPToSI(Value::Reg(u32::MAX), Value::ImmFloat(3.5)),
            Inst::Return(Value::Reg(u32::MAX)),
        ],
    );
    main.next_reg = u32::MAX;
    let llvm = try_generate_code(HashMap::from([("main".to_string(), main)]))
        .expect("extreme admitted raw result ID must not unwind or collide");
    assert!(
        llvm.contains("%reg4294967296 = fptosi double"),
        "fresh temporary did not widen beyond u32 raw IDs:\n{llvm}"
    );
    assert!(llvm.contains("%reg4294967295 = sitofp i32"), "{llvm}");
}
