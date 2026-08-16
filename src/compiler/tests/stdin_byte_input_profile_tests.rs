use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PROFILE_NAME: &str = "exact-i32-byte-input-v0";

const CHARACTERIZATION_SOURCE: &str = r#"
fn main() -> int {
    return 91;
}
"#;

const SOURCE_BYTE_BUFFER_PRODUCT: &str =
    include_str!("../../../examples/owned_byte_buffer_v0/source_owned_byte_buffer.aero");

const SINGLE_READ_SOURCE: &str = r#"
fn main() -> int {
    let read: Result<int, int> = stdin_read_byte();
    return match read {
        Ok(byte) => byte,
        Err(code) => 0 - code,
    };
}
"#;

const UNRESOLVED_CALL_SOURCE: &str = "fn main() -> int { return stdin_read_byte(); }";

const WHOLE_STREAM_PRODUCT: &str =
    include_str!("../../../examples/stdin_byte_input_v0/whole_stream_stdin.aero");

const MULTI_SIZE_SOURCE: &str = r#"
fn result_value(result: Result<int, int>) -> int {
    return match result {
        Ok(value) => value,
        Err(code) => 0 - code,
    };
}

fn main() -> int {
    let mut bytes: ByteBuffer = bytes_new();
    let mut reading: bool = 1 < 2;
    let mut sum: int = 0;
    while reading {
        let read: Result<int, int> = stdin_read_byte();
        let value: int = result_value(read);
        if value == -1 {
            reading = 1 > 2;
        } else {
            if value < 0 {
                return 10 - value;
            }
            let pushed: Result<int, int> = bytes_push(&mut bytes, value);
            if result_value(pushed) < 0 {
                return 20;
            }
            sum = sum + value;
        }
    }

    let length: int = bytes_len(&bytes);
    let capacity: int = bytes_capacity(&bytes);
    if length == 0 && capacity == 0 && sum == 0 {
        return 91;
    }
    if length == 3 && capacity == 8 && sum == 24 {
        let first: Result<int, int> = bytes_get(&bytes, 0);
        let last: Result<int, int> = bytes_get(&bytes, 2);
        if result_value(first) == 7 && result_value(last) == 9 {
            return 91;
        }
    }
    if length == 4097 && capacity == 8192 && sum == 522240 {
        let first: Result<int, int> = bytes_get(&bytes, 0);
        let middle: Result<int, int> = bytes_get(&bytes, 255);
        let last: Result<int, int> = bytes_get(&bytes, 4096);
        if result_value(first) == 0
            && result_value(middle) == 255
            && result_value(last) == 0 {
            return 91;
        }
    }
    return 1;
}
"#;

const INJECTED_FAILURE_SOURCE: &str = r#"
fn result_value(result: Result<int, int>) -> int {
    return match result {
        Ok(value) => value,
        Err(code) => 0 - code,
    };
}

fn main() -> int {
    let mut bytes: ByteBuffer = bytes_new();
    let mut reading: bool = 1 < 2;
    let mut failure: int = 0;
    while reading {
        let read: Result<int, int> = stdin_read_byte();
        let value: int = result_value(read);
        if value < 0 {
            failure = value;
            reading = 1 > 2;
        } else {
            let pushed: Result<int, int> = bytes_push(&mut bytes, value);
            if result_value(pushed) < 0 {
                return 70;
            }
        }
    }

    let length: int = bytes_len(&bytes);
    let capacity: int = bytes_capacity(&bytes);
    if failure == -2 && length == 2 {
        let first: Result<int, int> = bytes_get(&bytes, 0);
        let second: Result<int, int> = bytes_get(&bytes, 1);
        if result_value(first) == 65 && result_value(second) == 66 {
            return 91;
        }
    }
    if failure == -3 && length == 0 && capacity == 0 {
        return 92;
    }
    return 1;
}
"#;

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

