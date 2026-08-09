use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
            "aero-llvm-verifier-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create verifier test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, contents).expect("write verifier test fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy)]
enum FakeBehavior {
    Accept,
    Reject,
    RejectSecond,
    Timeout,
    WrongVersion,
}

#[derive(Clone, Copy)]
enum FakeTool {
    Opt,
    LlvmAs,
}

impl FakeTool {
    fn name(self) -> &'static str {
        match self {
            Self::Opt => "opt",
            Self::LlvmAs => "llvm-as",
        }
    }
}

impl FakeBehavior {
    fn name(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
            Self::RejectSecond => "reject_second",
            Self::Timeout => "timeout",
            Self::WrongVersion => "wrong_version",
        }
    }
}

fn write_fake_tool(
    workspace: &TestWorkspace,
    executable_name: &str,
    tool_id: &str,
    tool: FakeTool,
    behavior: FakeBehavior,
) -> PathBuf {
    #[cfg(windows)]
    let path = workspace.path(&format!("{executable_name}.cmd"));
    #[cfg(not(windows))]
    let path = workspace.path(executable_name);

    #[cfg(windows)]
    let script = format!(
        r#"@echo off
setlocal EnableExtensions EnableDelayedExpansion
if "%~1"=="--version" (
  if defined AERO_FAKE_LOG >>"%AERO_FAKE_LOG%" echo {tool_id} VERSION
  if "{behavior}"=="wrong_version" (echo LLVM version 21.0.0) else (echo LLVM version 22.1.0)
  exit /b 0
)
if "{tool}"=="opt" (
  if not "%~1"=="-passes=verify" goto bad_args
  if not "%~2"=="-disable-output" goto bad_args
  if not "%~3"=="-" goto bad_args
  if not "%~4"=="" goto bad_args
) else (
  if not "%~1"=="-o" goto bad_args
  if not "%~2"=="-" goto bad_args
  if not "%~3"=="-" goto bad_args
  if not "%~4"=="" goto bad_args
)
set "count_file=%AERO_FAKE_STATE_DIR%\{tool_id}.count"
set /a count=0
if exist "!count_file!" set /p count=<"!count_file!"
set /a count+=1
>"!count_file!" echo !count!
>>"%AERO_FAKE_LOG%" echo {tool_id} CALL !count!
set "stdin_file=%AERO_FAKE_STATE_DIR%\{tool_id}-!count!.stdin"
set "AERO_FAKE_STDIN_FILE=!stdin_file!"
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -Command "$inputStream=[Console]::OpenStandardInput(); $outputStream=[IO.File]::Create($env:AERO_FAKE_STDIN_FILE); try {{ $inputStream.CopyTo($outputStream) }} finally {{ $outputStream.Dispose() }}"
if errorlevel 1 exit /b 93
for %%A in ("!stdin_file!") do if %%~zA LEQ 0 (
  >&2 echo fake verifier received empty stdin
  exit /b 92
)
if defined AERO_FAKE_OUTPUT if exist "%AERO_FAKE_OUTPUT%" (
  >&2 echo fake verifier observed output before verification
  exit /b 91
)
if "{behavior}"=="timeout" "%SystemRoot%\System32\ping.exe" 127.0.0.1 -n 61 >nul
if "{behavior}"=="reject" (
  >&2 echo fake verifier rejected module
  exit /b 42
)
if "{behavior}"=="reject_second" if !count! GEQ 2 (
  >&2 echo fake verifier rejected transformed module
  exit /b 43
)
exit /b 0
:bad_args
>&2 echo fake {tool} received invalid verifier arguments: %*
exit /b 90
"#,
        behavior = behavior.name(),
        tool = tool.name(),
    );

    #[cfg(not(windows))]
    let script = format!(
        r#"#!/bin/sh
if [ "${{1:-}}" = "--version" ]; then
  if [ -n "${{AERO_FAKE_LOG:-}}" ]; then printf '%s\n' "{tool_id} VERSION" >> "$AERO_FAKE_LOG"; fi
  if [ "{behavior}" = "wrong_version" ]; then
    echo "LLVM version 21.0.0"
  else
    echo "LLVM version 22.1.0"
  fi
  exit 0
fi
if [ "{tool}" = "opt" ]; then
  if [ "$#" -ne 3 ] || [ "$1" != "-passes=verify" ] || [ "$2" != "-disable-output" ] || [ "$3" != "-" ]; then
    echo "fake opt received invalid verifier arguments: $*" >&2
    exit 90
  fi
else
  if [ "$#" -ne 3 ] || [ "$1" != "-o" ] || [ "$2" != "-" ] || [ "$3" != "-" ]; then
    echo "fake llvm-as received invalid verifier arguments: $*" >&2
    exit 90
  fi
fi
count_file="$AERO_FAKE_STATE_DIR/{tool_id}.count"
count=0
if [ -f "$count_file" ]; then read -r count < "$count_file"; fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
printf '%s\n' "{tool_id} CALL $count" >> "$AERO_FAKE_LOG"
stdin_file="$AERO_FAKE_STATE_DIR/{tool_id}-$count.stdin"
/bin/cat > "$stdin_file"
if [ ! -s "$stdin_file" ]; then
  echo "fake verifier received empty stdin" >&2
  exit 92
fi
if [ -n "${{AERO_FAKE_OUTPUT:-}}" ] && [ -e "$AERO_FAKE_OUTPUT" ]; then
  echo "fake verifier observed output before verification" >&2
  exit 91
fi
if [ "{behavior}" = "timeout" ]; then /bin/sleep 60; fi
if [ "{behavior}" = "reject" ]; then
  echo "fake verifier rejected module" >&2
  exit 42
fi
if [ "{behavior}" = "reject_second" ] && [ "$count" -ge 2 ]; then
  echo "fake verifier rejected transformed module" >&2
  exit 43
fi
exit 0
"#,
        behavior = behavior.name(),
        tool = tool.name(),
    );

    fs::write(&path, script).expect("write fake verifier");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("stat fake verifier")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake verifier executable");
    }
    path
}

