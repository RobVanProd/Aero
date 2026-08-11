use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
            "aero-cap-007-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-007 workspace");
        Self { root }
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, source).expect("write CAP-007 fixture");
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
    if metadata.is_dir()
        && let Ok(entries) = fs::read_dir(root)
    {
        for entry in entries.flatten() {
            clear_readonly_recursively(&entry.path());
        }
    }

    #[cfg(windows)]
    {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = fs::set_permissions(root, permissions);
        }
    }
}

fn run_aero(workspace: &TestWorkspace, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .expect("run Aero CLI")
}

fn visible_output(output: &Output) -> String {
    let raw = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut visible = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
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

#[test]
fn discovered_sources_must_pass_checked_admission_before_completion_is_reported() {
    let workspace = TestWorkspace::new("source-test-admission");
    workspace.write(
        "admission_test.aero",
        "fn main() { let quotient: int = 1 / 0; }",
    );

    let output = run_aero(&workspace, &["test"]);
    let rendered = visible_output(&output);
    assert!(
        !output.status.success(),
        "semantic-only test discovery falsely succeeded on a checked-admission rejection:\n{rendered}"
    );
    assert!(
        rendered.contains("IR Generation Error:")
            && rendered.contains("constant integer division by zero"),
        "canonical checked-admission diagnostic was not reported:\n{rendered}"
    );
    assert!(
        !rendered.contains("admission_test.aero analysis completed"),
        "source rejected by checked admission was reported completed:\n{rendered}"
    );
}
