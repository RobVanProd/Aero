use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PROFILE_NAME: &str = "exact-i32-byte-buffer-v0";

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

const LEGACY_BYTE_BUFFER_RECORD_SOURCE: &str = r#"
struct ByteBuffer {
    value: int,
}

fn main() -> int {
    let record: ByteBuffer = ByteBuffer { value: 91 };
    return record.value;
}
"#;

const SOURCE_BYTE_BUFFER_PRODUCT: &str =
    include_str!("../../../examples/owned_byte_buffer_v0/source_owned_byte_buffer.aero");

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

fn options(language_profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile,
        ..CompilerOptions::default()
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository_root().join("target"))
            .join("r1c-source-byte-buffer-tests");
        let root = parent.join(format!("{label}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&root).expect("create R1C native workspace");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write R1C native artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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

fn clang_link(
    label: &str,
    workspace: &TestWorkspace,
    llvm: &Path,
    runtime: &Path,
    harness: Option<&Path>,
    optimization: &str,
) -> PathBuf {
    let executable = workspace.path().join(if cfg!(windows) {
        format!("{label}-{optimization}.exe")
    } else {
        format!("{label}-{optimization}")
    });
    let mut command = Command::new("clang");
    command.args([
        "-std=c11",
        optimization,
        "-Wall",
        "-Wextra",
        "-Werror",
        "-Wno-override-module",
    ]);
    command.arg(llvm).arg(runtime);
    if let Some(harness) = harness {
        command.arg(harness);
    }
    let output = command
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for R1C native fixture");
    assert!(
        output.status.success(),
        "link {label} {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn compile_source(source: &str) -> String {
    compile_program(source, options(LanguageProfile::ExactI32ByteBufferV0))
        .expect("R1C source fixture compiles")
}

fn run_production_runtime_product(optimization: &str) {
    let workspace = TestWorkspace::new("product");
    let llvm = compile_source(SOURCE_BYTE_BUFFER_PRODUCT);
    verify_llvm_module(&llvm, LlvmVerificationMode::Required)
        .expect("R1C product passes required LLVM verification");
    let llvm_path = workspace.write("product.ll", &llvm);
    let executable = clang_link(
        "product",
        &workspace,
        &llvm_path,
        &repository_root().join("src/compiler/runtime/aero_runtime.c"),
        None,
        optimization,
    );
    let output = Command::new(&executable)
        .current_dir(workspace.path())
        .output()
        .expect("execute R1C production-runtime product");
    assert_exit_91(&output, &format!("R1C product {optimization}"));
    assert!(output.stdout.is_empty(), "R1C product emitted stdout");
    assert!(output.stderr.is_empty(), "R1C product emitted stderr");
}

#[derive(Clone, Copy)]
struct RuntimeExpectations {
    result: i32,
    allocations: u64,
    reallocations: u64,
    deallocations: u64,
}

fn run_test_runtime_case(
    label: &str,
    source: &str,
    fail_after: u64,
    expected: RuntimeExpectations,
) {
    let workspace = TestWorkspace::new(label);
    let llvm = compile_source(source);
    let renamed = llvm.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(llvm, renamed, "R1C source fixture omitted main");
    verify_llvm_module(&renamed, LlvmVerificationMode::Required)
        .expect("renamed R1C native fixture verifies");
    let llvm_path = workspace.write("program.ll", &renamed);
    let harness = format!(
        r#"
#include <stdint.h>

extern int aero_program_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

int main(void) {{
    if (aero_test_reset(UINT64_C({fail_after})) != 1) return 70;
    if (aero_program_main() != {result}) return 71;
    if (aero_test_alloc_calls() != UINT64_C({allocations})) return 72;
    if (aero_test_realloc_calls() != UINT64_C({reallocations})) return 73;
    if (aero_test_dealloc_calls() != UINT64_C({deallocations})) return 74;
    if (aero_test_live_allocations() != 0) return 75;
    if (aero_test_size_mismatch_calls() != 0) return 76;
    return 91;
}}
"#,
        result = expected.result,
        allocations = expected.allocations,
        reallocations = expected.reallocations,
        deallocations = expected.deallocations,
    );
    let harness_path = workspace.write("harness.c", &harness);
    let executable = clang_link(
        label,
        &workspace,
        &llvm_path,
        &repository_root().join("src/compiler/runtime/aero_test_runtime.c"),
        Some(&harness_path),
        "-O2",
    );
    let output = Command::new(&executable)
        .current_dir(workspace.path())
        .output()
        .expect("execute R1C test-runtime case");
    assert_exit_91(&output, label);
}

#[test]
fn accepted_profiles_and_owned_byte_substrates_are_frozen_before_r1c() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            md5_hex(llvm.as_bytes()),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R1C"
        );
        for forbidden in [
            "%aero.byte_buffer",
            "@aero_alloc",
            "@aero_realloc",
            "@aero_dealloc",
        ] {
            assert!(
                !llvm.contains(forbidden),
                "ordinary {profile:?} source unexpectedly emitted `{forbidden}`"
            );
        }
    }

    assert_eq!(
        check_program(VEC_NEW_SOURCE, CompilerOptions::default())
            .expect_err("Vec::new must remain absent during R1C"),
        "Semantic Analysis Error: enum `Vec` has no unique admitted definition"
    );
    assert_eq!(
        check_program(EMPTY_VEC_SOURCE, CompilerOptions::default())
            .expect_err("vec![] must remain a rejected fixed-array literal"),
        "IR Generation Error: empty array literals have no admitted logical element type"
    );
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::ExactI32RecordResultV0,
    ] {
        check_program(LEGACY_BYTE_BUFFER_RECORD_SOURCE, options(profile)).unwrap_or_else(|error| {
            panic!("{profile:?} user record named ByteBuffer drifted: {error}")
        });
    }

    let root = repository_root();
    for (relative, expected) in [
        (
            "src/compiler/runtime/aero_runtime.c",
            "993af1665a4e93249035b149dfc643be",
        ),
        (
            "src/compiler/runtime/aero_test_runtime.c",
            "5f1db08f29355e78a1dda31747ec7055",
        ),
        ("src/compiler/src/ir.rs", "e8b0f71c86d3345aa0c6756f609b0f1d"),
        (
            "src/compiler/src/ir_verifier.rs",
            "ecd6759c4ca5c96e0d3bdad4dc39c5a2",
        ),
    ] {
        assert_eq!(
            md5_hex(
                &fs::read(root.join(relative))
                    .unwrap_or_else(|error| panic!("read {relative}: {error}"))
            ),
            expected,
            "accepted authority `{relative}` drifted during R1C"
        );
    }

    for relative in ["src/compiler/src/ast.rs", "src/compiler/src/parser.rs"] {
        assert!(
            !read(&root, relative).contains("ByteBuffer"),
            "R1C must not add parser/AST syntax through `{relative}`"
        );
    }
}

