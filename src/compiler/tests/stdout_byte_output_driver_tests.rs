use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCEPTED_B1B_PRODUCT: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_llvm_emitter.aero";
const B1C_PRODUCT: &str = "../../examples/aero_frontend_v0/runtime_ascii_toolchain_driver.aero";
const WORKFLOW: &str = "../../.github/workflows/rust.yml";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-047 intentional product red: bounded stdout/toolchain driver is absent";
const ACCEPTED_B1B_SOURCE_LENGTH: usize = 237_201;
const ACCEPTED_B1B_SOURCE_MD5: &str = "ac6f9306d7eb1f660623f52f75b34fba";
const ACCEPTED_B1B_SEAL: i32 = 611_963;
const CANONICAL_DRIVEN_RAW_FOLD: i32 = 964_575;
const CANONICAL_DRIVEN_SEAL: i32 = 506_643;
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
const CANONICAL_INPUT: &[u8] = b"fn score()->int{return 1+2*3-4/2;}";
const PROFILE_NAME: &str = "exact-i32-byte-io-v0";
const SINGLE_WRITE_SOURCE: &str = r#"
fn main() -> int {
    let written: Result<int, int> = stdout_write_byte(65);
    return match written {
        Ok(status) => status,
        Err(code) => 0 - code,
    };
}
"#;

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);
static B1C_LLVM: OnceLock<String> = OnceLock::new();

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
            .join("cap047-stdout-driver-tests");
        let root = parent.join(format!(
            "cap047-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-047 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-047 test artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap047-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn accepted_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
}

fn io_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteIoV0,
        ..CompilerOptions::default()
    }
}

fn checksum_step(checksum: i32, word: i32) -> i32 {
    i32::try_from((i64::from(checksum) * 31 + i64::from(word)) % 1_000_003)
        .expect("bounded CAP-047 checksum")
}

fn driven_seal(
    bytes: &[u8],
    status: i32,
    runtime_code: i32,
    byte_index: i32,
    attempted: i32,
) -> (i32, i32) {
    let mut checksum = 59;
    for byte in bytes {
        checksum = checksum_step(checksum, i32::from(*byte));
    }
    let raw_fold = checksum;
    for word in [
        997,
        ACCEPTED_B1B_SEAL,
        status,
        runtime_code,
        if byte_index < 0 { 0 } else { byte_index + 1 },
        attempted,
        i32::try_from(bytes.len()).expect("bounded driven length"),
    ] {
        checksum = checksum_step(checksum, word);
    }
    (raw_fold, checksum)
}

fn clang_link(
    workspace: &TestWorkspace,
    label: &str,
    optimization: &str,
    inputs: &[&Path],
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
        .expect("execute Clang for CAP-047 independent oracle");
    assert!(
        output.status.success(),
        "CAP-047 oracle link failed at {optimization} (stdout={:?}, stderr={:?})",
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
        .expect("spawn CAP-047 child");
    child
        .stdin
        .take()
        .expect("piped CAP-047 stdin")
        .write_all(input)
        .expect("write CAP-047 binary stdin");
    child.wait_with_output().expect("wait for CAP-047 child")
}

fn llvm_bin() -> PathBuf {
    for variable in ["AERO_LLVM_BIN", "LLVM_BIN"] {
        if let Some(path) = std::env::var_os(variable).map(PathBuf::from)
            && path
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
            && path
                .join(if cfg!(windows) {
                    "llvm-as.exe"
                } else {
                    "llvm-as"
                })
                .is_file()
        {
            return path;
        }
    }
    let path = std::env::var_os("PATH").expect("CAP-047 tests require PATH");
    std::env::split_paths(&path)
        .find(|directory| {
            directory
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
                && directory
                    .join(if cfg!(windows) {
                        "llvm-as.exe"
                    } else {
                        "llvm-as"
                    })
                    .is_file()
        })
        .expect("CAP-047 tests require an explicit LLVM bin directory on PATH")
}

fn read(relative: &str) -> Option<String> {
    fs::read_to_string(repository_path(relative)).ok()
}

fn compiled_b1c() -> &'static String {
    B1C_LLVM.get_or_init(|| {
        let product = fs::read_to_string(repository_path(B1C_PRODUCT)).expect("read B1C product");
        compile_program(&product, io_options()).expect("tracked B1C product compiles")
    })
}

