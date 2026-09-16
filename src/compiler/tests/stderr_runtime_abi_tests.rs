//! Direct native coverage for the H1E-RUNTIME-001 diagnostic byte ABI.
//! These tests do not make the transport available to Aero source programs.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

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
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"))
            .join("h1e-stderr-runtime-tests");
        let root = parent.join(format!(
            "h1e-stderr-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create diagnostic runtime test workspace");
        Self {
            root: fs::canonicalize(root).expect("canonicalize test workspace"),
        }
    }

    fn compile(&self, optimization: &str, source: &str) -> PathBuf {
        let harness = self.root.join("harness.c");
        fs::write(&harness, source).expect("write native ABI harness");
        let executable = self.root.join(if cfg!(windows) {
            format!("harness-{optimization}.exe")
        } else {
            format!("harness-{optimization}")
        });
        let runtime_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime");
        let diagnostic_runtime = runtime_dir.join("aero_diagnostic_runtime.c");
        let mut clang = Command::new("clang");
        clang
            .args(["-std=c11", optimization, "-Wall", "-Wextra", "-Werror"])
            .arg(&harness)
            .arg(runtime_dir.join("aero_runtime.c"));
        // A missing implementation must still exercise the unresolved ABI export,
        // rather than failing only because Clang cannot open an input filename.
        if diagnostic_runtime.is_file() {
            clang.arg(diagnostic_runtime);
        }
        let output = clang
            .arg("-o")
            .arg(&executable)
            .output()
            .expect("execute Clang for diagnostic runtime ABI");
        assert!(
            output.status.success(),
            "H1E diagnostic runtime ABI link failed at {optimization}: status={}, stdout={:?}, stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        executable
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        if self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("h1e-stderr-"))
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn assert_output(output: Output, stdout: &[u8], stderr: &[u8]) {
    assert_eq!(
        output.status.code(),
        Some(91),
        "native ABI status={}, stdout={:?}, stderr={:?}",
        output.status,
        output.stdout,
        output.stderr
    );
    assert_eq!(output.stdout, stdout, "exact native stdout bytes");
    assert_eq!(output.stderr, stderr, "exact native stderr bytes");
}

#[test]
fn stderr_preserves_all_binary_bytes_and_flushes_before_success() {
    let workspace = TestWorkspace::new("binary");
    let source = r#"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int32_t aero_stderr_write_byte(int32_t value);
int main(void) {
    char buffer[4096];
    if (setvbuf(stderr, buffer, _IOFBF, sizeof(buffer)) != 0) return 70;
    for (int32_t byte = 0; byte <= 255; byte++) {
        if (aero_stderr_write_byte(byte) != 0) return 71;
    }
    /* _Exit bypasses stream cleanup, so success must already have flushed. */
    _Exit(91);
}
"#;
    let expected: Vec<u8> = (0..=255).collect();
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        assert_output(
            Command::new(executable).output().expect("run binary ABI"),
            b"",
            &expected,
        );
    }
}

#[test]
fn stderr_range_errors_are_sticky_and_write_nothing() {
    let workspace = TestWorkspace::new("range");
    let source = r#"
#include <stdint.h>
#include <stdlib.h>
extern int32_t aero_stderr_write_byte(int32_t value);
int main(int argc, char **argv) {
    if (argc != 2) return 70;
    int32_t value = (int32_t)strtol(argv[1], NULL, 10);
    if (aero_stderr_write_byte(value) != -3) return 71;
    if (aero_stderr_write_byte(0) != -3) return 72;
    if (aero_stderr_write_byte(255) != -3) return 73;
    if (aero_stderr_write_byte(value) != -3) return 74;
    return 91;
}
"#;
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        for value in ["-2147483648", "-1", "256", "2147483647"] {
            assert_output(
                Command::new(&executable)
                    .arg(value)
                    .output()
                    .expect("run range ABI"),
                b"",
                b"",
            );
        }
    }
}