#[test]
fn source_byte_buffer_profile_is_selector_red_first() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("R1C red: exact-i32-byte-buffer-v0 selector is absent");
    assert_eq!(profile.as_str(), PROFILE_NAME);

    check_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must pass the public check route");
    let first = compile_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must compile");
    let second = compile_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must compile deterministically");
    assert_eq!(first, second, "R1C LLVM must be deterministic");
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
    ] {
        assert!(first.contains(anchor), "R1C LLVM omitted `{anchor}`");
    }
    for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
        assert!(
            !first.contains(forbidden),
            "R1C exact product leaked forbidden LLVM `{forbidden}`"
        );
    }

    let root = repository_root();
    let profile_source = read(&root, "src/compiler/src/language_profile.rs");
    let types = read(&root, "src/compiler/src/types.rs");
    let semantics = read(&root, "src/compiler/src/semantic_analyzer.rs");
    let resolved = read(&root, "src/compiler/src/resolved_profile_shape.rs");
    let generator = read(&root, "src/compiler/src/ir_generator.rs");
    let library = read(&root, "src/compiler/src/lib.rs");
    let cli = read(&root, "src/compiler/src/main.rs");

    for anchor in [
        "EXACT_I32_BYTE_BUFFER_V0_NAME",
        "ExactI32ByteBufferV0",
        "exact-i32-byte-buffer-v0",
    ] {
        assert!(
            profile_source.contains(anchor),
            "R1C profile authority omitted `{anchor}`"
        );
    }
    assert!(types.contains("ByteBuffer,"), "R1C semantic type is absent");
    assert!(
        root.join("src/compiler/src/byte_buffer_source_contract.rs")
            .is_file(),
        "R1C shared source contract is absent"
    );
    for (source, anchor) in [
        (&semantics, "new_with_byte_buffer_source"),
        (&resolved, "ResolvedProfileCallArgumentKind"),
        (&generator, "new_with_byte_buffer_source"),
        (&generator, "CheckedByteBufferNew"),
        (&generator, "CheckedByteBufferDrop"),
        (&library, "mod byte_buffer_source_contract;"),
        (&cli, "exact-i32-byte-buffer-v0"),
    ] {
        assert!(source.contains(anchor), "R1C authority omitted `{anchor}`");
    }
}

