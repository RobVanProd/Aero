use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-stable-scalar-profile-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh stable-profile test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-stable-scalar-profile-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn run_check(workspace: &TestWorkspace, input: &Path, stable_profile: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.arg("check").arg(input);
    if stable_profile {
        command
            .arg("--language-profile")
            .arg("stable-scalar-v0");
    }
    command
        .current_dir(&workspace.root)
        .output()
        .expect("run aero check")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn strip_cli_colors(output: &str) -> String {
    output
        .replace("\x1b[1;31m", "")
        .replace("\x1b[1;32m", "")
        .replace("\x1b[1;34m", "")
        .replace("\x1b[1;36m", "")
        .replace("\x1b[0m", "")
}

#[test]
fn stable_scalar_profile_selects_a_valid_scalar_program() {
    let workspace = TestWorkspace::new("positive-red");
    let source = workspace.path("main.aero");
    fs::write(
        &source,
        r#"
fn advance(value: int, limit: int) -> int {
    if value < limit {
        return value + 3;
    }
    return value - 1;
}

fn main() -> int {
    let mut value: int = 2;
    while value < 11 {
        value = advance(value, 11);
    }
    return value * 7;
}
"#,
    )
    .expect("write stable scalar source");

    let output = run_check(&workspace, &source, true);
    assert!(
        output.status.success(),
        "stable-scalar-v0 selection is absent or rejected a valid profile program:\n{}",
        combined_output(&output)
    );
}

#[test]
fn stable_scalar_profile_rejects_an_out_of_profile_program_before_compilation() {
    let workspace = TestWorkspace::new("negative-red");
    let source = workspace.path("main.aero");
    fs::write(
        &source,
        "struct Reading { value: int } fn main() -> int { return 0; }",
    )
    .expect("write out-of-profile source");

    let experimental = run_check(&workspace, &source, false);
    assert!(
        experimental.status.success(),
        "experimental default control unexpectedly rejected the declaration:\n{}",
        combined_output(&experimental)
    );

    let stable = run_check(&workspace, &source, true);
    let diagnostics = strip_cli_colors(&combined_output(&stable));
    assert_eq!(stable.status.code(), Some(1), "{diagnostics}");
    assert!(
        diagnostics.contains(
            "Language Profile Error: stable-scalar-v0 rejects struct definitions"
        ),
        "profile rejection was not the compiler-owned diagnostic:\n{diagnostics}"
    );
    assert!(
        !diagnostics.contains("Usage:"),
        "profile selection was treated as a CLI invocation error:\n{diagnostics}"
    );
}
