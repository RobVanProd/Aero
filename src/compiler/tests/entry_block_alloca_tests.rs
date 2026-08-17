//! CORE-093 - every emitted `alloca` belongs to the entry block.
//!
//! LLVM never reclaims a non-entry `alloca` before the function returns, so a
//! loop body that allocates one grows the stack once per iteration. This target
//! proves that no emitted module places an `alloca` outside its entry block and
//! that a long loop over a checked `ByteBuffer` keeps its stack use constant.

use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_program, compile_program,
    verify_llvm_module,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const LOOP_STACK_SPECIMEN: &str = "../../examples/loop_stack_stability/main.aero";

/// Accepted Aero products whose emitted modules must also satisfy the rule.
const ACCEPTED_PRODUCTS: &[(&str, LanguageProfile)] = &[
    (
        "../../examples/owned_byte_buffer_v0/source_owned_byte_buffer.aero",
        LanguageProfile::ExactI32ByteBufferV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_lexer.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_parser.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_semantics.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_checked_ir.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_checked_ir_verifier.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_llvm_emitter.aero",
        LanguageProfile::ExactI32ByteInputV0,
    ),
    (
        "../../examples/aero_frontend_v0/runtime_ascii_toolchain_driver.aero",
        LanguageProfile::ExactI32ByteIoV0,
    ),
];

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn options(profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile: profile,
        ..CompilerOptions::default()
    }
}

/// Every `alloca` outside an entry block, reported as `(function, block, line)`.
fn allocas_outside_entry_blocks(llvm: &str) -> Vec<(String, String, String)> {
    let mut offenders = Vec::new();
    let mut function = String::new();
    let mut block = String::new();
    for line in llvm.lines() {
        if let Some(rest) = line.strip_prefix("define ") {
            function = rest
                .split_once('@')
                .map(|(_, tail)| tail.split('(').next().unwrap_or_default().to_string())
                .unwrap_or_default();
            block = "entry".to_string();
            continue;
        }
        if line == "}" {
            function.clear();
            block.clear();
            continue;
        }
        let trimmed = line.trim_end();
        if !trimmed.starts_with(' ')
            && trimmed.ends_with(':')
            && !trimmed.starts_with(';')
            && !function.is_empty()
        {
            block = trimmed.trim_end_matches(':').to_string();
            continue;
        }
        if trimmed.contains(" = alloca ") && block != "entry" && !function.is_empty() {
            offenders.push((function.clone(), block.clone(), trimmed.trim().to_string()));
        }
    }
    offenders
}

#[derive(Debug)]
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
            .unwrap_or_else(|| repository_path("../../target"))
            .join("core093-entry-block-alloca-tests");
        let root = parent.join(format!(
            "core093-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CORE-093 test workspace");
        let root = fs::canonicalize(root).expect("canonicalize CORE-093 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CORE-093 test artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("core093-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn llvm_bin() -> PathBuf {
    for variable in ["AERO_LLVM_BIN", "LLVM_BIN"] {
        if let Some(path) = std::env::var_os(variable).map(PathBuf::from)
            && path
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
        {
            return path;
        }
    }
    let path = std::env::var_os("PATH").expect("CORE-093 tests require PATH");
    std::env::split_paths(&path)
        .find(|directory| {
            directory
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
        })
        .expect("CORE-093 tests require an explicit LLVM bin directory")
}

fn clang_link(
    workspace: &TestWorkspace,
    label: &str,
    optimization: &str,
    inputs: &[&Path],
) -> PathBuf {
    let executable = workspace.root.join(if cfg!(windows) {
        format!("{label}-{optimization}.exe")
    } else {
        format!("{label}-{optimization}")
    });
    let output = Command::new(llvm_bin().join("clang"))
        .args([
            "-std=c11",
            optimization,
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wno-error=override-module",
        ])
        .args(inputs)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for CORE-093");
    assert!(
        output.status.success(),
        "CORE-093 link failed at {optimization} (stderr={:?})",
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

#[test]
fn the_loop_stack_specimen_is_tracked_and_checks() {
    let source = fs::read_to_string(repository_path(LOOP_STACK_SPECIMEN))
        .expect("read CORE-093 loop specimen");
    assert!(source.contains("while index < 400000"));
    assert!(source.contains("let pushed: Result<int, int> = bytes_push(&mut bytes, 65);"));
    assert!(source.contains("let found: Result<int, int> = bytes_get(&bytes, read_index);"));
    check_program(&source, options(LanguageProfile::ExactI32ByteBufferV0))
        .expect("CORE-093 loop specimen checks");
}

#[test]
fn every_emitted_module_places_allocas_in_the_entry_block() {
    let specimen = fs::read_to_string(repository_path(LOOP_STACK_SPECIMEN))
        .expect("read CORE-093 loop specimen");
    let mut subjects: Vec<(String, String)> = vec![(
        LOOP_STACK_SPECIMEN.to_string(),
        compile_program(&specimen, options(LanguageProfile::ExactI32ByteBufferV0))
            .expect("CORE-093 loop specimen compiles"),
    )];
    for (relative, profile) in ACCEPTED_PRODUCTS {
        let source = fs::read_to_string(repository_path(relative))
            .unwrap_or_else(|error| panic!("read {relative}: {error}"));
        let llvm = compile_program(&source, options(*profile))
            .unwrap_or_else(|error| panic!("compile {relative}: {error}"));
        subjects.push(((*relative).to_string(), llvm));
    }

    for (relative, llvm) in &subjects {
        let offenders = allocas_outside_entry_blocks(llvm);
        assert!(
            offenders.is_empty(),
            "{relative} emitted {} alloca(s) outside the entry block, first at {}::{} -> {}",
            offenders.len(),
            offenders[0].0,
            offenders[0].1,
            offenders[0].2
        );
        verify_llvm_module(llvm, LlvmVerificationMode::Required)
            .unwrap_or_else(|error| panic!("{relative} LLVM must still verify: {error}"));
    }
}

#[test]
fn a_long_checked_bytebuffer_loop_keeps_stack_use_constant() {
    let specimen = fs::read_to_string(repository_path(LOOP_STACK_SPECIMEN))
        .expect("read CORE-093 loop specimen");
    let llvm = compile_program(&specimen, options(LanguageProfile::ExactI32ByteBufferV0))
        .expect("CORE-093 loop specimen compiles");
    let repeated = compile_program(&specimen, options(LanguageProfile::ExactI32ByteBufferV0))
        .expect("CORE-093 loop specimen recompiles");
    assert_eq!(llvm, repeated, "CORE-093 LLVM became nondeterministic");

    let workspace = TestWorkspace::new("loop-stack");
    let module = workspace.write("loop-stack.ll", &llvm);
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            &workspace,
            "loop-stack",
            optimization,
            &[module.as_path(), runtime.as_path()],
        );
        let output = Command::new(executable)
            .output()
            .expect("run CORE-093 loop specimen");
        assert_eq!(
            output.status.code(),
            Some(91),
            "CORE-093 loop specimen did not survive 800,000 checked ByteBuffer operations at \
             {optimization} (stderr={:?})",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}
