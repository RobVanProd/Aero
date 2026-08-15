use compiler::{
    CompilerOptions, LanguageProfile, check_file, check_program, compile_file, compile_program,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const ROOT_EXPRESSION_SOURCE: &str = "1;\n";
const FUNCTION_EXPRESSION_SOURCE: &str = r#"
fn main() -> int {
    return 1;
}
"#;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

struct TempWorkspace(PathBuf);

impl TempWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aero-cap033-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).expect("create CAP-033 characterization workspace");
        Self(root)
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write CAP-033 characterization source");
        path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn accepted_root_and_function_expression_behavior_is_unchanged() {
    let options = CompilerOptions::default();
    let root_llvm = compile_program(ROOT_EXPRESSION_SOURCE, options.clone())
        .expect("accepted root expression must compile");
    let function_llvm = compile_program(FUNCTION_EXPRESSION_SOURCE, options.clone())
        .expect("accepted function expression must compile");
    assert_eq!(
        compile_program(ROOT_EXPRESSION_SOURCE, options.clone())
            .expect("repeated root expression compilation must succeed"),
        root_llvm,
        "root-expression LLVM became nondeterministic"
    );
    assert_eq!(
        compile_program(FUNCTION_EXPRESSION_SOURCE, options.clone())
            .expect("repeated function expression compilation must succeed"),
        function_llvm,
        "function-expression LLVM became nondeterministic"
    );

    let workspace = TempWorkspace::new();
    let root_path = workspace.source("root.aero", ROOT_EXPRESSION_SOURCE);
    let function_path = workspace.source("function.aero", FUNCTION_EXPRESSION_SOURCE);
    assert_eq!(
        compile_file(&root_path, options.clone()).expect("root file route must compile"),
        root_llvm,
        "root source/file LLVM drifted"
    );
    assert_eq!(
        compile_file(&function_path, options.clone()).expect("function file route must compile"),
        function_llvm,
        "function source/file LLVM drifted"
    );
    check_program(ROOT_EXPRESSION_SOURCE, options.clone())
        .expect("root source check must remain accepted");
    check_file(&root_path, options.clone()).expect("root file check must remain accepted");
    check_program(FUNCTION_EXPRESSION_SOURCE, options.clone())
        .expect("function source check must remain accepted");
    check_file(&function_path, options).expect("function file check must remain accepted");

    for (path, expected) in [(&root_path, 0), (&function_path, 1)] {
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .arg("run")
            .arg(path)
            .arg("--language-profile")
            .arg(LanguageProfile::Experimental.as_str())
            .current_dir(&workspace.0)
            .output()
            .expect("run accepted CAP-033 characterization source");
        assert_eq!(
            output.status.code(),
            Some(expected),
            "accepted native status drifted for {} (stdout={:?}, stderr={:?})",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

#[test]
fn resolved_surface_context_is_structural_red_first() {
    let authority = read(&repository_root().join("src/compiler/src/resolved_profile_shape.rs"));
    let production = authority
        .split("\n#[cfg(test)]")
        .next()
        .expect("resolved profile-shape source has a production prefix");

    assert!(
        production.contains("enum ResolvedProfileSurfaceContext"),
        "CAP-033 intentional structural red: surface context authority is absent"
    );
    for anchor in [
        "FileScope",
        "Function(ResolvedProfileOrigin)",
        "context: ResolvedProfileSurfaceContext",
        "fn surface_context(&self)",
        "self.function.clone()",
    ] {
        assert!(
            production.contains(anchor),
            "surface context authority omitted `{anchor}`"
        );
    }
    for forbidden in [
        "surface_contexts: Vec",
        "SourceLocation",
        "LanguageProfile",
        "authenticate_resolved_profile",
        "IrGenerator",
        "CheckedIr",
        "CopyDataLayout",
        "CodeGenerator",
    ] {
        assert!(
            !production.contains(forbidden),
            "surface context crossed its authority boundary via `{forbidden}`"
        );
    }
    assert_eq!(
        production.matches("for node in ast").count(),
        1,
        "surface context must reuse the single normalized-AST walk"
    );
    assert_eq!(
        production.matches("fn surface_context(&self)").count(),
        1,
        "surface context must have one projection from existing builder state"
    );
}