#[test]
fn accepted_b1b_characterization_is_unchanged_before_b1c() {
    let bytes = fs::read(repository_path(ACCEPTED_B1B_PRODUCT)).expect("read accepted B1B product");
    assert_eq!(bytes.len(), ACCEPTED_B1B_SOURCE_LENGTH);
    assert_eq!(
        format!("{:x}", md5::compute(&bytes)),
        ACCEPTED_B1B_SOURCE_MD5
    );
    let source = std::str::from_utf8(&bytes).expect("accepted B1B source is UTF-8");
    for marker in [
        "// CAP-045 B1A VERIFIER BEGIN",
        "// CAP-045 B1A VERIFIER END",
        "// CAP-046 B1B LLVM EMITTER BEGIN",
        "// CAP-046 B1B LLVM EMITTER END",
        "// CAP-046 TRACKED SELF-TEST",
    ] {
        assert_eq!(
            source.matches(marker).count(),
            1,
            "accepted B1B marker {marker} drifted"
        );
    }
    assert!(!source.contains("stdout_write_byte"));

    check_program(source, accepted_options()).expect("accepted B1B product still checks");
    let first = compile_program(source, accepted_options()).expect("accepted B1B still compiles");
    let second =
        compile_program(source, accepted_options()).expect("accepted B1B still recompiles");
    assert_eq!(
        first, second,
        "accepted B1B outer LLVM became nondeterministic"
    );
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("accepted B1B outer LLVM still verifies");
    assert!(!first.contains("aero_stdout_write_byte"));
}