fn captured_input(workspace: &TestWorkspace, tool_id: &str, call: usize) -> Vec<u8> {
    fs::read(workspace.path(&format!("{tool_id}-{call}.stdin")))
        .expect("read fake verifier stdin capture")
}

fn base_command(workspace: &TestWorkspace) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for key in [
        "AERO_LLVM_OPT",
        "AERO_LLVM_AS",
        "AERO_REQUIRE_LLVM_VERIFIER",
        "AERO_FAKE_LOG",
        "AERO_FAKE_OUTPUT",
        "AERO_FAKE_STATE_DIR",
    ] {
        command.env_remove(key);
    }
    command
}

fn add_args<I, S>(command: &mut Command, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command.args(args);
}

fn configure_fake(
    command: &mut Command,
    workspace: &TestWorkspace,
    variable: &str,
    tool: &Path,
    output: Option<&Path>,
) {
    command.env(variable, tool);
    command.env("AERO_FAKE_STATE_DIR", &workspace.root);
    command.env("AERO_FAKE_LOG", workspace.path("verifier.log"));
    if let Some(output) = output {
        command.env("AERO_FAKE_OUTPUT", output);
    }
}

fn empty_path(workspace: &TestWorkspace) -> OsString {
    let path = workspace.path("empty-path");
    fs::create_dir_all(&path).expect("create empty PATH directory");
    path.into_os_string()
}

fn run(mut command: Command) -> Output {
    command.output().expect("run aero command")
}

fn run_bounded(mut command: Command, timeout: Duration) -> Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn bounded aero command");
    let started = Instant::now();
    loop {
        if child.try_wait().expect("poll aero command").is_some() {
            return child
                .wait_with_output()
                .expect("collect aero command output");
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("collect killed aero command output");
            panic!(
                "aero did not enforce verifier timeout within {:?}: {}",
                timeout,
                diagnostics(&output)
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn valid_source(workspace: &TestWorkspace) -> PathBuf {
    workspace.write("main.aero", "let value = 10 + 5;\nreturn value;\n")
}

fn valid_llvm() -> &'static str {
    "define i32 @main() {\nentry:\n  ret i32 0\n}\n"
}

fn assert_llvm_failure(output: &Output, artifact: &Path, needle: &str) {
    let text = diagnostics(output);
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {text}"
    );
    assert!(
        text.contains("LLVM Verification Error:") && text.contains(needle),
        "missing stable LLVM verification diagnostic `{needle}`: {text}"
    );
    assert!(
        !artifact.exists(),
        "failed verification published {}",
        artifact.display()
    );
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
fn fake_verifier_control_is_executable_and_reports_llvm_22() {
    let workspace = TestWorkspace::new("fake-control");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "control",
        FakeTool::Opt,
        FakeBehavior::Accept,
    );
    let output = Command::new(verifier)
        .arg("--version")
        .output()
        .expect("execute fake verifier control");
    assert!(output.status.success(), "fake verifier control failed");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("LLVM version 22.1.0"),
        "fake verifier did not report LLVM 22"
    );
}