fn options(language_profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile,
        ..CompilerOptions::default()
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository_root().join("target"))
            .join("r2-stdin-byte-input-tests");
        let root = parent.join(format!("{label}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&root).expect("create R2 native workspace");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write R2 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn clang_link(
    label: &str,
    workspace: &TestWorkspace,
    inputs: &[&Path],
    optimization: &str,
) -> PathBuf {
    let executable = workspace.path().join(if cfg!(windows) {
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
    for input in inputs {
        command.arg(input);
    }
    let output = command
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for R2 fixture");
    assert!(
        output.status.success(),
        "link {label} {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn run_command_with_stdin(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn R2 native fixture");
    child
        .stdin
        .take()
        .expect("piped R2 stdin")
        .write_all(input)
        .expect("write R2 binary stdin");
    child
        .wait_with_output()
        .expect("wait for R2 native fixture")
}

fn run_with_stdin(executable: &Path, input: &[u8]) -> Output {
    run_command_with_stdin(&mut Command::new(executable), input)
}

fn assert_exit_91(output: &Output, label: &str) {
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

#[test]
fn accepted_profiles_and_byte_buffer_are_frozen_before_r2() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            md5_hex(llvm.as_bytes()),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R2"
        );
        assert!(
            !llvm.contains("aero_stdin_read_byte"),
            "{profile:?} unexpectedly acquired the R2 runtime ABI"
        );
    }

    let byte_buffer_llvm = compile_program(
        SOURCE_BYTE_BUFFER_PRODUCT,
        options(LanguageProfile::ExactI32ByteBufferV0),
    )
    .expect("accepted R1 source-owned byte-buffer product compiles");
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
    ] {
        assert!(
            byte_buffer_llvm.contains(anchor),
            "accepted R1 LLVM omitted `{anchor}`"
        );
    }
    assert!(
        !byte_buffer_llvm.contains("aero_stdin_read_byte"),
        "accepted R1 product unexpectedly acquired R2 input"
    );

    let root = repository_root();
    for (relative, expected) in [
        (
            "src/compiler/runtime/aero_runtime.c",
            "090c9c07a5fa0a4c374b43953af8306f",
        ),
        (
            "src/compiler/runtime/aero_test_runtime.c",
            "5f1db08f29355e78a1dda31747ec7055",
        ),
        ("src/compiler/src/ir.rs", "2b8288bcbb2825586a0e406f37fbe12d"),
        (
            "src/compiler/src/ir_verifier.rs",
            "d5fae602214665b724c48c9ae8090a06",
        ),
    ] {
        assert_eq!(
            md5_hex(
                &fs::read(root.join(relative))
                    .unwrap_or_else(|error| panic!("read {relative}: {error}"))
            ),
            expected,
            "accepted authority `{relative}` drifted before R2"
        );
    }
}