#[test]
fn independent_stream_and_external_tool_oracles_pass_before_product_red() {
    let bytes = CANONICAL_LLVM.as_bytes();
    assert_eq!(bytes.len(), 144);
    assert_eq!(
        format!("{:x}", md5::compute(bytes)),
        "fd2390d17d448d4539a72bf1991314dc"
    );
    assert_eq!(
        driven_seal(bytes, 0, 0, -1, 1),
        (CANONICAL_DRIVEN_RAW_FOLD, CANONICAL_DRIVEN_SEAL)
    );

    for fail_at in 0..bytes.len() {
        let prefix = &bytes[..fail_at];
        let (raw, seal) = driven_seal(
            prefix,
            2,
            1,
            i32::try_from(fail_at).expect("bounded failure index"),
            1,
        );
        assert!((0..1_000_003).contains(&raw));
        assert!((0..1_000_003).contains(&seal));
        assert_eq!(prefix, &CANONICAL_LLVM.as_bytes()[..fail_at]);
    }

    verify_llvm_module(CANONICAL_LLVM, LlvmVerificationMode::Required)
        .expect("independent canonical B1B module verifies");
    let workspace = TestWorkspace::new("independent-tools");
    let llvm = workspace.write("canonical.ll", CANONICAL_LLVM);
    let harness = workspace.write(
        "observer.c",
        concat!(
            "#include <stdint.h>\n",
            "extern int32_t aero_b1_entry(void);\n",
            "int main(void) { return aero_b1_entry() == 5 ? 91 : 1; }\n",
        ),
    );
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            &workspace,
            "canonical",
            optimization,
            &[llvm.as_path(), harness.as_path()],
        );
        let output = Command::new(executable)
            .output()
            .expect("run independent CAP-047 oracle");
        assert_eq!(
            output.status.code(),
            Some(91),
            "independent CAP-047 oracle failed at {optimization}"
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn exact_output_profile_is_closed_typed_and_deterministic() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("CAP-047 profile selector");
    assert_eq!(profile, LanguageProfile::ExactI32ByteIoV0);
    assert_eq!(profile.as_str(), PROFILE_NAME);

    check_program(SINGLE_WRITE_SOURCE, io_options()).expect("single checked write checks");
    let first = compile_program(SINGLE_WRITE_SOURCE, io_options()).expect("single write compiles");
    let second = compile_program(SINGLE_WRITE_SOURCE, io_options()).expect("write recompiles");
    assert_eq!(first, second, "CAP-047 LLVM became nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required).expect("single-write LLVM verifies");
    assert_eq!(
        first
            .matches("declare i32 @aero_stdout_write_byte(i32)")
            .count(),
        1
    );
    assert_eq!(
        first
            .matches("call i32 @aero_stdout_write_byte(i32 65)")
            .count(),
        1
    );

    let workspace = TestWorkspace::new("source-file-parity");
    let source = workspace.write("single-write.aero", SINGLE_WRITE_SOURCE);
    check_file(&source, io_options()).expect("single-write file checks");
    assert_eq!(
        compile_file(&source, io_options()).expect("single-write file compiles"),
        first
    );

    let unresolved_source = r#"
fn main() -> int {
    let written: Result<int, int> = stdout_write_byte(65);
    return 0;
}
"#;
    for earlier in [
        LanguageProfile::Experimental,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
        LanguageProfile::ExactI32ByteInputV0,
    ] {
        let error = check_program(
            unresolved_source,
            CompilerOptions {
                language_profile: earlier,
                ..CompilerOptions::default()
            },
        )
        .expect_err("earlier profile must not acquire stdout output");
        assert!(
            error.contains("Function `stdout_write_byte` is not defined"),
            "{earlier:?} stopped at the wrong boundary: {error}"
        );
        assert_eq!(
            compile_program(
                unresolved_source,
                CompilerOptions {
                    language_profile: earlier,
                    ..CompilerOptions::default()
                }
            )
            .expect_err("earlier profile unexpectedly compiled output"),
            error
        );
    }
    for (earlier, expected) in [
        (
            LanguageProfile::StableScalarV0,
            "Language Profile Error: stable-scalar-v0 rejects binding annotation types",
        ),
        (
            LanguageProfile::ExactI32ArrayV0,
            "Language Profile Error: exact-i32-array-v0 rejects binding annotation types",
        ),
    ] {
        let options = CompilerOptions {
            language_profile: earlier,
            ..CompilerOptions::default()
        };
        assert_eq!(
            check_program(unresolved_source, options.clone()).unwrap_err(),
            expected
        );
        assert_eq!(
            compile_program(unresolved_source, options).unwrap_err(),
            expected
        );
    }

    for (label, source, expected) in [
        (
            "zero arguments",
            "fn main() -> int { let value: Result<int, int> = stdout_write_byte(); return 0; }",
            "requires exactly one argument",
        ),
        (
            "two arguments",
            "fn main() -> int { let value: Result<int, int> = stdout_write_byte(1, 2); return 0; }",
            "requires exactly one argument",
        ),
        (
            "wrong type",
            "fn main() -> int { let value: Result<int, int> = stdout_write_byte(1 < 2); return 0; }",
            "argument has type bool, expected int",
        ),
        (
            "discarded",
            "fn main() -> int { stdout_write_byte(65); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "inferred",
            "fn main() -> int { let value = stdout_write_byte(65); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "nested",
            "fn consume(value: Result<int, int>) -> int { return 0; } fn main() -> int { return consume(stdout_write_byte(65)); }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "reserved definition",
            "fn stdout_write_byte(value: int) -> int { return value; } fn main() -> int { return 0; }",
            "intrinsic name `stdout_write_byte` is reserved",
        ),
        (
            "generic context",
            "fn emit<T>() -> int { let value: Result<int, int> = stdout_write_byte(65); return 0; } fn main() -> int { return 0; }",
            "requires a direct nongeneric source function body",
        ),
    ] {
        let error = match check_program(source, io_options()) {
            Ok(()) => panic!("{label} unexpectedly passed"),
            Err(error) => error,
        };
        assert!(
            error.contains(expected),
            "{label} stopped at the wrong boundary: {error}"
        );
        assert_eq!(
            compile_program(source, io_options()).expect_err("invalid source compiled"),
            error
        );
    }
}

#[test]
fn production_runtime_writes_raw_bytes_and_makes_failures_sticky() {
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    let workspace = TestWorkspace::new("runtime-bytes");
    let harness = workspace.write(
        "runtime-bytes.c",
        r#"
#include <stdint.h>
extern int32_t aero_stdout_write_byte(int32_t value);
int main(void) {
    const int32_t bytes[] = {0, 13, 10, 26, 255};
    for (unsigned i = 0; i < sizeof(bytes) / sizeof(bytes[0]); ++i) {
        if (aero_stdout_write_byte(bytes[i]) != 0) return 70 + (int)i;
    }
    if (aero_stdout_write_byte(256) != -3) return 80;
    if (aero_stdout_write_byte(65) != -3) return 81;
    return 91;
}
"#,
    );
    let executable = clang_link(
        &workspace,
        "runtime-bytes",
        "-O2",
        &[runtime.as_path(), harness.as_path()],
    );
    let output = Command::new(executable)
        .output()
        .expect("run production stdout runtime");
    assert_eq!(output.status.code(), Some(91));
    assert_eq!(output.stdout, [0, 13, 10, 26, 255]);
    assert!(output.stderr.is_empty());

    let closed = TestWorkspace::new("runtime-closed");
    let harness = closed.write(
        "runtime-closed.c",
        r#"
#include <stdint.h>
#ifdef _WIN32
#include <io.h>
#else
#include <unistd.h>
#endif
extern int32_t aero_stdout_write_byte(int32_t value);
int main(void) {
#ifdef _WIN32
    if (_close(1) != 0) return 70;
    if (aero_stdout_write_byte(65) != -2) return 71;
    if (aero_stdout_write_byte(66) != -2) return 72;
#else
    int descriptors[2];
    if (pipe(descriptors) != 0) return 70;
    if (close(descriptors[0]) != 0) return 71;
    if (dup2(descriptors[1], 1) == -1) return 72;
    if (close(descriptors[1]) != 0) return 73;
    if (aero_stdout_write_byte(65) != -1) return 74;
    if (aero_stdout_write_byte(66) != -1) return 75;
#endif
    return 91;
}
"#,
    );
    let executable = clang_link(
        &closed,
        "runtime-closed",
        "-O2",
        &[runtime.as_path(), harness.as_path()],
    );
    let status = Command::new(executable)
        .status()
        .expect("run closed stdout fixture");
    assert_eq!(
        status.code(),
        Some(91),
        "closed output did not stay in-process"
    );
}

#[test]
fn tracked_b1c_product_preserves_b1b_and_emits_the_exact_module_at_o0_and_o2() {
    let product = fs::read_to_string(repository_path(B1C_PRODUCT)).expect("read B1C product");
    let predecessor =
        fs::read_to_string(repository_path(ACCEPTED_B1B_PRODUCT)).expect("read B1B product");
    let section = |source: &str| {
        source
            .split_once("// CAP-046 B1B LLVM EMITTER BEGIN")
            .and_then(|(_, suffix)| {
                suffix
                    .split_once("// CAP-046 B1B LLVM EMITTER END")
                    .map(|(body, _)| body)
            })
            .expect("isolate accepted B1B section")
            .to_string()
    };
    assert_eq!(section(&product), section(&predecessor));
    assert_eq!(
        product.matches(": ByteBuffer = bytes_new();").count(),
        predecessor.matches(": ByteBuffer = bytes_new();").count(),
        "B1C added a ByteBuffer owner"
    );
    for marker in [
        "// CAP-047 B1C STDOUT DRIVER BEGIN",
        "// CAP-047 B1C STDOUT DRIVER END",
        "// CAP-047 TRACKED SELF-TEST",
    ] {
        assert_eq!(product.matches(marker).count(), 1);
    }

    check_program(&product, io_options()).expect("tracked B1C product checks");
    let first = compiled_b1c().clone();
    let second = compile_program(&product, io_options()).expect("tracked B1C product recompiles");
    assert_eq!(first, second, "tracked B1C LLVM became nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked B1C outer LLVM verifies");
    assert_eq!(
        first
            .matches("declare i32 @aero_stdout_write_byte(i32)")
            .count(),
        1
    );
    assert_eq!(
        first
            .matches("call i32 @aero_stdout_write_byte(i32")
            .count(),
        1,
        "the Aero loop must own repetition around one checked call site"
    );
    let workspace = TestWorkspace::new("tracked-product");
    let source_path = workspace.write("runtime-ascii-toolchain-driver.aero", &product);
    check_file(&source_path, io_options()).expect("tracked B1C file checks");
    assert_eq!(
        compile_file(&source_path, io_options()).expect("tracked B1C file compiles"),
        first
    );
    let llvm = workspace.write("runtime-ascii-toolchain-driver.ll", first);
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            &workspace,
            "runtime-ascii-toolchain-driver",
            optimization,
            &[llvm.as_path(), runtime.as_path()],
        );
        let output = run_command_with_stdin(&mut Command::new(executable), CANONICAL_INPUT);
        assert_eq!(output.status.code(), Some(91));
        assert_eq!(output.stdout, CANONICAL_LLVM.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn tracked_product_stops_at_every_output_failure_and_releases_all_owners() {
    let renamed =
        compiled_b1c().replacen("define i32 @main()", "define i32 @aero_product_main()", 1);
    assert_ne!(renamed, *compiled_b1c());
    verify_llvm_module(&renamed, LlvmVerificationMode::Required)
        .expect("renamed B1C product verifies");
    let workspace = TestWorkspace::new("all-output-boundaries");
    let llvm = workspace.write("product.ll", renamed);
    let harness = workspace.write(
        "boundaries.c",
        r#"
#include <stddef.h>
#include <stdint.h>
#include <string.h>

extern int32_t aero_product_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static const unsigned char input[] = "fn score()->int{return 1+2*3-4/2;}";
static const unsigned char expected[] =
    "define i32 @aero_b1_entry() {\n"
    "entry:\n"
    "  %r1 = mul i32 2, 3\n"
    "  %r2 = add i32 1, %r1\n"
    "  %r3 = sdiv i32 4, 2\n"
    "  %r4 = sub i32 %r2, %r3\n"
    "  ret i32 %r4\n"
    "}\n";
static size_t input_index;
static size_t output_length;
static size_t fail_at;
static int32_t output_sticky;
static unsigned char output[sizeof(expected) - 1];

int32_t aero_stdin_read_byte(void) {
    if (input_index < sizeof(input) - 1) return input[input_index++];
    return -1;
}

int32_t aero_stdout_write_byte(int32_t value) {
    if (output_sticky != 0) return output_sticky;
    if (value < 0 || value > 255) {
        output_sticky = -3;
        return output_sticky;
    }
    if (output_length == fail_at) {
        output_sticky = -1;
        return output_sticky;
    }
    output[output_length++] = (unsigned char)value;
    return 0;
}

static void reset_io(size_t boundary) {
    input_index = 0;
    output_length = 0;
    fail_at = boundary;
    output_sticky = 0;
    memset(output, 0, sizeof(output));
}

int main(void) {
    for (size_t boundary = 0; boundary <= sizeof(expected) - 1; ++boundary) {
        if (aero_test_reset(UINT64_MAX) != 1) return 60;
        reset_io(boundary);
        int32_t result = aero_product_main();
        int32_t expected_result = boundary < sizeof(expected) - 1 ? 95 : 91;
        size_t expected_length = boundary < sizeof(expected) - 1
                                     ? boundary
                                     : sizeof(expected) - 1;
        if (result != expected_result) return 61;
        if (output_length != expected_length) return 62;
        if (memcmp(output, expected, expected_length) != 0) return 63;
        if (aero_test_alloc_calls() != 14) return 64;
        if (aero_test_realloc_calls() != 58) return 65;
        if (aero_test_dealloc_calls() != 14) return 66;
        if (aero_test_live_allocations() != 0) return 67;
        if (aero_test_size_mismatch_calls() != 0) return 68;
    }
    return 91;
}
"#,
    );
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        &workspace,
        "all-output-boundaries",
        "-O2",
        &[llvm.as_path(), runtime.as_path(), harness.as_path()],
    );
    let output = Command::new(executable)
        .output()
        .expect("run every CAP-047 output boundary");
    assert_eq!(
        output.status.code(),
        Some(91),
        "boundary fixture failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn external_driver_is_explicit_transactional_and_green_at_o0_and_o2() {
    let workspace = TestWorkspace::new("external-driver");
    let emitter_llvm = workspace.write("tracked-b1c-emitter.ll", compiled_b1c().as_bytes());
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    let emitter = clang_link(
        &workspace,
        "tracked-b1c-emitter",
        "-O2",
        &[emitter_llvm.as_path(), runtime.as_path()],
    );
    let llvm_bin = llvm_bin();
    for level in ["0", "2"] {
        let output_dir = workspace.root.join(format!("driver-o{level}"));
        assert!(!output_dir.exists());
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .arg("bootstrap-drive-b1c")
            .arg(&emitter)
            .arg("--llvm-bin")
            .arg(&llvm_bin)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--opt-level")
            .arg(level)
            .stdin(Stdio::null())
            .output()
            .expect("run CAP-047 driver");
        assert_eq!(
            output.status.code(),
            Some(0),
            "driver O{level} failed (stdout={:?}, stderr={:?})",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(
            fs::read(output_dir.join("module.ll")).expect("read published module"),
            CANONICAL_LLVM.as_bytes()
        );
        for artifact in [
            "module.ll",
            "module.bc",
            if cfg!(windows) {
                "module.obj"
            } else {
                "module.o"
            },
            "observer.c",
            if cfg!(windows) {
                "observer.obj"
            } else {
                "observer.o"
            },
            if cfg!(windows) { "probe.exe" } else { "probe" },
        ] {
            assert!(output_dir.join(artifact).is_file(), "missing {artifact}");
        }

        let before = fs::read(output_dir.join("module.ll")).expect("read accepted module");
        let repeated = Command::new(env!("CARGO_BIN_EXE_aero"))
            .arg("bootstrap-drive-b1c")
            .arg(&emitter)
            .arg("--llvm-bin")
            .arg(&llvm_bin)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--opt-level")
            .arg(level)
            .output()
            .expect("rerun driver against existing directory");
        assert_eq!(repeated.status.code(), Some(1));
        assert_eq!(fs::read(output_dir.join("module.ll")).unwrap(), before);
    }

    let malformed_source = workspace.write(
        "malformed.c",
        r#"
#include <stdio.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif
int main(void) {
    static const unsigned char module[] =
        "xefine i32 @aero_b1_entry() {\n"
        "entry:\n"
        "  %r1 = mul i32 2, 3\n"
        "  %r2 = add i32 1, %r1\n"
        "  %r3 = sdiv i32 4, 2\n"
        "  %r4 = sub i32 %r2, %r3\n"
        "  ret i32 %r4\n"
        "}\n";
#ifdef _WIN32
    if (_setmode(_fileno(stdout), _O_BINARY) == -1) return 1;
#endif
    if (fwrite(module, 1, sizeof(module) - 1, stdout) != sizeof(module) - 1) return 1;
    return fflush(stdout) == 0 ? 91 : 1;
}
"#,
    );
    let malformed = clang_link(
        &workspace,
        "malformed",
        "-O2",
        &[malformed_source.as_path()],
    );
    let rejected_dir = workspace.root.join("rejected-stream");
    let rejected = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("bootstrap-drive-b1c")
        .arg(malformed)
        .arg("--llvm-bin")
        .arg(&llvm_bin)
        .arg("--output-dir")
        .arg(&rejected_dir)
        .arg("--opt-level")
        .arg("2")
        .output()
        .expect("run malformed stream rejection");
    assert_eq!(rejected.status.code(), Some(1));
    assert!(
        !rejected_dir.exists(),
        "rejected stream published artifacts"
    );

    let empty_tools = workspace.root.join("empty-tools");
    fs::create_dir(&empty_tools).expect("create explicit empty tool directory");
    let missing_tool_output = workspace.root.join("missing-tool-output");
    let rejected = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("bootstrap-drive-b1c")
        .arg(&emitter)
        .arg("--llvm-bin")
        .arg(&empty_tools)
        .arg("--output-dir")
        .arg(&missing_tool_output)
        .arg("--opt-level")
        .arg("0")
        .output()
        .expect("run no-PATH-fallback rejection");
    assert_eq!(rejected.status.code(), Some(1));
    assert!(!missing_tool_output.exists());

    let invocation = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("bootstrap-drive-b1c")
        .arg(&emitter)
        .arg("--llvm-bin")
        .arg(&llvm_bin)
        .arg("--output-dir")
        .arg("relative-output")
        .arg("--opt-level")
        .arg("1")
        .output()
        .expect("run parser rejection");
    assert_eq!(invocation.status.code(), Some(2));
}

#[test]
fn bounded_stdout_and_toolchain_driver_product_is_present() {
    let required = [
        (
            B1C_PRODUCT,
            &[
                "// CAP-047 B1C STDOUT DRIVER BEGIN",
                "stdout_write_byte",
                "driven_checksum",
                "// CAP-047 TRACKED SELF-TEST",
            ][..],
        ),
        (
            "../../src/compiler/src/byte_output_source_contract.rs",
            &["STDOUT_WRITE_BYTE", "stdout_write_byte"][..],
        ),
        (
            "../../src/compiler/runtime/aero_runtime.c",
            &["int32_t aero_stdout_write_byte(int32_t value)"][..],
        ),
        (
            "../../src/compiler/src/lib.rs",
            &["mod byte_output_source_contract;"][..],
        ),
        (
            "../../src/compiler/src/language_profile.rs",
            &["exact-i32-byte-io-v0", "ExactI32ByteIoV0"][..],
        ),
        (
            "../../src/compiler/src/ir.rs",
            &["CheckedStdoutWriteByte"][..],
        ),
        (
            "../../src/compiler/src/semantic_analyzer.rs",
            &["byte_output_source_enabled"][..],
        ),
        (
            "../../src/compiler/src/ir_generator.rs",
            &["CheckedStdoutWriteByte", "generate_byte_output_intrinsic"][..],
        ),
        (
            "../../src/compiler/src/ir_verifier.rs",
            &["CheckedStdoutWriteByte"][..],
        ),
        (
            "../../src/compiler/src/code_generator.rs",
            &[
                "declare i32 @aero_stdout_write_byte(i32)",
                "call i32 @aero_stdout_write_byte(i32",
            ][..],
        ),
        (
            "../../src/compiler/src/main.rs",
            &[
                "bootstrap-drive-b1c",
                "BootstrapDriveB1c",
                "B1C_CANONICAL_INPUT",
                "write_all(B1C_CANONICAL_INPUT)",
            ][..],
        ),
        (
            WORKFLOW,
            &[
                "Test runtime ASCII B1C toolchain driver",
                "emitter=\"${executable_o2}\"",
                "$emitter = $executableO2",
            ][..],
        ),
    ];

    let ready = required.iter().all(|(relative, anchors)| {
        read(relative)
            .is_some_and(|contents| anchors.iter().all(|anchor| contents.contains(anchor)))
    });
    if !ready {
        panic!("{INTENTIONAL_PRODUCT_RED}");
    }
}
