//! H1E-CHECKED-001 tests the checked backend, not stderr source admission.

use crate::LanguageProfile;

const PROFILE_NAME: &str = "exact-i32-byte-diagnostics-v0";

#[test]
fn diagnostic_profile_selector_is_available() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("H1E-CHECKED-001 diagnostic backend profile must be selectable");
    assert_eq!(profile.as_str(), PROFILE_NAME);
}

use super::{CodeGenerator, try_generate_code_with_authenticated_profile};
use crate::ir::{CheckedIr, Function, Inst, LogicalType, RawIr, ResultId, Value};
use crate::ir_verifier::{IrVerificationErrorKind, verify_ir};
use crate::language_profile::{ProfileTypeUse, validate_resolved_language_profile};
use crate::resolved_profile_authentication::authenticate_resolved_profile;
use crate::resolved_profile_shape::{
    ResolvedProfileOrigin, ResolvedProfileProgram, ResolvedProfileResolution,
    ResolvedProfileShapeId, ResolvedProfileUse,
};
use crate::{
    CompilerOptions, LlvmVerificationMode, check_program, compile_program, verify_llvm_module,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn profile() -> LanguageProfile {
    PROFILE_NAME.parse().expect("diagnostic profile selector")
}

fn main_ir(body: Vec<Inst>) -> RawIr {
    HashMap::from([(
        "main".to_string(),
        Function {
            name: "main".to_string(),
            body,
            next_reg: 256,
            next_ptr: 0,
        },
    )])
}

fn write(result: u32, value: Value) -> Inst {
    Inst::CheckedStderrWriteByte {
        result: Value::Reg(result),
        value,
    }
}

// This is an explicitly authored logical descriptor for the hand-authored IR's
// main() -> Int signature. It is not obtained from an unrelated source program.
// Authentication covers types/layout and actual verified metadata, not source
// body correspondence. No authenticated token is constructed or modified here.
fn main_descriptor(result: LogicalType) -> ResolvedProfileProgram {
    ResolvedProfileProgram {
        shapes: vec![result],
        nominals: vec![],
        uses: vec![ResolvedProfileUse {
            role: ProfileTypeUse::Result,
            function: Some(ResolvedProfileOrigin::Source {
                normalized: "main".to_string(),
            }),
            name: None,
            resolution: ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(0)),
        }],
        operations: vec![],
        surface: vec![],
    }
}

fn emit(checked: CheckedIr) -> String {
    let descriptor = main_descriptor(LogicalType::Int);
    validate_resolved_language_profile(&descriptor, profile())
        .expect("fixture profile admits types");
    let authenticated = authenticate_resolved_profile(descriptor, &checked)
        .expect("fixture types authenticate against actual checked IR");
    try_generate_code_with_authenticated_profile(checked, profile(), authenticated)
        .expect("verified stderr fixture lowers through the authenticated backend")
}

#[test]
fn checked_stderr_fixture_verifies_and_lowers() {
    let checked = verify_ir(main_ir(vec![
        Inst::Add(Value::Reg(8), Value::ImmInt(64), Value::ImmInt(1)),
        write(0, Value::Reg(8)),
        Inst::Return(Value::Reg(0)),
    ]))
    .expect("H1E-CHECKED-001 typed stderr operation must pass checked verification");
    assert_eq!(
        checked.metadata().functions["main"].results[&ResultId(0)],
        LogicalType::Int
    );
    let first = emit(checked.clone());
    assert_eq!(
        emit(checked),
        first,
        "checked stderr LLVM must be deterministic"
    );
    assert_eq!(
        first
            .matches("declare i32 @aero_stderr_write_byte(i32)")
            .count(),
        1
    );
    assert_eq!(
        first
            .matches("call i32 @aero_stderr_write_byte(i32 %reg8)")
            .count(),
        1
    );
    assert!(!first.contains("aero_stdout_write_byte"));
    verify_llvm_module(&first, LlvmVerificationMode::Required).expect("stderr LLVM verifies");
    let absent = emit(verify_ir(main_ir(vec![Inst::Return(Value::ImmInt(91))])).unwrap());
    assert!(
        !absent.contains("aero_stderr_write_byte"),
        "unused ABI must not be declared"
    );
}

