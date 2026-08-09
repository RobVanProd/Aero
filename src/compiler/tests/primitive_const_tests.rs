use compiler::ast::{AstNode, Expression, Statement, Type};
use compiler::{
    CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn parsed(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, Some("const_fixture.aero".to_string()))
        .expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn semantic(source: &str) -> Result<Vec<AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source))
        .map(|(_, ast)| ast)
}

fn checked(source: &str) -> Result<String, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source))
        .map(|ir| format!("{ir:?}"))
        .map_err(|error| error.to_string())
}

fn expect_rejected_everywhere(source: &str, marker: &str) {
    let semantic_error = semantic(source).expect_err("semantic analysis must reject");
    assert!(
        semantic_error.contains(marker),
        "semantic diagnostic omitted {marker:?}: {semantic_error}"
    );

    let admission_error = checked(source).expect_err("checked admission must reject");
    assert!(
        admission_error.contains(marker),
        "checked-admission diagnostic omitted {marker:?}: {admission_error}"
    );

    let public_error = compile_program(source, CompilerOptions::default())
        .expect_err("public compilation must reject");
    assert!(
        public_error.contains(marker),
        "public diagnostic omitted {marker:?}: {public_error}"
    );
}

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
            "aero-primitive-const-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create primitive-const test workspace");
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, contents).expect("write fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn parser_preserves_const_identity_annotation_initializer_and_span() {
    let ast = parsed("const ANSWER: int = 40 + 2;");
    let [
        AstNode::Statement(Statement::Const {
            name,
            type_annotation: Type::Named(annotation),
            value: Expression::Binary { .. },
            location,
        }),
    ] = ast.as_slice()
    else {
        panic!("parser did not preserve the const declaration: {ast:#?}");
    };
    assert_eq!(name, "ANSWER");
    assert_eq!(annotation, "int");
    assert_eq!(location.line, 1);
    assert_eq!(location.column, 1);
    assert_eq!(location.filename.as_deref(), Some("const_fixture.aero"));
}

#[test]
fn primitive_const_class_compiles_without_runtime_storage() {
    let source = r#"
        const BASE: int = 2 + 3;
        const ALIAS_INT: i32 = BASE;
        const SCALE: float = 1.5 + 2.0;
        const ALIAS_FLOAT: f64 = 2.5;
        const ENABLED: bool = BASE == 5 && ALIAS_INT == 5 && SCALE > 3.0 && ALIAS_FLOAT == 2.5;
        const LETTER: char = 'A';
        const NAME: String = "Aero";

        fn increment(value: int) -> int { return value + 1; }

        fn main() -> int {
            const OFFSET: int = BASE + 2;
            const SAME_LETTER: bool = LETTER == 'A';
            const SAME_NAME: bool = NAME == "Aero";
            let values = [OFFSET, BASE];
            if ENABLED && SAME_LETTER && SAME_NAME && values[1] == 5 {
                return increment(values[0]);
            }
            return 1;
        }
    "#;

    let analyzed = semantic(source).expect("primitive constants must pass semantics");
    assert!(
        !format!("{analyzed:#?}").contains("Const"),
        "semantic normalization retained executable const declarations"
    );

    let checked_debug = checked(source).expect("primitive constants must pass checked admission");
    for forbidden in [
        "BASE",
        "ALIAS_INT",
        "SCALE",
        "ALIAS_FLOAT",
        "ENABLED",
        "LETTER",
        "NAME",
        "OFFSET",
    ] {
        assert!(
            !checked_debug.contains(forbidden),
            "checked IR retained runtime identity {forbidden}:\n{checked_debug}"
        );
    }

    let llvm = compile_program(source, CompilerOptions::default())
        .expect("primitive constants should compile through the public pipeline");
    for forbidden in [
        "BASE",
        "ALIAS_INT",
        "SCALE",
        "ALIAS_FLOAT",
        "ENABLED",
        "LETTER",
        "NAME",
        "OFFSET",
    ] {
        assert!(
            !llvm.contains(forbidden),
            "LLVM retained runtime identity {forbidden}:\n{llvm}"
        );
    }
    assert!(
        llvm.contains("define i32 @main()"),
        "unexpected LLVM:\n{llvm}"
    );

    let raw_fstring = compile_program(
        r#"fn main() -> int {
            const TEMPLATE: String = f"raw {not_interpolated}";
            if TEMPLATE == "raw {not_interpolated}" { return 7; }
            return 1;
        }"#,
        CompilerOptions::default(),
    )
    .expect("existing raw non-print f-string spelling must remain a String literal");
    assert!(raw_fstring.contains("ret i32 7"), "{raw_fstring}");
}

#[test]
fn primitive_consts_obey_lexical_scope_and_declaration_order() {
    let source = r#"
        const VALUE: int = 2;
        fn main() -> int {
            const VALUE: int = VALUE + 3;
            {
                let VALUE = 9;
                if VALUE != 9 { return 1; }
            }
            return VALUE;
        }
    "#;
    let llvm = compile_program(source, CompilerOptions::default())
        .expect("inner constant and runtime shadow should compile");
    assert!(llvm.contains("ret i32 5"), "unexpected LLVM:\n{llvm}");

    expect_rejected_everywhere(
        "fn main() -> int { const LATER: int = MISSING; return LATER; }",
        "unknown or not-yet-declared constant `MISSING`",
    );
    expect_rejected_everywhere(
        "const FIRST: int = SECOND; const SECOND: int = FIRST; fn main() -> int { return 1; }",
        "unknown or not-yet-declared constant `SECOND`",
    );
    expect_rejected_everywhere(
        "fn main() -> int { const VALUE: int = 1; const VALUE: int = 2; return VALUE; }",
        "already defined in this lexical scope",
    );
}