#[test]
fn source_byte_buffer_product_verifies_and_runs_at_o0_and_o2() {
    run_production_runtime_product("-O0");
    run_production_runtime_product("-O2");

    let workspace = TestWorkspace::new("public-run");
    let source = workspace.write("product.aero", SOURCE_BYTE_BUFFER_PRODUCT);
    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .args([
            "run",
            source.to_str().expect("source path is UTF-8"),
            "--language-profile",
            PROFILE_NAME,
        ])
        .current_dir(workspace.path())
        .output()
        .expect("execute public R1C product");
    assert_exit_91(&output, "public R1C product");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1,
        "public R1C product omitted its exact exit report: {stdout}"
    );
    assert!(
        !stdout
            .lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:")),
        "public R1C product reported application output: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "public R1C product emitted stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn source_byte_buffer_profile_rejects_accelerators_before_artifact_creation() {
    let workspace = TestWorkspace::new("accelerator-boundary");
    let source = workspace.write("product.aero", SOURCE_BYTE_BUFFER_PRODUCT);
    for target in ["rocm", "cuda"] {
        let output_path = workspace.path().join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                source.to_str().expect("source path is UTF-8"),
                "-o",
                output_path.to_str().expect("output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(workspace.path())
            .output()
            .expect("execute R1C accelerator build rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("requires --target cpu without --gpu"),
            "{target} build stopped at the wrong boundary: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output_path.exists(),
            "{target} rejection created a requested artifact"
        );
    }

    let run = Command::new(env!("CARGO_BIN_EXE_aero"))
        .args([
            "run",
            source.to_str().expect("source path is UTF-8"),
            "--gpu",
            "sm_90",
            "--language-profile",
            PROFILE_NAME,
        ])
        .current_dir(workspace.path())
        .output()
        .expect("execute R1C accelerator run rejection");
    assert_eq!(run.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("requires --target cpu without --gpu"),
        "GPU run stopped at the wrong boundary: {:?}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !workspace.path().join("target").exists(),
        "accelerator rejection created a run-artifact tree"
    );
}

#[test]
fn source_byte_buffer_rejections_are_deterministic_across_source_and_file_routes() {
    let cases = [
        (
            "unannotated-owner",
            "fn main() -> int { let bytes = bytes_new(); return 0; }",
            "requires the explicit `ByteBuffer` annotation",
        ),
        (
            "uninitialized-owner",
            "fn main() -> int { let bytes: ByteBuffer; return 0; }",
            "must be initialized at declaration",
        ),
        (
            "branch-owner",
            "fn main() -> int { if 1 < 2 { let bytes: ByteBuffer = bytes_new(); } return 0; }",
            "outside control-flow topology",
        ),
        (
            "loop-owner",
            "fn main() -> int { while 1 < 2 { let bytes: ByteBuffer = bytes_new(); break; } return 0; }",
            "outside control-flow topology",
        ),
        (
            "use-after-move",
            "fn main() -> int { let first: ByteBuffer = bytes_new(); let second: ByteBuffer = first; return bytes_len(&first); }",
            "owner `first` is not live",
        ),
        (
            "immutable-push",
            "fn main() -> int { let bytes: ByteBuffer = bytes_new(); let pushed: Result<int, int> = bytes_push(&mut bytes, 1); return 0; }",
            "requires mutable owner `bytes`",
        ),
        (
            "wrong-borrow-topology",
            "fn main() -> int { let bytes: ByteBuffer = bytes_new(); return bytes_len(bytes); }",
            "requires exactly an immediate `&ByteBuffer` identifier",
        ),
        (
            "wrong-arity",
            "fn main() -> int { let bytes: ByteBuffer = bytes_new(); return bytes_get(&bytes); }",
            "requires exactly `&ByteBuffer, int` arguments",
        ),
        (
            "wrong-scalar",
            "fn main() -> int { let mut bytes: ByteBuffer = bytes_new(); let pushed: Result<int, int> = bytes_push(&mut bytes, 1 < 2); return 0; }",
            "scalar argument has type bool, expected int",
        ),
        (
            "discarded-result",
            "fn main() -> int { let mut bytes: ByteBuffer = bytes_new(); bytes_push(&mut bytes, 1); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "unannotated-result",
            "fn main() -> int { let bytes: ByteBuffer = bytes_new(); let found = bytes_get(&bytes, 0); return 0; }",
            "requires an explicit `Result<int, int>` context",
        ),
        (
            "reserved-name",
            "fn bytes_new() -> int { return 0; } fn main() -> int { return 0; }",
            "intrinsic name `bytes_new` is reserved",
        ),
        (
            "reserved-source-type",
            LEGACY_BYTE_BUFFER_RECORD_SOURCE,
            "source type name `ByteBuffer` is reserved",
        ),
        (
            "reserved-source-enum",
            "enum ByteBuffer { Empty } fn main() -> int { return 0; }",
            "source type name `ByteBuffer` is reserved",
        ),
        (
            "nested-struct-owner",
            "struct Holder { bytes: ByteBuffer } fn main() -> int { return 0; }",
            "source struct `Holder` cannot contain a ByteBuffer field",
        ),
        (
            "nested-enum-owner",
            "enum Holder { Bytes(ByteBuffer) } fn main() -> int { return 0; }",
            "source enum `Holder` cannot contain a ByteBuffer payload",
        ),
        (
            "nested-array-owner",
            "fn main() -> int { let nested: [ByteBuffer; 1]; return 0; }",
            "rejects surface statement `Let { mutable: false, annotated: true, initialized: false }`",
        ),
        (
            "parameter-transport",
            "fn consume(bytes: ByteBuffer) -> int { return 0; } fn main() -> int { return 0; }",
            "cannot transport ByteBuffer in a parameter or result",
        ),
        (
            "result-transport",
            "fn make() -> ByteBuffer { let bytes: ByteBuffer = bytes_new(); return bytes; } fn main() -> int { return 0; }",
            "cannot transport ByteBuffer in a parameter or result",
        ),
        (
            "reassignment",
            "fn main() -> int { let mut bytes: ByteBuffer = bytes_new(); bytes = bytes_new(); return 0; }",
            "ByteBuffer owners cannot be reassigned",
        ),
        (
            "stored-borrow",
            "fn main() -> int { let bytes: ByteBuffer = bytes_new(); let loan = &bytes; return 0; }",
            "ByteBuffer is not admitted Copy-data for immutable reference transport",
        ),
        (
            "ordinary-call-transport",
            "fn consume(value: int) -> int { return value; } fn main() -> int { let bytes: ByteBuffer = bytes_new(); return consume(bytes); }",
            "parameter `value` type mismatch: expected int, actual ByteBuffer",
        ),
    ];

    let workspace = TestWorkspace::new("rejections");
    let compiler_options = options(LanguageProfile::ExactI32ByteBufferV0);
    for (label, source, expected_fragment) in cases {
        let first = match check_program(source, compiler_options.clone()) {
            Ok(()) => panic!("{label} unexpectedly passed source check"),
            Err(error) => error,
        };
        let second = match check_program(source, compiler_options.clone()) {
            Ok(()) => panic!("{label} unexpectedly passed repeated source check"),
            Err(error) => error,
        };
        assert_eq!(
            first, second,
            "{label} source diagnostic was nondeterministic"
        );
        assert!(
            first.contains(expected_fragment),
            "{label} stopped at the wrong boundary: {first}"
        );
        assert_eq!(
            compile_program(source, compiler_options.clone())
                .expect_err("invalid R1C source must not reach LLVM"),
            first,
            "{label} compile route reordered its rejection"
        );

        let path = workspace.write(&format!("{label}.aero"), source);
        assert_eq!(
            check_file(&path, compiler_options.clone())
                .expect_err("invalid R1C file must fail check"),
            first,
            "{label} file check diverged"
        );
        assert_eq!(
            compile_file(&path, compiler_options.clone())
                .expect_err("invalid R1C file must fail compile"),
            first,
            "{label} file compile diverged"
        );
    }
}

#[test]
fn source_byte_buffer_maps_private_failures_and_closes_every_owner() {
    const RESULT_VALUE: &str = r#"
fn result_value(result: Result<int, int>) -> int {
    return match result {
        Ok(value) => value,
        Err(code) => 0 - code,
    };
}
"#;

    let invalid_byte = format!(
        "{RESULT_VALUE}\nfn main() -> int {{ let mut bytes: ByteBuffer = bytes_new(); let result: Result<int, int> = bytes_push(&mut bytes, 256); return result_value(result); }}"
    );
    run_test_runtime_case(
        "invalid-byte",
        &invalid_byte,
        u64::MAX,
        RuntimeExpectations {
            result: -1,
            allocations: 0,
            reallocations: 0,
            deallocations: 0,
        },
    );

    let allocation_failure = format!(
        "{RESULT_VALUE}\nfn main() -> int {{ let mut bytes: ByteBuffer = bytes_new(); let result: Result<int, int> = bytes_push(&mut bytes, 65); return result_value(result); }}"
    );
    run_test_runtime_case(
        "allocation-failure",
        &allocation_failure,
        0,
        RuntimeExpectations {
            result: -2,
            allocations: 1,
            reallocations: 0,
            deallocations: 0,
        },
    );

    let growth_failure = format!(
        r#"{RESULT_VALUE}
fn main() -> int {{
    let mut bytes: ByteBuffer = bytes_new();
    let p0: Result<int, int> = bytes_push(&mut bytes, 10);
    let p1: Result<int, int> = bytes_push(&mut bytes, 20);
    let p2: Result<int, int> = bytes_push(&mut bytes, 30);
    let p3: Result<int, int> = bytes_push(&mut bytes, 40);
    let p4: Result<int, int> = bytes_push(&mut bytes, 50);
    let p5: Result<int, int> = bytes_push(&mut bytes, 60);
    let p6: Result<int, int> = bytes_push(&mut bytes, 70);
    let p7: Result<int, int> = bytes_push(&mut bytes, 77);
    let failed: Result<int, int> = bytes_push(&mut bytes, 99);
    let preserved: Result<int, int> = bytes_get(&bytes, 7);
    if result_value(failed) == -2 {{
        return result_value(preserved);
    }}
    return 1;
}}
"#
    );
    run_test_runtime_case(
        "growth-failure-preserves-owner",
        &growth_failure,
        1,
        RuntimeExpectations {
            result: 77,
            allocations: 1,
            reallocations: 1,
            deallocations: 1,
        },
    );

    let out_of_bounds = format!(
        "{RESULT_VALUE}\nfn main() -> int {{ let bytes: ByteBuffer = bytes_new(); let result: Result<int, int> = bytes_get(&bytes, 0); return result_value(result); }}"
    );
    run_test_runtime_case(
        "out-of-bounds",
        &out_of_bounds,
        u64::MAX,
        RuntimeExpectations {
            result: -4,
            allocations: 0,
            reallocations: 0,
            deallocations: 0,
        },
    );

    let early_return = format!(
        "{RESULT_VALUE}\nfn main() -> int {{ let mut bytes: ByteBuffer = bytes_new(); let pushed: Result<int, int> = bytes_push(&mut bytes, 91); if result_value(pushed) == 1 {{ return 91; }} return 1; }}"
    );
    run_test_runtime_case(
        "early-return-cleanup",
        &early_return,
        u64::MAX,
        RuntimeExpectations {
            result: 91,
            allocations: 1,
            reallocations: 0,
            deallocations: 1,
        },
    );
}
