use compiler::{CompilerOptions, LanguageProfile, check_program, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

const CHARACTERIZATION_SOURCE: &str = r#"
fn main() -> int {
    return 91;
}
"#;

const VEC_NEW_SOURCE: &str = r#"
fn main() -> int {
    let mut bytes: Vec<int> = Vec::new();
    return 0;
}
"#;

const EMPTY_VEC_SOURCE: &str = r#"
fn main() -> int {
    let bytes: Vec<int> = vec![];
    return 0;
}
"#;

const PRODUCTION_HARNESS: &str = r#"
#include <stdint.h>
#include <stddef.h>

extern void *aero_alloc(uint64_t size);
extern void *aero_realloc(void *old, uint64_t old_size, uint64_t new_size);
extern void aero_dealloc(void *allocation, uint64_t size);

int main(void) {
    if (aero_alloc(0) != NULL) return 10;
    if (aero_realloc(NULL, 0, 8) != NULL) return 11;
    aero_dealloc(NULL, 0);

    unsigned char *bytes = (unsigned char *)aero_alloc(8);
    if (bytes == NULL) return 12;
    for (uint64_t index = 0; index < 8; ++index) {
        bytes[index] = (unsigned char)(index + 17);
    }

    unsigned char *grown = (unsigned char *)aero_realloc(bytes, 8, 16);
    if (grown == NULL) {
        aero_dealloc(bytes, 8);
        return 13;
    }
    for (uint64_t index = 0; index < 8; ++index) {
        if (grown[index] != (unsigned char)(index + 17)) return 14;
    }
    aero_dealloc(grown, 16);
    return 91;
}
"#;

const TEST_HARNESS: &str = r#"
#include <stdint.h>
#include <stddef.h>

extern void *aero_alloc(uint64_t size);
extern void *aero_realloc(void *old, uint64_t old_size, uint64_t new_size);
extern void aero_dealloc(void *allocation, uint64_t size);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static int success_case(void) {
    if (aero_test_reset(UINT64_MAX) != 1) return 20;
    unsigned char *bytes = (unsigned char *)aero_alloc(8);
    if (bytes == NULL) return 21;
    for (uint64_t index = 0; index < 8; ++index) {
        bytes[index] = (unsigned char)(index + 31);
    }
    unsigned char *grown = (unsigned char *)aero_realloc(bytes, 8, 16);
    if (grown == NULL) return 22;
    for (uint64_t index = 0; index < 8; ++index) {
        if (grown[index] != (unsigned char)(index + 31)) return 23;
    }
    aero_dealloc(grown, 16);
    if (aero_test_alloc_calls() != 1) return 24;
    if (aero_test_realloc_calls() != 1) return 25;
    if (aero_test_dealloc_calls() != 1) return 26;
    if (aero_test_live_allocations() != 0) return 27;
    if (aero_test_size_mismatch_calls() != 0) return 28;
    return 0;
}

static int mismatch_case(void) {
    if (aero_test_reset(UINT64_MAX) != 1) return 30;
    unsigned char *bytes = (unsigned char *)aero_alloc(8);
    if (bytes == NULL) return 31;
    if (aero_realloc(bytes, 7, 16) != NULL) return 32;
    if (aero_test_live_allocations() != 1) return 33;
    aero_dealloc(bytes, 7);
    if (aero_test_live_allocations() != 1) return 34;
    aero_dealloc(bytes, 8);
    if (aero_test_alloc_calls() != 1) return 35;
    if (aero_test_realloc_calls() != 1) return 36;
    if (aero_test_dealloc_calls() != 2) return 37;
    if (aero_test_live_allocations() != 0) return 38;
    if (aero_test_size_mismatch_calls() != 2) return 39;
    return 0;
}

static int failure_case(void) {
    if (aero_test_reset(1) != 1) return 40;
    unsigned char *bytes = (unsigned char *)aero_alloc(8);
    if (bytes == NULL) return 41;
    for (uint64_t index = 0; index < 8; ++index) {
        bytes[index] = (unsigned char)(index + 47);
    }
    if (aero_test_reset(0) != 0) return 42;
    if (aero_realloc(bytes, 8, 16) != NULL) return 43;
    for (uint64_t index = 0; index < 8; ++index) {
        if (bytes[index] != (unsigned char)(index + 47)) return 44;
    }
    if (aero_test_alloc_calls() != 1) return 45;
    if (aero_test_realloc_calls() != 1) return 46;
    if (aero_test_dealloc_calls() != 0) return 47;
    if (aero_test_live_allocations() != 1) return 48;
    if (aero_test_size_mismatch_calls() != 0) return 49;
    aero_dealloc(bytes, 8);
    if (aero_test_dealloc_calls() != 1) return 50;
    if (aero_test_live_allocations() != 0) return 51;
    return 0;
}

int main(void) {
    int result = success_case();
    if (result != 0) return result;
    result = mismatch_case();
    if (result != 0) return result;
    result = failure_case();
    if (result != 0) return result;
    return 91;
}
"#;

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "aero-r1a-runtime-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create R1A test workspace");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write R1A test file");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn options(profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile: profile,
        ..CompilerOptions::default()
    }
}