#[test]
fn primitive_const_exclusions_fail_closed_in_semantics_admission_and_public_compile() {
    let cases = [
        (
            "fn main() { const VALUE: bool = 1; }",
            "type annotation mismatch: expected bool, evaluated int",
        ),
        (
            "fn main() { const VALUE: [int; 1] = [1]; }",
            "unsupported composite annotation",
        ),
        (
            "fn helper(input: int) -> int { const VALUE: int = input; return VALUE; } fn main() -> int { return helper(1); }",
            "depends on runtime binding `input`",
        ),
        (
            "fn helper() -> int { return 1; } fn main() { const VALUE: int = helper(); }",
            "unsupported function call",
        ),
        (
            "fn main() { const VALUE: int = \"Aero\".len(); }",
            "unsupported method call",
        ),
        (
            "fn main() { const VALUE: int = [1]; }",
            "unsupported array expression",
        ),
        (
            "fn main() { const VALUE: int = (1, 2); }",
            "unsupported tuple expression",
        ),
        (
            "struct Pair { value: int } fn main() { const VALUE: int = Pair { value: 1 }; }",
            "unsupported struct expression",
        ),
        (
            "enum Choice { A } fn main() { const VALUE: int = Choice::A; }",
            "unsupported enum expression",
        ),
        (
            "fn main() { let source = 1; const VALUE: int = &source; }",
            "unsupported borrow expression",
        ),
        (
            "fn main() { let source = 1; const VALUE: int = *&source; }",
            "unsupported dereference expression",
        ),
        (
            "fn main() { const VALUE: int = |item: int| item; }",
            "unsupported closure expression",
        ),
        (
            "fn main() { const VALUE: int = match 1 { _ => 1 }; }",
            "unsupported match expression",
        ),
        (
            "fn main() { const VALUE: int = 2147483647 + 1; }",
            "overflowed the admitted i32 range",
        ),
        (
            "fn main() { const VALUE: int = 1 / 0; }",
            "integer division by zero",
        ),
        (
            "fn main() { const VALUE: float = 1e308 * 1e308; }",
            "produced a non-finite result",
        ),
        (
            "fn main() { const VALUE: int = 5 % 2; }",
            "operator `%` is not admitted",
        ),
        (
            "fn main() { const VALUE: int = 1; VALUE = 2; }",
            "Cannot assign to primitive constant `VALUE`",
        ),
        (
            "fn main() { const VALUE: int = 1; let VALUE = 2; }",
            "already defined as a primitive constant",
        ),
    ];

    for (source, marker) in cases {
        expect_rejected_everywhere(source, marker);
    }
}

#[test]
fn direct_modules_normalize_constants_independently() {
    let tracked = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/primitive_consts/main.aero");
    let tracked_llvm = compile_file(&tracked, CompilerOptions::default())
        .expect("tracked multi-file primitive-const specimen should compile");
    assert!(tracked_llvm.contains("define i32 @primitive_const_result()"));
    assert!(!tracked_llvm.contains("BASE"));

    let workspace = TestWorkspace::new("module");
    let root = workspace.write(
        "main.aero",
        "mod values; fn main() -> int { return module_value(); }",
    );
    workspace.write(
        "values.aero",
        "fn module_value() -> int { const MODULE_VALUE: int = 11 + 2; return MODULE_VALUE; }",
    );
    let llvm = compile_file(&root, CompilerOptions::default())
        .expect("module-local primitive constants should compile");
    assert!(llvm.contains("define i32 @module_value()"));
    assert!(!llvm.contains("MODULE_VALUE"));

    let isolated = TestWorkspace::new("module-isolation");
    let isolated_root = isolated.write(
        "main.aero",
        "const ROOT_ONLY: int = 7; mod values; fn main() -> int { return module_value(); }",
    );
    isolated.write(
        "values.aero",
        "fn module_value() -> int { return ROOT_ONLY; }",
    );
    let error = compile_file(&isolated_root, CompilerOptions::default())
        .expect_err("root constants must not leak into a direct module");
    assert!(
        error.contains("Use of undeclared variable `ROOT_ONLY`"),
        "{error}"
    );
}

#[test]
fn cli_check_build_and_run_reject_invalid_consts_without_artifacts() {
    let workspace = TestWorkspace::new("cli-negative");
    let source = workspace.write(
        "invalid.aero",
        "fn main() -> int { let runtime = 1; const VALUE: int = runtime; return VALUE; }",
    );
    let artifact = workspace.root.join("invalid.ll");
    let binary = Path::new(env!("CARGO_BIN_EXE_aero"));

    for command in ["check", "run"] {
        let output = Command::new(binary)
            .current_dir(&workspace.root)
            .arg(command)
            .arg(&source)
            .output()
            .expect("run Aero CLI");
        let diagnostics = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.status.success(), "{command} unexpectedly succeeded");
        assert!(diagnostics.contains("depends on runtime binding `runtime`"));
    }

    let output = Command::new(binary)
        .current_dir(&workspace.root)
        .arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&artifact)
        .output()
        .expect("run Aero CLI build");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.status.success(), "build unexpectedly succeeded");
    assert!(diagnostics.contains("depends on runtime binding `runtime`"));
    assert!(
        !artifact.exists(),
        "failed build published an LLVM artifact"
    );
}
