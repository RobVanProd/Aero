use super::{
    BuildConfig, LLVM_VERIFIER_TEST_ENVIRONMENT_LOCK as ENVIRONMENT_LOCK,
    compile_to_llvm_ir_with_optimizer,
};
use compiler::PerformanceOptimizer;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-llvm-cache-unit-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create cache test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, contents).expect("write cache test fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct EnvironmentGuard {
    _lock: MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<OsString>)>,
}

impl EnvironmentGuard {
    fn acquire() -> Self {
        let lock = ENVIRONMENT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let saved = [
            "AERO_LLVM_OPT",
            "AERO_LLVM_AS",
            "AERO_REQUIRE_LLVM_VERIFIER",
            "AERO_TEST_CACHE_VERIFIER_INPUT",
            "AERO_TEST_CACHE_OUTPUT",
            "PATH",
        ]
        .into_iter()
        .map(|key| (key, std::env::var_os(key)))
        .collect();
        Self { _lock: lock, saved }
    }

    fn set(key: &str, value: impl AsRef<std::ffi::OsStr>) {
        // SAFETY: this unit test serializes all verifier-environment mutations.
        unsafe { std::env::set_var(key, value) };
    }

    fn remove(key: &str) {
        // SAFETY: this unit test serializes all verifier-environment mutations.
        unsafe { std::env::remove_var(key) };
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..) {
            // SAFETY: the environment lock is held until restoration completes.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

fn write_rejecting_opt(workspace: &TestWorkspace) -> PathBuf {
    #[cfg(windows)]
    let (name, script) = (
        "rejecting-opt.cmd",
        r#"@echo off
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
if not "%~1"=="-passes=verify" goto bad_args
if not "%~2"=="-disable-output" goto bad_args
if not "%~3"=="-" goto bad_args
if not "%~4"=="" goto bad_args
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -Command "$inputStream=[Console]::OpenStandardInput(); $outputStream=[IO.File]::Create($env:AERO_TEST_CACHE_VERIFIER_INPUT); try { $inputStream.CopyTo($outputStream) } finally { $outputStream.Dispose() }"
if errorlevel 1 exit /b 93
for %%A in ("%AERO_TEST_CACHE_VERIFIER_INPUT%") do if %%~zA LEQ 0 (
  echo cached verifier received empty stdin 1>&2
  exit /b 92
)
if defined AERO_TEST_CACHE_OUTPUT if exist "%AERO_TEST_CACHE_OUTPUT%" (
  echo cached verifier observed output before verification 1>&2
  exit /b 91
)
echo cached verifier rejection 1>&2
exit /b 19
:bad_args
echo cached opt received invalid verifier arguments: %* 1>&2
exit /b 90
"#,
    );
    #[cfg(not(windows))]
    let (name, script) = (
        "rejecting-opt",
        r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
if [ "$#" -ne 3 ] || [ "$1" != "-passes=verify" ] || [ "$2" != "-disable-output" ] || [ "$3" != "-" ]; then
  echo "cached opt received invalid verifier arguments: $*" >&2
  exit 90
fi
/bin/cat > "${AERO_TEST_CACHE_VERIFIER_INPUT}"
if [ ! -s "${AERO_TEST_CACHE_VERIFIER_INPUT}" ]; then
  echo "cached verifier received empty stdin" >&2
  exit 92
fi
if [ -n "${AERO_TEST_CACHE_OUTPUT:-}" ] && [ -e "$AERO_TEST_CACHE_OUTPUT" ]; then
  echo "cached verifier observed output before verification" >&2
  exit 91
fi
echo "cached verifier rejection" >&2
exit 19
"#,
    );

    let path = workspace.write(name, script);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("stat rejecting verifier")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make rejecting verifier executable");
    }
    path
}

fn write_accepting_opt(workspace: &TestWorkspace) -> PathBuf {
    #[cfg(windows)]
    let (name, script) = (
        "accepting-opt.cmd",
        r#"@echo off
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
if not "%~1"=="-passes=verify" goto bad_args
if not "%~2"=="-disable-output" goto bad_args
if not "%~3"=="-" goto bad_args
if not "%~4"=="" goto bad_args
"%SystemRoot%\System32\more.com" >nul
exit /b 0
:bad_args
echo accepting opt received invalid verifier arguments: %* 1>&2
exit /b 90
"#,
    );
    #[cfg(not(windows))]
    let (name, script) = (
        "accepting-opt",
        r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
if [ "$#" -ne 3 ] || [ "$1" != "-passes=verify" ] || [ "$2" != "-disable-output" ] || [ "$3" != "-" ]; then
  echo "accepting opt received invalid verifier arguments: $*" >&2
  exit 90
fi
/bin/cat >/dev/null
exit 0
"#,
    );

    let path = workspace.write(name, script);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("stat accepting verifier")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make accepting verifier executable");
    }
    path
}