#[test]
fn build_verifies_final_graph_transformed_and_retargeted_bytes_before_publication() {
    let workspace = TestWorkspace::new("build-final-bytes");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "override-opt",
        FakeTool::Opt,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
            OsStr::new("--target"),
            OsStr::new("rocm"),
        ],
    );
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &verifier,
        Some(&artifact),
    );

    let output = run(command);
    assert!(
        output.status.success(),
        "verified build failed: {}",
        diagnostics(&output)
    );
    assert!(artifact.exists(), "verified build did not publish LLVM");
    let verified = captured_input(&workspace, "override-opt", 1);
    let published = fs::read(&artifact).expect("read published LLVM");
    assert_eq!(
        verified, published,
        "externally verified source-build bytes differ from published bytes"
    );
    let verified = String::from_utf8(verified).expect("verified LLVM is UTF-8 text");
    assert!(
        verified.contains("; aero.graph_compilation=enabled"),
        "verifier saw pre-graph LLVM: {verified}"
    );
    assert!(
        verified.contains("target triple = \"amdgcn-amd-amdhsa\""),
        "verifier saw pre-retarget LLVM: {verified}"
    );
}

#[test]
fn build_external_failure_matrix_is_fail_closed() {
    for (label, behavior, needle) in [
        ("reject", FakeBehavior::Reject, "rejected"),
        ("wrong-version", FakeBehavior::WrongVersion, "22"),
    ] {
        let workspace = TestWorkspace::new(label);
        let source = valid_source(&workspace);
        let artifact = workspace.path("program.ll");
        let verifier = write_fake_tool(&workspace, "fake-opt", label, FakeTool::Opt, behavior);
        let mut command = base_command(&workspace);
        add_args(
            &mut command,
            [
                OsStr::new("build"),
                source.as_os_str(),
                OsStr::new("-o"),
                artifact.as_os_str(),
            ],
        );
        configure_fake(
            &mut command,
            &workspace,
            "AERO_LLVM_OPT",
            &verifier,
            Some(&artifact),
        );
        assert_llvm_failure(&run(command), &artifact, needle);
    }

    let workspace = TestWorkspace::new("missing-override");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("AERO_LLVM_OPT", workspace.path("missing-opt"));
    assert_llvm_failure(&run(command), &artifact, "not found");
}

#[test]
fn source_ir_failure_precedes_external_verification_and_publication() {
    let workspace = TestWorkspace::new("source-ir-before-external");
    let source = workspace.write("invalid.aero", "fn main() { let compared = [1] == [1]; }\n");
    let artifact = workspace.path("invalid.ll");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "must-not-run",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &verifier,
        Some(&artifact),
    );

    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        !output.status.success(),
        "invalid source build succeeded: {text}"
    );
    assert!(
        text.contains("IR Generation Error:"),
        "source IR generation failure lost precedence over external verification: {text}"
    );
    assert!(!artifact.exists(), "source IR failure published LLVM");
    assert!(
        !workspace.path("verifier.log").exists(),
        "external verifier ran after a source IR failure"
    );
}

