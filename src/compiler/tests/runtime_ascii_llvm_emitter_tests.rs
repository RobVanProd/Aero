use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCT_RELATIVE_PATH: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_llvm_emitter.aero";
const PREDECESSOR_RELATIVE_PATH: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_checked_ir_verifier.aero";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const RUNTIME_RELATIVE_PATH: &str = "../../src/compiler/runtime/aero_runtime.c";
const TEST_RUNTIME_RELATIVE_PATH: &str = "../../src/compiler/runtime/aero_test_runtime.c";
const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const B1A_BEGIN: &str = "// CAP-045 B1A VERIFIER BEGIN";
const B1A_END: &str = "// CAP-045 B1A VERIFIER END";
const B1B_BEGIN: &str = "// CAP-046 B1B LLVM EMITTER BEGIN";
const B1B_END: &str = "// CAP-046 B1B LLVM EMITTER END";
const SELF_TEST_MARKER: &str = "// CAP-046 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-046 intentional product red: tracked runtime ASCII LLVM emitter is absent";
const MAX_EMITTED_BYTES: usize = 21_438;
const CANONICAL_B1A_CHECKSUM: i32 = 592_819;
const CANONICAL_INPUT: &[u8] = b"fn score()->int{return 1+2*3-4/2;}";
const CANONICAL_LLVM: &str = concat!(
    "define i32 @aero_b1_entry() {\n",
    "entry:\n",
    "  %r1 = mul i32 2, 3\n",
    "  %r2 = add i32 1, %r1\n",
    "  %r3 = sdiv i32 4, 2\n",
    "  %r4 = sub i32 %r2, %r3\n",
    "  ret i32 %r4\n",
    "}\n",
);

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    opcode: i32,
    left_kind: i32,
    left_payload: i32,
    right_kind: i32,
    right_payload: i32,
    origin: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VerifierExpectation {
    attempted: i32,
    status: i32,
    word_index: i32,
    record_id: i32,
    code: i32,
    expected: i32,
    actual: i32,
    instruction_count: i32,
    result_count: i32,
    root_value: i32,
    result_values: i32,
    checksum: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmitterExpectation {
    attempted: i32,
    status: i32,
    byte_index: i32,
    record_id: i32,
    length: i32,
    checksum: i32,
}

const VERIFIED_SUCCESS: VerifierExpectation = VerifierExpectation {
    attempted: 1,
    status: 0,
    word_index: -1,
    record_id: 0,
    code: 0,
    expected: 0,
    actual: 0,
    instruction_count: 5,
    result_count: 4,
    root_value: 5,
    result_values: 4,
    checksum: CANONICAL_B1A_CHECKSUM,
};

const EMITTED_SUCCESS: EmitterExpectation = EmitterExpectation {
    attempted: 1,
    status: 0,
    byte_index: -1,
    record_id: 0,
    length: 144,
    checksum: 611_963,
};

const EMITTED_SKIP: EmitterExpectation = EmitterExpectation {
    attempted: 0,
    status: 0,
    byte_index: -1,
    record_id: 0,
    length: 0,
    checksum: 0,
};

#[derive(Debug)]
struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository_path("../../target"))
            .join("cap046-runtime-llvm-emitter-tests");
        let root = parent.join(format!(
            "cap046-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-046 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-046 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap046-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
}

fn run_command_with_stdin(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CAP-046 child");
    child
        .stdin
        .take()
        .expect("CAP-046 child stdin")
        .write_all(input)
        .expect("write CAP-046 child stdin");
    child.wait_with_output().expect("wait for CAP-046 child")
}

fn assert_silent_exit_91(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(91),
        "{label} failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "{label} emitted stdout");
    assert!(output.stderr.is_empty(), "{label} emitted stderr");
}

fn checksum_step(checksum: i32, word: i32) -> i32 {
    i32::try_from((i64::from(checksum) * 31 + i64::from(word)) % 1_000_003)
        .expect("bounded checksum")
}

fn emission_seal(
    bytes: &[u8],
    verified_checksum: i32,
    instruction_count: i32,
    result_count: i32,
) -> (i32, i32) {
    let mut checksum = 43;
    for byte in bytes {
        checksum = checksum_step(checksum, i32::from(*byte));
    }
    let byte_fold = checksum;
    for word in [
        991,
        verified_checksum,
        0,
        0,
        0,
        1,
        i32::try_from(bytes.len()).expect("bounded output length"),
        instruction_count,
        result_count,
    ] {
        checksum = checksum_step(checksum, word);
    }
    (byte_fold, checksum)
}

fn operand(kind: i32, payload: i32) -> String {
    match kind {
        1 => payload.to_string(),
        2 => format!("%r{payload}"),
        _ => panic!("oracle received unverified operand kind {kind}"),
    }
}

fn emit_verified_module(words: &[i32]) -> Vec<u8> {
    let instruction_count = usize::try_from(words[3]).expect("positive instruction count");
    let mut llvm = String::from("define i32 @aero_b1_entry() {\nentry:\n");
    for index in 0..instruction_count {
        let base = 25 + index * 11;
        let id = words[base + 1];
        let opcode = words[base + 2];
        let left = operand(words[base + 5], words[base + 6]);
        match opcode {
            1..=4 => {
                let mnemonic = match opcode {
                    1 => "add",
                    2 => "sub",
                    3 => "mul",
                    4 => "sdiv",
                    _ => unreachable!(),
                };
                let right = operand(words[base + 7], words[base + 8]);
                llvm.push_str(&format!("  %r{id} = {mnemonic} i32 {left}, {right}\n"));
            }
            5 => llvm.push_str(&format!("  %r{id} = sub i32 0, {left}\n")),
            6 => llvm.push_str(&format!("  ret i32 {left}\n")),
            _ => panic!("oracle received unverified opcode {opcode}"),
        }
    }
    llvm.push_str("}\n");
    llvm.into_bytes()
}

fn module(function_node: i32, instructions: &[Instruction], root: (i32, i32)) -> Vec<i32> {
    assert!(!instructions.is_empty());
    assert_eq!(instructions.last().expect("terminal instruction").opcode, 6);
    let instruction_count = i32::try_from(instructions.len()).expect("bounded instructions");
    let result_count = instruction_count - 1;
    let mut words = vec![
        1,
        1,
        1,
        instruction_count,
        result_count,
        1,
        root.0,
        root.1,
        1,
        1,
        1,
        1,
        function_node,
        0,
        1,
        1,
        1,
        instruction_count,
        2,
        1,
        1,
        1,
        0,
        1,
        instruction_count,
    ];
    for (index, instruction) in instructions.iter().enumerate() {
        let id = i32::try_from(index + 1).expect("bounded instruction ID");
        let is_return = instruction.opcode == 6;
        words.extend([
            3,
            id,
            instruction.opcode,
            if is_return { 0 } else { id },
            if is_return { 0 } else { 1 },
            instruction.left_kind,
            instruction.left_payload,
            instruction.right_kind,
            instruction.right_payload,
            instruction.origin,
            1,
        ]);
    }
    for (index, instruction) in instructions[..instructions.len() - 1].iter().enumerate() {
        let id = i32::try_from(index + 1).expect("bounded result ID");
        words.extend([4, 1, id, 1, id, instruction.origin]);
    }
    words
}

fn canonical_words() -> Vec<i32> {
    module(
        11,
        &[
            Instruction {
                opcode: 3,
                left_kind: 1,
                left_payload: 2,
                right_kind: 1,
                right_payload: 3,
                origin: 4,
            },
            Instruction {
                opcode: 1,
                left_kind: 1,
                left_payload: 1,
                right_kind: 2,
                right_payload: 1,
                origin: 5,
            },
            Instruction {
                opcode: 4,
                left_kind: 1,
                left_payload: 4,
                right_kind: 1,
                right_payload: 2,
                origin: 8,
            },
            Instruction {
                opcode: 2,
                left_kind: 2,
                left_payload: 2,
                right_kind: 2,
                right_payload: 3,
                origin: 9,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 4,
                right_kind: 0,
                right_payload: 0,
                origin: 10,
            },
        ],
        (2, 4),
    )
}

fn invocation_arguments(
    fault_word: i32,
    fault_value: i32,
    verified: VerifierExpectation,
    emitted: EmitterExpectation,
) -> String {
    let arguments = [
        0,
        -1,
        0,
        0,
        0,
        0,
        2,
        20,
        11,
        11,
        586_661,
        0,
        0,
        -1,
        0,
        0,
        0,
        0,
        0,
        11,
        1,
        11,
        1,
        827_574,
        1,
        0,
        0,
        -1,
        0,
        0,
        0,
        0,
        0,
        9,
        5,
        4,
        104,
        2,
        4,
        1,
        355_067,
        fault_word,
        fault_value,
        verified.attempted,
        verified.status,
        verified.word_index,
        verified.record_id,
        verified.code,
        verified.expected,
        verified.actual,
        verified.instruction_count,
        verified.result_count,
        verified.root_value,
        verified.result_values,
        verified.checksum,
        emitted.attempted,
        emitted.status,
        emitted.byte_index,
        emitted.record_id,
        emitted.length,
        emitted.checksum,
    ];
    assert_eq!(arguments.len(), 61, "CAP-046 invocation arity changed");
    arguments
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn verifier_fault_cases() -> Vec<(&'static str, i32, i32, VerifierExpectation)> {
    vec![
        (
            "format family",
            0,
            2,
            VerifierExpectation {
                attempted: 1,
                status: 1,
                word_index: 0,
                record_id: 0,
                code: 2,
                expected: 1,
                actual: 2,
                instruction_count: 0,
                result_count: 0,
                root_value: 0,
                result_values: 0,
                checksum: 510_488,
            },
        ),
        (
            "topology family",
            9,
            2,
            VerifierExpectation {
                attempted: 1,
                status: 2,
                word_index: 9,
                record_id: 0,
                code: 1,
                expected: 1,
                actual: 2,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 0,
                checksum: 716_292,
            },
        ),
        (
            "instruction family",
            27,
            99,
            VerifierExpectation {
                attempted: 1,
                status: 3,
                word_index: 27,
                record_id: 1,
                code: 2,
                expected: 99,
                actual: 99,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 0,
                checksum: 333_688,
            },
        ),
        (
            "operand family",
            30,
            9,
            VerifierExpectation {
                attempted: 1,
                status: 4,
                word_index: 30,
                record_id: 1,
                code: 1,
                expected: 1,
                actual: 9,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 0,
                checksum: 251_078,
            },
        ),
        (
            "arithmetic family",
            55,
            0,
            VerifierExpectation {
                attempted: 1,
                status: 5,
                word_index: 49,
                record_id: 3,
                code: 4,
                expected: 0,
                actual: 0,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 2,
                checksum: 276_369,
            },
        ),
        (
            "result family",
            80,
            3,
            VerifierExpectation {
                attempted: 1,
                status: 6,
                word_index: 80,
                record_id: 1,
                code: 1,
                expected: 4,
                actual: 3,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 4,
                checksum: 319_541,
            },
        ),
        (
            "root family",
            7,
            3,
            VerifierExpectation {
                attempted: 1,
                status: 7,
                word_index: 7,
                record_id: 5,
                code: 2,
                expected: 4,
                actual: 3,
                instruction_count: 5,
                result_count: 4,
                root_value: 0,
                result_values: 4,
                checksum: 458_247,
            },
        ),
        (
            "outside-view selector",
            104,
            0,
            VerifierExpectation {
                attempted: 1,
                status: 1,
                word_index: 104,
                record_id: 0,
                code: 1,
                expected: 104,
                actual: 0,
                instruction_count: 0,
                result_count: 0,
                root_value: 0,
                result_values: 0,
                checksum: 971_129,
            },
        ),
        ("same-value enabled selector", 0, 1, VERIFIED_SUCCESS),
    ]
}

fn one_result(opcode: i32) -> Vec<i32> {
    module(
        4,
        &[
            Instruction {
                opcode,
                left_kind: 1,
                left_payload: 7,
                right_kind: if opcode == 5 { 0 } else { 1 },
                right_payload: if opcode == 5 { 0 } else { 3 },
                origin: 1,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 1,
                right_kind: 0,
                right_payload: 0,
                origin: 3,
            },
        ],
        (2, 1),
    )
}

fn clang_link(
    label: &str,
    workspace: &TestWorkspace,
    inputs: &[&Path],
    optimization: &str,
) -> PathBuf {
    let executable = workspace.root.join(if cfg!(windows) {
        format!("{label}-{optimization}.exe")
    } else {
        format!("{label}-{optimization}")
    });
    let output = Command::new("clang")
        .args([
            "-std=c11",
            optimization,
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wno-error=override-module",
        ])
        .args(inputs)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for CAP-046 oracle");
    assert!(
        output.status.success(),
        "link {label} {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn capture_test_runtime() -> String {
    let mut runtime = fs::read_to_string(repository_path(TEST_RUNTIME_RELATIVE_PATH))
        .expect("read accepted Aero test runtime");
    let globals = "static uint64_t fail_after_successes = UINT64_MAX;";
    assert_eq!(runtime.matches(globals).count(), 1);
    runtime = runtime.replacen(
        globals,
        concat!(
            "static uint64_t fail_after_successes = UINT64_MAX;\n",
            "static uint8_t captured_first_deallocation[21438];\n",
            "static uint64_t captured_first_deallocation_size;\n",
        ),
        1,
    );

    let deallocation = concat!(
        "    header->metadata.magic = 0;\n",
        "    free(header);\n",
        "    --live_allocations;",
    );
    assert_eq!(runtime.matches(deallocation).count(), 1);
    runtime = runtime.replacen(
        deallocation,
        concat!(
            "    if (dealloc_calls == UINT64_C(1)) {\n",
            "        captured_first_deallocation_size = size;\n",
            "        uint64_t capture = size < UINT64_C(21438) ? size : UINT64_C(21438);\n",
            "        for (uint64_t index = 0; index < capture; ++index) {\n",
            "            captured_first_deallocation[index] = ((uint8_t *)allocation)[index];\n",
            "        }\n",
            "    }\n",
            "    header->metadata.magic = 0;\n",
            "    free(header);\n",
            "    --live_allocations;",
        ),
        1,
    );

    let reset = "    fail_after_successes = requested_fail_after_successes;";
    assert_eq!(runtime.matches(reset).count(), 1);
    runtime = runtime.replacen(
        reset,
        concat!(
            "    fail_after_successes = requested_fail_after_successes;\n",
            "    captured_first_deallocation_size = 0;",
        ),
        1,
    );
    runtime.push_str(concat!(
        "\nuint64_t aero_test_capture_size(void) {\n",
        "    return captured_first_deallocation_size;\n",
        "}\n\n",
        "int32_t aero_test_capture_byte(uint64_t index) {\n",
        "    if (index >= captured_first_deallocation_size || index >= UINT64_C(21438)) return -1;\n",
        "    return captured_first_deallocation[index];\n",
        "}\n",
    ));
    runtime
}

fn capture_harness() -> String {
    let input = CANONICAL_INPUT
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let expected = CANONICAL_LLVM
        .as_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
#include <stddef.h>
#include <stdint.h>

extern int32_t aero_product_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);
extern uint64_t aero_test_capture_size(void);
extern int32_t aero_test_capture_byte(uint64_t index);

static const uint8_t input_bytes[] = {{ {input} }};
static const uint8_t expected_bytes[] = {{ {expected} }};
static size_t input_index;
static int32_t sticky_status;

int32_t aero_stdin_read_byte(void) {{
    if (sticky_status != 0) return sticky_status;
    if (input_index < sizeof(input_bytes)) return input_bytes[input_index++];
    sticky_status = -1;
    return sticky_status;
}}

int main(void) {{
    if (aero_test_reset(UINT64_MAX) != 1) return 40;
    input_index = 0;
    sticky_status = 0;
    if (aero_product_main() != 91) return 41;
    if (aero_test_live_allocations() != 0) return 42;
    if (aero_test_size_mismatch_calls() != 0) return 43;
    if (aero_test_alloc_calls() != UINT64_C(14)) return 44;
    if (aero_test_realloc_calls() != UINT64_C(58)) return 45;
    if (aero_test_dealloc_calls() != UINT64_C(14)) return 46;
    if (aero_test_capture_size() < sizeof(expected_bytes) ||
        aero_test_capture_size() > UINT64_C(32768)) return 47;
    for (uint64_t index = 0; index < sizeof(expected_bytes); ++index) {{
        if (aero_test_capture_byte(index) != expected_bytes[index]) return 48;
    }}
    return 91;
}}
"#,
    )
}

fn fault_and_allocation_harness() -> String {
    use std::fmt::Write as _;

    let mut calls = String::new();
    for (case_index, (label, fault_word, fault_value, verified)) in
        verifier_fault_cases().into_iter().enumerate()
    {
        let arguments = invocation_arguments(fault_word, fault_value, verified, EMITTED_SKIP);
        writeln!(
            calls,
            "    /* {label} */\n    if (aero_test_reset(UINT64_MAX) != 1) return 50;\n    reset_input();\n    if (run_runtime_ascii_llvm_emitter({arguments}) != 91) return 51;\n    if (aero_test_live_allocations() != 0) return 52;\n    if (aero_test_size_mismatch_calls() != 0) return 53;\n    completed = {completed};",
            completed = case_index + 1,
        )
        .expect("write CAP-046 verifier-fault case");
    }

    let parameter_types = std::iter::repeat_n("int32_t", 61)
        .collect::<Vec<_>>()
        .join(", ");
    let input = CANONICAL_INPUT
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let canonical = invocation_arguments(-1, 0, VERIFIED_SUCCESS, EMITTED_SUCCESS);
    let expected_cases = verifier_fault_cases().len();
    format!(
        r#"
#include <stddef.h>
#include <stdint.h>

extern int32_t run_runtime_ascii_llvm_emitter({parameter_types});
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static const uint8_t input_bytes[] = {{ {input} }};
static size_t input_index;
static int32_t sticky_status;

static void reset_input(void) {{ input_index = 0; sticky_status = 0; }}

int32_t aero_stdin_read_byte(void) {{
    if (sticky_status != 0) return sticky_status;
    if (input_index < sizeof(input_bytes)) return input_bytes[input_index++];
    sticky_status = -1;
    return sticky_status;
}}

int main(void) {{
    size_t completed = 0;
{calls}
    if (completed != {expected_cases}) return 54;
    for (uint64_t threshold = 0; threshold <= UINT64_C(72); ++threshold) {{
        if (aero_test_reset(threshold) != 1) return 55;
        reset_input();
        int32_t result = run_runtime_ascii_llvm_emitter({canonical});
        if (threshold < UINT64_C(72) && result == 91) return 56;
        if (threshold == UINT64_C(72) && result != 91) return 57;
        if (aero_test_live_allocations() != 0) return 58;
        if (aero_test_size_mismatch_calls() != 0) return 59;
        if (threshold == UINT64_C(72) &&
            (aero_test_alloc_calls() != UINT64_C(14) ||
             aero_test_realloc_calls() != UINT64_C(58) ||
             aero_test_dealloc_calls() != UINT64_C(14))) return 60;
    }}
    return 91;
}}
"#,
    )
}

#[test]
fn independent_b1b_oracle_freezes_canonical_bytes_and_seal() {
    let first = emit_verified_module(&canonical_words());
    let second = emit_verified_module(&canonical_words());
    assert_eq!(first, second);
    assert_eq!(first, CANONICAL_LLVM.as_bytes());
    assert_eq!(first.len(), 144);
    assert_eq!(
        format!("{:x}", md5::compute(&first)),
        "fd2390d17d448d4539a72bf1991314dc"
    );
    assert_eq!(
        emission_seal(&first, CANONICAL_B1A_CHECKSUM, 5, 4),
        (629_434, 611_963)
    );
    verify_llvm_module(
        std::str::from_utf8(&first).expect("oracle output is ASCII"),
        LlvmVerificationMode::Required,
    )
    .expect("canonical B1B oracle LLVM verifies");
}

#[test]
fn independent_b1b_oracle_covers_every_mapping_and_the_bound() {
    let literal = module(
        3,
        &[Instruction {
            opcode: 6,
            left_kind: 1,
            left_payload: 7,
            right_kind: 0,
            right_payload: 0,
            origin: 2,
        }],
        (1, 7),
    );
    let literal_llvm = emit_verified_module(&literal);
    assert_eq!(
        std::str::from_utf8(&literal_llvm).expect("literal output is ASCII"),
        "define i32 @aero_b1_entry() {\nentry:\n  ret i32 7\n}\n"
    );
    verify_llvm_module(
        std::str::from_utf8(&literal_llvm).expect("literal output is ASCII"),
        LlvmVerificationMode::Required,
    )
    .expect("literal Return verifies");

    for (opcode, line) in [
        (1, "  %r1 = add i32 7, 3\n"),
        (2, "  %r1 = sub i32 7, 3\n"),
        (3, "  %r1 = mul i32 7, 3\n"),
        (4, "  %r1 = sdiv i32 7, 3\n"),
        (5, "  %r1 = sub i32 0, 7\n"),
    ] {
        let llvm = emit_verified_module(&one_result(opcode));
        let text = std::str::from_utf8(&llvm).expect("mapped output is ASCII");
        assert!(text.contains(line), "opcode {opcode} emitted:\n{text}");
        assert!(text.contains("  ret i32 %r1\n"));
        verify_llvm_module(text, LlvmVerificationMode::Required)
            .unwrap_or_else(|error| panic!("opcode {opcode} failed LLVM verification: {error}"));
    }

    let boundary = module(
        5,
        &[
            Instruction {
                opcode: 2,
                left_kind: 1,
                left_payload: 0,
                right_kind: 1,
                right_payload: i32::MAX,
                origin: 1,
            },
            Instruction {
                opcode: 2,
                left_kind: 2,
                left_payload: 1,
                right_kind: 1,
                right_payload: 1,
                origin: 2,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 2,
                right_kind: 0,
                right_payload: 0,
                origin: 4,
            },
        ],
        (2, 2),
    );
    let boundary_llvm = emit_verified_module(&boundary);
    let boundary_text = std::str::from_utf8(&boundary_llvm).expect("boundary output is ASCII");
    assert!(boundary_text.contains("sub i32 0, 2147483647"));
    assert!(boundary_text.contains("sub i32 %r1, 1"));
    verify_llvm_module(boundary_text, LlvmVerificationMode::Required)
        .expect("signed i32 boundary LLVM verifies");

    let mut instructions = Vec::with_capacity(510);
    for origin in 1..=509 {
        instructions.push(Instruction {
            opcode: 2,
            left_kind: 1,
            left_payload: i32::MAX,
            right_kind: 1,
            right_payload: i32::MAX,
            origin,
        });
    }
    instructions.push(Instruction {
        opcode: 6,
        left_kind: 2,
        left_payload: 509,
        right_kind: 0,
        right_payload: 0,
        origin: 511,
    });
    let maximum = emit_verified_module(&module(512, &instructions, (2, 509)));
    assert_eq!(maximum.len(), 20_816);
    assert!(maximum.len() <= MAX_EMITTED_BYTES);
    let maximum_text = std::str::from_utf8(&maximum).expect("maximum output is ASCII");
    assert!(maximum_text.contains("  %r509 = sub i32 2147483647, 2147483647\n"));
    assert!(maximum_text.ends_with("  ret i32 %r509\n}\n"));
    verify_llvm_module(maximum_text, LlvmVerificationMode::Required)
        .expect("maximum B1B oracle LLVM verifies");
}

#[test]
fn independent_b1b_canonical_llvm_lowers_and_executes_at_o0_and_o2() {
    let workspace = TestWorkspace::new("oracle-native");
    let llvm = workspace.write("canonical.ll", CANONICAL_LLVM);
    let harness = workspace.write(
        "harness.c",
        concat!(
            "#include <stdint.h>\n",
            "extern int32_t aero_b1_entry(void);\n",
            "int main(void) { return aero_b1_entry() == 5 ? 91 : 1; }\n",
        ),
    );
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "canonical",
            &workspace,
            &[llvm.as_path(), harness.as_path()],
            optimization,
        );
        let output = Command::new(executable)
            .output()
            .expect("execute CAP-046 oracle");
        assert_eq!(
            output.status.code(),
            Some(91),
            "canonical B1B oracle failed at {optimization} (stdout={:?}, stderr={:?})",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn tracked_runtime_ascii_llvm_emitter_is_deterministic_captured_and_native() {
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    let product = fs::read_to_string(&product_path).expect("read tracked CAP-046 product");

    check_program(&product, options()).expect("tracked CAP-046 product checks");
    let first = compile_program(&product, options()).expect("tracked CAP-046 product compiles");
    let second = compile_program(&product, options()).expect("tracked CAP-046 product recompiles");
    assert_eq!(first, second, "tracked CAP-046 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked CAP-046 outer LLVM verifies");
    assert!(first.contains("define i32 @run_runtime_ascii_llvm_emitter("));
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
        "declare i32 @aero_stdin_read_byte()",
    ] {
        assert!(first.contains(anchor), "tracked LLVM omitted `{anchor}`");
    }
    for forbidden in [
        "double", "fptosi", "sitofp", " nsw ", " nuw ", "@malloc", "@free",
    ] {
        assert!(
            !first.contains(forbidden),
            "tracked LLVM leaked `{forbidden}`"
        );
    }

    let workspace = TestWorkspace::new("tracked-emitter");
    let source_path = workspace.write("runtime_ascii_llvm_emitter.aero", &product);
    check_file(&source_path, options()).expect("tracked CAP-046 file checks");
    assert_eq!(
        compile_file(&source_path, options()).expect("tracked CAP-046 file compiles"),
        first,
        "tracked CAP-046 source/file LLVM diverged"
    );
    let llvm_path = workspace.write("tracked.ll", &first);
    let runtime = repository_path(RUNTIME_RELATIVE_PATH);
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "tracked-emitter",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            optimization,
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), CANONICAL_INPUT),
            &format!("tracked CAP-046 outer product {optimization}"),
        );
    }

    let renamed = first.replacen("define i32 @main()", "define i32 @aero_product_main()", 1);
    assert_ne!(renamed, first, "tracked CAP-046 LLVM omitted main");
    let captured_llvm = workspace.write("captured.ll", renamed);
    let captured_runtime = workspace.write("capture_runtime.c", capture_test_runtime());
    let captured_harness = workspace.write("capture_harness.c", capture_harness());
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "captured-emitter",
            &workspace,
            &[
                captured_llvm.as_path(),
                captured_harness.as_path(),
                captured_runtime.as_path(),
            ],
            optimization,
        );
        assert_silent_exit_91(
            &Command::new(executable)
                .output()
                .expect("run CAP-046 capture harness"),
            &format!("CAP-046 captured bytes {optimization}"),
        );
    }

    let mutation_harness = workspace.write("faults.c", fault_and_allocation_harness());
    let test_runtime = repository_path(TEST_RUNTIME_RELATIVE_PATH);
    let mutation_executable = clang_link(
        "faults",
        &workspace,
        &[
            captured_llvm.as_path(),
            mutation_harness.as_path(),
            test_runtime.as_path(),
        ],
        "-O2",
    );
    assert_silent_exit_91(
        &Command::new(mutation_executable)
            .output()
            .expect("run CAP-046 fault/allocation harness"),
        "CAP-046 verifier-gate and allocation replay",
    );

    let mut public = Command::new(env!("CARGO_BIN_EXE_aero"));
    public
        .args([
            "run",
            source_path.to_str().expect("public source path is UTF-8"),
            "--language-profile",
            PROFILE_NAME,
        ])
        .current_dir(&workspace.root);
    let public_output = run_command_with_stdin(&mut public, CANONICAL_INPUT);
    assert_eq!(public_output.status.code(), Some(91));
    let public_stdout = String::from_utf8_lossy(&public_output.stdout);
    assert_eq!(
        public_stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1
    );
    assert!(
        !public_stdout
            .lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:"))
    );
    assert!(public_output.stderr.is_empty());

    for target in ["rocm", "cuda"] {
        let output_path = workspace.root.join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                source_path
                    .to_str()
                    .expect("accelerator source path is UTF-8"),
                "-o",
                output_path
                    .to_str()
                    .expect("accelerator output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(&workspace.root)
            .output()
            .expect("execute CAP-046 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !output_path.exists(),
            "{target} rejection created an artifact"
        );
    }
}