fn source_cache_key(source: &str, config: &BuildConfig) -> String {
    format!(
        "{:x}",
        md5::compute(format!(
            "{}::target={}::gpu={}",
            source,
            config.target.as_str(),
            config.gpu_arch_or_default()
        ))
    )
}

fn call_with_optimizer(
    source: &str,
    input: &Path,
    output: &Path,
    config: &BuildConfig,
    optimizer: &mut PerformanceOptimizer,
) -> Result<(), String> {
    let input = input.to_string_lossy().into_owned();
    let output = output.to_string_lossy().into_owned();
    compile_to_llvm_ir_with_optimizer(source, &output, &input, config, optimizer)
}

#[test]
fn cache_hit_is_reverified_and_missing_verifier_forces_a_fresh_checked_rebuild() {
    let _environment = EnvironmentGuard::acquire();
    let workspace = TestWorkspace::new();
    let source = "let value = 10 + 5;\nreturn value;\n";
    let input = workspace.write("main.aero", source);
    let config = BuildConfig::default();
    let source_hash = source_cache_key(source, &config);
    let cached_llvm =
        "; AERO_TEST_CACHE_HIT_SENTINEL\ndefine i32 @cached_main() {\nentry:\n  ret i32 73\n}\n";

    let verifier_input = workspace.path("verifier-input.ll");
    let rejecting_opt = write_rejecting_opt(&workspace);
    EnvironmentGuard::set("AERO_LLVM_OPT", &rejecting_opt);
    EnvironmentGuard::remove("AERO_LLVM_AS");
    EnvironmentGuard::remove("AERO_REQUIRE_LLVM_VERIFIER");
    EnvironmentGuard::set("AERO_TEST_CACHE_VERIFIER_INPUT", &verifier_input);

    let mut hit_optimizer = PerformanceOptimizer::new();
    hit_optimizer
        .get_compilation_cache()
        .cache_llvm(source_hash.clone(), cached_llvm.to_string());
    let rejected_output = workspace.path("cached-reject.ll");
    EnvironmentGuard::set("AERO_TEST_CACHE_OUTPUT", &rejected_output);
    let error = call_with_optimizer(
        source,
        &input,
        &rejected_output,
        &config,
        &mut hit_optimizer,
    )
    .expect_err("rejecting verifier accepted cached final LLVM");
    assert!(
        error.contains("LLVM Verification Error:") && error.contains("cached verifier rejection"),
        "cache-hit rejection lost the stable verifier diagnostic: {error}"
    );
    assert!(
        !rejected_output.exists(),
        "rejected cache hit was published"
    );
    assert_eq!(
        fs::read_to_string(&verifier_input).expect("read cache-hit verifier input"),
        cached_llvm,
        "external verifier did not receive the exact cached final LLVM bytes"
    );
    let (hits, _, _) = hit_optimizer.get_compilation_cache().get_cache_stats();
    assert_eq!(hits, 1, "cache-hit path did not record exactly one hit");

    let fresh_verifier_input = workspace.path("fresh-verifier-input.ll");
    let fresh_output = workspace.path("fresh-reject.ll");
    EnvironmentGuard::set("AERO_TEST_CACHE_VERIFIER_INPUT", &fresh_verifier_input);
    EnvironmentGuard::set("AERO_TEST_CACHE_OUTPUT", &fresh_output);
    let mut fresh_optimizer = PerformanceOptimizer::new();
    let error = call_with_optimizer(source, &input, &fresh_output, &config, &mut fresh_optimizer)
        .expect_err("rejecting verifier accepted fresh final LLVM");
    assert!(
        error.contains("LLVM Verification Error:") && error.contains("cached verifier rejection"),
        "fresh-output rejection lost the stable verifier diagnostic: {error}"
    );
    assert!(!fresh_output.exists(), "rejected fresh LLVM was published");
    let rejected_fresh = fs::read_to_string(&fresh_verifier_input)
        .expect("read rejected fresh final LLVM verifier input");
    assert!(
        rejected_fresh.contains("; aero.graph_compilation=enabled")
            && rejected_fresh.contains("target triple ="),
        "verifier did not receive post-transform/post-retarget fresh LLVM: {rejected_fresh}"
    );
    assert!(
        fresh_optimizer
            .get_compilation_cache()
            .get_cached_llvm(&source_hash)
            .is_none(),
        "rejected fresh final LLVM was cached"
    );

    EnvironmentGuard::remove("AERO_LLVM_OPT");
    EnvironmentGuard::remove("AERO_LLVM_AS");
    EnvironmentGuard::remove("AERO_TEST_CACHE_OUTPUT");
    let empty_path = workspace.path("empty-path");
    fs::create_dir_all(&empty_path).expect("create verifier-free PATH");
    EnvironmentGuard::set("PATH", &empty_path);

    let mut missing_optimizer = PerformanceOptimizer::new();
    missing_optimizer
        .get_compilation_cache()
        .cache_llvm(source_hash, cached_llvm.to_string());
    let rebuilt_output = workspace.path("internal-only-rebuild.ll");
    call_with_optimizer(
        source,
        &input,
        &rebuilt_output,
        &config,
        &mut missing_optimizer,
    )
    .expect("missing optional verifier should bypass cache and rebuild with internal checks");
    let rebuilt = fs::read_to_string(&rebuilt_output).expect("read fresh checked rebuild");
    assert!(
        !rebuilt.contains("AERO_TEST_CACHE_HIT_SENTINEL") && rebuilt.contains("define i32 @main"),
        "missing verifier published cached bytes instead of a fresh checked rebuild: {rebuilt}"
    );
    let (hits, _, _) = missing_optimizer.get_compilation_cache().get_cache_stats();
    assert_eq!(
        hits, 1,
        "missing-verifier path did not observe the seeded hit"
    );
}

