use crate::LanguageProfile;
use crate::code_generator::{CodeGenerationError, try_generate_code_with_profile};
use crate::ir::{
    ByteBufferId, ByteBufferPlaceRole, CheckedIr, Function, Inst, LogicalType, PlaceId, RawIr,
    Value,
};
use crate::ir_verifier::verify_ir;
use crate::llvm_verifier::{LlvmVerificationMode, verify_llvm_module};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn program(body: Vec<Inst>) -> RawIr {
    HashMap::from([(
        "main".to_string(),
        Function {
            name: "main".to_string(),
            body,
            next_reg: 64,
            next_ptr: 0,
        },
    )])
}

fn exercised_body() -> Vec<Inst> {
    vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::CheckedByteBufferMutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferPush {
            result: Value::Reg(2),
            reference: Value::Reg(1),
            byte: Value::ImmInt(65),
        },
        Inst::CheckedByteBufferMutableBorrowEnd {
            reference: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferImmutableBorrow {
            result: Value::Reg(3),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferLength {
            result: Value::Reg(4),
            reference: Value::Reg(3),
        },
        Inst::CheckedByteBufferCapacity {
            result: Value::Reg(5),
            reference: Value::Reg(3),
        },
        Inst::CheckedByteBufferGet {
            result: Value::Reg(6),
            reference: Value::Reg(3),
            index: Value::ImmInt(0),
        },
        Inst::CheckedByteBufferImmutableBorrowEnd {
            reference: Value::Reg(3),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::Reg(6)),
    ]
}

fn empty_drop_body() -> Vec<Inst> {
    vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::ImmInt(0)),
    ]
}

fn assert_rejected(body: Vec<Inst>, expected: &[&str]) {
    assert_raw_rejected(program(body), expected);
}

fn assert_raw_rejected(raw: RawIr, expected: &[&str]) {
    let first = verify_ir(raw.clone())
        .expect_err("corrupt checked byte-buffer program must fail")
        .to_string();
    let second = verify_ir(raw)
        .expect_err("repeated corrupt checked byte-buffer program must fail")
        .to_string();
    assert_eq!(
        first, second,
        "checked resource rejection was nondeterministic"
    );
    let lowercase = first.to_ascii_lowercase();
    for fragment in expected {
        assert!(
            lowercase.contains(&fragment.to_ascii_lowercase()),
            "rejection omitted `{fragment}`: {first}"
        );
    }
}

fn checked_llvm(body: Vec<Inst>) -> String {
    let checked = verify_ir(program(body)).expect("valid checked byte-buffer program");
    let first = try_generate_code_with_profile(checked.clone(), LanguageProfile::StableScalarV0)
        .expect("verified checked byte-buffer program lowers");
    let second = try_generate_code_with_profile(checked, LanguageProfile::StableScalarV0)
        .expect("repeated checked byte-buffer program lowers");
    assert_eq!(
        first, second,
        "checked byte-buffer LLVM was nondeterministic"
    );
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .unwrap_or_else(|error| panic!("R1B LLVM failed verification: {error}\n{first}"));
    first
}

fn shared_read_body() -> Vec<Inst> {
    vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::CheckedByteBufferImmutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferImmutableBorrow {
            result: Value::Reg(2),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferLength {
            result: Value::Reg(3),
            reference: Value::Reg(1),
        },
        Inst::CheckedByteBufferCapacity {
            result: Value::Reg(4),
            reference: Value::Reg(2),
        },
        Inst::CheckedByteBufferGet {
            result: Value::Reg(5),
            reference: Value::Reg(1),
            index: Value::ImmInt(0),
        },
        Inst::CheckedByteBufferImmutableBorrowEnd {
            reference: Value::Reg(2),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferImmutableBorrowEnd {
            reference: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::ImmInt(0)),
    ]
}

fn outer_owner_loop_body() -> Vec<Inst> {
    vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::Jump("loop".to_string()),
        Inst::Label("loop".to_string()),
        Inst::CheckedByteBufferImmutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferLength {
            result: Value::Reg(2),
            reference: Value::Reg(1),
        },
        Inst::CheckedByteBufferImmutableBorrowEnd {
            reference: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::ICmp {
            op: "eq".to_string(),
            result: Value::Reg(3),
            left: Value::ImmInt(0),
            right: Value::ImmInt(1),
        },
        Inst::Branch {
            condition: Value::Reg(3),
            true_label: "loop".to_string(),
            false_label: "exit".to_string(),
        },
        Inst::Label("exit".to_string()),
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::ImmInt(0)),
    ]
}

