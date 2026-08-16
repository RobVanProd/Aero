use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_program, compile_program,
    verify_llvm_module,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

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

fn read(relative: &str) -> Option<String> {
    fs::read_to_string(repository_path(relative)).ok()
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
            &["bootstrap-drive-b1c", "BootstrapDriveB1c"][..],
        ),
        (WORKFLOW, &["Test runtime ASCII B1C toolchain driver"][..]),
    ];

    let ready = required.iter().all(|(relative, anchors)| {
        read(relative)
            .is_some_and(|contents| anchors.iter().all(|anchor| contents.contains(anchor)))
    });
    if !ready {
        panic!("{INTENTIONAL_PRODUCT_RED}");
    }
}