fn configure_accepting_verifier(workspace: &TestWorkspace) -> EnvironmentGuard {
    let environment = EnvironmentGuard::acquire();
    let accepting_opt = write_accepting_opt(workspace);
    EnvironmentGuard::set("AERO_LLVM_OPT", &accepting_opt);
    EnvironmentGuard::remove("AERO_LLVM_AS");
    EnvironmentGuard::remove("AERO_REQUIRE_LLVM_VERIFIER");
    EnvironmentGuard::remove("AERO_TEST_CACHE_OUTPUT");
    environment
}

fn module_source() -> &'static str {
    "mod helper; fn main() { helper(); }"
}

fn module_cache_stats(optimizer: &mut PerformanceOptimizer) -> (usize, usize) {
    let (hits, misses, _) = optimizer.get_compilation_cache().get_cache_stats();
    (hits, misses)
}

fn push_cache_frame(bytes: &mut Vec<u8>, label: &str, payload: &[u8]) {
    bytes.extend_from_slice(label.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    bytes.extend_from_slice(payload);
}

fn exact_module_cache_key_known_vector() -> String {
    let mut bytes = b"AERO_MODULE_CACHE_V1\0".to_vec();
    push_cache_frame(&mut bytes, "root", module_source().as_bytes());
    push_cache_frame(&mut bytes, "target", b"cpu");
    push_cache_frame(&mut bytes, "gpu", b"x86_64");
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    push_cache_frame(&mut bytes, "name", b"helper");
    push_cache_frame(&mut bytes, "candidate", b"helper.aero");
    push_cache_frame(&mut bytes, "source", b"fn helper() {}\n");
    format!("{:x}", md5::compute(bytes))
}

#[test]
fn legacy_no_module_identity_remains_an_actual_verified_cache_hit() {
    let workspace = TestWorkspace::new();
    let source = "fn main() {}";
    let input = workspace.write("main.aero", source);
    let output = workspace.path("cached.ll");
    let config = BuildConfig::default();
    let cached_llvm =
        "; AERO_LEGACY_NO_MODULE_CACHE\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n";
    let mut optimizer = PerformanceOptimizer::new();
    optimizer
        .get_compilation_cache()
        .cache_llvm(source_cache_key(source, &config), cached_llvm.to_string());

    let environment = configure_accepting_verifier(&workspace);
    let result = call_with_optimizer(source, &input, &output, &config, &mut optimizer);
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    result.expect("verified legacy no-module entry did not hit");
    assert_eq!(
        fs::read_to_string(&output).expect("read verified legacy cache output"),
        cached_llvm
    );
    assert_eq!(stats, (1, 0), "legacy no-module key changed");
}

#[test]
fn module_cache_identity_matches_frozen_v1_known_vector() {
    let workspace = TestWorkspace::new();
    let root_source = module_source();
    let input = workspace.write("main.aero", root_source);
    workspace.write("helper.aero", "fn helper() {}\n");
    let output = workspace.path("known-vector.ll");
    let config = BuildConfig::default();
    let cached_llvm =
        "; AERO_MODULE_CACHE_V1_KNOWN_VECTOR\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n";
    let expected_key = "84dfd6b8372f03e679696f9f3a8ab4a9";
    let mut optimizer = PerformanceOptimizer::new();
    optimizer
        .get_compilation_cache()
        .cache_llvm(expected_key.to_string(), cached_llvm.to_string());

    let environment = configure_accepting_verifier(&workspace);
    let result = call_with_optimizer(root_source, &input, &output, &config, &mut optimizer);
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    assert_eq!(
        exact_module_cache_key_known_vector(),
        expected_key,
        "test vector no longer represents the frozen V1 byte stream"
    );
    result.expect("frozen module-bearing cache key did not hit");
    assert_eq!(
        fs::read_to_string(&output).expect("read known-vector cache output"),
        cached_llvm,
        "production module cache identity does not match the frozen V1 stream"
    );
    assert_eq!(stats, (1, 0), "known-vector entry was not a cache hit");
}

#[test]
fn exact_module_source_byte_change_misses_final_llvm_cache() {
    let workspace = TestWorkspace::new();
    let root_source = module_source();
    let input = workspace.write("main.aero", root_source);
    let helper = workspace.write("helper.aero", "fn helper() {}\n");
    let config = BuildConfig::default();
    let mut optimizer = PerformanceOptimizer::new();

    let environment = configure_accepting_verifier(&workspace);
    let first = call_with_optimizer(
        root_source,
        &input,
        &workspace.path("first.ll"),
        &config,
        &mut optimizer,
    );
    fs::write(&helper, "fn helper() {}\n\n").expect("change exact module bytes");
    let second = call_with_optimizer(
        root_source,
        &input,
        &workspace.path("second.ll"),
        &config,
        &mut optimizer,
    );
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    first.expect("initial module compilation failed");
    second.expect("byte-mutated module compilation failed");
    assert_eq!(stats, (0, 2), "exact module byte change reused cache");
}

#[test]
fn same_module_bytes_moved_between_resolver_candidates_miss_cache() {
    let workspace = TestWorkspace::new();
    let root_source = module_source();
    let input = workspace.write("main.aero", root_source);
    let file_candidate = workspace.write("helper.aero", "fn helper() {}\n");
    let config = BuildConfig::default();
    let mut optimizer = PerformanceOptimizer::new();

    let environment = configure_accepting_verifier(&workspace);
    let first = call_with_optimizer(
        root_source,
        &input,
        &workspace.path("file-layout.ll"),
        &config,
        &mut optimizer,
    );
    let directory_candidate = workspace.path("helper").join("mod.aero");
    fs::create_dir_all(directory_candidate.parent().expect("module directory"))
        .expect("create directory-layout module directory");
    fs::rename(&file_candidate, &directory_candidate).expect("move module between candidates");
    let second = call_with_optimizer(
        root_source,
        &input,
        &workspace.path("directory-layout.ll"),
        &config,
        &mut optimizer,
    );
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    first.expect("file-layout module compilation failed");
    second.expect("directory-layout module compilation failed");
    assert_eq!(stats, (0, 2), "resolver candidate move reused cache");
}

#[test]
fn identical_module_inputs_across_entry_roots_share_cache_identity() {
    let verifier_workspace = TestWorkspace::new();
    let first_workspace = TestWorkspace::new();
    let second_workspace = TestWorkspace::new();
    let root_source = module_source();
    let first_input = first_workspace.write("main.aero", root_source);
    first_workspace.write("helper.aero", "fn helper() {}\n");
    let second_input = second_workspace.write("main.aero", root_source);
    second_workspace.write("helper.aero", "fn helper() {}\n");
    let config = BuildConfig::default();
    let mut optimizer = PerformanceOptimizer::new();

    let environment = configure_accepting_verifier(&verifier_workspace);
    let first = call_with_optimizer(
        root_source,
        &first_input,
        &first_workspace.path("first.ll"),
        &config,
        &mut optimizer,
    );
    let second = call_with_optimizer(
        root_source,
        &second_input,
        &second_workspace.path("second.ll"),
        &config,
        &mut optimizer,
    );
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    first.expect("first-root module compilation failed");
    second.expect("second-root module compilation failed");
    assert_eq!(
        stats,
        (1, 1),
        "entry directory or canonical host path entered module cache identity"
    );
}

#[test]
fn deleted_module_fails_before_cache_lookup_or_output() {
    let workspace = TestWorkspace::new();
    let root_source = module_source();
    let input = workspace.write("main.aero", root_source);
    let helper = workspace.write("helper.aero", "fn helper() {}\n");
    let config = BuildConfig::default();
    let mut optimizer = PerformanceOptimizer::new();

    let environment = configure_accepting_verifier(&workspace);
    let initial = call_with_optimizer(
        root_source,
        &input,
        &workspace.path("initial.ll"),
        &config,
        &mut optimizer,
    );
    fs::remove_file(&helper).expect("delete direct module");
    let output = workspace.path("deleted-module.ll");
    let deleted = call_with_optimizer(root_source, &input, &output, &config, &mut optimizer);
    let stats = module_cache_stats(&mut optimizer);
    drop(environment);

    initial.expect("initial module compilation failed");
    let error = deleted.expect_err("deleted module reused cached final LLVM");
    assert!(
        error.contains("Module resolution failed for `helper`")
            && error.contains("Cannot find module `helper`"),
        "deleted module lost shared diagnostic: {error}"
    );
    assert!(!output.exists(), "deleted module published cached LLVM");
    assert_eq!(stats, (0, 1), "deleted module reached cache lookup");
}