#[test]
fn check_is_identical_with_a_rejecting_verifier_and_with_no_verifier() {
    let workspace = TestWorkspace::new("check-tool-independent");
    let source = valid_source(&workspace);
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "must-not-run",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    let mut found = base_command(&workspace);
    add_args(&mut found, [OsStr::new("check"), source.as_os_str()]);
    configure_fake(&mut found, &workspace, "AERO_LLVM_OPT", &verifier, None);
    found.env("AERO_REQUIRE_LLVM_VERIFIER", "true");
    let found = run(found);

    let mut missing = base_command(&workspace);
    add_args(&mut missing, [OsStr::new("check"), source.as_os_str()]);
    missing.env("PATH", empty_path(&workspace));
    missing.env("AERO_REQUIRE_LLVM_VERIFIER", "true");
    let missing = run(missing);

    assert!(
        found.status.success() && missing.status.success(),
        "valid check depended on external LLVM tools: found={} missing={}",
        diagnostics(&found),
        diagnostics(&missing)
    );
    for (label, output) in [("found", &found), ("missing", &missing)] {
        let text = diagnostics(output);
        assert!(
            text.contains("Semantic analysis completed successfully")
                && !text.contains("LLVM Verification")
                && !text.contains("InternalOnly"),
            "{label}-verifier check changed its tool-independent result: {text}"
        );
    }
    assert!(
        !workspace.path("verifier.log").exists(),
        "check invoked a found external verifier"
    );
}

#[test]
fn prefer_external_missing_verifier_builds_fresh_and_reports_internal_only() {
    let workspace = TestWorkspace::new("fresh-internal-only");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", empty_path(&workspace));

    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        output.status.success(),
        "fresh PreferExternal build failed without an LLVM tool: {text}"
    );
    assert!(
        artifact.exists(),
        "fresh InternalOnly build wrote no artifact"
    );
    assert!(
        text.contains("InternalOnly"),
        "fresh PreferExternal build omitted visible InternalOnly status: {text}"
    );
}

#[test]
fn verifier_timeout_is_bounded_and_fail_closed() {
    let workspace = TestWorkspace::new("timeout");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "timeout",
        FakeTool::Opt,
        FakeBehavior::Timeout,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &verifier,
        Some(&artifact),
    );
    // This is only an outer deadlock guard. It is deliberately longer than the
    // verifier adapter's implementation timeout and does not configure production.
    let output = run_bounded(command, Duration::from_secs(30));
    assert_llvm_failure(&output, &artifact, "timed out");
}

#[cfg(unix)]
#[test]
fn explicit_verifier_launch_failure_is_fail_closed() {
    use std::os::unix::fs::PermissionsExt;

    let workspace = TestWorkspace::new("launch-failure");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let verifier = workspace.write("non-executable-opt", "#!/bin/sh\nexit 0\n");
    fs::set_permissions(&verifier, fs::Permissions::from_mode(0o644))
        .expect("make verifier fixture non-executable");
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("AERO_LLVM_OPT", &verifier);
    assert_llvm_failure(&run(command), &artifact, "launch");
}

