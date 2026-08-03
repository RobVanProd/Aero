use serde_json::Value;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SUCCESS: i32 = 0;
const OPERATIONAL_FAILURE: i32 = 1;
const INVOCATION_FAILURE: i32 = 2;
const LIVE_REGISTRY_DISABLED: &str =
    "live registry transport is disabled pending a reviewed protocol and trust boundary";

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-cli-status-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CLI status workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, contents).expect("write fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        clear_readonly_recursively(&self.root);
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn clear_readonly_recursively(root: &Path) {
    let Ok(metadata) = fs::symlink_metadata(root) else {
        return;
    };
    if metadata.is_dir() {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                clear_readonly_recursively(&entry.path());
            }
        }
    }
    make_writable(root);
}

fn make_writable(path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    let mut permissions = metadata.permissions();

    #[cfg(windows)]
    {
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = permissions.mode();
        if mode & 0o200 == 0 {
            permissions.set_mode(mode | 0o200);
            let _ = fs::set_permissions(path, permissions);
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        #[allow(clippy::permissions_set_readonly_false)]
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn path_arg(path: &Path) -> OsString {
    path.as_os_str().to_os_string()
}

fn run_aero(workspace: &TestWorkspace, arguments: &[OsString]) -> Output {
    run_aero_configured(workspace, arguments, |_| {})
}

fn run_aero_configured(
    workspace: &TestWorkspace,
    arguments: &[OsString],
    configure: impl FnOnce(&mut Command),
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command
        .current_dir(&workspace.root)
        .args(arguments)
        .stdin(Stdio::null());
    configure(&mut command);
    command.output().expect("run Aero CLI")
}

fn run_aero_with_stdin(workspace: &TestWorkspace, arguments: &[OsString], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Aero CLI with controlled stdin");
    child
        .stdin
        .take()
        .expect("take Aero CLI stdin")
        .write_all(input)
        .expect("write Aero CLI stdin");
    child.wait_with_output().expect("wait for Aero CLI")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn strip_ansi(input: &str) -> String {
    let mut visible = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            visible.push(ch);
        }
    }
    visible
}

fn has_source_analysis_line(output: &str, expected_name: &str) -> bool {
    output.lines().any(|line| {
        let Some(path) = line.trim_start().strip_prefix("Analyzing ") else {
            return false;
        };
        let normalized = path.trim_end().replace('\\', "/");
        normalized == expected_name || normalized.ends_with(&format!("/{expected_name}"))
    })
}

fn record_case(
    failures: &mut Vec<String>,
    label: &str,
    output: &Output,
    expected_code: i32,
    stdout_fragments: &[&str],
    stderr_fragments: &[&str],
    stdout_must_be_empty: bool,
    stderr_must_be_empty: bool,
    forbidden_fragments: &[&str],
) {
    let out = stdout(output);
    let err = stderr(output);
    let actual_code = output.status.code();
    if actual_code != Some(expected_code) {
        failures.push(format!(
            "{label}: expected exit {expected_code}, received {actual_code:?}; stdout={out:?}; stderr={err:?}"
        ));
    }
    for fragment in stdout_fragments {
        if !out.contains(fragment) {
            failures.push(format!(
                "{label}: stdout omitted {fragment:?}; stdout={out:?}; stderr={err:?}"
            ));
        }
        if err.contains(fragment) {
            failures.push(format!(
                "{label}: stdout fragment {fragment:?} appeared on stderr; stdout={out:?}; stderr={err:?}"
            ));
        }
    }
    for fragment in stderr_fragments {
        if !err.contains(fragment) {
            failures.push(format!(
                "{label}: stderr omitted {fragment:?}; stdout={out:?}; stderr={err:?}"
            ));
        }
        if out.contains(fragment) {
            failures.push(format!(
                "{label}: stderr fragment {fragment:?} appeared on stdout; stdout={out:?}; stderr={err:?}"
            ));
        }
    }
    if stdout_must_be_empty && !out.is_empty() {
        failures.push(format!(
            "{label}: expected empty stdout; stdout={out:?}; stderr={err:?}"
        ));
    }
    if stderr_must_be_empty && !err.is_empty() {
        failures.push(format!(
            "{label}: expected empty stderr; stdout={out:?}; stderr={err:?}"
        ));
    }
    for fragment in forbidden_fragments {
        if out.contains(fragment) || err.contains(fragment) {
            failures.push(format!(
                "{label}: output unexpectedly contained {fragment:?}; stdout={out:?}; stderr={err:?}"
            ));
        }
    }
}

fn assert_no_failures(failures: Vec<String>) {
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).expect("stat fake tool").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make fake tool executable");
}

fn write_fake_verifier(workspace: &TestWorkspace, label: &str, accept: bool) -> PathBuf {
    #[cfg(windows)]
    let path = workspace.path(&format!("tools/{label}-opt.cmd"));
    #[cfg(not(windows))]
    let path = workspace.path(&format!("tools/{label}-opt"));

    #[cfg(windows)]
    let script = format!(
        r#"@echo off
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
"%SystemRoot%\System32\more.com" >nul
exit /b {}
"#,
        if accept { 0 } else { 42 }
    );

    #[cfg(not(windows))]
    let script = format!(
        r#"#!/bin/sh
if [ "${{1:-}}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
/bin/cat >/dev/null
exit {}
"#,
        if accept { 0 } else { 42 }
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fake verifier directory");
    }
    fs::write(&path, script).expect("write fake verifier");
    #[cfg(unix)]
    make_executable(&path);
    path
}

fn configure_verifier(command: &mut Command, workspace: &TestWorkspace, verifier: &Path) {
    let empty_path = workspace.path("empty-path");
    fs::create_dir_all(&empty_path).expect("create empty PATH");
    command
        .env("AERO_LLVM_OPT", verifier)
        .env_remove("AERO_LLVM_AS")
        .env_remove("AERO_REQUIRE_LLVM_VERIFIER")
        .env("PATH", empty_path);
    #[cfg(windows)]
    command.env("PATHEXT", ".CMD;.EXE");
}

fn write_fake_native_tools(workspace: &TestWorkspace, child_exit: i32) -> PathBuf {
    let bin = workspace.path("native-tools");
    fs::create_dir_all(&bin).expect("create fake native tool directory");

    #[cfg(windows)]
    {
        let source = workspace.write(
            "fake-native.rs",
            &format!(
                r#"use std::env;
use std::fs;
use std::path::PathBuf;

fn output_arg() -> PathBuf {{
    let args: Vec<_> = env::args_os().collect();
    let index = args.iter().position(|arg| arg == "-o").expect("missing -o");
    PathBuf::from(&args[index + 1])
}}

fn main() {{
    let executable = env::current_exe().expect("current exe");
    let name = executable.file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
    if env::args_os().any(|arg| arg == "--version") {{
        println!("LLVM version 22.1.0");
        return;
    }}
    if name == "llc" {{
        fs::write(output_arg(), b"fake object").expect("write object");
        return;
    }}
    if name == "clang" {{
        fs::copy(&executable, output_arg()).expect("copy runner");
        return;
    }}
    std::process::exit({child_exit});
}}
"#
            ),
        );
        let llc = bin.join("llc.exe");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc)
            .arg(&source)
            .arg("-o")
            .arg(&llc)
            .output()
            .expect("compile fake native tool");
        assert!(
            output.status.success(),
            "fake native tool compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::copy(&llc, bin.join("clang.exe")).expect("copy fake clang");
    }

    #[cfg(not(windows))]
    {
        let llc = bin.join("llc");
        fs::write(
            &llc,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then out="$2"; shift 2; else shift; fi
done
printf '%s\n' 'fake object' > "$out"
"#,
        )
        .expect("write fake llc");
        make_executable(&llc);

        let clang = bin.join("clang");
        fs::write(
            &clang,
            format!(
                r#"#!/bin/sh
if [ "${{1:-}}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then out="$2"; shift 2; else shift; fi
done
printf '%s\n' '#!/bin/sh' 'exit {child_exit}' > "$out"
/bin/chmod +x "$out"
"#
            ),
        )
        .expect("write fake clang");
        make_executable(&clang);
    }

    bin
}

fn write_no_run_native_tools(workspace: &TestWorkspace) -> PathBuf {
    let bin = workspace.path("no-run-native-tools");
    fs::create_dir_all(&bin).expect("create no-run native tool directory");

    #[cfg(windows)]
    {
        let source = workspace.write(
            "no-run-native.rs",
            r#"use std::env;
use std::fs;
use std::path::PathBuf;

fn output_arg() -> PathBuf {
    let args: Vec<_> = env::args_os().collect();
    let index = args.iter().position(|arg| arg == "-o").expect("missing -o");
    PathBuf::from(&args[index + 1])
}

fn mark_forbidden_invocation(kind: &str) {
    let sentinel = env::var_os("AERO_TEST_EXECUTION_SENTINEL").expect("sentinel path");
    fs::write(sentinel, kind.as_bytes()).expect("write forbidden invocation sentinel");
}

fn main() {
    let executable = env::current_exe().expect("current exe");
    let name = executable.file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
    if env::args_os().any(|arg| arg == "--version") {
        println!("LLVM version 22.1.0");
        return;
    }
    if name == "llc" {
        fs::copy(&executable, output_arg()).expect("copy executable object sentinel");
        return;
    }
    if name == "clang" {
        mark_forbidden_invocation("link invoked");
        std::process::exit(98);
    }
    mark_forbidden_invocation("emitted object executed");
    std::process::exit(97);
}
"#,
        );
        let llc = bin.join("llc.exe");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc)
            .arg(&source)
            .arg("-o")
            .arg(&llc)
            .output()
            .expect("compile no-run native tool");
        assert!(
            output.status.success(),
            "no-run native tool compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::copy(&llc, bin.join("clang.exe")).expect("copy no-run fake clang");
    }

    #[cfg(not(windows))]
    {
        let llc = bin.join("llc");
        fs::write(
            &llc,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then out="$2"; shift 2; else shift; fi
done
printf '%s\n' '#!/bin/sh' 'printf "%s\n" "emitted object executed" > "$AERO_TEST_EXECUTION_SENTINEL"' 'exit 97' > "$out"
/bin/chmod +x "$out"
"#,
        )
        .expect("write no-run fake llc");
        make_executable(&llc);

        let clang = bin.join("clang");
        fs::write(
            &clang,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
printf '%s\n' 'link invoked' > "$AERO_TEST_EXECUTION_SENTINEL"
exit 98
"#,
        )
        .expect("write no-run fake clang");
        make_executable(&clang);
    }

    bin
}

fn write_failing_native_tools(workspace: &TestWorkspace) -> PathBuf {
    let bin = workspace.path("failing-native-tools");
    fs::create_dir_all(&bin).expect("create failing native tool directory");

    #[cfg(windows)]
    {
        let source = workspace.write(
            "failing-clang.rs",
            r#"use std::env;
use std::fs;
use std::path::PathBuf;

fn output_arg() -> PathBuf {
    let args: Vec<_> = env::args_os().collect();
    let index = args.iter().position(|arg| arg == "-o").expect("missing -o");
    PathBuf::from(&args[index + 1])
}

fn main() {
    if std::env::args_os().any(|arg| arg == "--version") {
        println!("LLVM version 22.1.0");
        return;
    }
    let executable = env::current_exe().expect("current exe");
    let name = executable.file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
    if name == "llc" {
        fs::write(output_arg(), b"fake object").expect("write object");
        return;
    }
    eprintln!("forced clang failure");
    std::process::exit(42);
}
"#,
        );
        let llc = bin.join("llc.exe");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc)
            .arg(&source)
            .arg("-o")
            .arg(&llc)
            .output()
            .expect("compile failing native tools");
        assert!(
            output.status.success(),
            "failing native tool compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::copy(&llc, bin.join("clang.exe")).expect("copy failing clang");
    }

    #[cfg(not(windows))]
    {
        let llc = bin.join("llc");
        fs::write(
            &llc,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then out="$2"; shift 2; else shift; fi
done
printf '%s\n' 'fake object' > "$out"
"#,
        )
        .expect("write fake llc for failing native path");
        make_executable(&llc);

        let clang = bin.join("clang");
        fs::write(
            &clang,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
echo "forced clang failure" >&2
exit 42
"#,
        )
        .expect("write failing clang");
        make_executable(&clang);
    }

    bin
}

fn write_non_regular_output_native_tools(workspace: &TestWorkspace) -> PathBuf {
    let bin = workspace.path("non-regular-output-native-tools");
    fs::create_dir_all(&bin).expect("create non-regular-output native tool directory");

    #[cfg(windows)]
    {
        let source = workspace.write(
            "non-regular-output-llc.rs",
            r#"use std::path::PathBuf;

fn main() {
    if std::env::args_os().any(|arg| arg == "--version") {
        println!("LLVM version 22.1.0");
        return;
    }
    let args: Vec<_> = std::env::args_os().collect();
    let index = args.iter().position(|arg| arg == "-o").expect("missing -o");
    std::fs::create_dir(PathBuf::from(&args[index + 1])).expect("create directory at output path");
}
"#,
        );
        let llc = bin.join("llc.exe");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc)
            .arg(&source)
            .arg("-o")
            .arg(&llc)
            .output()
            .expect("compile non-regular-output llc");
        assert!(
            output.status.success(),
            "non-regular-output llc compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(not(windows))]
    {
        let llc = bin.join("llc");
        fs::write(
            &llc,
            r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then echo "LLVM version 22.1.0"; exit 0; fi
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then out="$2"; shift 2; else shift; fi
done
/bin/mkdir "$out"
"#,
        )
        .expect("write non-regular-output llc");
        make_executable(&llc);
    }

    bin
}

fn configure_native_run(command: &mut Command, verifier: &Path, native_tools: &Path) {
    command
        .env("AERO_LLVM_OPT", verifier)
        .env_remove("AERO_LLVM_AS")
        .env_remove("AERO_REQUIRE_LLVM_VERIFIER")
        .env("PATH", native_tools);
    #[cfg(windows)]
    command.env("PATHEXT", ".EXE;.CMD");
}

#[test]
fn explicit_help_and_version_are_successful() {
    let workspace = TestWorkspace::new("help-success");
    let mut failures = Vec::new();
    for (label, command_args, expected_stdout) in [
        ("help-long", args(&["--help"]), "USAGE:"),
        ("help-short", args(&["-h"]), "USAGE:"),
        (
            "version-long",
            args(&["--version"]),
            concat!("Aero compiler version ", env!("CARGO_PKG_VERSION")),
        ),
        (
            "version-short",
            args(&["-v"]),
            concat!("Aero compiler version ", env!("CARGO_PKG_VERSION")),
        ),
        (
            "registry-help-word",
            args(&["registry", "help"]),
            "registry.aero commands",
        ),
        (
            "registry-help-long",
            args(&["registry", "--help"]),
            "registry.aero commands",
        ),
        (
            "registry-help-short",
            args(&["registry", "-h"]),
            "registry.aero commands",
        ),
    ] {
        let output = run_aero(&workspace, &command_args);
        record_case(
            &mut failures,
            label,
            &output,
            SUCCESS,
            &[expected_stdout],
            &[],
            false,
            true,
            &[],
        );
    }
    assert_no_failures(failures);
}

#[test]
fn ambiguous_gpu_aliases_fail_invocation_before_source_access() {
    const DIAGNOSTIC: &str = "target `gpu` is ambiguous and does not prove a usable device; choose cpu, rocm, or cuda explicitly";

    let workspace = TestWorkspace::new("ambiguous-gpu");
    let source = workspace.path("must-not-be-read.aero");
    let mut failures = Vec::new();
    for (label, command, option) in [
        ("build-target-gpu", "build", "--target"),
        ("build-backend-gpu", "build", "--backend"),
        ("run-target-gpu", "run", "--target"),
        ("run-backend-gpu", "run", "--backend"),
    ] {
        let output_path = workspace.path(&format!("{label}.ll"));
        let mut command_args = vec![OsString::from(command), path_arg(&source)];
        if command == "build" {
            command_args.extend([OsString::from("-o"), path_arg(&output_path)]);
        }
        command_args.extend([OsString::from(option), OsString::from("gpu")]);
        let output = run_aero(&workspace, &command_args);
        record_case(
            &mut failures,
            label,
            &output,
            INVOCATION_FAILURE,
            &[],
            &[DIAGNOSTIC],
            true,
            false,
            &["completed successfully", "Program executed successfully"],
        );
        if output_path.exists() {
            failures.push(format!("{label}: ambiguous target published output"));
        }
    }
    assert_no_failures(failures);
}

#[test]
fn invalid_invocations_return_two_with_command_appropriate_streams() {
    let workspace = TestWorkspace::new("invocation-errors");
    let source = workspace.write("valid.aero", "fn main() { let value: int = 1; }\n");
    let source_text = path_arg(&source);
    let bare_source = vec![source_text.clone()];
    let mut failures = Vec::new();

    let cases: Vec<(&str, Vec<OsString>, &[&str], &[&str])> = vec![
        ("no-command", vec![], &["USAGE:"], &[]),
        (
            "unknown-command",
            args(&["not-a-command"]),
            &[],
            &["Unknown command: not-a-command"],
        ),
        (
            "bare-benchmark-source",
            bare_source,
            &[],
            &["Unknown command:"],
        ),
        ("help-extra", args(&["--help", "extra"]), &["USAGE:"], &[]),
        (
            "version-extra",
            args(&["--version", "extra"]),
            &[concat!("Aero compiler version ", env!("CARGO_PKG_VERSION"))],
            &[],
        ),
        ("build-missing", args(&["build"]), &[], &["Usage:"]),
        (
            "build-unrecognized-target",
            vec![
                OsString::from("build"),
                source_text.clone(),
                OsString::from("-o"),
                path_arg(&workspace.path("build.ll")),
                OsString::from("--target"),
                OsString::from("not-a-target"),
            ],
            &[],
            &["unsupported target `not-a-target`"],
        ),
        ("run-missing", args(&["run"]), &[], &["Usage:"]),
        (
            "run-unrecognized-target",
            vec![
                OsString::from("run"),
                OsString::from("--target"),
                OsString::from("not-a-target"),
                source_text.clone(),
            ],
            &[],
            &["unsupported target `not-a-target`"],
        ),
        ("check-missing", args(&["check"]), &[], &["Usage:"]),
        (
            "check-extra",
            vec![
                OsString::from("check"),
                source_text.clone(),
                OsString::from("extra"),
            ],
            &[],
            &["Usage:"],
        ),
        ("test-extra", args(&["test", "extra"]), &[], &["Usage:"]),
        ("fmt-missing", args(&["fmt"]), &[], &["Usage:"]),
        (
            "fmt-extra",
            vec![
                OsString::from("fmt"),
                source_text.clone(),
                OsString::from("extra"),
            ],
            &[],
            &["Usage:"],
        ),
        ("doc-missing", args(&["doc"]), &[], &["Usage:"]),
        (
            "doc-incomplete-output",
            vec![
                OsString::from("doc"),
                source_text.clone(),
                OsString::from("-o"),
            ],
            &[],
            &["Usage:"],
        ),
        ("profile-missing", args(&["profile"]), &[], &["Usage:"]),
        (
            "profile-incomplete-output",
            vec![
                OsString::from("profile"),
                source_text.clone(),
                OsString::from("-o"),
            ],
            &[],
            &["Usage:"],
        ),
        ("graph-missing", args(&["graph-opt"]), &[], &["Usage:"]),
        (
            "graph-incomplete-output",
            args(&["graph-opt", "missing.ll", "-o"]),
            &[],
            &["Usage:"],
        ),
        (
            "graph-omitted-output",
            args(&["graph-opt", "missing.ll", "--backend", "cpu"]),
            &[],
            &["Usage:"],
        ),
        (
            "graph-incomplete-backend",
            args(&["graph-opt", "missing.ll", "-o", "graph.ll", "--backend"]),
            &[],
            &["Usage:"],
        ),
        (
            "graph-incomplete-gpu",
            args(&["graph-opt", "missing.ll", "-o", "graph.ll", "--gpu"]),
            &[],
            &["Usage:"],
        ),
        (
            "graph-unknown-option",
            args(&["graph-opt", "missing.ll", "-o", "graph.ll", "--unknown"]),
            &[],
            &["Usage:"],
        ),
        (
            "graph-unrecognized-backend",
            args(&[
                "graph-opt",
                "missing.ll",
                "-o",
                "graph.ll",
                "--backend",
                "not-a-backend",
            ]),
            &[],
            &["unsupported backend `not-a-backend`"],
        ),
        ("quantize-missing", args(&["quantize"]), &[], &["Usage:"]),
        (
            "quantize-incomplete-output",
            args(&["quantize", "missing.ll", "-o"]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-incomplete-mode",
            args(&["quantize", "missing.ll", "-o", "quant.ll", "--mode"]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-incomplete-backend",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "int8",
                "--backend",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-incomplete-gpu",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "int8",
                "--gpu",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-incomplete-calibration",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "int8",
                "--calibration",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-unknown-option",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "int8",
                "--unknown",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-omitted-output",
            args(&[
                "quantize",
                "missing.ll",
                "--mode",
                "int8",
                "--backend",
                "cpu",
                "--annotation-only",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-omitted-mode",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--backend",
                "cpu",
                "--annotation-only",
            ]),
            &[],
            &["Usage:"],
        ),
        (
            "quantize-unrecognized-mode",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "not-a-mode",
            ]),
            &[],
            &["unsupported quantization mode `not-a-mode`"],
        ),
        (
            "quantize-unrecognized-backend",
            args(&[
                "quantize",
                "missing.ll",
                "-o",
                "quant.ll",
                "--mode",
                "int8",
                "--backend",
                "not-a-backend",
            ]),
            &[],
            &["unsupported backend `not-a-backend`"],
        ),
        (
            "registry-missing",
            args(&["registry"]),
            &["registry.aero commands"],
            &[],
        ),
        (
            "registry-unknown",
            args(&["registry", "not-a-subcommand"]),
            &["registry.aero commands"],
            &[],
        ),
        (
            "registry-help-extra",
            args(&["registry", "help", "extra"]),
            &["registry.aero commands"],
            &[],
        ),
        (
            "registry-search-missing-query",
            args(&["registry", "search"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-search-incomplete-index",
            args(&["registry", "search", "vision", "--index"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-search-incomplete-registry",
            args(&["registry", "search", "vision", "--registry"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-search-incomplete-token",
            args(&["registry", "search", "vision", "--token"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-search-incomplete-token-file",
            args(&["registry", "search", "vision", "--token-file"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-search-unknown-option",
            args(&["registry", "search", "vision", "--unknown"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-publish-missing-package",
            args(&["registry", "publish"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-publish-incomplete-registry",
            args(&["registry", "publish", ".", "--registry"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-publish-incomplete-token",
            args(&["registry", "publish", ".", "--token"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-publish-incomplete-token-file",
            args(&["registry", "publish", ".", "--token-file"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-publish-unknown-option",
            args(&["registry", "publish", ".", "--unknown"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-missing-package",
            args(&["registry", "install"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-version",
            args(&["registry", "install", "vision", "--version"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-registry",
            args(&["registry", "install", "vision", "--registry"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-target",
            args(&["registry", "install", "vision", "--target"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-token",
            args(&["registry", "install", "vision", "--token"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-token-file",
            args(&["registry", "install", "vision", "--token-file"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-incomplete-digest",
            args(&["registry", "install", "vision", "--expected-sha256"]),
            &[],
            &["Usage:"],
        ),
        (
            "registry-install-unknown-option",
            args(&["registry", "install", "vision", "--unknown"]),
            &[],
            &["Usage:"],
        ),
        (
            "conformance-bad-flag",
            args(&["conformance", "--bad-flag"]),
            &[],
            &["Usage:"],
        ),
        (
            "conformance-incomplete-output",
            args(&["conformance", "-o"]),
            &[],
            &["Usage:"],
        ),
        (
            "init-extra",
            args(&["init", "one", "two"]),
            &[],
            &["Usage:"],
        ),
        ("lsp-extra", args(&["lsp", "extra"]), &[], &["Usage:"]),
    ];

    for (label, command_args, expected_stdout, expected_stderr) in cases {
        let output = run_aero(&workspace, &command_args);
        record_case(
            &mut failures,
            label,
            &output,
            INVOCATION_FAILURE,
            expected_stdout,
            expected_stderr,
            expected_stdout.is_empty(),
            expected_stderr.is_empty(),
            &[],
        );
    }

    let formatted = fs::read_to_string(&source).expect("read strict-arity format fixture");
    if formatted != "fn main() { let value: int = 1; }\n" {
        failures.push("fmt-extra mutated its source before rejecting the invocation".to_string());
    }
    assert_no_failures(failures);
}

#[test]
fn pre_publication_and_established_operational_failures_return_one() {
    let workspace = TestWorkspace::new("operational-errors");
    let missing = workspace.path("missing.aero");
    let missing_text = path_arg(&missing);
    let malformed = workspace.write("malformed.aero", "let = ;\n");
    let mut failures = Vec::new();

    let cases: Vec<(&str, Vec<OsString>, &str, Vec<PathBuf>)> = vec![
        (
            "build-missing-input",
            vec![
                OsString::from("build"),
                missing_text.clone(),
                OsString::from("-o"),
                path_arg(&workspace.path("missing-build.ll")),
            ],
            "Error reading file",
            vec![workspace.path("missing-build.ll")],
        ),
        (
            "run-missing-input",
            vec![OsString::from("run"), missing_text.clone()],
            "Error reading file",
            vec![],
        ),
        (
            "check-missing-input",
            vec![OsString::from("check"), missing_text.clone()],
            "could not read file",
            vec![],
        ),
        (
            "fmt-missing-input",
            vec![OsString::from("fmt"), missing_text.clone()],
            "could not read file",
            vec![],
        ),
        (
            "doc-missing-input",
            vec![
                OsString::from("doc"),
                missing_text.clone(),
                OsString::from("-o"),
                path_arg(&workspace.path("missing-doc.md")),
            ],
            "could not read file",
            vec![workspace.path("missing-doc.md")],
        ),
        (
            "profile-missing-input",
            vec![
                OsString::from("profile"),
                missing_text.clone(),
                OsString::from("-o"),
                path_arg(&workspace.path("missing-profile.json")),
            ],
            "could not read file",
            vec![workspace.path("missing-profile.json")],
        ),
        (
            "graph-missing-input",
            vec![
                OsString::from("graph-opt"),
                path_arg(&workspace.path("missing.ll")),
                OsString::from("-o"),
                path_arg(&workspace.path("missing-graph.ll")),
            ],
            "could not read file",
            vec![workspace.path("missing-graph.ll")],
        ),
        (
            "quantize-missing-input",
            vec![
                OsString::from("quantize"),
                path_arg(&workspace.path("missing.ll")),
                OsString::from("-o"),
                path_arg(&workspace.path("missing-quant.ll")),
                OsString::from("--mode"),
                OsString::from("int8"),
            ],
            "could not read file",
            vec![workspace.path("missing-quant.ll")],
        ),
        (
            "check-parser-failure",
            vec![OsString::from("check"), path_arg(&malformed)],
            "Parse error",
            vec![],
        ),
        (
            "registry-local-index-failure",
            vec![
                OsString::from("registry"),
                OsString::from("search"),
                OsString::from("vision"),
                OsString::from("--index"),
                path_arg(&workspace.path("missing-index.json")),
            ],
            "failed to read registry index",
            vec![],
        ),
        (
            "registry-live-quarantine",
            args(&["registry", "search", "vision", "--live"]),
            LIVE_REGISTRY_DISABLED,
            vec![],
        ),
    ];

    for (label, command_args, diagnostic, artifacts) in cases {
        let output = run_aero(&workspace, &command_args);
        record_case(
            &mut failures,
            label,
            &output,
            OPERATIONAL_FAILURE,
            &[],
            &[diagnostic],
            label != "registry-local-index-failure",
            false,
            &["completed successfully"],
        );
        for artifact in artifacts {
            if artifact.exists() {
                failures.push(format!(
                    "{label}: pre-publication failure created {}",
                    artifact.display()
                ));
            }
        }
    }

    let failed_test = workspace.write("broken_test.aero", "let = ;\n");
    let output = run_aero(&workspace, &args(&["test"]));
    record_case(
        &mut failures,
        "discovered-test-failure",
        &output,
        OPERATIONAL_FAILURE,
        &["1 failed"],
        &[],
        false,
        true,
        &["0 failed"],
    );
    drop(failed_test);

    let report_directory = workspace.path("report-directory");
    fs::create_dir_all(&report_directory).expect("create report output directory");
    let output = run_aero(
        &workspace,
        &[
            OsString::from("conformance"),
            OsString::from("-o"),
            path_arg(&report_directory),
        ],
    );
    record_case(
        &mut failures,
        "conformance-report-write-failure",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["could not write"],
        true,
        false,
        &["Wrote conformance report"],
    );

    let existing_project = workspace.path("existing-project");
    fs::create_dir_all(&existing_project).expect("create existing project");
    fs::write(existing_project.join("aero.toml"), "existing").expect("write existing manifest");
    let output = run_aero(
        &workspace,
        &[OsString::from("init"), path_arg(&existing_project)],
    );
    record_case(
        &mut failures,
        "init-existing-project",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["refusing to overwrite existing manifest"],
        true,
        false,
        &["Initialized Aero project"],
    );

    let output = run_aero_with_stdin(&workspace, &args(&["lsp"]), b"Broken-Header\n\n");
    record_case(
        &mut failures,
        "lsp-read-failure",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["LSP read error: missing Content-Length header"],
        true,
        false,
        &[],
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let partial_project = workspace.path("partial-project");
        let partial_src = partial_project.join("src");
        fs::create_dir_all(&partial_src).expect("create partial project source directory");
        let path_blocker = partial_project.join("path-blocker");
        fs::write(&path_blocker, "regular file blocks child traversal")
            .expect("write partial-init path blocker");
        symlink(
            path_blocker.join("impossible-child"),
            partial_src.join("main.aero"),
        )
        .expect("create deterministic dangling source symlink");

        let output = run_aero(
            &workspace,
            &[OsString::from("init"), path_arg(&partial_project)],
        );
        record_case(
            &mut failures,
            "init-partial-write-failure",
            &output,
            OPERATIONAL_FAILURE,
            &[],
            &["failed to write source"],
            true,
            false,
            &["Initialized Aero project"],
        );
        if !partial_project.join("aero.toml").exists() {
            failures.push(
                "init-partial-write-failure: expected the allowed partial manifest evidence"
                    .to_string(),
            );
        }
    }

    assert_no_failures(failures);
}

#[test]
fn write_backend_and_recognized_unavailable_failures_return_one_without_false_success() {
    let workspace = TestWorkspace::new("write-backend-errors");
    let source = workspace.write("valid.aero", "fn main() { let value: int = 1; }\n");
    let llvm = workspace.write("valid.ll", "define i32 @main() {\nentry:\n  ret i32 0\n}\n");
    let verifier = write_fake_verifier(&workspace, "accept", true);
    let rejecting_verifier = write_fake_verifier(&workspace, "reject", false);
    let failing_native_tools = write_failing_native_tools(&workspace);
    let mut failures = Vec::new();

    // Windows read-only files reject replacement writes. Linux procfs text is
    // readable but rejects writes even for a privileged test process, avoiding a
    // mode-bit fixture that changes behavior under CAP_DAC_OVERRIDE.
    #[cfg(any(windows, target_os = "linux"))]
    {
        #[cfg(windows)]
        let write_failure_source = {
            let path = workspace.write("readonly.aero", "fn main() { }   \n");
            let mut permissions = fs::metadata(&path)
                .expect("stat read-only fixture")
                .permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&path, permissions).expect("make source read-only");
            path
        };
        #[cfg(target_os = "linux")]
        let write_failure_source = PathBuf::from("/proc/version");

        let output = run_aero(
            &workspace,
            &[OsString::from("fmt"), path_arg(&write_failure_source)],
        );
        record_case(
            &mut failures,
            "fmt-write-failure",
            &output,
            OPERATIONAL_FAILURE,
            &[],
            &["could not write file"],
            true,
            false,
            &["Formatted"],
        );
        #[cfg(windows)]
        make_writable(&write_failure_source);
    }

    for (label, command_args, diagnostic, configure_accepting_verifier) in [
        (
            "doc-write-failure",
            vec![
                OsString::from("doc"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&workspace.root),
            ],
            "could not write docs",
            false,
        ),
        (
            "profile-write-failure",
            vec![
                OsString::from("profile"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&workspace.root),
            ],
            "failed to write trace file",
            true,
        ),
        (
            "build-write-failure",
            vec![
                OsString::from("build"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&workspace.root),
            ],
            "Error writing to file",
            true,
        ),
        (
            "graph-write-failure",
            vec![
                OsString::from("graph-opt"),
                path_arg(&llvm),
                OsString::from("-o"),
                path_arg(&workspace.root),
            ],
            "could not write file",
            true,
        ),
        (
            "quantize-write-failure",
            vec![
                OsString::from("quantize"),
                path_arg(&llvm),
                OsString::from("-o"),
                path_arg(&workspace.root),
                OsString::from("--mode"),
                OsString::from("int8"),
            ],
            "could not write file",
            true,
        ),
    ] {
        let output = run_aero_configured(&workspace, &command_args, |command| {
            if configure_accepting_verifier {
                configure_verifier(command, &workspace, &verifier);
            }
        });
        let forbidden_success: &[&str] = match label {
            "doc-write-failure" => &["Generated documentation"],
            "profile-write-failure" => &["Wrote trace file"],
            "build-write-failure" => {
                &["Performance-optimized compilation process completed successfully."]
            }
            "graph-write-failure" => &["Wrote graph-optimized IR"],
            "quantize-write-failure" => &["Wrote quantization IR"],
            _ => &[],
        };
        record_case(
            &mut failures,
            label,
            &output,
            OPERATIONAL_FAILURE,
            &[],
            &[diagnostic],
            label == "doc-write-failure",
            false,
            forbidden_success,
        );
    }

    let verifier_output = workspace.path("rejected-graph.ll");
    let output = run_aero_configured(
        &workspace,
        &[
            OsString::from("graph-opt"),
            path_arg(&llvm),
            OsString::from("-o"),
            path_arg(&verifier_output),
        ],
        |command| configure_verifier(command, &workspace, &rejecting_verifier),
    );
    record_case(
        &mut failures,
        "graph-verifier-failure",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["rejected"],
        false,
        false,
        &["Wrote graph-optimized"],
    );
    if verifier_output.exists() {
        failures.push("graph verifier failure published output".to_string());
    }

    let calibration_output = workspace.path("calibration.ll");
    let output = run_aero_configured(
        &workspace,
        &[
            OsString::from("quantize"),
            path_arg(&llvm),
            OsString::from("-o"),
            path_arg(&calibration_output),
            OsString::from("--mode"),
            OsString::from("int8"),
            OsString::from("--calibration"),
            path_arg(&workspace.path("missing-calibration.json")),
        ],
        |command| configure_verifier(command, &workspace, &verifier),
    );
    record_case(
        &mut failures,
        "quantize-calibration-failure",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["calibration"],
        false,
        false,
        &["Wrote quantization"],
    );
    if calibration_output.exists() {
        failures.push("quantization calibration failure published output".to_string());
    }

    let output = run_aero_configured(
        &workspace,
        &[
            OsString::from("run"),
            path_arg(&source),
            OsString::from("--target"),
            OsString::from("cuda"),
        ],
        |command| configure_verifier(command, &workspace, &verifier),
    );
    record_case(
        &mut failures,
        "recognized-unavailable-cuda-run",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["CUDA run", "not implemented", "--target cpu"],
        false,
        false,
        &["Program executed successfully"],
    );

    let output = run_aero_configured(
        &workspace,
        &[OsString::from("run"), path_arg(&source)],
        |command| configure_native_run(command, &verifier, &failing_native_tools),
    );
    record_case(
        &mut failures,
        "native-tool-failure",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &["Error running clang"],
        false,
        false,
        &["Program executed successfully"],
    );

    assert_no_failures(failures);
}

fn assert_run_workspace_is_clean(workspace: &TestWorkspace) {
    let run_root = workspace.path("target/aero-run");
    if run_root.exists() {
        let entries = fs::read_dir(&run_root)
            .expect("read run artifact root")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect run artifact entries");
        assert!(
            entries.is_empty(),
            "temporary run artifacts remain: {entries:?}"
        );
    }
}

#[test]
fn rocm_run_with_regular_object_fails_closed_without_execution() {
    let workspace = TestWorkspace::new("rocm-object-only");
    let source = workspace.write("valid.aero", "fn main() { let value: int = 1; }\n");
    let verifier = write_fake_verifier(&workspace, "rocm-object", true);
    let native_tools = write_no_run_native_tools(&workspace);
    let execution_sentinel = workspace.path("forbidden-link-or-execution.txt");
    let output = run_aero_configured(
        &workspace,
        &[
            OsString::from("run"),
            path_arg(&source),
            OsString::from("--target"),
            OsString::from("rocm"),
        ],
        |command| {
            configure_native_run(command, &verifier, &native_tools);
            command.env("AERO_TEST_EXECUTION_SENTINEL", &execution_sentinel);
        },
    );
    let mut failures = Vec::new();
    record_case(
        &mut failures,
        "rocm-object-only",
        &output,
        OPERATIONAL_FAILURE,
        &[
            "ROCm object stage complete: llc produced a temporary file; no link or execution occurred.",
        ],
        &[
            "ROCm run is unavailable: HIP link and device launch are not implemented; no program was executed.",
        ],
        false,
        false,
        &[
            "Program executed successfully",
            "object generation validated",
            "staged for HIP launcher integration",
        ],
    );
    if execution_sentinel.exists() {
        failures.push(format!(
            "rocm-object-only: forbidden native stage ran: {}",
            fs::read_to_string(&execution_sentinel)
                .unwrap_or_else(|error| format!("unreadable sentinel: {error}"))
        ));
    }

    let cuda_output = run_aero_configured(
        &workspace,
        &[
            OsString::from("run"),
            path_arg(&source),
            OsString::from("--backend"),
            OsString::from("cuda"),
        ],
        |command| configure_verifier(command, &workspace, &verifier),
    );
    record_case(
        &mut failures,
        "cuda-unavailable-no-execution",
        &cuda_output,
        OPERATIONAL_FAILURE,
        &[],
        &[
            "CUDA run is unavailable: object generation, link, and device launch are not implemented; no program was executed. Use --target cpu for execution.",
        ],
        false,
        false,
        &[
            "Program executed successfully",
            "CUDA run target is not implemented",
        ],
    );
    assert_no_failures(failures);
    assert_run_workspace_is_clean(&workspace);
}

#[test]
fn rocm_run_requires_a_regular_object_emission_postcondition() {
    let workspace = TestWorkspace::new("rocm-non-regular-object");
    let source = workspace.write("valid.aero", "fn main() { let value: int = 1; }\n");
    let verifier = write_fake_verifier(&workspace, "rocm-non-regular", true);
    let native_tools = write_non_regular_output_native_tools(&workspace);
    let output = run_aero_configured(
        &workspace,
        &[
            OsString::from("run"),
            path_arg(&source),
            OsString::from("--backend"),
            OsString::from("rocm"),
        ],
        |command| configure_native_run(command, &verifier, &native_tools),
    );
    let mut failures = Vec::new();
    record_case(
        &mut failures,
        "rocm-non-regular-object",
        &output,
        OPERATIONAL_FAILURE,
        &[],
        &[
            "ROCm object generation failed: llc reported success but did not create the requested regular object file.",
        ],
        false,
        false,
        &[
            "ROCm object stage complete",
            "Program executed successfully",
            "object generation validated",
        ],
    );
    assert_no_failures(failures);
    assert_run_workspace_is_clean(&workspace);
}

#[test]
fn successful_command_families_remain_compatible() {
    let workspace = TestWorkspace::new("success-controls");
    let source = workspace.write("valid.aero", "fn main() { let value: int = 1; }\n");
    workspace.write("sample_test.aero", "fn main() { let value: int = 1; }\n");
    let llvm = workspace.write("valid.ll", "define i32 @main() {\nentry:\n  ret i32 0\n}\n");
    let index = workspace.write(
        "index.json",
        r#"{"packages":[{"name":"vision-core","version":"0.2.0","downloads":1}]}"#,
    );
    let verifier = write_fake_verifier(&workspace, "success", true);
    let mut failures = Vec::new();

    let build_output = workspace.path("build.ll");
    let check_cases: Vec<(&str, Vec<OsString>, &str, Option<PathBuf>, bool)> = vec![
        (
            "build-success",
            vec![
                OsString::from("build"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&build_output),
            ],
            "completed successfully",
            Some(build_output.clone()),
            true,
        ),
        (
            "check-success",
            vec![OsString::from("check"), path_arg(&source)],
            "Checking",
            None,
            false,
        ),
        (
            "test-success",
            args(&["test"]),
            "1 completed, 0 failed, 1 total; no tests were executed",
            None,
            false,
        ),
        (
            "fmt-success",
            vec![OsString::from("fmt"), path_arg(&source)],
            "Formatted",
            None,
            false,
        ),
        (
            "doc-success",
            vec![
                OsString::from("doc"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&workspace.path("api.md")),
            ],
            "Generated documentation",
            Some(workspace.path("api.md")),
            false,
        ),
        (
            "profile-success",
            vec![
                OsString::from("profile"),
                path_arg(&source),
                OsString::from("-o"),
                path_arg(&workspace.path("trace.json")),
            ],
            "Wrote trace file",
            Some(workspace.path("trace.json")),
            true,
        ),
        (
            "graph-success",
            vec![
                OsString::from("graph-opt"),
                path_arg(&llvm),
                OsString::from("-o"),
                path_arg(&workspace.path("graph.ll")),
                OsString::from("--backend"),
                OsString::from("cuda"),
            ],
            "Wrote graph-optimized IR",
            Some(workspace.path("graph.ll")),
            true,
        ),
        (
            "quantize-success",
            vec![
                OsString::from("quantize"),
                path_arg(&llvm),
                OsString::from("-o"),
                path_arg(&workspace.path("quant.ll")),
                OsString::from("--mode"),
                OsString::from("int8"),
                OsString::from("--backend"),
                OsString::from("rocm"),
            ],
            "Wrote quantization IR",
            Some(workspace.path("quant.ll")),
            true,
        ),
        (
            "registry-local-success",
            vec![
                OsString::from("registry"),
                OsString::from("search"),
                OsString::from("vision"),
                OsString::from("--index"),
                path_arg(&index),
            ],
            "Found 1 package(s)",
            None,
            false,
        ),
        (
            "conformance-success",
            vec![
                OsString::from("conformance"),
                OsString::from("-o"),
                path_arg(&workspace.path("conformance.json")),
            ],
            "Conformance cases:",
            Some(workspace.path("conformance.json")),
            false,
        ),
        (
            "init-success",
            vec![
                OsString::from("init"),
                path_arg(&workspace.path("new-project")),
            ],
            "Initialized Aero project",
            Some(workspace.path("new-project/src/main.aero")),
            false,
        ),
        ("lsp-eof-success", args(&["lsp"]), "", None, false),
    ];

    for (label, command_args, expected_stdout, artifact, use_verifier) in check_cases {
        let output = run_aero_configured(&workspace, &command_args, |command| {
            if use_verifier {
                configure_verifier(command, &workspace, &verifier);
            }
        });
        let expected_stdout_fragments = if expected_stdout.is_empty() {
            Vec::new()
        } else {
            vec![expected_stdout]
        };
        record_case(
            &mut failures,
            label,
            &output,
            SUCCESS,
            &expected_stdout_fragments,
            &[],
            expected_stdout.is_empty(),
            true,
            &["Unknown command", "Usage:"],
        );
        if let Some(artifact) = artifact
            && !artifact.exists()
        {
            failures.push(format!(
                "{label}: successful command omitted {}",
                artifact.display()
            ));
        }
    }

    assert_no_failures(failures);
}

#[test]
fn semantic_only_test_command_reports_analysis_without_execution_claims() {
    const BUILD_GUIDE: &str = include_str!("../../../BUILD.md");
    const FORBIDDEN: &[&str] = &[
        "Compiling test suite",
        "run",
        "Running",
        "passed",
        "test result",
        "Program executed successfully",
    ];

    let mut failures = Vec::new();

    let successful = TestWorkspace::new("test-analysis-success");
    successful.write("sample_test.aero", "fn main() { let value: int = 1; }\n");
    let success_output = run_aero(&successful, &args(&["test"]));
    let success_stdout = strip_ansi(&stdout(&success_output));
    if success_output.status.code() != Some(SUCCESS) {
        failures.push(format!(
            "analysis success: expected exit {SUCCESS}, received {:?}; stdout={success_stdout:?}; stderr={:?}",
            success_output.status.code(),
            stderr(&success_output)
        ));
    }
    for fragment in [
        "Analyzing Aero test sources (parse, direct modules, semantics only; no execution)...",
        "sample_test.aero analysis completed (not executed)",
        "analysis result: 1 completed, 0 failed, 1 total; no tests were executed",
    ] {
        if !success_stdout.contains(fragment) {
            failures.push(format!(
                "analysis success: omitted {fragment:?}; stdout={success_stdout:?}"
            ));
        }
    }
    if success_stdout.matches("Analyzing").count() != 2 {
        failures.push(format!(
            "analysis success: expected one suite and one source analysis label; stdout={success_stdout:?}"
        ));
    }
    if !has_source_analysis_line(&success_stdout, "sample_test.aero") {
        failures.push(format!(
            "analysis success: missing exact source label `Analyzing <path-to-sample_test.aero>`; stdout={success_stdout:?}"
        ));
    }
    for fragment in FORBIDDEN {
        if success_stdout.contains(fragment) {
            failures.push(format!(
                "analysis success: retained execution/pass claim {fragment:?}; stdout={success_stdout:?}"
            ));
        }
    }
    if !stderr(&success_output).is_empty() {
        failures.push(format!(
            "analysis success: expected empty stderr; stderr={:?}",
            stderr(&success_output)
        ));
    }

    let empty = TestWorkspace::new("test-analysis-empty");
    let empty_output = run_aero(&empty, &args(&["test"]));
    let empty_stdout = strip_ansi(&stdout(&empty_output));
    if empty_output.status.code() != Some(SUCCESS) {
        failures.push(format!(
            "empty analysis: expected exit {SUCCESS}, received {:?}; stdout={empty_stdout:?}; stderr={:?}",
            empty_output.status.code(),
            stderr(&empty_output)
        ));
    }
    for fragment in [
        "Analyzing Aero test sources (parse, direct modules, semantics only; no execution)...",
        "no Aero test source files found (*_test.aero, *_tests.aero); no tests were executed",
    ] {
        if !empty_stdout.contains(fragment) {
            failures.push(format!(
                "empty analysis: omitted {fragment:?}; stdout={empty_stdout:?}"
            ));
        }
    }
    for fragment in FORBIDDEN {
        if empty_stdout.contains(fragment) {
            failures.push(format!(
                "empty analysis: retained execution/pass claim {fragment:?}; stdout={empty_stdout:?}"
            ));
        }
    }
    if !stderr(&empty_output).is_empty() {
        failures.push(format!(
            "empty analysis: expected empty stderr; stderr={:?}",
            stderr(&empty_output)
        ));
    }

    let failing = TestWorkspace::new("test-analysis-failure");
    failing.write(
        "semantic_test.aero",
        "fn main() { let observed = missing_value; }\n",
    );
    let failure_output = run_aero(&failing, &args(&["test"]));
    let failure_stdout = strip_ansi(&stdout(&failure_output));
    if failure_output.status.code() != Some(OPERATIONAL_FAILURE) {
        failures.push(format!(
            "analysis failure: expected exit {OPERATIONAL_FAILURE}, received {:?}; stdout={failure_stdout:?}; stderr={:?}",
            failure_output.status.code(),
            stderr(&failure_output)
        ));
    }
    for fragment in [
        "semantic_test.aero analysis failed:",
        "Use of undeclared variable `missing_value`",
        "analysis result: 0 completed, 1 failed, 1 total; no tests were executed",
    ] {
        if !failure_stdout.contains(fragment) {
            failures.push(format!(
                "analysis failure: omitted {fragment:?}; stdout={failure_stdout:?}"
            ));
        }
    }
    if failure_stdout.matches("Analyzing").count() != 2 {
        failures.push(format!(
            "analysis failure: expected one suite and one source analysis label; stdout={failure_stdout:?}"
        ));
    }
    if !has_source_analysis_line(&failure_stdout, "semantic_test.aero") {
        failures.push(format!(
            "analysis failure: missing exact source label `Analyzing <path-to-semantic_test.aero>`; stdout={failure_stdout:?}"
        ));
    }
    for fragment in FORBIDDEN {
        if failure_stdout.contains(fragment) {
            failures.push(format!(
                "analysis failure: retained execution/pass claim {fragment:?}; stdout={failure_stdout:?}"
            ));
        }
    }
    if !stderr(&failure_output).is_empty() {
        failures.push(format!(
            "analysis failure: expected empty stderr; stderr={:?}",
            stderr(&failure_output)
        ));
    }

    let help_output = run_aero(&empty, &args(&["--help"]));
    let help_stdout = strip_ansi(&stdout(&help_output));
    let test_help_line = help_stdout
        .lines()
        .find(|line| line.trim_start().starts_with("test "));
    if help_output.status.code() != Some(SUCCESS)
        || !test_help_line.is_some_and(|line| {
            line.contains("Discover and semantically analyze *_test.aero files (no execution)")
        })
    {
        failures.push(format!(
            "help: missing exact analysis-only description or wrong status; status={:?}; stdout={help_stdout:?}",
            help_output.status.code()
        ));
    }
    if help_stdout.contains("Discover and run *_test.aero files") {
        failures.push(format!("help: retained run claim; stdout={help_stdout:?}"));
    }
    for fragment in FORBIDDEN {
        if test_help_line.is_some_and(|line| line.contains(fragment)) {
            failures.push(format!(
                "help: test description retained execution/pass claim {fragment:?}; line={test_help_line:?}"
            ));
        }
    }
    if !stderr(&help_output).is_empty() {
        failures.push(format!(
            "help: expected empty stderr; stderr={:?}",
            stderr(&help_output)
        ));
    }

    for fragment in [
        "`aero test`: discover and semantically analyze `*_test.aero` and `*_tests.aero` files",
        "does not execute tests or generate IR",
    ] {
        if !BUILD_GUIDE.contains(fragment) {
            failures.push(format!("BUILD.md omitted {fragment:?}"));
        }
    }
    if BUILD_GUIDE.contains("`aero test`: discover and run `*_test.aero` files") {
        failures.push("BUILD.md retained the discover-and-run claim".to_string());
    }

    assert_no_failures(failures);
}

#[test]
fn delegated_cpu_program_exit_is_passed_through_outside_cli_owned_statuses() {
    const CHILD_EXIT: i32 = 7;

    let workspace = TestWorkspace::new("child-exit");
    let source = workspace.write("child.aero", "fn main() { }\n");
    let verifier = write_fake_verifier(&workspace, "child", true);
    let native_tools = write_fake_native_tools(&workspace, CHILD_EXIT);
    let output = run_aero_configured(
        &workspace,
        &[OsString::from("run"), path_arg(&source)],
        |command| configure_native_run(command, &verifier, &native_tools),
    );
    let mut failures = Vec::new();
    record_case(
        &mut failures,
        "delegated-child-exit",
        &output,
        CHILD_EXIT,
        &["Program executed successfully.", "Exit code: 7"],
        &[],
        false,
        true,
        &["Usage:", "Unknown command"],
    );
    assert_no_failures(failures);
}

#[test]
fn invalid_python_compilation_measurements_are_quarantined_without_deleting_evidence() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let claims_path = repo_root.join("claim-verification/claims.json");
    let claims: Value =
        serde_json::from_str(&fs::read_to_string(&claims_path).expect("read claim index"))
            .expect("parse claim index");
    let mut failures = Vec::new();

    let expected_claims: &[(&str, &str, &[&str])] = &[
        (
            "aero_public_master_compilation_benchmark_20260528",
            "invalid_measurement",
            &[
                "benchmarks/results/performance_results_1780007130.json",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/run_performance_benchmarks.stdout.log",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/run_performance_benchmarks.stderr.log",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/exit_code.txt",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/command_config.txt",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/hardware_software_metadata.txt",
            ],
        ),
        (
            "aero_public_master_lexer_criterion_20260528",
            "verified_current",
            &[
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/run_performance_benchmarks.stdout.log",
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/hardware_software_metadata.txt",
            ],
        ),
        (
            "aero_public_master_gguf_llama_cli_20260528",
            "verified_reference_only",
            &[
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/claim_result.json",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/gguf_compare.stdout.log",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/gguf_compare.stderr.log",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/report.md",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/config.local.json",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/command_config.txt",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/hardware_software_metadata.txt",
            ],
        ),
        (
            "aero_public_master_gpu_claims_20260528",
            "blocked",
            &[
                "claim-verification/results/aero_public_master_7900xtx_20260528T223000Z/hardware_software_metadata.txt",
                "claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/claim_result.json",
            ],
        ),
        (
            "aero_post_reboot_compilation_benchmark_20260528",
            "invalid_measurement",
            &[
                "benchmarks/results/performance_results_1780006109.json",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/claim_result.json",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/run_performance_benchmarks.stdout.log",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/run_performance_benchmarks.stderr.log",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/exit_code.txt",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/generated_files.txt",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/hardware_software_metadata.txt",
            ],
        ),
        (
            "aero_post_reboot_lexer_criterion_20260528",
            "historical_artifact",
            &[
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/claim_result.json",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/run_performance_benchmarks.stdout.log",
                "claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/hardware_software_metadata.txt",
            ],
        ),
    ];
    let invalid_ids: BTreeSet<&str> = [
        "aero_public_master_compilation_benchmark_20260528",
        "aero_post_reboot_compilation_benchmark_20260528",
    ]
    .into_iter()
    .collect();

    let Some(items) = claims["claims"].as_array() else {
        panic!("claim index omitted claims array");
    };
    if items.len() != expected_claims.len() {
        failures.push(format!(
            "claim index expected {} preserved/split records, received {}",
            expected_claims.len(),
            items.len()
        ));
    }

    let actual_ids: BTreeSet<&str> = items
        .iter()
        .filter_map(|claim| claim["id"].as_str())
        .collect();
    let expected_ids: BTreeSet<&str> = expected_claims.iter().map(|(id, _, _)| *id).collect();
    if actual_ids != expected_ids {
        failures.push(format!(
            "claim identity set changed: expected {expected_ids:?}, received {actual_ids:?}"
        ));
    }

    for (id, expected_status, expected_artifacts) in expected_claims {
        let Some(claim) = items.iter().find(|item| item["id"] == *id) else {
            failures.push(format!("claim index omitted {id}"));
            continue;
        };
        if claim["status"] != *expected_status {
            failures.push(format!(
                "{id}: expected {expected_status}, received {:?}",
                claim["status"]
            ));
        }
        let actual_artifacts: Option<Vec<&str>> = claim["artifacts"]
            .as_array()
            .and_then(|artifacts| artifacts.iter().map(Value::as_str).collect());
        let expected_artifact_set: BTreeSet<&str> = expected_artifacts.iter().copied().collect();
        let actual_artifact_set = actual_artifacts
            .as_ref()
            .map(|artifacts| artifacts.iter().copied().collect::<BTreeSet<_>>());
        if actual_artifacts.as_ref().map(Vec::len) != Some(expected_artifacts.len())
            || actual_artifact_set.as_ref() != Some(&expected_artifact_set)
        {
            failures.push(format!(
                "{id}: evidence inventory changed; expected {expected_artifacts:?}, received {actual_artifacts:?}"
            ));
        }
        for relative in *expected_artifacts {
            if !repo_root.join(relative).exists() {
                failures.push(format!("{id}: preserved artifact is missing: {relative}"));
            }
        }

        if invalid_ids.contains(id) {
            let claim_text = claim["claim"]
                .as_str()
                .unwrap_or_default()
                .to_ascii_lowercase();
            for fragment in [
                "invalid measurement",
                "performance_benchmark.py",
                "bare source",
            ] {
                if !claim_text.contains(fragment) {
                    failures.push(format!("{id}: invalid claim text omitted {fragment:?}"));
                }
            }
            if claim_text.contains("verified current") || claim_text.contains("valid compilation") {
                failures.push(format!("{id}: invalid claim text still implies validity"));
            }
        }
    }

    let actual_invalid_ids: BTreeSet<&str> = items
        .iter()
        .filter(|claim| claim["status"] == "invalid_measurement")
        .filter_map(|claim| claim["id"].as_str())
        .collect();
    if actual_invalid_ids != invalid_ids {
        failures.push(format!(
            "only the two Python compilation series may be invalidated; expected {invalid_ids:?}, received {actual_invalid_ids:?}"
        ));
    }

    let readme = fs::read_to_string(repo_root.join("README.md")).expect("read README");
    for fragment in [
        "Invalid compilation measurements",
        "performance_benchmark.py",
        "bare source path",
        "post-reboot",
    ] {
        if !readme.contains(fragment) {
            failures.push(format!("README claim quarantine omitted {fragment:?}"));
        }
    }
    let verified_section = readme
        .split_once("Verified current results:")
        .and_then(|(_, tail)| tail.split_once("Invalid compilation measurements"))
        .map(|(section, _)| section);
    match verified_section {
        Some(section)
            if !section.contains("Python harness")
                && !section.contains("compilation benchmark cases")
                && !section.contains("compilation mean times") => {}
        Some(_) => failures.push(
            "README still presents invalid Python compilation numbers as verified current"
                .to_string(),
        ),
        None => failures.push("README claim sections are not auditable".to_string()),
    }

    let benchmark_guide =
        fs::read_to_string(repo_root.join("benchmarks/README.md")).expect("read benchmark guide");
    for fragment in [
        "Quarantined benchmark driver",
        "performance_benchmark.py",
        "invalid compilation measurement",
        "bare source path",
    ] {
        if !benchmark_guide.contains(fragment) {
            failures.push(format!("benchmark guide quarantine omitted {fragment:?}"));
        }
    }
    if benchmark_guide.contains("Python Benchmarks (Recommended)") {
        failures.push("benchmark guide still recommends the invalid Python driver".to_string());
    }

    for (label, relative, fragments) in [
        (
            "capability audit",
            "CURRENT_CAPABILITY_AUDIT.md",
            &[
                "performance_benchmark.py",
                "invalid as a compilation",
                "lexer Criterion run is genuine historical",
                "llama.cpp observation",
            ][..],
        ),
        (
            "implementation matrix",
            "SPEC_IMPLEMENTATION_MATRIX.md",
            &[
                "performance_benchmark.py",
                "invalid measurement",
                "Criterion lexer",
                "llama.cpp",
            ][..],
        ),
        (
            "project state",
            "PROJECT_STATE.md",
            &[
                "performance_benchmark.py",
                "invalid measurement",
                "lexer evidence",
                "GGUF",
            ][..],
        ),
        (
            "task ledger",
            "TASK_LEDGER.md",
            &[
                "performance_benchmark.py",
                "invalid measurements",
                "Lexer",
                "llama.cpp",
            ][..],
        ),
        (
            "decision log",
            "DECISION_LOG.md",
            &["performance_benchmark.py", "invalid", "lexer", "llama.cpp"][..],
        ),
    ] {
        let document = fs::read_to_string(repo_root.join(relative))
            .unwrap_or_else(|error| panic!("read {relative}: {error}"));
        for fragment in fragments {
            if !document.contains(fragment) {
                failures.push(format!("{label} omitted claim qualification {fragment:?}"));
            }
        }
    }

    assert_no_failures(failures);
}