#[test]
fn checked_stderr_rejects_bad_results_and_operand_ssa() {
    let cases = [
        (
            "duplicate",
            vec![
                write(0, Value::ImmInt(65)),
                write(0, Value::ImmInt(66)),
                Inst::Return(Value::ImmInt(0)),
            ],
            IrVerificationErrorKind::DuplicateResultDefinition(ResultId(0)),
        ),
        (
            "nonidentifier",
            vec![
                Inst::CheckedStderrWriteByte {
                    result: Value::ImmInt(0),
                    value: Value::ImmInt(65),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            IrVerificationErrorKind::ExpectedResultIdentifier("instruction"),
        ),
        (
            "undefined",
            vec![write(0, Value::Reg(99)), Inst::Return(Value::ImmInt(0))],
            IrVerificationErrorKind::UndefinedResultUse(ResultId(99)),
        ),
        (
            "forward",
            vec![
                write(0, Value::Reg(1)),
                Inst::Add(Value::Reg(1), Value::ImmInt(64), Value::ImmInt(1)),
                Inst::Return(Value::ImmInt(0)),
            ],
            IrVerificationErrorKind::ResultUseBeforeDefinition(ResultId(1)),
        ),
        (
            "dominance",
            vec![
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(0),
                    right: Value::ImmInt(0),
                },
                Inst::Branch {
                    condition: Value::Reg(0),
                    true_label: "left".to_string(),
                    false_label: "right".to_string(),
                },
                Inst::Label("left".to_string()),
                Inst::Add(Value::Reg(1), Value::ImmInt(64), Value::ImmInt(1)),
                Inst::Jump("join".to_string()),
                Inst::Label("right".to_string()),
                Inst::Jump("join".to_string()),
                Inst::Label("join".to_string()),
                write(2, Value::Reg(1)),
                Inst::Return(Value::ImmInt(0)),
            ],
            IrVerificationErrorKind::ResultDoesNotDominateUse(ResultId(1)),
        ),
    ];
    for (label, body, expected) in cases {
        let error = verify_ir(main_ir(body)).expect_err(label);
        assert_eq!(error.kind, expected, "{label}: {error}");
    }
    for body in [
        vec![
            write(0, Value::ImmFloat(65.0)),
            Inst::Return(Value::ImmInt(0)),
        ],
        vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmInt(0),
                right: Value::ImmInt(0),
            },
            write(1, Value::Reg(0)),
            Inst::Return(Value::ImmInt(0)),
        ],
    ] {
        let error = verify_ir(main_ir(body)).expect_err("non-Int byte operand must fail");
        assert!(
            matches!(error.kind, IrVerificationErrorKind::TypeMismatch { .. }),
            "{error}"
        );
    }
}

#[test]
fn checked_stderr_rejects_runtime_symbol_collisions_and_raw_calls() {
    let runtime = "aero_stderr_write_byte";
    let mut module_collision = main_ir(vec![
        write(0, Value::ImmInt(65)),
        Inst::Return(Value::ImmInt(0)),
    ]);
    module_collision.insert(
        runtime.to_string(),
        Function {
            name: runtime.to_string(),
            body: vec![Inst::Return(Value::ImmInt(0))],
            next_reg: 0,
            next_ptr: 0,
        },
    );
    let mut nested_collision = main_ir(vec![
        Inst::FunctionDef {
            name: runtime.to_string(),
            parameters: vec![],
            return_type: Some("int".to_string()),
            body: vec![Inst::Return(Value::ImmInt(0))],
        },
        write(0, Value::ImmInt(65)),
        Inst::Return(Value::ImmInt(0)),
    ]);
    nested_collision.insert(
        "helper".to_string(),
        Function {
            name: "helper".to_string(),
            body: vec![Inst::Return(Value::ImmInt(0))],
            next_reg: 0,
            next_ptr: 0,
        },
    );
    for raw in [module_collision, nested_collision] {
        let error = verify_ir(raw).expect_err("runtime ABI collision must fail");
        assert!(
            matches!(error.kind, IrVerificationErrorKind::MetadataMismatch(_)),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("aero_stderr_write_byte` is reserved"),
            "{error}"
        );
    }
    let raw_call = main_ir(vec![
        Inst::Call {
            function: runtime.to_string(),
            arguments: vec![Value::ImmInt(65)],
            result: Some(Value::Reg(0)),
        },
        Inst::Return(Value::Reg(0)),
    ]);
    assert_eq!(
        verify_ir(raw_call).unwrap_err().kind,
        IrVerificationErrorKind::UnknownFunction(runtime.to_string())
    );
}