#[test]
fn stderr_state_is_independent_of_stdin_and_stdout() {
    let workspace = TestWorkspace::new("isolation");
    let source = r#"
#include <stdint.h>
extern int32_t aero_stdin_read_byte(void);
extern int32_t aero_stdout_write_byte(int32_t value);
extern int32_t aero_stderr_write_byte(int32_t value);
int main(int argc, char **argv) {
    const int32_t bytes[] = {0, 13, 10, 26, 255};
    if (argc != 2) return 70;
    if (argv[1][0] == 'e') {
        if (aero_stderr_write_byte(-1) != -3) return 71;
        for (unsigned int i = 0; i < sizeof(bytes) / sizeof(bytes[0]); i++) {
            if (aero_stdin_read_byte() != bytes[i]) return 72;
            if (aero_stdout_write_byte(bytes[i]) != 0) return 73;
        }
        if (aero_stdin_read_byte() != -1) return 74;
        if (aero_stderr_write_byte(65) != -3) return 75;
    } else {
        if (aero_stdout_write_byte(256) != -3) return 76;
        if (aero_stdin_read_byte() != -1) return 77;
        for (unsigned int i = 0; i < sizeof(bytes) / sizeof(bytes[0]); i++) {
            if (aero_stderr_write_byte(bytes[i]) != 0) return 78;
        }
        if (aero_stdout_write_byte(65) != -3) return 79;
        if (aero_stdin_read_byte() != -1) return 80;
    }
    return 91;
}
"#;
    let bytes = [0, 13, 10, 26, 255];
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        let mut child = Command::new(&executable)
            .arg("error-on-stderr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run stderr isolation ABI");
        child
            .stdin
            .take()
            .expect("child stdin")
            .write_all(&bytes)
            .expect("supply binary input");
        assert_output(
            child.wait_with_output().expect("capture isolation"),
            &bytes,
            b"",
        );
        assert_output(
            Command::new(&executable)
                .arg("stdout-error")
                .stdin(Stdio::null())
                .output()
                .expect("run stdout isolation ABI"),
            b"",
            &bytes,
        );
    }
}

#[cfg(windows)]
#[test]
fn stderr_invalid_windows_handle_is_sticky_after_restore() {
    let workspace = TestWorkspace::new("invalid-handle");
    let source = r#"
#include <stdint.h>
#include <windows.h>
extern int32_t aero_stdout_write_byte(int32_t value);
extern int32_t aero_stderr_write_byte(int32_t value);
int main(int argc, char **argv) {
    if (argc != 2) return 70;
    HANDLE saved = GetStdHandle(STD_ERROR_HANDLE);
    HANDLE invalid = argv[1][0] == 'n' ? NULL : INVALID_HANDLE_VALUE;
    if (!SetStdHandle(STD_ERROR_HANDLE, invalid)) return 71;
    if (aero_stderr_write_byte(65) != -2) return 72;
    if (aero_stderr_write_byte(256) != -2) return 73;
    if (!SetStdHandle(STD_ERROR_HANDLE, saved)) return 74;
    if (aero_stderr_write_byte(66) != -2) return 75;
    if (aero_stdout_write_byte(79) != 0) return 76;
    return 91;
}
"#;
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        for handle in ["null", "invalid"] {
            assert_output(
                Command::new(&executable)
                    .arg(handle)
                    .output()
                    .expect("run Windows invalid handle ABI"),
                b"O",
                b"",
            );
        }
    }
}

