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