#[test]
fn checked_stderr_keeps_profile_and_authentication_guards() {
    let checked = verify_ir(main_ir(vec![
        write(0, Value::ImmInt(65)),
        Inst::Return(Value::Reg(0)),
    ]))
    .unwrap();
    for earlier in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
        LanguageProfile::ExactI32ByteInputV0,
        LanguageProfile::ExactI32ByteIoV0,
    ] {
        let authenticated =
            authenticate_resolved_profile(main_descriptor(LogicalType::Int), &checked).unwrap();
        let error =
            try_generate_code_with_authenticated_profile(checked.clone(), earlier, authenticated)
                .expect_err("earlier backend must refuse stderr");
        assert!(
            error
                .to_string()
                .contains("checked stderr byte writes require exact-i32-byte-diagnostics-v0"),
            "{earlier}: {error}"
        );
    }
    let missing = CodeGenerator::new()
        .try_generate_code_with_profile(checked.clone(), profile())
        .expect_err("direct checked backend needs authentication");
    assert!(
        missing
            .to_string()
            .contains("missing verifier-authenticated resolved profile token"),
        "{missing}"
    );
    let mismatch = authenticate_resolved_profile(main_descriptor(LogicalType::Bool), &checked)
        .expect_err("mismatched descriptor must not produce a token");
    assert!(
        mismatch.to_string().contains("FunctionSignatureMismatch"),
        "{mismatch}"
    );
    let authenticated =
        authenticate_resolved_profile(main_descriptor(LogicalType::Int), &checked).unwrap();
    let changed = verify_ir(main_ir(vec![
        write(0, Value::ImmInt(65)),
        Inst::Add(Value::Reg(1), Value::Reg(0), Value::ImmInt(1)),
        Inst::Return(Value::Reg(1)),
    ]))
    .unwrap();
    let stale = try_generate_code_with_authenticated_profile(changed, profile(), authenticated)
        .expect_err("stale metadata token must fail");
    assert!(
        stale
            .to_string()
            .contains("does not match re-verified metadata count"),
        "{stale}"
    );
}