#[test]
fn stderr_closed_descriptor_errors_are_sticky_after_restore() {
    let workspace = TestWorkspace::new("closed");
    let source = r#"
#include <stdint.h>
#include <stdio.h>
#ifdef _WIN32
#include <io.h>
#define descriptor_close _close
#define descriptor_dup _dup
#define descriptor_dup2 _dup2
#else
#include <unistd.h>
#define descriptor_close close
#define descriptor_dup dup
#define descriptor_dup2 dup2
#endif
extern int32_t aero_stdout_write_byte(int32_t value);
extern int32_t aero_stderr_write_byte(int32_t value);
int main(int argc, char **argv) {
    if (argc != 2) return 70;
    int initialized = argv[1][0] == 'i';
    if (setvbuf(stderr, NULL, _IONBF, 0) != 0) return 69;
    int saved = descriptor_dup(2);
    if (saved < 0) return 71;
    if (initialized && aero_stderr_write_byte(83) != 0) return 72;
    if (descriptor_close(2) != 0) return 73;
    int32_t expected = -1;
#ifdef _WIN32
    if (!initialized) expected = -2;
#endif
    if (aero_stderr_write_byte(65) != expected) return 74;
    /* The channel's earlier error wins over a newly invalid value. */
    if (aero_stderr_write_byte(256) != expected) return 75;
    if (descriptor_dup2(saved, 2) < 0) return 76;
    if (descriptor_close(saved) != 0) return 77;
    clearerr(stderr);
    if (aero_stderr_write_byte(66) != expected) return 78;
    if (aero_stdout_write_byte(79) != 0) return 79;
    return 91;
}
"#;
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        for (mode, stderr) in [("cold", &b""[..]), ("initialized", &b"S"[..])] {
            assert_output(
                Command::new(&executable)
                    .arg(mode)
                    .output()
                    .expect("run closed descriptor ABI"),
                b"O",
                stderr,
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn stderr_broken_pipe_returns_sticky_error_instead_of_sigpipe() {
    let workspace = TestWorkspace::new("broken-pipe");
    let source = r#"
#include <stdint.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
extern int32_t aero_stdout_write_byte(int32_t value);
extern int32_t aero_stderr_write_byte(int32_t value);
int main(void) {
    /* Force the broken pipe to fail at fflush, after fputc buffers its byte. */
    char buffer[4096];
    if (setvbuf(stderr, buffer, _IOFBF, sizeof(buffer)) != 0) return 69;
    int descriptors[2];
    int saved = dup(2);
    if (saved < 0) return 70;
    if (pipe(descriptors) != 0) return 71;
    if (close(descriptors[0]) != 0) return 72;
    if (dup2(descriptors[1], 2) < 0) return 73;
    if (close(descriptors[1]) != 0) return 74;
    if (signal(SIGPIPE, SIG_DFL) == SIG_ERR) return 75;
    if (aero_stderr_write_byte(65) != -1) return 76;
    if (aero_stderr_write_byte(-1) != -1) return 77;
    if (dup2(saved, 2) < 0) return 78;
    if (close(saved) != 0) return 79;
    clearerr(stderr);
    if (aero_stderr_write_byte(66) != -1) return 80;
    if (aero_stdout_write_byte(79) != 0) return 81;
    _Exit(91);
}
"#;
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        assert_output(
            Command::new(executable)
                .output()
                .expect("run broken pipe ABI"),
            b"O",
            b"",
        );
    }
}

#[cfg(unix)]
#[test]
fn stderr_sigpipe_setup_failure_is_sticky_and_precedes_io() {
    let workspace = TestWorkspace::new("signal-setup");
    let source = r#"
#include <stdint.h>
#include <signal.h>
static int signal_calls = 0;
/* Fault injection affects only signal setup; the production ABI is linked. */
void (*signal(int number, void (*handler)(int)))(int) {
    if (number != SIGPIPE || handler != SIG_IGN) return SIG_DFL;
    signal_calls++;
    return SIG_ERR;
}
extern int32_t aero_stderr_write_byte(int32_t value);
int main(void) {
    if (aero_stderr_write_byte(65) != -1) return 70;
    if (aero_stderr_write_byte(256) != -1) return 71;
    if (aero_stderr_write_byte(66) != -1) return 72;
    if (signal_calls != 1) return 73;
    return 91;
}
"#;
    for optimization in ["-O0", "-O2"] {
        let executable = workspace.compile(optimization, source);
        assert_output(
            Command::new(executable)
                .output()
                .expect("run setup failure ABI"),
            b"",
            b"",
        );
    }
}