#[cfg(unix)]
#[test]
fn discovery_uses_llvm_as_only_when_opt_is_absent_and_never_after_opt_rejection() {
    let accepted = TestWorkspace::new("as-fallback");
    let source = valid_source(&accepted);
    let artifact = accepted.path("program.ll");
    write_fake_tool(
        &accepted,
        "llvm-as-22",
        "as",
        FakeTool::LlvmAs,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&accepted);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", &accepted.root);
    command.env("AERO_FAKE_STATE_DIR", &accepted.root);
    command.env("AERO_FAKE_LOG", accepted.path("verifier.log"));
    command.env("AERO_FAKE_OUTPUT", &artifact);
    let output = run(command);
    assert!(
        output.status.success(),
        "llvm-as fallback failed: {}",
        diagnostics(&output)
    );
    assert!(
        fs::read_to_string(accepted.path("verifier.log"))
            .expect("read llvm-as log")
            .contains("as CALL 1"),
        "llvm-as fallback was not invoked"
    );

    for (label, executable_name, tool, tool_id) in [
        ("unversioned-opt", "opt", FakeTool::Opt, "plain-opt"),
        ("unversioned-as", "llvm-as", FakeTool::LlvmAs, "plain-as"),
    ] {
        let workspace = TestWorkspace::new(label);
        let source = valid_source(&workspace);
        let artifact = workspace.path("program.ll");
        write_fake_tool(
            &workspace,
            executable_name,
            tool_id,
            tool,
            FakeBehavior::Accept,
        );
        let mut command = base_command(&workspace);
        add_args(
            &mut command,
            [
                OsStr::new("build"),
                source.as_os_str(),
                OsStr::new("-o"),
                artifact.as_os_str(),
            ],
        );
        command.env("PATH", &workspace.root);
        command.env("AERO_FAKE_STATE_DIR", &workspace.root);
        command.env("AERO_FAKE_LOG", workspace.path("verifier.log"));
        command.env("AERO_FAKE_OUTPUT", &artifact);
        let output = run(command);
        assert!(
            output.status.success(),
            "compatible {label} discovery failed: {}",
            diagnostics(&output)
        );
        let log = fs::read_to_string(workspace.path("verifier.log"))
            .expect("read unversioned discovery log");
        assert!(
            log.contains(&format!("{tool_id} VERSION"))
                && log.contains(&format!("{tool_id} CALL 1")),
            "unversioned tool was not version-validated before use: {log}"
        );
        assert_eq!(
            captured_input(&workspace, tool_id, 1),
            fs::read(&artifact).expect("read unversioned verified artifact"),
            "unversioned verifier did not receive published bytes"
        );
    }

    let incompatible = TestWorkspace::new("incompatible-unversioned-opt");
    let source = valid_source(&incompatible);
    let artifact = incompatible.path("program.ll");
    write_fake_tool(
        &incompatible,
        "opt",
        "wrong-opt",
        FakeTool::Opt,
        FakeBehavior::WrongVersion,
    );
    write_fake_tool(
        &incompatible,
        "llvm-as",
        "compatible-as",
        FakeTool::LlvmAs,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&incompatible);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", &incompatible.root);
    command.env("AERO_FAKE_STATE_DIR", &incompatible.root);
    command.env("AERO_FAKE_LOG", incompatible.path("verifier.log"));
    command.env("AERO_FAKE_OUTPUT", &artifact);
    let output = run(command);
    assert_llvm_failure(&output, &artifact, "22");
    let log = fs::read_to_string(incompatible.path("verifier.log"))
        .expect("read incompatible discovery log");
    assert!(
        log.contains("wrong-opt VERSION")
            && !log.contains("wrong-opt CALL")
            && !log.contains("compatible-as VERSION")
            && !log.contains("compatible-as CALL"),
        "wrong-version opt was not fatal or fell through to llvm-as: {log}"
    );

    let rejected = TestWorkspace::new("opt-no-fallback");
    let source = valid_source(&rejected);
    let artifact = rejected.path("program.ll");
    write_fake_tool(
        &rejected,
        "opt-22",
        "opt",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    write_fake_tool(
        &rejected,
        "llvm-as-22",
        "as",
        FakeTool::LlvmAs,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&rejected);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", &rejected.root);
    command.env("AERO_FAKE_STATE_DIR", &rejected.root);
    command.env("AERO_FAKE_LOG", rejected.path("verifier.log"));
    command.env("AERO_FAKE_OUTPUT", &artifact);
    let output = run(command);
    assert_llvm_failure(&output, &artifact, "rejected");
    let log = fs::read_to_string(rejected.path("verifier.log")).expect("read discovery log");
    assert!(log.contains("opt CALL 1"), "opt was not invoked: {log}");
    assert!(
        !log.contains("as CALL"),
        "llvm-as ran after opt rejection: {log}"
    );
}

#[cfg(unix)]
#[test]
fn explicit_opt_and_llvm_as_overrides_are_authoritative_over_accepting_path_tools() {
    let workspace = TestWorkspace::new("authoritative-override");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let override_tool = write_fake_tool(
        &workspace,
        "override-opt",
        "override",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    write_fake_tool(
        &workspace,
        "opt-22",
        "path-opt",
        FakeTool::Opt,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", &workspace.root);
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &override_tool,
        Some(&artifact),
    );
    let output = run(command);
    assert_llvm_failure(&output, &artifact, "rejected");
    let log = fs::read_to_string(workspace.path("verifier.log")).expect("read override log");
    assert!(
        log.contains("override CALL 1"),
        "override was not invoked: {log}"
    );
    assert!(
        !log.contains("path-opt CALL"),
        "PATH tool bypassed override: {log}"
    );

    let workspace = TestWorkspace::new("authoritative-as-override");
    let source = valid_source(&workspace);
    let artifact = workspace.path("program.ll");
    let override_tool = write_fake_tool(
        &workspace,
        "override-as",
        "override-as",
        FakeTool::LlvmAs,
        FakeBehavior::Reject,
    );
    write_fake_tool(
        &workspace,
        "opt-22",
        "path-opt",
        FakeTool::Opt,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("-o"),
            artifact.as_os_str(),
        ],
    );
    command.env("PATH", &workspace.root);
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_AS",
        &override_tool,
        Some(&artifact),
    );
    let output = run(command);
    assert_llvm_failure(&output, &artifact, "rejected");
    let log = fs::read_to_string(workspace.path("verifier.log")).expect("read as override log");
    assert!(
        log.contains("override-as CALL 1") && !log.contains("path-opt CALL"),
        "AERO_LLVM_AS did not remain authoritative: {log}"
    );
}