#[test]
fn diagnostic_profile_inherits_byte_io_without_stderr_source_admission() {
    let source = r#"
fn result_value(value: Result<int, int>) -> int {
    return match value { Ok(status) => status, Err(code) => 0 - code, };
}
fn main() -> int {
    let mut bytes: ByteBuffer = bytes_new();
    let input: Result<int, int> = stdin_read_byte();
    let pushed: Result<int, int> = bytes_push(&mut bytes, result_value(input));
    let output: Result<int, int> = stdout_write_byte(result_value(pushed));
    return result_value(output);
}
"#;
    let inherited = CompilerOptions {
        language_profile: profile(),
        ..CompilerOptions::default()
    };
    check_program(source, inherited.clone()).expect("inherited source checks");
    let actual = compile_program(source, inherited.clone()).expect("inherited source compiles");
    let old = compile_program(
        source,
        CompilerOptions {
            language_profile: LanguageProfile::ExactI32ByteIoV0,
            ..CompilerOptions::default()
        },
    )
    .unwrap();
    assert_eq!(actual, old, "inherited source LLVM must remain unchanged");
    for name in ["stderr_write_byte", "aero_stderr_write_byte"] {
        for unresolved in [
            format!("fn main() -> int {{ let written: Result<int, int> = {name}(65); return 0; }}"),
            format!(
                "fn helper(value: int) -> int {{ let written: Result<int, int> = {name}(value); return 0; }} fn main() -> int {{ return helper(65); }}"
            ),
        ] {
            let error = check_program(&unresolved, inherited.clone())
                .expect_err("stderr source is not admitted yet");
            assert!(
                error.contains(&format!("Function `{name}` is not defined")),
                "{error}"
            );
            assert_eq!(
                compile_program(&unresolved, inherited.clone()).unwrap_err(),
                error
            );
        }
    }
    let ordinary = "fn stderr_write_byte(value: int) -> int { return value; } fn main() -> int { return stderr_write_byte(91); }";
    let ordinary_llvm =
        compile_program(ordinary, inherited).expect("ordinary source helper remains ordinary");
    assert_eq!(
        ordinary_llvm,
        compile_program(
            ordinary,
            CompilerOptions {
                language_profile: LanguageProfile::ExactI32ByteIoV0,
                ..CompilerOptions::default()
            }
        )
        .unwrap()
    );
    assert!(
        !ordinary_llvm.contains("@aero_stderr_write_byte"),
        "ordinary helper must not acquire the runtime ABI"
    );
}

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);
struct NativeWorkspace(PathBuf);
impl NativeWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"))
            .join(format!(
                "h1e-checked-{}-{nonce}-{serial}",
                std::process::id()
            ));
        fs::create_dir_all(&root).unwrap();
        Self(fs::canonicalize(root).unwrap())
    }
}
impl Drop for NativeWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn checked_sequence(values: &[i64], expected: i64) -> CheckedIr {
    let mut body = vec![];
    for (index, value) in values.iter().enumerate() {
        let result = u32::try_from(index * 2).unwrap();
        body.push(write(result, Value::ImmInt(*value)));
        body.push(Inst::ICmp {
            op: "eq".to_string(),
            result: Value::Reg(result + 1),
            left: Value::Reg(result),
            right: Value::ImmInt(expected),
        });
        let next = format!("next_{index}");
        body.push(Inst::Branch {
            condition: Value::Reg(result + 1),
            true_label: next.clone(),
            false_label: "failed".to_string(),
        });
        body.push(Inst::Label(next));
    }
    body.push(Inst::Return(Value::ImmInt(91)));
    body.push(Inst::Label("failed".to_string()));
    body.push(Inst::Return(Value::ImmInt(71)));
    verify_ir(main_ir(body)).expect("native checked sequence verifies")
}

#[test]
fn checked_stderr_native_binary_range_and_sticky_statuses_at_o0_o2() {
    let workspace = NativeWorkspace::new();
    for (label, values, status, stderr) in [
        (
            "binary",
            &[0, 13, 10, 26, 255][..],
            0,
            &[0, 13, 10, 26, 255][..],
        ),
        ("low", &[-1, 65, 255][..], -3, &[][..]),
        ("high", &[256, 65, 0][..], -3, &[][..]),
    ] {
        let llvm = emit(checked_sequence(values, status));
        verify_llvm_module(&llvm, LlvmVerificationMode::Required)
            .expect("native fixture LLVM verifies");
        let calls: Vec<_> = llvm
            .lines()
            .filter(|line| line.contains("call i32 @aero_stderr_write_byte"))
            .map(|line| {
                line.split("(i32 ")
                    .nth(1)
                    .unwrap()
                    .trim_end_matches(')')
                    .parse::<i64>()
                    .unwrap()
            })
            .collect();
        assert_eq!(calls, values, "exact call order and operands");
        let llvm_file = workspace.0.join(format!("{label}.ll"));
        fs::write(&llvm_file, llvm).unwrap();
        for optimization in ["-O0", "-O2"] {
            let executable = workspace.0.join(format!(
                "{label}-{optimization}{}",
                if cfg!(windows) { ".exe" } else { "" }
            ));
            let output = Command::new("clang")
                .args([
                    "-std=c11",
                    optimization,
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-Wno-error=override-module",
                ])
                .arg(&llvm_file)
                .arg(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("runtime/aero_diagnostic_runtime.c"),
                )
                .arg("-o")
                .arg(&executable)
                .output()
                .expect("compile native diagnostic fixture");
            assert!(
                output.status.success(),
                "{label} {optimization}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = Command::new(executable)
                .output()
                .expect("run native diagnostic fixture");
            assert_eq!(
                output.status.code(),
                Some(91),
                "{label} {optimization}: {output:?}"
            );
            assert!(
                output.stdout.is_empty(),
                "{label} {optimization}: unexpected stdout"
            );
            assert_eq!(
                output.stderr, stderr,
                "{label} {optimization}: exact binary stderr"
            );
        }
    }
}
