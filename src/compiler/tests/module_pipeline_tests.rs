use compiler::{CompilerOptions, compile_program};
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
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-module-pipeline-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create module test workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create module fixture directory");
        }
        fs::write(&path, contents).expect("write module fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_aero(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for arg in args {
        command.arg(arg);
    }
    command.output().expect("run Aero CLI")
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).expect("stat fake tool").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make fake tool executable");
}

fn write_marker_verifier(workspace: &TestWorkspace) -> PathBuf {
    #[cfg(windows)]
    let path = workspace.path("tools/fake-opt.cmd");
    #[cfg(not(windows))]
    let path = workspace.path("tools/fake-opt");

    #[cfg(windows)]
    let script = r#"@echo off
>>"%AERO_MODULE_VERIFIER_MARKER%" echo verifier %*
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
if not "%~1"=="-passes=verify" exit /b 90
if not "%~2"=="-disable-output" exit /b 90
if not "%~3"=="-" exit /b 90
if not "%~4"=="" exit /b 90
"%SystemRoot%\System32\more.com" >nul
exit /b 0
"#;

    #[cfg(not(windows))]
    let script = r#"#!/bin/sh
printf '%s\n' "verifier $*" >> "$AERO_MODULE_VERIFIER_MARKER"
if [ "${1:-}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
if [ "$#" -ne 3 ] || [ "$1" != "-passes=verify" ] || [ "$2" != "-disable-output" ] || [ "$3" != "-" ]; then
  exit 90
fi
/bin/cat >/dev/null
exit 0
"#;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fake verifier directory");
    }
    fs::write(&path, script).expect("write marker verifier");
    #[cfg(unix)]
    make_executable(&path);
    path
}

fn write_marker_native_tools(workspace: &TestWorkspace) -> PathBuf {
    let bin = workspace.path("fake-native-bin");
    fs::create_dir_all(&bin).expect("create fake native tool directory");

    #[cfg(windows)]
    {
        let source = workspace.write(
            "tools/fake-native.rs",
            r#"use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    let marker = std::env::var_os("AERO_MODULE_NATIVE_MARKER")
        .expect("native marker environment");
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(marker)
        .expect("open native marker");
    let executable = std::env::current_exe().expect("current executable");
    writeln!(output, "{} invoked", executable.display()).expect("write native marker");
    std::process::exit(97);
}
"#,
        );
        let clang = bin.join("clang.exe");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc)
            .arg(&source)
            .arg("-o")
            .arg(&clang)
            .output()
            .expect("compile marker native tool");
        assert!(
            output.status.success(),
            "failed to compile marker native tool: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::copy(&clang, bin.join("llc.exe")).expect("copy marker llc tool");
    }

    #[cfg(not(windows))]
    for tool in ["clang", "llc"] {
        let path = bin.join(tool);
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' '{tool} invoked' >> \"$AERO_MODULE_NATIVE_MARKER\"\nexit 97\n"
        );

        fs::write(&path, script).expect("write marker native tool");
        make_executable(&path);
    }

    bin
}

fn run_aero_with_phase_markers(
    workspace: &TestWorkspace,
    args: &[&Path],
    verifier_marker: &Path,
    native_marker: &Path,
) -> Output {
    let verifier = write_marker_verifier(workspace);
    let fake_native_bin = write_marker_native_tools(workspace);
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command
        .current_dir(&workspace.root)
        .args(args)
        .env("AERO_LLVM_OPT", verifier)
        .env_remove("AERO_LLVM_AS")
        .env_remove("AERO_REQUIRE_LLVM_VERIFIER")
        .env("AERO_MODULE_VERIFIER_MARKER", verifier_marker)
        .env("AERO_MODULE_NATIVE_MARKER", native_marker)
        .env("PATH", fake_native_bin);
    #[cfg(windows)]
    command.env("PATHEXT", ".CMD;.EXE");
    command.output().expect("run marked Aero CLI")
}