#[test]
fn required_flag_environment_and_forced_commands_cannot_degrade_to_internal_only() {
    let workspace = TestWorkspace::new("required-modes");
    let source = valid_source(&workspace);
    let empty_path = empty_path(&workspace);

    for (label, extra_args, required_env) in [
        (
            "flag",
            vec![OsString::from("--require-llvm-verifier")],
            Some("false"),
        ),
        ("environment", Vec::new(), Some("true")),
    ] {
        let artifact = workspace.path(&format!("{label}.ll"));
        let mut args = vec![
            OsString::from("build"),
            source.as_os_str().to_os_string(),
            OsString::from("-o"),
            artifact.as_os_str().to_os_string(),
        ];
        args.extend(extra_args);
        let mut command = base_command(&workspace);
        add_args(&mut command, &args);
        command.env("PATH", &empty_path);
        if let Some(value) = required_env {
            command.env("AERO_REQUIRE_LLVM_VERIFIER", value);
        }
        assert_llvm_failure(&run(command), &artifact, "not found");
    }

    let input = workspace.write("input.ll", valid_llvm());
    for (command_name, extra_args) in [
        ("graph-opt", Vec::new()),
        (
            "quantize",
            vec![OsString::from("--mode"), OsString::from("int8")],
        ),
    ] {
        let artifact = workspace.path(&format!("{command_name}.ll"));
        let mut args = vec![
            OsString::from(command_name),
            input.as_os_str().to_os_string(),
            OsString::from("-o"),
            artifact.as_os_str().to_os_string(),
        ];
        args.extend(extra_args);
        let mut command = base_command(&workspace);
        add_args(&mut command, &args);
        command.env("PATH", &empty_path);
        command.env("AERO_REQUIRE_LLVM_VERIFIER", "false");
        assert_llvm_failure(&run(command), &artifact, "not found");
    }
}

#[test]
fn run_rejects_before_native_tool_discovery_or_object_publication() {
    let workspace = TestWorkspace::new("run-ordering");
    let source = valid_source(&workspace);
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "run",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("run"),
            source.as_os_str(),
            OsStr::new("--target"),
            OsStr::new("cpu"),
        ],
    );
    configure_fake(&mut command, &workspace, "AERO_LLVM_OPT", &verifier, None);
    command.env("PATH", empty_path(&workspace));

    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        !output.status.success(),
        "rejecting verifier allowed run to succeed: {text}"
    );
    assert!(
        text.contains("LLVM Verification Error:") && text.contains("rejected"),
        "native tool discovery happened before final LLVM verification: {text}"
    );
    assert!(
        !text.contains("clang") && !text.contains("llc"),
        "native tool diagnostics escaped before verifier rejection: {text}"
    );
    assert_run_artifacts_clean(&workspace);

    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("run"),
            source.as_os_str(),
            OsStr::new("--target"),
            OsStr::new("cpu"),
        ],
    );
    command.env("PATH", empty_path(&workspace));
    command.env("AERO_REQUIRE_LLVM_VERIFIER", "false");
    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        !output.status.success(),
        "forced run degraded to InternalOnly with no verifier: {text}"
    );
    assert!(
        text.contains("LLVM Verification Error:") && text.contains("not found"),
        "missing verifier did not fail forced run before native tools: {text}"
    );
    assert!(
        !text.contains("clang") && !text.contains("llc"),
        "native tool discovery ran after missing required verifier: {text}"
    );
    assert_run_artifacts_clean(&workspace);

    let accepting_verifier = write_fake_tool(
        &workspace,
        "accepting-opt",
        "run-accept",
        FakeTool::Opt,
        FakeBehavior::Accept,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("run"),
            source.as_os_str(),
            OsStr::new("--target"),
            OsStr::new("cpu"),
        ],
    );
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &accepting_verifier,
        None,
    );
    command.env("PATH", empty_path(&workspace));
    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        !output.status.success() && text.contains("clang"),
        "accepted LLVM did not reach the expected missing-native-tool failure: {text}"
    );
    assert_run_artifacts_clean(&workspace);
}