fn branch_drop_body() -> Vec<Inst> {
    vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::ICmp {
            op: "eq".to_string(),
            result: Value::Reg(1),
            left: Value::ImmInt(0),
            right: Value::ImmInt(1),
        },
        Inst::Branch {
            condition: Value::Reg(1),
            true_label: "left".to_string(),
            false_label: "right".to_string(),
        },
        Inst::Label("left".to_string()),
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::ImmInt(1)),
        Inst::Label("right".to_string()),
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::ImmInt(2)),
    ]
}

fn mutable_push_body(bytes: &[i64], return_push: bool) -> Vec<Inst> {
    let mut body = vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::CheckedByteBufferMutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
        },
    ];
    let mut next = 2_u32;
    let mut last_push = None;
    for byte in bytes {
        let result = Value::Reg(next);
        next += 1;
        body.push(Inst::CheckedByteBufferPush {
            result: result.clone(),
            reference: Value::Reg(1),
            byte: Value::ImmInt(*byte),
        });
        last_push = Some(result);
    }
    body.push(Inst::CheckedByteBufferMutableBorrowEnd {
        reference: Value::Reg(1),
        source: Value::Reg(0),
    });
    body.push(Inst::CheckedByteBufferDrop {
        owner: Value::Reg(0),
    });
    body.push(Inst::Return(if return_push {
        last_push.expect("return-push fixture requires a push")
    } else {
        Value::ImmInt(0)
    }));
    body
}