fn assert_phase_markers_absent(verifier_marker: &Path, native_marker: &Path, output: &Output) {
    let text = diagnostics(output);
    let mut reached = Vec::new();
    if verifier_marker.exists() {
        reached.push("external LLVM verifier");
    }
    if native_marker.exists() {
        reached.push("native tool discovery/execution");
    }
    assert!(
        reached.is_empty(),
        "missing module reached {}: {text}",
        reached.join(" and ")
    );
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn missing_module_diagnostic(text: &str) -> bool {
    text.contains("Module resolution failed for `absent`")
        && text.contains("Cannot find module `absent`")
}

fn assert_missing_module_failure(output: &Output, artifacts: &[&Path]) {
    let text = diagnostics(output);
    assert!(
        !output.status.success(),
        "missing module command unexpectedly succeeded: {text}"
    );
    assert!(
        missing_module_diagnostic(&text),
        "missing shared module-resolution diagnostic: {text}"
    );
    assert!(
        !text.contains("completed successfully"),
        "missing module command printed a success message: {text}"
    );
    for artifact in artifacts {
        assert!(
            !artifact.exists(),
            "missing module command published {}: {text}",
            artifact.display()
        );
    }
}

fn assert_run_artifacts_clean(workspace: &TestWorkspace) {
    let run_root = workspace.path("target").join("aero-run");
    if !run_root.exists() {
        return;
    }
    let entries = fs::read_dir(&run_root)
        .expect("read run artifact root")
        .map(|entry| entry.expect("read run artifact entry").path())
        .collect::<Vec<_>>();
    assert!(
        entries.is_empty(),
        "failed run left temporary artifact directories/files: {entries:?}"
    );
}

#[test]
fn source_only_compile_program_rejects_module_without_entry_context() {
    let error = compile_program("mod absent; fn main() {}", CompilerOptions::default())
        .expect_err("source-only compilation silently accepted a file-backed module");

    assert_eq!(
        error,
        "Module resolution failed for `absent`: module declarations require an entry-file context"
    );
}

#[test]
fn module_free_source_only_compilation_remains_accepted() {
    let result = compile_program("fn main() {}", CompilerOptions::default());
    assert!(
        result.is_ok(),
        "module-free source stopped compiling: {result:?}"
    );
}

#[test]
fn build_rejects_missing_direct_module_without_llvm_artifact() {
    let workspace = TestWorkspace::new("missing-build");
    let root = workspace.write("main.aero", "mod absent; fn main() {}");
    let artifact = workspace.path("program.ll");
    let verifier_marker = workspace.path("verifier-invoked");
    let native_marker = workspace.path("native-invoked");

    let output = run_aero_with_phase_markers(
        &workspace,
        &[Path::new("build"), &root, Path::new("-o"), &artifact],
        &verifier_marker,
        &native_marker,
    );

    assert_phase_markers_absent(&verifier_marker, &native_marker, &output);
    assert_missing_module_failure(&output, &[&artifact]);
}

#[test]
fn run_rejects_missing_direct_module_before_verifier_or_native_tools_and_cleans_up() {
    let workspace = TestWorkspace::new("missing-run");
    let root = workspace.write("main.aero", "mod absent; fn main() {}");
    let verifier_marker = workspace.path("verifier-invoked");
    let native_marker = workspace.path("native-invoked");

    let output = run_aero_with_phase_markers(
        &workspace,
        &[Path::new("run"), &root],
        &verifier_marker,
        &native_marker,
    );
    assert_phase_markers_absent(&verifier_marker, &native_marker, &output);
    assert_missing_module_failure(&output, &[]);

    let text = diagnostics(&output);
    for forbidden in [
        "Semantic Analysis Result:",
        "Optimized IR generation completed",
        "LLVM Verification Error:",
        "Error executing llc",
        "Error executing clang",
        "Error running llc",
        "Error running clang",
    ] {
        assert!(
            !text.contains(forbidden),
            "missing module reached a later compiler/native phase `{forbidden}`: {text}"
        );
    }
    assert_run_artifacts_clean(&workspace);
}

#[test]
fn check_profile_test_and_doc_share_missing_module_failure_without_artifacts() {
    let workspace = TestWorkspace::new("missing-shared-routes");
    let root = workspace.write("module_test.aero", "mod absent; fn main() {}");
    let trace = workspace.path("profile.json");
    let markdown = workspace.path("api.md");

    let routes = [
        (
            "check",
            run_aero(&workspace, &[Path::new("check"), &root]),
            Vec::<&Path>::new(),
        ),
        (
            "profile",
            run_aero(
                &workspace,
                &[Path::new("profile"), &root, Path::new("-o"), &trace],
            ),
            vec![trace.as_path()],
        ),
        (
            "test",
            run_aero(&workspace, &[Path::new("test")]),
            Vec::<&Path>::new(),
        ),
        (
            "doc",
            run_aero(
                &workspace,
                &[Path::new("doc"), &root, Path::new("-o"), &markdown],
            ),
            vec![markdown.as_path()],
        ),
    ];

    let mut failures = Vec::new();
    for (route, output, artifacts) in routes {
        let text = diagnostics(&output);
        if output.status.success() {
            failures.push(format!("{route} succeeded: {text}"));
        }
        if !missing_module_diagnostic(&text) {
            failures.push(format!("{route} lost shared diagnostic: {text}"));
        }
        for artifact in artifacts {
            if artifact.exists() {
                failures.push(format!("{route} published {}: {text}", artifact.display()));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn direct_file_and_directory_module_layouts_remain_accepted() {
    for (label, module_path) in [
        ("file-layout", "helper.aero"),
        ("directory-layout", "helper/mod.aero"),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write("main.aero", "mod helper; fn main() { helper(); }");
        workspace.write(module_path, "fn helper() {}");

        let output = run_aero(&workspace, &[Path::new("check"), &root]);
        assert!(
            output.status.success(),
            "{module_path} stopped resolving: {}",
            diagnostics(&output)
        );
    }
}

#[test]
fn nested_module_declaration_fails_closed_until_graph_semantics_are_frozen() {
    let workspace = TestWorkspace::new("nested-unsupported");
    let root = workspace.write("main.aero", "mod outer; fn main() { outer(); }");
    workspace.write("outer.aero", "mod inner; fn outer() {}");
    workspace.write("inner.aero", "fn inner() {}");

    let output = run_aero(&workspace, &[Path::new("check"), &root]);
    let text = diagnostics(&output);
    assert!(
        !output.status.success(),
        "nested module was silently accepted: {text}"
    );
    assert!(
        text.contains("Module resolution failed for `inner`")
            && text
                .contains("nested module declarations are not supported by the current compiler"),
        "nested module lost the frozen unsupported diagnostic: {text}"
    );
}