#[test]
fn whole_stream_binary_stdin_profile_is_selector_red_first() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("R2 red: exact-i32-byte-input-v0 selector is absent");
    assert_eq!(profile.as_str(), PROFILE_NAME);
    check_program(SINGLE_READ_SOURCE, options(profile)).expect("R2 single read checks");
    let llvm =
        compile_program(SINGLE_READ_SOURCE, options(profile)).expect("R2 single read compiles");
    assert_eq!(
        llvm.matches("declare i32 @aero_stdin_read_byte()").count(),
        1
    );
    assert_eq!(llvm.matches("call i32 @aero_stdin_read_byte()").count(), 1);

    check_program(WHOLE_STREAM_PRODUCT, options(profile)).expect("R2 product checks");
    let first =
        compile_program(WHOLE_STREAM_PRODUCT, options(profile)).expect("R2 product compiles");
    let second = compile_program(WHOLE_STREAM_PRODUCT, options(profile))
        .expect("R2 product compiles deterministically");
    assert_eq!(first, second, "R2 product LLVM must be deterministic");
    assert_eq!(
        first.matches("call i32 @aero_stdin_read_byte()").count(),
        1,
        "the Aero loop must own repetition around one checked call site"
    );

    let workspace = TestWorkspace::new("source-parity");
    let source_path = workspace.write("whole-stream.aero", WHOLE_STREAM_PRODUCT);
    check_file(&source_path, options(profile)).expect("R2 product file checks");
    assert_eq!(
        compile_file(&source_path, options(profile)).expect("R2 product file compiles"),
        first,
        "R2 source/file LLVM diverged"
    );

    let unresolved = "Semantic Analysis Error: Unsupported function call `stdin_read_byte`: Error: Function `stdin_read_byte` is not defined.";
    for earlier in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
    ] {
        assert_eq!(
            check_program(UNRESOLVED_CALL_SOURCE, options(earlier))
                .expect_err("earlier profile must not acquire stdin input"),
            unresolved,
            "{earlier:?} changed its ordinary unresolved-call boundary"
        );
    }

    for (label, source, expected) in [
        (
            "argument-bearing",
            "fn main() -> int { let read: Result<int, int> = stdin_read_byte(1); return 0; }",
            "requires exactly zero arguments",
        ),
        (
            "discarded",
            "fn main() -> int { stdin_read_byte(); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "inferred",
            "fn main() -> int { let read = stdin_read_byte(); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "nested-call-argument",
            "fn consume(value: Result<int, int>) -> int { return 0; } fn main() -> int { return consume(stdin_read_byte()); }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "reserved-definition",
            "fn stdin_read_byte() -> int { return 0; } fn main() -> int { return 0; }",
            "intrinsic name `stdin_read_byte` is reserved",
        ),
        (
            "generic-context",
            "fn read<T>() -> int { let value: Result<int, int> = stdin_read_byte(); return 0; } fn main() -> int { return 0; }",
            "requires a direct nongeneric source function body",
        ),
    ] {
        let error = match check_program(source, options(profile)) {
            Ok(()) => panic!("{label} unexpectedly passed"),
            Err(error) => error,
        };
        assert!(
            error.contains(expected),
            "{label} stopped at the wrong boundary: {error}"
        );
        assert_eq!(
            compile_program(source, options(profile)).expect_err("invalid R2 source compiles"),
            error,
            "{label} compile diagnostic diverged"
        );
    }
}

#[test]
fn production_runtime_preserves_binary_bytes_sticky_eof_and_io_error() {
    let runtime = repository_root().join("src/compiler/runtime/aero_runtime.c");

    let binary = TestWorkspace::new("runtime-binary");
    let binary_harness = binary.write(
        "binary.c",
        r#"
#include <stdint.h>

extern int32_t aero_stdin_read_byte(void);

int main(void) {
    const int32_t expected[] = {0, 13, 10, 26, 255};
    for (unsigned i = 0; i < sizeof(expected) / sizeof(expected[0]); ++i) {
        if (aero_stdin_read_byte() != expected[i]) return 70 + (int)i;
    }
    if (aero_stdin_read_byte() != -1) return 80;
    if (aero_stdin_read_byte() != -1) return 81;
    return 91;
}
"#,
    );
    let executable = clang_link(
        "runtime-binary",
        &binary,
        &[runtime.as_path(), binary_harness.as_path()],
        "-O2",
    );
    assert_exit_91(
        &run_with_stdin(&executable, &[0, 13, 10, 26, 255]),
        "production runtime binary/sticky EOF",
    );

    let closed = TestWorkspace::new("runtime-closed");
    let closed_harness = closed.write(
        "closed.c",
        r#"
#include <stdint.h>
#ifdef _WIN32
#include <io.h>
#else
#include <unistd.h>
#endif

extern int32_t aero_stdin_read_byte(void);

int main(void) {
#ifdef _WIN32
    if (_close(0) != 0) return 70;
#else
    if (close(0) != 0) return 70;
#endif
    if (aero_stdin_read_byte() != -2) return 71;
    if (aero_stdin_read_byte() != -2) return 72;
    return 91;
}
"#,
    );
    let executable = clang_link(
        "runtime-closed",
        &closed,
        &[runtime.as_path(), closed_harness.as_path()],
        "-O2",
    );
    assert_exit_91(
        &run_with_stdin(&executable, &[]),
        "production runtime sticky I/O failure",
    );
}