#[test]
fn tracked_runtime_ascii_llvm_emitter_is_structurally_complete() {
    let product = repository_path(PRODUCT_RELATIVE_PATH);
    assert!(product.is_file(), "{INTENTIONAL_PRODUCT_RED}");

    let product = fs::read_to_string(product).expect("read tracked CAP-046 product");
    let predecessor = fs::read_to_string(repository_path(PREDECESSOR_RELATIVE_PATH))
        .expect("read accepted CAP-045 product");
    assert!(product.is_ascii(), "CAP-046 product must remain raw ASCII");
    assert!(product.contains(SELF_TEST_MARKER));
    assert_eq!(
        product.matches(": ByteBuffer = bytes_new();").count(),
        14,
        "CAP-046 must own exactly the accepted thirteen owners plus emitted_llvm"
    );
    assert!(product.contains(concat!(
        "let mut verified_results: ByteBuffer = bytes_new();\n",
        "    let mut emitted_llvm: ByteBuffer = bytes_new();",
    )));

    let b1a_section = |source: &str| {
        source
            .split_once(B1A_BEGIN)
            .and_then(|(_, suffix)| suffix.split_once(B1A_END))
            .map(|(section, _)| section)
            .expect("isolate accepted B1A verifier section")
            .to_owned()
    };
    assert_eq!(
        b1a_section(&product),
        b1a_section(&predecessor),
        "CAP-046 changed the accepted B1A verifier body"
    );

    let emitter = product
        .split_once(B1B_BEGIN)
        .and_then(|(_, suffix)| suffix.split_once(B1B_END))
        .map(|(section, _)| section)
        .expect("isolate CAP-046 emitter section");
    for anchor in [
        "verified_attempted == 1 && verified_status == 0",
        "verification_fault_word == -1 && verification_fault_value == 0",
        "bytes_get(&checked_ir",
        "bytes_push(\n                    &mut emitted_llvm",
        "emitted_checksum = checksum_step(emitted_checksum, 991)",
        "emitted_checksum = checksum_step(emitted_checksum, verified_checksum)",
        "verified_instruction_count",
        "emitted_opcode",
        "emitter_fixed_byte",
        "unsigned_decimal_digit",
    ] {
        assert!(
            emitter.contains(anchor),
            "CAP-046 emitter omitted `{anchor}`"
        );
    }
    for forbidden in [
        "bytes_get(&source",
        "bytes_get(&names",
        "bytes_get(&tokens",
        "bytes_get(&nodes",
        "bytes_get(&values",
        "bytes_get(&operators",
        "bytes_get(&origins",
        "bytes_get(&symbols",
        "bytes_get(&facts",
        "bytes_get(&checked_values",
        "bytes_get(&checked_instructions",
        "bytes_get(&verified_results",
        "bytes_push(&mut checked_ir",
        "checked_checksum",
        "checked_root_",
        "expected_",
        "println!",
        "target triple",
        "target datalayout",
    ] {
        assert!(
            !emitter.contains(forbidden),
            "CAP-046 emitter crossed its consumption boundary via `{forbidden}`"
        );
    }
    for anchor in [
        "expected_emitted_attempted: int",
        "expected_emitted_status: int",
        "expected_emitted_byte_index: int",
        "expected_emitted_record_id: int",
        "expected_emitted_length: int",
        "expected_emitted_checksum: int",
        "1, 0, -1, 0, 144, 611963",
        "return 94;",
    ] {
        assert!(
            product.contains(anchor),
            "CAP-046 product omitted `{anchor}`"
        );
    }

    let workflow = fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH))
        .expect("read protected workflow");
    for step in [
        "Test runtime ASCII LLVM emitter at O0 and O2",
        "Test runtime ASCII LLVM emitter on Windows at O0 and O2",
    ] {
        assert!(
            workflow.contains(step),
            "protected workflow omitted `{step}`"
        );
    }
}
