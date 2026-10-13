use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const PROFILE: &str = "exact-i32-byte-diagnostics-v0";
static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        let root = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"))
            .join(format!(
                "h1e-cli-{}-{}",
                std::process::id(),
                NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&root).unwrap();
        Self(fs::canonicalize(root).unwrap())
    }

    fn source(&self, source: &str) -> PathBuf {
        let path = self.0.join("diagnostic.aero");
        fs::write(&path, source).unwrap();
        path
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
        command.current_dir(&self.0);
        command
    }

    fn run(&self, source: &str) -> Output {
        let source_file = self.source(source);
        let output = self
            .command()
            .arg("run")
            .arg(source_file)
            .args(["--language-profile", PROFILE])
            .output()
            .expect("execute diagnostic source through CLI");
        let artifacts = self.0.join("target/aero-run");
        if artifacts.exists() {
            assert_eq!(
                fs::read_dir(artifacts).unwrap().count(),
                0,
                "run artifacts cleaned"
            );
        }
        output
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_run(output: &Output, status: i32, stdout_lines: &[&str]) {
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "CLI uses its established output presentation: {output:?}"
    );
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    for line in stdout_lines {
        assert!(
            stdout.lines().any(|actual| actual == *line),
            "missing {line:?}: {stdout:?}"
        );
    }
}

#[test]
fn diagnostic_source_checks_builds_and_runs_from_embedded_runtime() {
    let workspace = Workspace::new();
    let source = r#"
fn main() -> int {
    let written: Result<int, int> = stderr_write_byte(65);
    return match written { Ok(status) => 91, Err(code) => 0 - code, };
}
"#;
    let path = workspace.source(source);
    let checked = workspace
        .command()
        .arg("check")
        .arg(&path)
        .args(["--language-profile", PROFILE])
        .output()
        .unwrap();
    assert!(checked.status.success(), "{checked:?}");
    let llvm = workspace.0.join("diagnostic.ll");
    let built = workspace
        .command()
        .arg("build")
        .arg(&path)
        .arg("-o")
        .arg(&llvm)
        .args(["--language-profile", PROFILE, "--require-llvm-verifier"])
        .output()
        .unwrap();
    assert!(built.status.success(), "{built:?}");
    let text = fs::read_to_string(&llvm).unwrap();
    assert_eq!(
        text.matches("declare i32 @aero_stderr_write_byte(i32)")
            .count(),
        1
    );
    assert_eq!(text.matches("call i32 @aero_stderr_write_byte").count(), 1);
    assert_run(
        &workspace.run(source),
        91,
        &["Exit code: 91", "Error output: A"],
    );
}

#[test]
fn diagnostic_cli_preserves_typed_failure_and_channel_isolation() {
    let workspace = Workspace::new();
    let range = r#"
fn main() -> int {
    let written: Result<int, int> = stderr_write_byte(256);
    return match written { Ok(status) => 71, Err(code) => code, };
}
"#;
    let output = workspace.run(range);
    assert_run(&output, 3, &["Exit code: 3"]);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Error output:"));
    let isolated = r#"
fn value(result: Result<int, int>) -> int {
    return match result { Ok(status) => status, Err(code) => code, };
}
fn main() -> int {
    let a: Result<int, int> = stdout_write_byte(65);
    let bad: Result<int, int> = stderr_write_byte(256);
    let sticky: Result<int, int> = stderr_write_byte(67);
    let b: Result<int, int> = stdout_write_byte(66);
    if value(a) == 0 && value(b) == 0 && value(bad) == 3 && value(sticky) == 3 {
        return 91;
    }
    return 71;
}
"#;
    let output = workspace.run(isolated);
    assert_run(&output, 91, &["Exit code: 91", "Output: AB"]);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Error output:"));
}

#[test]
fn earlier_profile_rejects_intrinsic_without_leaving_build_or_run_artifacts() {
    let workspace = Workspace::new();
    let source = workspace.source(
        "fn main() -> int { let value: Result<int, int> = stderr_write_byte(65); return 0; }",
    );
    let llvm = workspace.0.join("rejected.ll");
    for mode in ["check", "build", "run"] {
        let mut command = workspace.command();
        command
            .arg(mode)
            .arg(&source)
            .args(["--language-profile", "exact-i32-byte-io-v0"]);
        if mode == "build" {
            command.arg("-o").arg(&llvm);
        }
        let output = command.output().unwrap();
        assert!(!output.status.success(), "{mode}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("Function `stderr_write_byte` is not defined"),
            "{mode}: {output:?}"
        );
        assert!(!llvm.exists());
    }
    let artifacts = workspace.0.join("target/aero-run");
    if artifacts.exists() {
        assert_eq!(fs::read_dir(artifacts).unwrap().count(), 0);
    }
}

#[test]
fn failed_second_runtime_compilation_cleans_partial_artifacts() {
    let workspace = Workspace::new();
    let tool_source = workspace.0.join("failing-clang.c");
    fs::write(
        &tool_source,
        r#"
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <string.h>
int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--version") == 0) {
        puts("clang version 22.1.8");
        return 0;
    }
    for (int i = 1; i < argc; ++i) {
        if (strstr(argv[i], "aero_diagnostic_runtime.c")) {
            fputs("forced diagnostic compilation failure\n", stderr);
            return 71;
        }
    }
    for (int i = 1; i + 1 < argc; ++i) {
        if (strcmp(argv[i], "-o") == 0) {
            FILE *object = fopen(argv[i + 1], "wb");
            if (!object) return 72;
            if (fputc(0, object) == EOF) { fclose(object); return 73; }
            return fclose(object) == 0 ? 0 : 74;
        }
    }
    return 75;
}
"#,
    )
    .unwrap();
    let tools = workspace.0.join("failing-tools");
    fs::create_dir(&tools).unwrap();
    let compiler = tools.join(if cfg!(windows) { "clang.exe" } else { "clang" });
    let compiled = Command::new("clang")
        .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror"])
        .arg(&tool_source)
        .arg("-o")
        .arg(&compiler)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let input = workspace.source("fn main() -> int { return 91; }");
    let mut paths = vec![tools];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let output = workspace
        .command()
        .arg("run")
        .arg(input)
        .args(["--language-profile", PROFILE])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("forced diagnostic compilation failure"),
        "{output:?}"
    );
    assert_eq!(
        fs::read_dir(workspace.0.join("target/aero-run"))
            .unwrap()
            .count(),
        0,
        "original object and diagnostic source must be removed together"
    );
}

#[test]
fn invalid_diagnostic_context_stops_before_artifacts_are_published() {
    let workspace = Workspace::new();
    let input = workspace.source("fn main() -> int { stderr_write_byte(65); return 0; }");
    let output = workspace
        .command()
        .arg("run")
        .arg(input)
        .args(["--language-profile", PROFILE])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("explicit `Result<int, int>` context"),
        "{output:?}"
    );
    assert_eq!(
        fs::read_dir(workspace.0.join("target/aero-run"))
            .unwrap()
            .count(),
        0
    );
}
