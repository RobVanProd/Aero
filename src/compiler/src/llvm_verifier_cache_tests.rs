use super::{BuildConfig, compile_to_llvm_ir_with_optimizer};
use crate::performance_optimizations::PerformanceOptimizer;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

static ENVIRONMENT_LOCK: Mutex<()> = Mutex::new(());
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
        let lock = ENVIRONMENT_LOCK.lock().expect("lock verifier environment");
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