fn run_transform(
    workspace: &TestWorkspace,
    command_name: &str,
    input: &Path,
    output: &Path,
    verifier: &Path,
) -> Output {
    let mut command = base_command(workspace);
    let mut args = vec![
        OsString::from(command_name),
        input.as_os_str().to_os_string(),
        OsString::from("-o"),
        output.as_os_str().to_os_string(),
    ];
    if command_name == "quantize" {
        args.extend([OsString::from("--mode"), OsString::from("int8")]);
    }
    add_args(&mut command, &args);
    configure_fake(
        &mut command,
        workspace,
        "AERO_LLVM_OPT",
        verifier,
        Some(output),
    );
    run(command)
}

#[test]
fn graph_opt_and_quantize_verify_input_and_output_before_publication() {
    for command_name in ["graph-opt", "quantize"] {
        let valid = TestWorkspace::new(&format!("{command_name}-valid"));
        let input = valid.write("input.ll", valid_llvm());
        let output_path = valid.path("output.ll");
        let verifier = write_fake_tool(
            &valid,
            "fake-opt",
            "transform",
            FakeTool::Opt,
            FakeBehavior::Accept,
        );
        let output = run_transform(&valid, command_name, &input, &output_path, &verifier);
        assert!(
            output.status.success(),
            "valid {command_name} failed: {}",
            diagnostics(&output)
        );
        assert!(
            output_path.exists(),
            "valid {command_name} did not publish output"
        );
        let published = fs::read(&output_path).expect("read transformed output");
        assert_eq!(
            captured_input(&valid, "transform", 1),
            fs::read(&input).expect("read transform input"),
            "{command_name} did not verify the exact original input first"
        );
        assert_eq!(
            captured_input(&valid, "transform", 2),
            published,
            "{command_name} did not verify the exact final published bytes second"
        );
        let published_text = String::from_utf8(published).expect("transformed LLVM is UTF-8");
        let transform_marker = if command_name == "graph-opt" {
            "; aero.graph_compilation=enabled"
        } else {
            "; aero.quantization.mode=int8"
        };
        assert!(
            published_text.contains(transform_marker),
            "{command_name} output lacked transform marker `{transform_marker}`: {published_text}"
        );
        assert_eq!(
            fs::read_to_string(valid.path("transform.count"))
                .expect("read transform verifier count")
                .trim(),
            "2",
            "{command_name} did not verify both input and final output"
        );

        let invalid_input = TestWorkspace::new(&format!("{command_name}-bad-input"));
        let input = invalid_input.write("input.ll", "INVALID INPUT\n");
        let output_path = invalid_input.path("output.ll");
        let verifier = write_fake_tool(
            &invalid_input,
            "fake-opt",
            "bad-input",
            FakeTool::Opt,
            FakeBehavior::Reject,
        );
        let output = run_transform(
            &invalid_input,
            command_name,
            &input,
            &output_path,
            &verifier,
        );
        assert_llvm_failure(&output, &output_path, "rejected");
        assert_eq!(
            captured_input(&invalid_input, "bad-input", 1),
            b"INVALID INPUT\n",
            "{command_name} did not submit the exact arbitrary input before transformation"
        );
        assert!(
            !invalid_input.path("bad-input-2.stdin").exists(),
            "{command_name} continued to final-output verification after input rejection"
        );
        assert_eq!(
            fs::read_to_string(invalid_input.path("bad-input.count"))
                .expect("read invalid-input count")
                .trim(),
            "1",
            "{command_name} transformed input after verification rejection"
        );

        let invalid_output = TestWorkspace::new(&format!("{command_name}-bad-output"));
        let input = invalid_output.write("input.ll", valid_llvm());
        let output_path = invalid_output.path("output.ll");
        let verifier = write_fake_tool(
            &invalid_output,
            "fake-opt",
            "bad-output",
            FakeTool::Opt,
            FakeBehavior::RejectSecond,
        );
        let output = run_transform(
            &invalid_output,
            command_name,
            &input,
            &output_path,
            &verifier,
        );
        assert_llvm_failure(&output, &output_path, "transformed");
        assert_eq!(
            captured_input(&invalid_output, "bad-output", 1),
            fs::read(&input).expect("read final-rejection input"),
            "{command_name} did not verify original input before transforming"
        );
        let rejected_final = String::from_utf8(captured_input(&invalid_output, "bad-output", 2))
            .expect("rejected final LLVM is UTF-8");
        let transform_marker = if command_name == "graph-opt" {
            "; aero.graph_compilation=enabled"
        } else {
            "; aero.quantization.mode=int8"
        };
        assert!(
            rejected_final.contains(transform_marker),
            "{command_name} did not verify transformed final output: {rejected_final}"
        );
    }
}

