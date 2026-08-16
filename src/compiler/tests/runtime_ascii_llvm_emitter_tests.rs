use compiler::{LlvmVerificationMode, verify_llvm_module};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCT_RELATIVE_PATH: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_llvm_emitter.aero";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-046 intentional product red: tracked runtime ASCII LLVM emitter is absent";
const MAX_EMITTED_BYTES: usize = 21_438;
const CANONICAL_B1A_CHECKSUM: i32 = 592_819;
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
fn tracked_runtime_ascii_llvm_emitter_is_structurally_complete() {
    let product = repository_path(PRODUCT_RELATIVE_PATH);
    assert!(product.is_file(), "{INTENTIONAL_PRODUCT_RED}");
}