#[test]
fn tracked_aero_product_consumes_binary_stdin_at_o0_and_o2() {
    let profile = LanguageProfile::ExactI32ByteInputV0;
    let llvm = compile_program(WHOLE_STREAM_PRODUCT, options(profile))
        .expect("R2 tracked product compiles");
    verify_llvm_module(&llvm, LlvmVerificationMode::Required)
        .expect("R2 tracked product passes required LLVM verification");
    let runtime = repository_root().join("src/compiler/runtime/aero_runtime.c");
    let input = [0, 13, 10, 26, 255, 1, 2, 3, 4];
    for optimization in ["-O0", "-O2"] {
        let workspace = TestWorkspace::new("tracked-product");
        let llvm_path = workspace.write("whole-stream.ll", &llvm);
        let executable = clang_link(
            "whole-stream",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            optimization,
        );
        assert_exit_91(
            &run_with_stdin(&executable, &input),
            &format!("R2 tracked product {optimization}"),
        );
    }
}

#[test]
fn aero_owned_loop_handles_empty_short_and_large_streams() {
    let llvm = compile_program(
        MULTI_SIZE_SOURCE,
        options(LanguageProfile::ExactI32ByteInputV0),
    )
    .expect("R2 multi-size product compiles");
    verify_llvm_module(&llvm, LlvmVerificationMode::Required)
        .expect("R2 multi-size product passes required LLVM verification");
    let workspace = TestWorkspace::new("multi-size");
    let llvm_path = workspace.write("multi-size.ll", &llvm);
    let runtime = repository_root().join("src/compiler/runtime/aero_runtime.c");
    let executable = clang_link(
        "multi-size",
        &workspace,
        &[llvm_path.as_path(), runtime.as_path()],
        "-O2",
    );
    let large = (0..4097)
        .map(|index| (index % 256) as u8)
        .collect::<Vec<_>>();
    for (label, input) in [
        ("empty", &[][..]),
        ("short", &[7, 8, 9][..]),
        ("large", large.as_slice()),
    ] {
        assert_exit_91(
            &run_with_stdin(&executable, input),
            &format!("R2 {label} whole stream"),
        );
    }
}

#[test]
fn injected_input_failures_preserve_prefix_map_error_and_close_owner() {
    let llvm = compile_program(
        INJECTED_FAILURE_SOURCE,
        options(LanguageProfile::ExactI32ByteInputV0),
    )
    .expect("R2 injected-failure product compiles");
    let renamed = llvm.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(llvm, renamed, "R2 injected-failure fixture omitted main");
    verify_llvm_module(&renamed, LlvmVerificationMode::Required)
        .expect("R2 renamed injected-failure fixture verifies");

    let workspace = TestWorkspace::new("injected-failure");
    let llvm_path = workspace.write("program.ll", renamed);
    let harness = workspace.write(
        "harness.c",
        r#"
#include <stdint.h>

extern int aero_program_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static int32_t mock_mode;
static int32_t mock_index;
static int32_t mock_sticky;

static void reset_mock(int32_t mode) {
    mock_mode = mode;
    mock_index = 0;
    mock_sticky = 0;
}

int32_t aero_stdin_read_byte(void) {
    if (mock_sticky != 0) return mock_sticky;
    if (mock_mode == 0) {
        const int32_t sequence[] = {65, 66, -2};
        int32_t value = sequence[mock_index < 3 ? mock_index : 2];
        if (mock_index < 3) ++mock_index;
        if (value < 0) mock_sticky = value;
        return value;
    }
    mock_sticky = -3;
    return mock_sticky;
}

static int clean_runtime(uint64_t allocations, uint64_t deallocations) {
    return aero_test_alloc_calls() == allocations &&
           aero_test_realloc_calls() == 0 &&
           aero_test_dealloc_calls() == deallocations &&
           aero_test_live_allocations() == 0 &&
           aero_test_size_mismatch_calls() == 0;
}

int main(void) {
    if (aero_test_reset(UINT64_MAX) != 1) return 70;
    reset_mock(0);
    if (aero_program_main() != 91) return 71;
    if (!clean_runtime(1, 1)) return 72;

    if (aero_test_reset(UINT64_MAX) != 1) return 73;
    reset_mock(1);
    if (aero_program_main() != 92) return 74;
    if (!clean_runtime(0, 0)) return 75;
    return 91;
}
"#,
    );
    let runtime = repository_root().join("src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        "injected-failure",
        &workspace,
        &[llvm_path.as_path(), runtime.as_path(), harness.as_path()],
        "-O2",
    );
    assert_exit_91(
        &Command::new(executable)
            .output()
            .expect("execute injected R2 failure fixture"),
        "R2 injected failures",
    );
}