#[test]
fn profiler_reports_internal_only_and_rejection_prevents_trace_publication() {
    let workspace = TestWorkspace::new("profile-policy");
    let source = valid_source(&workspace);
    let trace = workspace.path("internal-only-trace.json");
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("profile"),
            source.as_os_str(),
            OsStr::new("-o"),
            trace.as_os_str(),
        ],
    );
    command.env("PATH", empty_path(&workspace));
    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        output.status.success(),
        "InternalOnly profile failed: {text}"
    );
    assert!(
        text.contains("InternalOnly"),
        "stdout omitted InternalOnly: {text}"
    );
    let trace_text = fs::read_to_string(&trace).expect("read InternalOnly trace");
    assert!(
        trace_text.contains("InternalOnly"),
        "trace metadata omitted InternalOnly: {trace_text}"
    );

    let rejected_trace = workspace.path("rejected-trace.json");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "profile",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("profile"),
            source.as_os_str(),
            OsStr::new("-o"),
            rejected_trace.as_os_str(),
        ],
    );
    configure_fake(
        &mut command,
        &workspace,
        "AERO_LLVM_OPT",
        &verifier,
        Some(&rejected_trace),
    );
    assert_llvm_failure(&run(command), &rejected_trace, "rejected");
}

#[test]
fn conformance_is_tool_independent_and_writes_the_complete_requested_report() {
    let workspace = TestWorkspace::new("conformance-policy");
    let report = workspace.path("conformance.json");
    let verifier = write_fake_tool(
        &workspace,
        "fake-opt",
        "must-not-run",
        FakeTool::Opt,
        FakeBehavior::Reject,
    );
    let mut command = base_command(&workspace);
    add_args(
        &mut command,
        [
            OsStr::new("conformance"),
            OsStr::new("-o"),
            report.as_os_str(),
        ],
    );
    configure_fake(&mut command, &workspace, "AERO_LLVM_OPT", &verifier, None);
    let output = run(command);
    let text = diagnostics(&output);
    assert!(
        output.status.success(),
        "tool-independent conformance failed: {text}"
    );
    let report_text = fs::read_to_string(&report).expect("read requested conformance report");
    for required in [
        "\"total_cases\": 3",
        "\"total_mechanized_checks\": 4",
        "basic_arithmetic_well_typed",
        "undefined_variable_rejected",
        "function_call_well_typed",
        "lexer_determinism",
        "parser_determinism",
        "ir_generation_determinism",
        "lowering_pipeline_determinism",
    ] {
        assert!(
            report_text.contains(required),
            "requested conformance report omitted `{required}`: {report_text}"
        );
    }
    assert!(
        !workspace.path("verifier.log").exists(),
        "conformance consulted the external LLVM verifier"
    );
}
