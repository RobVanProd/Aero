use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
            "aero-registry-quarantine-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create registry test workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create registry fixture directory");
        }
        fs::write(&path, contents).expect("write registry fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_aero(workspace: &TestWorkspace, args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(args)
        .output()
        .expect("run Aero CLI")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_live_cli_quarantined(output: &Output) {
    let text = diagnostics(output);
    assert!(
        !output.status.success(),
        "live registry command unexpectedly succeeded: {text}"
    );
    assert!(
        text.contains(LIVE_REGISTRY_DISABLED),
        "live registry command did not report the frozen quarantine: {text}"
    );
    assert!(
        !text.contains("registry token is empty"),
        "credential resolution preceded the quarantine: {text}"
    );
}

#[test]
fn cli_live_search_is_quarantined_before_credentials_or_transport() {
    let workspace = TestWorkspace::new("live-search");
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "search".to_string(),
            "vision".to_string(),
            "--live".to_string(),
            "--registry".to_string(),
            "not-a-url".to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );

    assert_live_cli_quarantined(&output);
}

#[test]
fn cli_live_publish_is_quarantined_before_credentials_or_package_access() {
    let workspace = TestWorkspace::new("live-publish");
    let missing_package = workspace.path("missing-package");
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "publish".to_string(),
            missing_package.display().to_string(),
            "--registry".to_string(),
            "not-a-url".to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );

    assert_live_cli_quarantined(&output);
    assert!(!missing_package.exists());
}

#[test]
fn cli_live_install_is_quarantined_before_credentials_resolution_or_writes() {
    let workspace = TestWorkspace::new("live-install");
    let target = workspace.path("install-target");
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "install".to_string(),
            "../escape".to_string(),
            "--version".to_string(),
            "../../outside".to_string(),
            "--target".to_string(),
            target.display().to_string(),
            "--registry".to_string(),
            "not-a-url".to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );

    assert_live_cli_quarantined(&output);
    assert!(!target.exists());
}

#[test]
fn cli_local_search_is_credential_free() {
    let workspace = TestWorkspace::new("local-search");
    let index = workspace.write(
        "index.json",
        r#"{"packages":[{"name":"vision-core","version":"0.2.0","description":"vision kernels","downloads":120}]}"#,
    );
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "search".to_string(),
            "vision".to_string(),
            "--index".to_string(),
            index.display().to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );
    let text = diagnostics(&output);

    assert!(output.status.success(), "local search failed: {text}");
    assert!(text.contains("Found 1 package(s)"), "{text}");
    assert!(text.contains("vision-core 0.2.0"), "{text}");
}

#[test]
fn cli_publish_dry_run_is_credential_free_and_local() {
    let workspace = TestWorkspace::new("publish-preview");
    let package = workspace.path("package");
    workspace.write(
        "package/aero.toml",
        "name = \"vision-core\"\nversion = \"0.2.0\"\n",
    );
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "publish".to_string(),
            package.display().to_string(),
            "--dry-run".to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );
    let text = diagnostics(&output);

    assert!(output.status.success(), "publish preview failed: {text}");
    assert!(text.contains("Registry publish preview:"), "{text}");
    assert!(text.contains("\"package_name\": \"vision-core\""), "{text}");
    assert!(text.contains("\"dry_run\": true"), "{text}");
}

#[test]
fn cli_install_dry_run_is_credential_free_local_and_write_free() {
    let workspace = TestWorkspace::new("install-plan");
    let target = workspace.path("install-target");
    let output = run_aero(
        &workspace,
        &[
            "registry".to_string(),
            "install".to_string(),
            "vision-core".to_string(),
            "--version".to_string(),
            "0.2.0".to_string(),
            "--target".to_string(),
            target.display().to_string(),
            "--dry-run".to_string(),
            "--token".to_string(),
            String::new(),
        ],
    );
    let text = diagnostics(&output);

    assert!(output.status.success(), "install plan failed: {text}");
    assert!(text.contains("Registry install plan:"), "{text}");
    assert!(text.contains("\"package_name\": \"vision-core\""), "{text}");
    assert!(text.contains("\"dry_run\": true"), "{text}");
    assert!(!target.exists(), "dry-run created install target");
}

#[test]
fn cli_registry_help_states_quarantine_and_preserved_local_workflows() {
    let workspace = TestWorkspace::new("registry-help");
    let output = run_aero(&workspace, &["registry".to_string(), "help".to_string()]);
    let text = diagnostics(&output);

    assert!(output.status.success(), "registry help failed: {text}");
    assert!(
        text.contains(
            "Local index search and publish/install --dry-run are credential-free and network-free."
        ),
        "registry help omitted preserved local workflows: {text}"
    );
    assert!(
        text.contains(LIVE_REGISTRY_DISABLED),
        "registry help omitted the live quarantine: {text}"
    );
}