fn assert_exit_91(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(91),
        "{label} failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_c_harness(
    workspace: &TestWorkspace,
    runtime: &Path,
    harness_name: &str,
    harness: &str,
) -> Output {
    let harness_path = workspace.write(harness_name, harness);
    let executable = workspace.path().join(if cfg!(windows) {
        format!("{harness_name}.exe")
    } else {
        harness_name.to_string()
    });
    let compile = Command::new("clang")
        .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror"])
        .arg(runtime)
        .arg(&harness_path)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for R1A runtime harness");
    assert!(
        compile.status.success(),
        "compile {harness_name} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    Command::new(&executable)
        .output()
        .expect("execute R1A runtime harness")
}

#[test]
fn accepted_profiles_source_boundaries_and_native_exit_are_frozen_before_r1a() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            format!("{:x}", md5::compute(llvm.as_bytes())),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R1A"
        );
    }

    assert_eq!(
        check_program(VEC_NEW_SOURCE, CompilerOptions::default())
            .expect_err("Vec::new must remain absent before R1C"),
        "Semantic Analysis Error: enum `Vec` has no unique admitted definition"
    );
    assert_eq!(
        check_program(EMPTY_VEC_SOURCE, CompilerOptions::default())
            .expect_err("vec![] must remain a rejected fixed-array literal"),
        "IR Generation Error: empty array literals have no admitted logical element type"
    );

    let workspace = TestWorkspace::new();
    let source = workspace.write("characterization.aero", CHARACTERIZATION_SOURCE);
    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("run")
        .arg(&source)
        .current_dir(workspace.path())
        .output()
        .expect("run accepted native characterization");
    assert_exit_91(&output, "accepted native characterization");

    let run_root = workspace.path().join("target/aero-run");
    if run_root.exists() {
        assert_eq!(
            fs::read_dir(&run_root)
                .expect("read native run directory")
                .count(),
            0,
            "native run left a unique artifact directory behind"
        );
    }
}

#[test]
fn r1a_runtime_and_driver_boundary_executes_with_deterministic_failure() {
    let root = repository_root();
    let production_runtime_path = root.join("src/compiler/runtime/aero_runtime.c");
    let test_runtime_path = root.join("src/compiler/runtime/aero_test_runtime.c");
    assert!(
        production_runtime_path.is_file(),
        "R1A red: the production allocator runtime is absent"
    );
    assert!(
        test_runtime_path.is_file(),
        "R1A red: the deterministic allocator test runtime is absent"
    );

    let main =
        fs::read_to_string(root.join("src/compiler/src/main.rs")).expect("read compiler driver");
    assert!(
        !main.contains("mod runtime_link;"),
        "R1A red: runtime linking must remain in the canonical CLI driver"
    );
    for anchor in [
        "const PRODUCTION_RUNTIME_SOURCE",
        "include_bytes!(\"../runtime/aero_runtime.c\")",
        "fn compile_production_runtime",
        "compile_production_runtime",
        "runtime_source_file",
        "runtime_obj_file",
        "-std=c11",
        "-O2",
        "-c",
    ] {
        assert!(main.contains(anchor), "compiler driver omitted `{anchor}`");
    }
    assert_eq!(
        main.matches("&runtime_obj_path").count(),
        2,
        "both CPU link paths must consume the compiled runtime object"
    );
    let production_runtime =
        fs::read_to_string(&production_runtime_path).expect("read production allocator runtime");
    let test_runtime = fs::read_to_string(&test_runtime_path).expect("read test allocator runtime");
    for symbol in ["aero_alloc", "aero_realloc", "aero_dealloc"] {
        assert!(
            production_runtime.contains(symbol),
            "production runtime omitted `{symbol}`"
        );
        assert!(
            test_runtime.contains(symbol),
            "test runtime omitted `{symbol}`"
        );
    }
    for symbol in [
        "aero_test_reset",
        "aero_test_alloc_calls",
        "aero_test_realloc_calls",
        "aero_test_dealloc_calls",
        "aero_test_live_allocations",
        "aero_test_size_mismatch_calls",
    ] {
        assert!(
            test_runtime.contains(symbol),
            "test runtime omitted `{symbol}`"
        );
        assert!(
            !production_runtime.contains(symbol),
            "production runtime exposed test symbol `{symbol}`"
        );
    }

    let workspace = TestWorkspace::new();
    assert_exit_91(
        &compile_c_harness(
            &workspace,
            &production_runtime_path,
            "production-runtime-harness.c",
            PRODUCTION_HARNESS,
        ),
        "production allocator runtime harness",
    );
    assert_exit_91(
        &compile_c_harness(
            &workspace,
            &test_runtime_path,
            "test-runtime-harness.c",
            TEST_HARNESS,
        ),
        "deterministic allocator runtime harness",
    );
}
