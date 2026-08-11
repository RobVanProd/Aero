use compiler::{CompilerOptions, compile_file, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
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
            "aero-file-compilation-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create file-compilation test workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture directory");
        }
        fs::write(&path, contents).expect("write Aero fixture");
        path
    }

    fn assert_no_llvm_artifact(&self) {
        fn visit(path: &Path, found: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(path).expect("read fixture directory") {
                let path = entry.expect("read fixture entry").path();
                if path.is_dir() {
                    visit(&path, found);
                } else if path.extension().is_some_and(|extension| extension == "ll") {
                    found.push(path);
                }
            }
        }

        let mut found = Vec::new();
        visit(&self.root, &mut found);
        assert!(
            found.is_empty(),
            "compile_file wrote LLVM artifacts: {found:?}"
        );
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn module_free_file_output_is_byte_identical_to_source_only_compilation() {
    let workspace = TestWorkspace::new("module-free-parity");
    let source = "fn answer() -> int { return 42; } fn main() -> int { return answer(); }";
    let root = workspace.write("nested/main.aero", source);

    let from_source = compile_program(source, CompilerOptions::default())
        .expect("module-free source-only compilation must remain accepted");
    let from_file = compile_file(&root, CompilerOptions::default())
        .expect("module-free file compilation must be accepted");

    assert_eq!(from_file, from_source);
    workspace.assert_no_llvm_artifact();
}

#[test]
fn direct_file_directory_and_multiple_modules_use_the_existing_flattened_contract() {
    for (label, helper_path) in [
        ("file-layout", "project/helper.aero"),
        ("directory-layout", "project/helper/mod.aero"),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write(
            "project/main.aero",
            "mod helper; mod values; fn main() -> int { return helper() + value(); }",
        );
        workspace.write(helper_path, "fn helper() -> int { return 19; }");
        workspace.write("project/values.aero", "fn value() -> int { return 23; }");

        let llvm = compile_file(&root, CompilerOptions::default())
            .expect("accepted direct modules must compile through the file API");
        assert!(llvm.contains("define i32 @helper()"), "{llvm}");
        assert!(llvm.contains("define i32 @value()"), "{llvm}");
        assert!(llvm.contains("define i32 @main()"), "{llvm}");
        workspace.assert_no_llvm_artifact();
    }
}

#[test]
fn root_read_lex_parse_and_semantic_failures_return_without_artifacts() {
    let missing_workspace = TestWorkspace::new("missing-root");
    let missing = missing_workspace.path("missing/main.aero");
    let error = compile_file(&missing, CompilerOptions::default())
        .expect_err("missing root must fail before compilation");
    assert!(
        error.starts_with("Could not read Aero source file `")
            && error.contains("missing/main.aero"),
        "{error}"
    );
    missing_workspace.assert_no_llvm_artifact();

    let option_error = compile_file(
        &missing,
        CompilerOptions {
            optimize: true,
            ..CompilerOptions::default()
        },
    )
    .expect_err("unsupported options must win before root-file I/O");
    assert_eq!(
        option_error,
        "Unsupported CompilerOptions: optimize, debug_info, and target behavior is not implemented; language_profile is the only supported nondefault option"
    );

    for (label, bytes, expected) in [
        (
            "invalid-utf8",
            vec![0xff, 0xfe],
            "Could not read Aero source file",
        ),
        ("strict-lex", b"fn main() { @; }".to_vec(), "Lex error:"),
        ("fatal-parse", b"fn main( {".to_vec(), "Parse error:"),
        (
            "semantic",
            b"fn main() -> int { return missing; }".to_vec(),
            "Semantic Analysis Error:",
        ),
        (
            "checked-admission",
            b"fn main() { let quotient: int = 1 / 0; }".to_vec(),
            "IR Generation Error:",
        ),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.path("main.aero");
        fs::write(&root, bytes).expect("write failing root fixture");
        let error = compile_file(&root, CompilerOptions::default())
            .expect_err("invalid root program must fail");
        assert!(error.contains(expected), "{label}: {error}");
        workspace.assert_no_llvm_artifact();
    }
}

#[test]
fn direct_module_failures_preserve_the_accepted_early_boundary() {
    let missing_workspace = TestWorkspace::new("missing-module");
    let missing_root = missing_workspace.write("main.aero", "mod absent; fn main() {}");
    let error = compile_file(&missing_root, CompilerOptions::default())
        .expect_err("missing direct module must fail");
    assert!(
        error.contains("Module resolution failed for `absent`")
            && error.contains("Cannot find module `absent`"),
        "{error}"
    );
    missing_workspace.assert_no_llvm_artifact();

    let nested_workspace = TestWorkspace::new("nested-module");
    let nested_root = nested_workspace.write("main.aero", "mod outer; fn main() {}");
    nested_workspace.write("outer.aero", "mod inner; fn outer() {}");
    nested_workspace.write("inner.aero", "fn inner() {}");
    let error = compile_file(&nested_root, CompilerOptions::default())
        .expect_err("nested modules must remain unsupported");
    assert!(
        error.contains("Module resolution failed for `inner`")
            && error.contains("nested module declarations are not supported"),
        "{error}"
    );
    nested_workspace.assert_no_llvm_artifact();

    for (label, module_bytes, expected) in [
        (
            "module-invalid-utf8",
            vec![0xff],
            "Module resolution failed for `bad`",
        ),
        (
            "module-strict-lex",
            b"fn bad() { @; }".to_vec(),
            "Lex error:",
        ),
        ("module-fatal-parse", b"fn bad( {".to_vec(), "Parse error:"),
        (
            "module-semantic",
            b"fn bad() -> int { return missing; }".to_vec(),
            "Semantic Analysis Error:",
        ),
        (
            "module-checked-admission",
            b"fn bad() { let quotient: int = 1 / 0; }".to_vec(),
            "IR Generation Error:",
        ),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write("main.aero", "mod bad; fn main() {}");
        fs::write(workspace.path("bad.aero"), module_bytes).expect("write failing module");
        let error = compile_file(&root, CompilerOptions::default())
            .expect_err("invalid direct module must fail");
        assert!(error.contains(expected), "{label}: {error}");
        workspace.assert_no_llvm_artifact();
    }
}

#[test]
fn source_only_module_rejection_remains_exact() {
    let error = compile_program(
        "mod helper; fn main() { helper(); }",
        CompilerOptions::default(),
    )
    .expect_err("source-only compilation cannot acquire a file-system base directory");
    assert_eq!(
        error,
        "Module resolution failed for `helper`: module declarations require an entry-file context"
    );
}