fn push_then_get_body(bytes: &[i64], index: i64) -> Vec<Inst> {
    let mut body = vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "bytes".to_string(),
        },
        Inst::CheckedByteBufferMutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
        },
    ];
    let mut next = 2_u32;
    for byte in bytes {
        body.push(Inst::CheckedByteBufferPush {
            result: Value::Reg(next),
            reference: Value::Reg(1),
            byte: Value::ImmInt(*byte),
        });
        next += 1;
    }
    body.extend([
        Inst::CheckedByteBufferMutableBorrowEnd {
            reference: Value::Reg(1),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferImmutableBorrow {
            result: Value::Reg(next),
            source: Value::Reg(0),
        },
    ]);
    let reference = next;
    next += 1;
    body.push(Inst::CheckedByteBufferGet {
        result: Value::Reg(next),
        reference: Value::Reg(reference),
        index: Value::ImmInt(index),
    });
    let read = next;
    body.extend([
        Inst::CheckedByteBufferImmutableBorrowEnd {
            reference: Value::Reg(reference),
            source: Value::Reg(0),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(0),
        },
        Inst::Return(Value::Reg(read)),
    ]);
    body
}

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-r1b-byte-buffer-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create R1B native workspace");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write R1B native artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn assert_exit_91(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(91),
        "{label} failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_native_case(
    label: &str,
    body: Vec<Inst>,
    fail_after: u64,
    expected_result: i32,
    expected_alloc: u64,
    expected_realloc: u64,
    expected_dealloc: u64,
) {
    let workspace = TestWorkspace::new();
    let llvm = checked_llvm(body);
    let renamed = llvm.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(llvm, renamed, "R1B native fixture did not contain main");
    verify_llvm_module(&renamed, LlvmVerificationMode::Required)
        .expect("renamed R1B native fixture verifies");
    let llvm_path = workspace.write("program.ll", &renamed);
    let harness = format!(
        r#"
#include <stdint.h>

extern int aero_program_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

int main(void) {{
    if (aero_test_reset(UINT64_C({fail_after})) != 1) return 70;
    if (aero_program_main() != {expected_result}) return 71;
    if (aero_test_alloc_calls() != UINT64_C({expected_alloc})) return 72;
    if (aero_test_realloc_calls() != UINT64_C({expected_realloc})) return 73;
    if (aero_test_dealloc_calls() != UINT64_C({expected_dealloc})) return 74;
    if (aero_test_live_allocations() != 0) return 75;
    if (aero_test_size_mismatch_calls() != 0) return 76;
    return 91;
}}
"#
    );
    let harness_path = workspace.write("harness.c", &harness);
    let executable = workspace
        .path()
        .join(if cfg!(windows) { "case.exe" } else { "case" });
    let compile = Command::new("clang")
        .args([
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wno-override-module",
        ])
        .arg(&llvm_path)
        .arg(repository_root().join("src/compiler/runtime/aero_test_runtime.c"))
        .arg(&harness_path)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for R1B native fixture");
    assert!(
        compile.status.success(),
        "compile {label} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = Command::new(&executable)
        .output()
        .expect("execute R1B native fixture");
    assert_exit_91(&output, label);
}

#[test]
fn checked_byte_buffer_smoke_verifies_and_lowers() {
    let llvm = checked_llvm(exercised_body());
    for expected in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
        "call ptr @aero_alloc(i64",
        "call ptr @aero_realloc(ptr",
        "call void @aero_dealloc(ptr",
        "icmp sle i32",
        "1073741823",
        "i32 8",
        "phi i32 [ -1",
        "[ -2, %aero.bytes.push.",
        "[ -3, %aero.bytes.push.",
        "[ -4, %aero.bytes.get.",
        "zext i8",
    ] {
        assert!(
            llvm.contains(expected),
            "LLVM omitted `{expected}`:\n{llvm}"
        );
    }
    let byte_check = llvm.find("icmp sge i32 65, 0").expect("byte range check");
    let allocation = llvm.find("call ptr @aero_alloc").expect("allocation call");
    assert!(
        byte_check < allocation,
        "byte validation must precede allocation:\n{llvm}"
    );

    let moved = checked_llvm(vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "first".to_string(),
        },
        Inst::CheckedByteBufferMove {
            result: Value::Reg(1),
            source: Value::Reg(0),
            name: "second".to_string(),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(1),
        },
        Inst::Return(Value::ImmInt(0)),
    ]);
    assert!(
        moved.contains("%ptr1 = alloca %aero.byte_buffer, align 8"),
        "move omitted destination descriptor:\n{moved}"
    );
    assert_eq!(
        moved
            .matches("store %aero.byte_buffer zeroinitializer, ptr %ptr0, align 8")
            .count(),
        2,
        "new must initialize and move must clear the source:\n{moved}"
    );
}

