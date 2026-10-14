use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_program, compile_program,
    verify_llvm_module,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE: &str = r#"
fn main() -> int {
    let written: Result<int, int> = stderr_write_byte(65);
    return match written { Ok(status) => status, Err(code) => 0 - code, };
}
"#;

fn options(profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile: profile,
        ..CompilerOptions::default()
    }
}

fn diagnostic_options() -> CompilerOptions {
    options(LanguageProfile::ExactI32ByteDiagnosticsV0)
}

#[test]
fn direct_stderr_source_reaches_authenticated_checked_output() {
    check_program(SOURCE, diagnostic_options()).expect("H1E source stderr must check");
    let first = compile_program(SOURCE, diagnostic_options()).expect("H1E stderr source compiles");
    assert_eq!(
        compile_program(SOURCE, diagnostic_options()).unwrap(),
        first
    );
    assert_eq!(
        first
            .matches("declare i32 @aero_stderr_write_byte(i32)")
            .count(),
        1
    );
    assert_eq!(
        first
            .matches("call i32 @aero_stderr_write_byte(i32 65)")
            .count(),
        1
    );
    assert!(!first.contains("aero_stdout_write_byte"));
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("stderr source LLVM verifies");
}

#[test]
fn earlier_profiles_preserve_unresolved_and_ordinary_helper_boundaries() {
    let ordinary = "fn stderr_write_byte(value: int) -> int { return value; } fn main() -> int { return stderr_write_byte(91); }";
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
        LanguageProfile::ExactI32ByteInputV0,
        LanguageProfile::ExactI32ByteIoV0,
    ] {
        let error = check_program(SOURCE, options(profile))
            .expect_err("earlier profile rejects stderr source");
        assert_eq!(
            compile_program(SOURCE, options(profile)).unwrap_err(),
            error
        );
        let expected = match profile {
            LanguageProfile::StableScalarV0 | LanguageProfile::ExactI32ArrayV0 => {
                "rejects binding annotation types"
            }
            _ => "Function `stderr_write_byte` is not defined",
        };
        assert!(error.contains(expected), "{profile}: {error}");
        let llvm =
            compile_program(ordinary, options(profile)).expect("earlier ordinary helper compiles");
        assert!(
            !llvm.contains("aero_stderr_write_byte"),
            "{profile} acquired the stderr ABI"
        );
    }
    let error = check_program(ordinary, diagnostic_options())
        .expect_err("diagnostic profile reserves stderr name");
    assert!(
        error.contains("intrinsic name `stderr_write_byte` is reserved"),
        "{error}"
    );
}