#[test]
fn public_run_forwards_binary_stdin_and_accelerators_fail_before_artifacts() {
    let workspace = TestWorkspace::new("public-run");
    let source = workspace.write("whole-stream.aero", WHOLE_STREAM_PRODUCT);
    let mut run = Command::new(env!("CARGO_BIN_EXE_aero"));
    run.args([
        "run",
        source.to_str().expect("R2 source path is UTF-8"),
        "--language-profile",
        PROFILE_NAME,
    ])
    .current_dir(workspace.path());
    let output = run_command_with_stdin(&mut run, &[0, 13, 10, 26, 255, 1, 2, 3, 4]);
    assert_eq!(
        output.status.code(),
        Some(91),
        "public R2 run failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1,
        "public R2 run omitted its exit report: {stdout}"
    );
    assert!(output.stderr.is_empty(), "public R2 run emitted stderr");

    for target in ["rocm", "cuda"] {
        let output_path = workspace.path().join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                source.to_str().expect("R2 source path is UTF-8"),
                "-o",
                output_path.to_str().expect("R2 output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(workspace.path())
            .output()
            .expect("execute R2 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("requires --target cpu without --gpu"),
            "{target} stopped at the wrong boundary: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output_path.exists(),
            "{target} created a rejected artifact"
        );
    }
}

#[test]
fn r2_authority_is_structurally_closed_and_workflow_replays_binary_input() {
    let root = repository_root();
    let contract = read(&root, "src/compiler/src/byte_input_source_contract.rs");
    let ir = read(&root, "src/compiler/src/ir.rs");
    let verifier = read(&root, "src/compiler/src/ir_verifier.rs");
    let backend = read(&root, "src/compiler/src/code_generator.rs");
    let runtime = read(&root, "src/compiler/runtime/aero_runtime.c");
    let driver = read(&root, "src/compiler/src/main.rs");
    let workflow = read(&root, ".github/workflows/rust.yml");

    assert_eq!(
        contract.matches("pub(crate) const STDIN_READ_BYTE").count(),
        1
    );
    assert_eq!(ir.matches("CheckedStdinReadByte {").count(), 1);
    assert!(
        ir.contains("CheckedStdinReadByte {\n        result: Value,"),
        "R2 checked instruction acquired operands or a resource identity"
    );
    assert!(verifier.contains("| Inst::CheckedStdinReadByte { .. }"));
    assert!(verifier.contains("Some(LogicalType::Int)"));
    assert!(backend.contains("declare i32 @aero_stdin_read_byte()"));
    assert!(backend.contains("call i32 @aero_stdin_read_byte()"));
    assert_eq!(
        runtime
            .matches("int32_t aero_stdin_read_byte(void) {")
            .count(),
        1
    );
    for anchor in ["_O_BINARY", "fgetc(stdin)", "feof(stdin) ? -1 : -2"] {
        assert!(runtime.contains(anchor), "R2 runtime omitted `{anchor}`");
    }
    assert!(driver.contains("Stdio::inherit()"));
    assert!(driver.contains("Stdio::null()"));
    assert_eq!(
        workflow
            .matches("Test whole-stream binary stdin profile at O0 and O2")
            .count(),
        1
    );
    assert_eq!(
        workflow
            .matches("Test whole-stream binary stdin profile on Windows at O0 and O2")
            .count(),
        1
    );
    assert!(workflow.contains("[byte[]] $binaryInput = @(0, 13, 10, 26, 255, 1, 2, 3, 4)"));

    for relative in [
        "src/compiler/src/ast.rs",
        "src/compiler/src/parser.rs",
        "src/compiler/src/types.rs",
        "src/compiler/runtime/aero_test_runtime.c",
    ] {
        assert!(
            !read(&root, relative).contains("stdin_read_byte"),
            "R2 leaked its closed input identity into `{relative}`"
        );
    }
}