#[test]
fn checked_byte_buffer_metadata_and_balanced_control_flow_are_exact() {
    for body in [
        empty_drop_body(),
        shared_read_body(),
        outer_owner_loop_body(),
        branch_drop_body(),
        vec![
            Inst::CheckedByteBufferNew {
                result: Value::Reg(0),
                name: "first".to_string(),
            },
            Inst::CheckedByteBufferMove {
                result: Value::Reg(1),
                source: Value::Reg(0),
                name: "second".to_string(),
            },
            Inst::CheckedByteBufferDrop {
                owner: Value::Reg(1),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
    ] {
        verify_ir(program(body)).expect("balanced checked byte-buffer program verifies");
    }

    let checked = verify_ir(program(exercised_body())).expect("exercise metadata");
    let function = &checked.metadata().functions["main"];
    assert_eq!(function.byte_buffers.len(), 3);
    assert_eq!(function.byte_buffers[&PlaceId(0)].identity, ByteBufferId(0));
    assert_eq!(
        function.byte_buffers[&PlaceId(0)].role,
        ByteBufferPlaceRole::Owner { moved_from: None }
    );
    assert_eq!(
        function.byte_buffers[&PlaceId(1)].role,
        ByteBufferPlaceRole::MutableLoan { owner: PlaceId(0) }
    );
    assert_eq!(
        function.byte_buffers[&PlaceId(3)].role,
        ByteBufferPlaceRole::ImmutableLoan { owner: PlaceId(0) }
    );
    for place in [PlaceId(0), PlaceId(1), PlaceId(3)] {
        assert_eq!(function.places[&place].pointee, LogicalType::ByteBuffer);
    }

    let moved = verify_ir(program(vec![
        Inst::CheckedByteBufferNew {
            result: Value::Reg(0),
            name: "first".to_string(),
        },
        Inst::CheckedByteBufferMove {
            result: Value::Reg(1),
            source: Value::Reg(0),
            name: "second".to_string(),
        },
        Inst::CheckedByteBufferDrop {
            owner: Value::Reg(1),
        },
        Inst::Return(Value::ImmInt(0)),
    ]))
    .expect("moved resource verifies");
    let moved = &moved.metadata().functions["main"].byte_buffers;
    assert_eq!(moved[&PlaceId(0)].identity, ByteBufferId(0));
    assert_eq!(moved[&PlaceId(1)].identity, ByteBufferId(0));
    assert_eq!(
        moved[&PlaceId(1)].role,
        ByteBufferPlaceRole::Owner {
            moved_from: Some(PlaceId(0))
        }
    );
}

#[test]
fn checked_byte_buffer_lifecycle_and_loan_corruption_is_fail_closed() {
    let cases: Vec<(&str, Vec<Inst>, Vec<&str>)> = vec![
        (
            "missing drop",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["resources", "closed", "return"],
        ),
        (
            "duplicate drop",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["drop", "moved", "dropped", "loaned"],
        ),
        (
            "use after move",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "first".to_string(),
                },
                Inst::CheckedByteBufferMove {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    name: "second".to_string(),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(1),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["drop", "moved"],
        ),
        (
            "use after drop",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["immutable", "live", "owner"],
        ),
        (
            "move under shared loan",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "first".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferMove {
                    result: Value::Reg(2),
                    source: Value::Reg(0),
                    name: "second".to_string(),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["move", "loaned", "owner"],
        ),
        (
            "drop under shared loan",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["drop", "loaned"],
        ),
        (
            "exclusive while shared",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferMutableBorrow {
                    result: Value::Reg(2),
                    source: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["mutable", "exclusive", "owner"],
        ),
        (
            "shared while exclusive",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferMutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(2),
                    source: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["immutable", "exclusive", "owner"],
        ),
        (
            "push through shared loan",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferPush {
                    result: Value::Reg(2),
                    reference: Value::Reg(1),
                    byte: Value::ImmInt(1),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["push", "mutable", "loan"],
        ),
        (
            "read through exclusive loan",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferMutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferLength {
                    result: Value::Reg(2),
                    reference: Value::Reg(1),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["shared", "immutable", "loan"],
        ),
        (
            "mismatched loan end",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "left".to_string(),
                },
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(1),
                    name: "right".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(2),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferImmutableBorrowEnd {
                    reference: Value::Reg(2),
                    source: Value::Reg(1),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["borrow", "end", "inconsistent"],
        ),
        (
            "duplicate loan end",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferImmutableBorrowEnd {
                    reference: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::CheckedByteBufferImmutableBorrowEnd {
                    reference: Value::Reg(1),
                    source: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["loan", "end", "not active"],
        ),
        (
            "generic store",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["generic", "store", "bytebuffer"],
        ),
        (
            "generic load",
            vec![
                Inst::CheckedByteBufferNew {
                    result: Value::Reg(0),
                    name: "bytes".to_string(),
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::CheckedByteBufferDrop {
                    owner: Value::Reg(0),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["generic", "load", "bytebuffer"],
        ),
        (
            "legacy vec",
            vec![
                Inst::VecAlloca {
                    result: Value::Reg(0),
                    element_type: "i32".to_string(),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            vec!["unsupported", "vec", "alloca"],
        ),
    ];

    for (label, body, fragments) in cases {
        assert_rejected(body, &fragments);
        assert!(!label.is_empty());
    }
}

#[test]
fn checked_byte_buffer_control_flow_requires_exact_resource_state() {
    assert_rejected(
        vec![
            Inst::CheckedByteBufferNew {
                result: Value::Reg(0),
                name: "bytes".to_string(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(1),
                left: Value::ImmInt(0),
                right: Value::ImmInt(1),
            },
            Inst::Branch {
                condition: Value::Reg(1),
                true_label: "drop".to_string(),
                false_label: "keep".to_string(),
            },
            Inst::Label("drop".to_string()),
            Inst::CheckedByteBufferDrop {
                owner: Value::Reg(0),
            },
            Inst::Jump("merge".to_string()),
            Inst::Label("keep".to_string()),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ],
        &["resource", "state", "join", "backedge"],
    );

    assert_rejected(
        vec![
            Inst::CheckedByteBufferNew {
                result: Value::Reg(0),
                name: "first".to_string(),
            },
            Inst::Jump("loop".to_string()),
            Inst::Label("loop".to_string()),
            Inst::CheckedByteBufferMove {
                result: Value::Reg(1),
                source: Value::Reg(0),
                name: "second".to_string(),
            },
            Inst::Jump("loop".to_string()),
        ],
        &["resource", "state", "join", "backedge"],
    );

    assert_rejected(
        vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmInt(0),
                right: Value::ImmInt(1),
            },
            Inst::Branch {
                condition: Value::Reg(0),
                true_label: "create".to_string(),
                false_label: "skip".to_string(),
            },
            Inst::Label("create".to_string()),
            Inst::CheckedByteBufferNew {
                result: Value::Reg(1),
                name: "bytes".to_string(),
            },
            Inst::Jump("merge".to_string()),
            Inst::Label("skip".to_string()),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ],
        &["resource", "state", "join", "backedge"],
    );
}

#[test]
fn checked_byte_buffer_schema_and_runtime_boundaries_are_private() {
    assert_rejected(
        vec![
            Inst::CheckedByteBufferNew {
                result: Value::Reg(0),
                name: "bytes".to_string(),
            },
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
                pointee: LogicalType::ByteBuffer,
            },
            Inst::Return(Value::ImmInt(0)),
        ],
        &[
            "unsupported",
            "logical",
            "type",
            "immutable reference pointee",
            "bytebuffer",
        ],
    );
    assert_rejected(
        vec![
            Inst::CheckedByteBufferNew {
                result: Value::Reg(0),
                name: "bytes".to_string(),
            },
            Inst::CheckedByteBufferMutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
            },
            Inst::CheckedByteBufferMove {
                result: Value::Reg(2),
                source: Value::Reg(1),
                name: "moved".to_string(),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
        &["move", "source", "no", "resolvable", "owner", "identity"],
    );
    assert_rejected(
        vec![
            Inst::CheckedByteBufferMove {
                result: Value::Reg(0),
                source: Value::Reg(1),
                name: "left".to_string(),
            },
            Inst::CheckedByteBufferMove {
                result: Value::Reg(1),
                source: Value::Reg(0),
                name: "right".to_string(),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
        &["move", "source", "no", "resolvable", "identity"],
    );

    let mut reserved = program(empty_drop_body());
    reserved.insert(
        "aero_alloc".to_string(),
        Function {
            name: "aero_alloc".to_string(),
            body: vec![Inst::Return(Value::ImmInt(0))],
            next_reg: 0,
            next_ptr: 0,
        },
    );
    assert_raw_rejected(reserved, &["aero_alloc", "reserved", "runtime", "abi"]);

    let ordinary_runtime_name = HashMap::from([
        (
            "main".to_string(),
            Function {
                name: "main".to_string(),
                body: vec![Inst::Return(Value::ImmInt(0))],
                next_reg: 0,
                next_ptr: 0,
            },
        ),
        (
            "aero_alloc".to_string(),
            Function {
                name: "aero_alloc".to_string(),
                body: vec![Inst::Return(Value::ImmInt(0))],
                next_reg: 0,
                next_ptr: 0,
            },
        ),
    ]);
    verify_ir(ordinary_runtime_name)
        .expect("runtime symbol is reserved only when the private resource is present");

    let transport = HashMap::from([
        (
            "main".to_string(),
            Function {
                name: "main".to_string(),
                body: vec![
                    Inst::CheckedFunctionDef {
                        name: "transport".to_string(),
                        parameters: vec![("bytes".to_string(), LogicalType::ByteBuffer)],
                        result: LogicalType::Int,
                        body: vec![Inst::Return(Value::ImmInt(0))],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
                next_reg: 0,
                next_ptr: 0,
            },
        ),
        (
            "transport".to_string(),
            Function {
                name: "transport".to_string(),
                body: Vec::new(),
                next_reg: 0,
                next_ptr: 0,
            },
        ),
    ]);
    assert_raw_rejected(transport, &["bytebuffer", "transport", "parameter"]);
}

#[test]
fn checked_wrapper_metadata_is_reverified_before_byte_buffer_codegen() {
    let checked = verify_ir(program(exercised_body())).expect("valid checked resource");
    let mut mutations = Vec::new();

    let mut metadata = checked.metadata().clone();
    metadata
        .functions
        .get_mut("main")
        .unwrap()
        .byte_buffers
        .get_mut(&PlaceId(0))
        .unwrap()
        .identity = ByteBufferId(99);
    mutations.push(CheckedIr::new(checked.raw().clone(), metadata));

    let mut metadata = checked.metadata().clone();
    metadata
        .functions
        .get_mut("main")
        .unwrap()
        .byte_buffers
        .get_mut(&PlaceId(1))
        .unwrap()
        .role = ByteBufferPlaceRole::ImmutableLoan { owner: PlaceId(0) };
    mutations.push(CheckedIr::new(checked.raw().clone(), metadata));

    let mut metadata = checked.metadata().clone();
    metadata
        .functions
        .get_mut("main")
        .unwrap()
        .places
        .get_mut(&PlaceId(0))
        .unwrap()
        .pointee = LogicalType::Int;
    mutations.push(CheckedIr::new(checked.raw().clone(), metadata));

    for corrupt in mutations {
        let first =
            try_generate_code_with_profile(corrupt.clone(), LanguageProfile::StableScalarV0)
                .expect_err("forged checked metadata must fail before LLVM");
        let second = try_generate_code_with_profile(corrupt, LanguageProfile::StableScalarV0)
            .expect_err("repeated forged checked metadata must fail before LLVM");
        assert_eq!(first.to_string(), second.to_string());
        assert!(matches!(first, CodeGenerationError::IrVerification(_)));
    }

    let experimental = try_generate_code_with_profile(
        verify_ir(program(empty_drop_body())).unwrap(),
        LanguageProfile::Experimental,
    )
    .expect_err("checked byte buffers require exact i32 lowering");
    assert!(matches!(
        experimental,
        CodeGenerationError::LanguageProfileContract { .. }
    ));
}

#[test]
fn checked_byte_buffer_native_success_and_failure_preserve_runtime_state() {
    run_native_case(
        "allocate-grow-read-drop",
        push_then_get_body(&[91, 2, 3, 4, 5, 6, 7, 8, 9], 0),
        u64::MAX,
        91,
        1,
        1,
        1,
    );
    run_native_case(
        "allocation-failure",
        mutable_push_body(&[65], true),
        0,
        -2,
        1,
        0,
        0,
    );
    run_native_case(
        "reallocation-failure-preserves-prefix",
        push_then_get_body(&[10, 20, 30, 40, 50, 60, 70, 77, 99], 7),
        1,
        77,
        1,
        1,
        1,
    );
    run_native_case(
        "invalid-byte-precedes-allocation",
        mutable_push_body(&[256], true),
        u64::MAX,
        -1,
        0,
        0,
        0,
    );
    run_native_case(
        "out-of-bounds-get",
        push_then_get_body(&[], 0),
        u64::MAX,
        -4,
        0,
        0,
        0,
    );
}