#[test]
fn stderr_source_rejects_invalid_context_argument_and_raw_abi() {
    for (label, source, expected) in [
        (
            "no arguments",
            "fn main() -> int { let value: Result<int, int> = stderr_write_byte(); return 0; }",
            "requires exactly one argument",
        ),
        (
            "two arguments",
            "fn main() -> int { let value: Result<int, int> = stderr_write_byte(1, 2); return 0; }",
            "requires exactly one argument",
        ),
        (
            "bool",
            "fn main() -> int { let value: Result<int, int> = stderr_write_byte(1 < 2); return 0; }",
            "argument has type bool, expected int",
        ),
        (
            "discarded",
            "fn main() -> int { stderr_write_byte(65); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "inferred",
            "fn main() -> int { let value = stderr_write_byte(65); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "wrong annotation",
            "fn main() -> int { let value: int = stderr_write_byte(65); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "nested",
            "fn consume(value: Result<int, int>) -> int { return 0; } fn main() -> int { return consume(stderr_write_byte(65)); }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "return",
            "fn emit() -> Result<int, int> { return stderr_write_byte(65); } fn main() -> int { return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "generic",
            "fn emit<T>() -> int { let value: Result<int, int> = stderr_write_byte(65); return 0; } fn main() -> int { return 0; }",
            "requires a direct nongeneric source function body",
        ),
        (
            "global initializer",
            "let value: Result<int, int> = stderr_write_byte(65); fn main() -> int { return 0; }",
            "requires a direct nongeneric source function body",
        ),
        (
            "shadow",
            "fn stderr_write_byte(value: int) -> int { return value; } fn main() -> int { return 0; }",
            "intrinsic name `stderr_write_byte` is reserved",
        ),
        (
            "raw ABI",
            "fn main() -> int { return aero_stderr_write_byte(65); }",
            "Function `aero_stderr_write_byte` is not defined",
        ),
    ] {
        let error = check_program(source, diagnostic_options()).expect_err(label);
        assert!(error.contains(expected), "{label}: {error}");
        assert_eq!(
            compile_program(source, diagnostic_options()).unwrap_err(),
            error,
            "{label}: compile/check parity"
        );
    }
    let collision = "fn aero_stderr_write_byte(value: int) -> int { return value; } fn main() -> int { let result: Result<int, int> = stderr_write_byte(65); return 0; }";
    let error = check_program(collision, diagnostic_options())
        .expect_err("runtime symbol collision must stop before backend");
    assert!(
        error.contains("aero_stderr_write_byte` is reserved"),
        "{error}"
    );
    assert_eq!(
        compile_program(collision, diagnostic_options()).unwrap_err(),
        error
    );
}

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);
struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
        let root = parent.join(format!(
            "h1e-source-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        Self(fs::canonicalize(root).unwrap())
    }

    fn assert_native(&self, label: &str, source: &str, stdout: &[u8], stderr: &[u8]) {
        let llvm = compile_program(source, diagnostic_options()).expect("native source compiles");
        verify_llvm_module(&llvm, LlvmVerificationMode::Required).expect("native LLVM verifies");
        let llvm_file = self.0.join(format!("{label}.ll"));
        fs::write(&llvm_file, llvm).unwrap();
        for optimization in ["-O0", "-O2"] {
            let executable = self.0.join(format!(
                "{label}-{optimization}{}",
                if cfg!(windows) { ".exe" } else { "" }
            ));
            let runtime = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime");
            let linked = Command::new("clang")
                .args([
                    "-std=c11",
                    optimization,
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-Wno-error=override-module",
                ])
                .arg(&llvm_file)
                .arg(runtime.join("aero_runtime.c"))
                .arg(runtime.join("aero_diagnostic_runtime.c"))
                .arg("-o")
                .arg(&executable)
                .output()
                .expect("link source diagnostics");
            assert!(
                linked.status.success(),
                "{label} {optimization}: {linked:?}"
            );
            let output = Command::new(executable)
                .output()
                .expect("run source diagnostics");
            assert_eq!(
                output.status.code(),
                Some(91),
                "{label} {optimization}: {output:?}"
            );
            assert_eq!(output.stdout, stdout, "{label} {optimization}: stdout");
            assert_eq!(output.stderr, stderr, "{label} {optimization}: stderr");
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        if self
            .0
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("h1e-source-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn source_stderr_binary_and_computed_bytes_at_o0_o2() {
    let source = r#"
fn computed(value: int) -> int { return value + 1; }
fn head(values: [int; 1]) -> int { return values[0]; }
fn emit(value: int) -> int {
    let written: Result<int, int> = stderr_write_byte(value);
    return match written { Ok(status) => status, Err(code) => code, };
}
fn main() -> int {
    let mut value: int = 0;
    while value < 256 {
        let status: int = emit(value);
        if status != 0 { return 71; }
        value = value + 1;
    }
    let written: Result<int, int> = stderr_write_byte(computed(64));
    let status: int = match written { Ok(status) => status, Err(code) => code, };
    if status != 0 { return 72; }
    let array_written: Result<int, int> = stderr_write_byte(head([computed(64)]));
    let array_status: int = match array_written { Ok(status) => status, Err(code) => code, };
    if array_status != 0 { return 73; }
    return 91;
}
"#;
    let mut expected: Vec<u8> = (0..=255).collect();
    expected.push(65);
    expected.push(65);
    Workspace::new().assert_native("binary", source, b"", &expected);
}

#[test]
fn source_stderr_range_errors_are_typed_and_sticky_at_o0_o2() {
    let workspace = Workspace::new();
    for (label, invalid) in [
        ("min", "0 - 2147483647 - 1"),
        ("negative", "0 - 1"),
        ("high", "256"),
        ("max", "2147483647"),
    ] {
        let source = format!(
            r#"
fn main() -> int {{
    let failed: Result<int, int> = stderr_write_byte({invalid});
    let first: int = match failed {{ Ok(status) => 71, Err(code) => code, }};
    if first != 3 {{ return 72; }}
    let sticky: Result<int, int> = stderr_write_byte(65);
    let second: int = match sticky {{ Ok(status) => 73, Err(code) => code, }};
    if second != 3 {{ return 74; }}
    return 91;
}}
"#
        );
        workspace.assert_native(label, &source, b"", b"");
    }
}

#[test]
fn source_stdout_and_stderr_states_are_independent_at_o0_o2() {
    let workspace = Workspace::new();
    for (label, failed_channel, live_channel, stdout, stderr) in [
        (
            "stderr-failed",
            "stderr_write_byte",
            "stdout_write_byte",
            &[0, 13, 10, 26, 255][..],
            &[][..],
        ),
        (
            "stdout-failed",
            "stdout_write_byte",
            "stderr_write_byte",
            &[][..],
            &[0, 13, 10, 26, 255][..],
        ),
    ] {
        let source = format!(
            r#"
fn main() -> int {{
    let failed: Result<int, int> = {failed_channel}(256);
    let error: int = match failed {{ Ok(status) => 71, Err(code) => code, }};
    if error != 3 {{ return 72; }}
    let values: [int; 5] = [0, 13, 10, 26, 255];
    let mut index: int = 0;
    while index < 5 {{
        let written: Result<int, int> = {live_channel}(values[index]);
        let status: int = match written {{ Ok(status) => status, Err(code) => code, }};
        if status != 0 {{ return 73; }}
        index = index + 1;
    }}
    let sticky: Result<int, int> = {failed_channel}(65);
    let final_error: int = match sticky {{ Ok(status) => 74, Err(code) => code, }};
    if final_error != 3 {{ return 75; }}
    return 91;
}}
"#
        );
        workspace.assert_native(label, &source, stdout, stderr);
    }
}
