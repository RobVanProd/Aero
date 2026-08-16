use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const PRODUCT_RELATIVE_PATH: &str =
    "../../examples/compiler_storage_v0/deterministic_compiler_storage.aero";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const SELF_TEST_MARKER: &str = "// CAP-040 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-040 intentional product red: tracked owned compiler-storage arena is absent";
const CHECKSUM_MODULUS: i32 = 1_000_003;
const CANONICAL_CHECKSUM: i32 = 639_832;

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NameRecord {
    start: i32,
    length: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenRecord {
    kind: i32,
    start: i32,
    length: i32,
    name_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScopeRecord {
    name_id: i32,
    leaf_node_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeRecord {
    kind: i32,
    payload: i32,
    left_id: i32,
    right_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StorageModel {
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    scopes: Vec<ScopeRecord>,
    nodes: Vec<NodeRecord>,
    root: i32,
    checksum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputError {
    ZeroLength {
        offset: usize,
    },
    Truncated {
        offset: usize,
        declared: usize,
        available: usize,
    },
    InvalidIdentifier {
        offset: usize,
        position: usize,
        byte: u8,
    },
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn fold_checksum(checksum: i32, word: i32) -> i32 {
    assert!((0..=i32::MAX).contains(&word));
    ((checksum * 31) + word) % CHECKSUM_MODULUS
}

fn storage_checksum(
    names: &[NameRecord],
    tokens: &[TokenRecord],
    scopes: &[ScopeRecord],
    nodes: &[NodeRecord],
    root: i32,
) -> i32 {
    let mut checksum = 17;
    for name in names {
        checksum = fold_checksum(checksum, name.start);
        checksum = fold_checksum(checksum, name.length);
    }
    checksum = fold_checksum(checksum, 991);
    for token in tokens {
        for word in [token.kind, token.start, token.length, token.name_id] {
            checksum = fold_checksum(checksum, word);
        }
    }
    checksum = fold_checksum(checksum, 992);
    for scope in scopes {
        checksum = fold_checksum(checksum, scope.name_id);
        checksum = fold_checksum(checksum, scope.leaf_node_id);
    }
    checksum = fold_checksum(checksum, 993);
    for node in nodes {
        for word in [node.kind, node.payload, node.left_id, node.right_id] {
            checksum = fold_checksum(checksum, word);
        }
    }
    checksum = fold_checksum(checksum, 994);
    for word in [
        names.len() as i32,
        tokens.len() as i32,
        nodes.len() as i32,
        root,
    ] {
        checksum = fold_checksum(checksum, word);
    }
    checksum
}

fn reference_storage(input: &[u8]) -> Result<StorageModel, InputError> {
    let mut names = Vec::<NameRecord>::new();
    let mut tokens = Vec::<TokenRecord>::new();
    let mut scopes = Vec::<ScopeRecord>::new();
    let mut nodes = Vec::<NodeRecord>::new();
    let mut root = 0;
    let mut offset = 0usize;

    while offset < input.len() {
        let declared = usize::from(input[offset]);
        if declared == 0 {
            return Err(InputError::ZeroLength { offset });
        }
        let start = offset + 1;
        let available = input.len().saturating_sub(start);
        if available < declared {
            return Err(InputError::Truncated {
                offset,
                declared,
                available,
            });
        }
        let end = start + declared;
        for (position, byte) in input[start..end].iter().copied().enumerate() {
            let valid = if position == 0 {
                is_identifier_start(byte)
            } else {
                is_identifier_continue(byte)
            };
            if !valid {
                return Err(InputError::InvalidIdentifier {
                    offset: start + position,
                    position,
                    byte,
                });
            }
        }

        let name_id = names
            .iter()
            .position(|record| {
                record.length as usize == declared
                    && input[record.start as usize..record.start as usize + declared]
                        == input[start..end]
            })
            .map(|index| index as i32 + 1)
            .unwrap_or_else(|| {
                names.push(NameRecord {
                    start: start as i32,
                    length: declared as i32,
                });
                names.len() as i32
            });

        tokens.push(TokenRecord {
            kind: 1,
            start: start as i32,
            length: declared as i32,
            name_id,
        });
        let leaf_node_id = nodes.len() as i32 + 1;
        nodes.push(NodeRecord {
            kind: 1,
            payload: name_id,
            left_id: 0,
            right_id: 0,
        });
        scopes.push(ScopeRecord {
            name_id,
            leaf_node_id,
        });
        if root == 0 {
            root = leaf_node_id;
        } else {
            let sequence_node_id = nodes.len() as i32 + 1;
            nodes.push(NodeRecord {
                kind: 2,
                payload: 0,
                left_id: root,
                right_id: leaf_node_id,
            });
            root = sequence_node_id;
        }
        offset = end;
    }

    let checksum = storage_checksum(&names, &tokens, &scopes, &nodes, root);
    Ok(StorageModel {
        names,
        tokens,
        scopes,
        nodes,
        root,
        checksum,
    })
}

fn encode_names(names: &[&[u8]]) -> Vec<u8> {
    let mut input = Vec::new();
    for name in names {
        assert!((1..=63).contains(&name.len()));
        input.push(name.len() as u8);
        input.extend_from_slice(name);
    }
    input
}

fn canonical_input() -> Vec<u8> {
    encode_names(&[b"alpha", b"beta", b"alpha", b"_x9"])
}

fn large_input() -> Vec<u8> {
    let mut names = Vec::<&[u8]>::new();
    for index in 0..1_025 {
        names.push(if index % 2 == 0 { b"alpha" } else { b"beta" });
    }
    let input = encode_names(&names);
    assert!(input.len() >= 4_097);
    input
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocationExpectation {
    result_is_success: bool,
    alloc_calls: u64,
    realloc_calls: u64,
    dealloc_calls: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct BufferState {
    length: usize,
    capacity: usize,
}

fn simulate_pushes(
    state: &mut BufferState,
    count: usize,
    fail_after: u64,
    successful_events: &mut u64,
    alloc_calls: &mut u64,
    realloc_calls: &mut u64,
) -> bool {
    for _ in 0..count {
        if state.length == state.capacity {
            if state.capacity == 0 {
                *alloc_calls += 1;
            } else {
                *realloc_calls += 1;
            }
            if *successful_events >= fail_after {
                return false;
            }
            *successful_events += 1;
            state.capacity = if state.capacity == 0 {
                8
            } else {
                state.capacity * 2
            };
        }
        state.length += 1;
    }
    true
}

fn allocation_expectation(
    input: &[u8],
    model: &StorageModel,
    fail_after: u64,
) -> AllocationExpectation {
    let mut buffers = [BufferState::default(); 5];
    let mut successful_events = 0;
    let mut alloc_calls = 0;
    let mut realloc_calls = 0;
    let mut completed = simulate_pushes(
        &mut buffers[0],
        input.len(),
        fail_after,
        &mut successful_events,
        &mut alloc_calls,
        &mut realloc_calls,
    );
    let mut seen_names = BTreeSet::new();

    if completed {
        for (index, token) in model.tokens.iter().enumerate() {
            if seen_names.insert(token.name_id) {
                completed = simulate_pushes(
                    &mut buffers[1],
                    8,
                    fail_after,
                    &mut successful_events,
                    &mut alloc_calls,
                    &mut realloc_calls,
                );
            }
            for (buffer, bytes) in [(2usize, 16usize), (4, 16), (3, 8)] {
                if completed {
                    completed = simulate_pushes(
                        &mut buffers[buffer],
                        bytes,
                        fail_after,
                        &mut successful_events,
                        &mut alloc_calls,
                        &mut realloc_calls,
                    );
                }
            }
            if completed && index > 0 {
                completed = simulate_pushes(
                    &mut buffers[4],
                    16,
                    fail_after,
                    &mut successful_events,
                    &mut alloc_calls,
                    &mut realloc_calls,
                );
            }
            if !completed {
                break;
            }
        }
    }

    AllocationExpectation {
        result_is_success: completed,
        alloc_calls,
        realloc_calls,
        dealloc_calls: buffers.iter().filter(|buffer| buffer.capacity != 0).count() as u64,
    }
}

fn options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
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
            .join("cap040-compiler-storage-tests");
        let root = parent.join(format!("{label}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&root).expect("create CAP-040 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-040 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap040-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn run_command_with_stdin(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CAP-040 child");
    child
        .stdin
        .take()
        .expect("CAP-040 child stdin")
        .write_all(input)
        .expect("write CAP-040 child stdin");
    child.wait_with_output().expect("wait for CAP-040 child")
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
    let mut command = Command::new("clang");
    command.args([
        "-std=c11",
        optimization,
        "-Wall",
        "-Wextra",
        "-Werror",
        "-Wno-override-module",
    ]);
    command.args(inputs).arg("-o").arg(&executable);
    let output = command.output().expect("execute Clang for CAP-040");
    assert!(
        output.status.success(),
        "link {label} {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
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

fn generated_program(kernel_prefix: &str, model: &StorageModel, checksum_delta: i32) -> String {
    format!(
        "{}\n\nfn main() -> int {{\n    return run_compiler_storage({}, {}, {}, {}, {});\n}}\n",
        kernel_prefix.trim_end(),
        model.names.len(),
        model.tokens.len(),
        model.nodes.len(),
        model.root,
        model.checksum + checksum_delta,
    )
}

fn compile_generated(program: &str) -> String {
    check_program(program, options()).expect("generated CAP-040 program checks");
    let first = compile_program(program, options()).expect("generated CAP-040 program compiles");
    let second = compile_program(program, options()).expect("generated CAP-040 program recompiles");
    assert_eq!(first, second, "generated CAP-040 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("generated CAP-040 LLVM verifies");
    first
}

fn workflow_step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let marker = format!("    - name: {name}\n");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("workflow step `{name}` is absent"));
    let remainder = &workflow[start + marker.len()..];
    let end = remainder.find("\n    - name: ").unwrap_or(remainder.len());
    &remainder[..end]
}

fn allocation_harness(input: &[u8], model: &StorageModel) -> String {
    let fail_after = [0u64, 1, 3, 5, 10, 17, 18];
    let mut cases = String::new();
    for threshold in fail_after {
        let expected = allocation_expectation(input, model, threshold);
        writeln!(
            cases,
            "    {{ UINT64_C({threshold}), {}, UINT64_C({}), UINT64_C({}), UINT64_C({}) }},",
            i32::from(expected.result_is_success),
            expected.alloc_calls,
            expected.realloc_calls,
            expected.dealloc_calls,
        )
        .expect("write allocation case");
    }
    let input_bytes = input
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
#include <stddef.h>
#include <stdint.h>

extern int aero_program_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static const uint8_t input_bytes[] = {{ {input_bytes} }};
static size_t input_index;
static int32_t sticky_status;

static void reset_input(void) {{
    input_index = 0;
    sticky_status = 0;
}}

int32_t aero_stdin_read_byte(void) {{
    if (sticky_status != 0) return sticky_status;
    if (input_index < sizeof(input_bytes)) return input_bytes[input_index++];
    sticky_status = -1;
    return sticky_status;
}}

struct Case {{
    uint64_t fail_after;
    int32_t success;
    uint64_t allocations;
    uint64_t reallocations;
    uint64_t deallocations;
}};

int main(void) {{
    const struct Case cases[] = {{
{cases}    }};
    for (size_t index = 0; index < sizeof(cases) / sizeof(cases[0]); ++index) {{
        const struct Case *test = &cases[index];
        if (aero_test_reset(test->fail_after) != 1) return 70;
        reset_input();
        int32_t result = aero_program_main();
        if ((result == 91) != test->success) return 71;
        if (aero_test_alloc_calls() != test->allocations) return 72;
        if (aero_test_realloc_calls() != test->reallocations) return 73;
        if (aero_test_dealloc_calls() != test->deallocations) return 74;
        if (aero_test_live_allocations() != 0) return 75;
        if (aero_test_size_mismatch_calls() != 0) return 76;
    }}
    return 91;
}}
"#,
    )
}

#[test]
fn independent_oracle_freezes_names_tokens_scopes_nodes_and_failures() {
    let canonical = canonical_input();
    let model = reference_storage(&canonical).expect("canonical input is valid");
    assert_eq!(
        model.names,
        vec![
            NameRecord {
                start: 1,
                length: 5
            },
            NameRecord {
                start: 7,
                length: 4
            },
            NameRecord {
                start: 18,
                length: 3
            },
        ]
    );
    assert_eq!(
        model.tokens,
        vec![
            TokenRecord {
                kind: 1,
                start: 1,
                length: 5,
                name_id: 1
            },
            TokenRecord {
                kind: 1,
                start: 7,
                length: 4,
                name_id: 2
            },
            TokenRecord {
                kind: 1,
                start: 12,
                length: 5,
                name_id: 1
            },
            TokenRecord {
                kind: 1,
                start: 18,
                length: 3,
                name_id: 3
            },
        ]
    );
    assert_eq!(
        model.scopes,
        vec![
            ScopeRecord {
                name_id: 1,
                leaf_node_id: 1
            },
            ScopeRecord {
                name_id: 2,
                leaf_node_id: 2
            },
            ScopeRecord {
                name_id: 1,
                leaf_node_id: 4
            },
            ScopeRecord {
                name_id: 3,
                leaf_node_id: 6
            },
        ]
    );
    assert_eq!(model.nodes.len(), 7);
    assert_eq!(
        model.nodes[2],
        NodeRecord {
            kind: 2,
            payload: 0,
            left_id: 1,
            right_id: 2
        }
    );
    assert_eq!(
        model.nodes[6],
        NodeRecord {
            kind: 2,
            payload: 0,
            left_id: 5,
            right_id: 6
        }
    );
    assert_eq!(model.root, 7);
    assert_eq!(model.checksum, CANONICAL_CHECKSUM);

    let empty = reference_storage(&[]).expect("empty input is valid");
    assert!(empty.names.is_empty());
    assert!(empty.tokens.is_empty());
    assert!(empty.nodes.is_empty());
    assert_eq!((empty.root, empty.checksum), (0, 577_557));

    assert_eq!(
        reference_storage(&[0]),
        Err(InputError::ZeroLength { offset: 0 })
    );
    assert_eq!(
        reference_storage(&[3, b'a']),
        Err(InputError::Truncated {
            offset: 0,
            declared: 3,
            available: 1,
        })
    );
    assert_eq!(
        reference_storage(&[1, b'9']),
        Err(InputError::InvalidIdentifier {
            offset: 1,
            position: 0,
            byte: b'9',
        })
    );
    assert_eq!(
        reference_storage(&[2, b'a', 0xff]),
        Err(InputError::InvalidIdentifier {
            offset: 2,
            position: 1,
            byte: 0xff,
        })
    );

    let max_name = vec![b'a'; 63];
    let max = encode_names(&[max_name.as_slice()]);
    let max_model = reference_storage(&max).expect("63-byte name is valid");
    assert_eq!(
        (
            max_model.names.len(),
            max_model.tokens.len(),
            max_model.root
        ),
        (1, 1, 1)
    );

    let large = large_input();
    let first = reference_storage(&large).expect("large input is valid");
    let second = reference_storage(&large).expect("large input is deterministic");
    assert_eq!(first, second);
    assert_eq!(first.names.len(), 2);
    assert_eq!(first.tokens.len(), 1_025);
    assert_eq!(first.nodes.len(), 2_049);
    assert!(first.root > 255);

    let success = allocation_expectation(&canonical, &model, 18);
    assert_eq!(
        success,
        AllocationExpectation {
            result_is_success: true,
            alloc_calls: 5,
            realloc_calls: 13,
            dealloc_calls: 5,
        }
    );
    assert!(!allocation_expectation(&canonical, &model, 17).result_is_success);
}

#[test]
fn accepted_r2_runtime_ir_verifier_and_profile_remain_frozen_before_d1() {
    let product = fs::read_to_string(repository_path(
        "../../examples/stdin_byte_input_v0/whole_stream_stdin.aero",
    ))
    .expect("read accepted R2 product");
    check_program(&product, options()).expect("accepted R2 product checks");
    let first = compile_program(&product, options()).expect("accepted R2 product compiles");
    let second = compile_program(&product, options()).expect("accepted R2 product recompiles");
    assert_eq!(first, second, "accepted R2 LLVM drifted before D1");
    assert_eq!(first.matches("call i32 @aero_stdin_read_byte()").count(), 1);

    for (relative, expected) in [
        (
            "../../src/compiler/runtime/aero_runtime.c",
            "090c9c07a5fa0a4c374b43953af8306f",
        ),
        (
            "../../src/compiler/runtime/aero_test_runtime.c",
            "5f1db08f29355e78a1dda31747ec7055",
        ),
        (
            "../../src/compiler/src/ir.rs",
            "2b8288bcbb2825586a0e406f37fbe12d",
        ),
        (
            "../../src/compiler/src/ir_verifier.rs",
            "d5fae602214665b724c48c9ae8090a06",
        ),
    ] {
        assert_eq!(
            md5_hex(&fs::read(repository_path(relative)).expect("read accepted authority")),
            expected,
            "accepted authority `{relative}` drifted during D1"
        );
    }
}

#[test]
fn tracked_owned_compiler_storage_arena_matches_oracle_and_platform_gate() {
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    assert!(product_path.is_file(), "{INTENTIONAL_PRODUCT_RED}");
    let product = fs::read_to_string(&product_path).expect("read tracked CAP-040 product");
    let (kernel_prefix, tracked_main) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked product retains one kernel/self-test boundary");

    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert_eq!(product.matches("fn run_compiler_storage(").count(), 1);
    assert!(tracked_main.contains("fn main() -> int"));
    assert_eq!(product.matches("let sequence_left: int = root;").count(), 1);
    for owner in ["source", "names", "tokens", "scopes", "nodes"] {
        assert!(
            product.contains(&format!("let mut {owner}: ByteBuffer = bytes_new();")),
            "tracked product omitted `{owner}` owner"
        );
    }
    for forbidden in [
        "String", "Vec", "HashMap", "print", "mod ", "use ", "unsafe",
    ] {
        assert!(
            !product.contains(forbidden),
            "tracked product contains `{forbidden}`"
        );
    }

    check_program(&product, options()).expect("tracked CAP-040 product checks");
    let first = compile_program(&product, options()).expect("tracked CAP-040 product compiles");
    let second = compile_program(&product, options()).expect("tracked CAP-040 product recompiles");
    assert_eq!(first, second, "tracked CAP-040 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked CAP-040 LLVM verifies");
    assert_eq!(
        first.matches("declare i32 @aero_stdin_read_byte()").count(),
        1
    );
    assert_eq!(first.matches("call i32 @aero_stdin_read_byte()").count(), 1);
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
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

    let workspace = TestWorkspace::new("tracked");
    let tracked_source = workspace.write("tracked.aero", &product);
    check_file(&tracked_source, options()).expect("tracked CAP-040 file checks");
    assert_eq!(
        compile_file(&tracked_source, options()).expect("tracked CAP-040 file compiles"),
        first,
        "tracked source/file LLVM diverged"
    );

    let canonical = canonical_input();
    let canonical_model = reference_storage(&canonical).expect("canonical model");
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    let llvm_path = workspace.write("tracked.ll", &first);
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "tracked",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            optimization,
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), &canonical),
            &format!("tracked CAP-040 {optimization}"),
        );
    }

    for (label, input) in [
        ("empty", Vec::new()),
        ("shadow", encode_names(&[b"x", b"y", b"x"])),
        ("maximum", encode_names(&[vec![b'a'; 63].as_slice()])),
        ("large", large_input()),
    ] {
        let model = reference_storage(&input).expect("valid generated fixture");
        let program = generated_program(kernel_prefix, &model, 0);
        let llvm = compile_generated(&program);
        let case = TestWorkspace::new(label);
        let llvm_path = case.write("case.ll", llvm);
        let executable = clang_link(
            label,
            &case,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O2",
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), &input),
            &format!("generated CAP-040 {label}"),
        );
    }

    let wrong_program = generated_program(kernel_prefix, &canonical_model, 1);
    let wrong_llvm = compile_generated(&wrong_program);
    let wrong = TestWorkspace::new("wrong-oracle");
    let wrong_path = wrong.write("wrong.ll", wrong_llvm);
    let wrong_executable = clang_link(
        "wrong",
        &wrong,
        &[wrong_path.as_path(), runtime.as_path()],
        "-O2",
    );
    assert_ne!(
        run_command_with_stdin(&mut Command::new(wrong_executable), &canonical)
            .status
            .code(),
        Some(91),
        "wrong independent expectation was accepted"
    );

    let corrupted_kernel = kernel_prefix.replacen(
        "let sequence_left: int = root;",
        "let sequence_left: int = node_count + 2;",
        1,
    );
    assert_ne!(
        corrupted_kernel, kernel_prefix,
        "corruption anchor was absent"
    );
    let corrupt_program = generated_program(&corrupted_kernel, &canonical_model, 0);
    let corrupt_llvm = compile_generated(&corrupt_program);
    let corrupt = TestWorkspace::new("corrupt-arena");
    let corrupt_path = corrupt.write("corrupt.ll", corrupt_llvm);
    let corrupt_executable = clang_link(
        "corrupt",
        &corrupt,
        &[corrupt_path.as_path(), runtime.as_path()],
        "-O2",
    );
    assert_ne!(
        run_command_with_stdin(&mut Command::new(corrupt_executable), &canonical)
            .status
            .code(),
        Some(91),
        "forward-child arena corruption was accepted"
    );

    for invalid in [vec![0], vec![3, b'a'], vec![1, b'9'], vec![2, b'a', 0xff]] {
        let executable = clang_link(
            "invalid",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O2",
        );
        assert_ne!(
            run_command_with_stdin(&mut Command::new(executable), &invalid)
                .status
                .code(),
            Some(91),
            "invalid framing returned success"
        );
    }

    let renamed = first.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(renamed, first, "tracked product omitted main");
    let allocation = TestWorkspace::new("allocation");
    let allocation_llvm = allocation.write("program.ll", renamed);
    let harness = allocation.write(
        "harness.c",
        allocation_harness(&canonical, &canonical_model),
    );
    let test_runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        "allocation",
        &allocation,
        &[
            allocation_llvm.as_path(),
            test_runtime.as_path(),
            harness.as_path(),
        ],
        "-O2",
    );
    assert_silent_exit_91(
        &Command::new(executable)
            .output()
            .expect("execute allocation harness"),
        "CAP-040 allocation/failure matrix",
    );

    let public = TestWorkspace::new("public-run");
    let public_source = public.write(
        "storage.aero",
        generated_program(kernel_prefix, &canonical_model, 0),
    );
    let mut run = Command::new(env!("CARGO_BIN_EXE_aero"));
    run.args([
        "run",
        public_source.to_str().expect("public source path is UTF-8"),
        "--language-profile",
        PROFILE_NAME,
    ])
    .current_dir(&public.root);
    let output = run_command_with_stdin(&mut run, &canonical);
    assert_eq!(
        output.status.code(),
        Some(91),
        "public CAP-040 runner failed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1
    );
    assert!(
        !stdout
            .lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:"))
    );
    assert!(
        output.stderr.is_empty(),
        "public CAP-040 runner emitted stderr"
    );

    for target in ["rocm", "cuda"] {
        let output_path = public.root.join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                public_source.to_str().expect("public source path is UTF-8"),
                "-o",
                output_path.to_str().expect("output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(&public.root)
            .output()
            .expect("execute CAP-040 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !output_path.exists(),
            "{target} rejection created an artifact"
        );
    }

    let workflow = fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH))
        .expect("read Rust workflow")
        .replace("\r\n", "\n");
    let linux = workflow_step(
        &workflow,
        "Test deterministic compiler storage arena at O0 and O2",
    );
    let windows = workflow_step(
        &workflow,
        "Test deterministic compiler storage arena on Windows at O0 and O2",
    );
    for anchor in [
        "deterministic_compiler_storage.aero",
        "exact-i32-byte-input-v0",
        "llvm-as-22",
        "opt-22",
        "llc-22",
        "-O0",
        "-O2",
        "Exit code: 91",
    ] {
        assert!(linux.contains(anchor), "Linux D1 gate omitted `{anchor}`");
    }
    for anchor in [
        "deterministic_compiler_storage.aero",
        "exact-i32-byte-input-v0",
        "llvm-as.exe",
        "opt.exe",
        "llc.exe",
        "-O0",
        "-O2",
        "Exit code: 91",
    ] {
        assert!(
            windows.contains(anchor),
            "Windows D1 gate omitted `{anchor}`"
        );
    }
}
