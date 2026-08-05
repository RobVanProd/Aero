use compiler::ast::{AstNode, Block, ComparisonOp, Expression, Parameter, Statement, Type};
use compiler::errors::SourceLocation;
use compiler::{
    CodeGenerationError, CodeGenerator, CompilerOptions, IrGenerationError, IrGenerator,
    SemanticAnalyzer, compile_program, generate_code, parse_with_locations, try_generate_code,
    try_tokenize_with_locations,
};
use std::ffi::OsStr;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-checked-ir-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create checked-IR test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-checked-ir-"));
        if self.root.starts_with(std::env::temp_dir()) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn analyzed_ast(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("source must lex");
    let ast = parse_with_locations(tokens).expect("source must parse");
    let mut analyzer = SemanticAnalyzer::new();
    let (_message, typed_ast) = analyzer.analyze(ast).expect("source must pass semantics");
    typed_ast
}

#[test]
fn checked_apis_are_additive_and_keep_structured_error_types() {
    let ast = analyzed_ast("fn main() { let value: int = 1 + 2; }");

    let mut checked_ir_generator = IrGenerator::new();
    let checked_ir: Result<_, IrGenerationError> =
        checked_ir_generator.try_generate_ir(ast.clone());
    let checked_ir = checked_ir.expect("valid source must pass checked IR generation");

    let mut legacy_ir_generator = IrGenerator::new();
    let legacy_ir = legacy_ir_generator.generate_ir(ast);

    let free_checked: Result<String, CodeGenerationError> = try_generate_code(checked_ir.clone());
    let mut method_generator = CodeGenerator::new();
    let method_checked: Result<String, CodeGenerationError> =
        method_generator.try_generate_code(checked_ir.clone());

    for (label, llvm) in [
        ("free checked", free_checked.expect("free checked codegen")),
        (
            "method checked",
            method_checked.expect("method checked codegen"),
        ),
        ("free legacy", generate_code(legacy_ir.clone())),
        (
            "method legacy",
            CodeGenerator::new().generate_code(legacy_ir),
        ),
    ] {
        assert!(!llvm.trim().is_empty(), "{label} returned empty IR");
        assert!(
            !llvm.contains("IR Generation Error:")
                && !llvm.contains("IR Verification Error:")
                && !llvm.contains("Code Generation Error:"),
            "{label} embedded an error as IR: {llvm}"
        );
    }
}

#[test]
fn checked_ir_generator_reuse_starts_from_a_clean_module_state() {
    let first =
        analyzed_ast("fn helper(value: int) -> int { return value; } fn main() { helper(1); }");
    let second = analyzed_ast("fn main() { let value: int = 7; }");
    let mut generator = IrGenerator::new();

    let first = generator
        .try_generate_ir(first)
        .expect("first checked module must compile");
    assert!(
        first.metadata().functions.len() > 1,
        "first module must exercise non-main generator state"
    );

    let second = generator
        .try_generate_ir(second)
        .expect("reused generator must compile a fresh module");
    let names = second
        .metadata()
        .functions
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec!["main"],
        "reused checked generator leaked functions from the prior module"
    );
    let llvm = try_generate_code(second).expect("fresh second module must pass checked codegen");
    assert!(
        !llvm.contains("@helper"),
        "stale helper leaked into LLVM:\n{llvm}"
    );
    assert!(
        !llvm.contains("@closure_"),
        "stale closure leaked into LLVM:\n{llvm}"
    );
}

#[test]
fn known_scalar_top_level_call_arity_fails_at_checked_admission() {
    fn named(name: &str) -> Type {
        Type::Named(name.to_string())
    }

    fn parameter(name: &str, ty: Type) -> Parameter {
        Parameter {
            name: name.to_string(),
            param_type: ty,
        }
    }

    fn function(
        name: &str,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        type_params: Vec<&str>,
        statements: Vec<Statement>,
    ) -> AstNode {
        AstNode::Statement(Statement::Function {
            name: name.to_string(),
            parameters,
            return_type,
            body: Block {
                statements,
                expression: None,
            },
            type_params: type_params.into_iter().map(str::to_string).collect(),
            trait_bounds: Vec::new(),
        })
    }

    fn main_function(statements: Vec<Statement>) -> AstNode {
        function("main", Vec::new(), None, Vec::new(), statements)
    }

    fn int_function(name: &str, parameters: Vec<Parameter>) -> AstNode {
        function(
            name,
            parameters,
            Some(named("int")),
            Vec::new(),
            vec![Statement::Return(Some(Expression::IntegerLiteral(1)))],
        )
    }

    fn call(name: &str, arguments: Vec<Expression>) -> Expression {
        Expression::FunctionCall {
            name: name.to_string(),
            arguments,
        }
    }

    fn checked_error(ast: Vec<AstNode>, label: &str) -> IrGenerationError {
        match IrGenerator::new().try_generate_ir(ast) {
            Ok(_) => panic!("{label}: checked IR unexpectedly succeeded"),
            Err(error) => error,
        }
    }

    fn assert_admission(ast: Vec<AstNode>, expected: &str, label: &str) {
        match checked_error(ast, label) {
            IrGenerationError::Admission(message) => {
                assert_eq!(message, expected, "{label}: changed Admission diagnostic")
            }
            IrGenerationError::Verification(error) => {
                panic!("{label}: reached verification instead of Admission: {error}")
            }
        }
    }

    fn assert_verification(ast: Vec<AstNode>, expected_fragment: &str, label: &str) {
        match checked_error(ast, label) {
            IrGenerationError::Verification(error) => assert!(
                error.to_string().contains(expected_fragment),
                "{label}: wrong verifier diagnostic: {error}"
            ),
            IrGenerationError::Admission(message) => {
                panic!("{label}: verifier precedence changed to Admission: {message}")
            }
        }
    }

    fn assert_checked(ast: Vec<AstNode>, label: &str) {
        if let Err(error) = IrGenerator::new().try_generate_ir(ast) {
            panic!("{label}: valid checked AST failed: {error}");
        }
    }

    assert_admission(
        vec![
            int_function("child_first", vec![parameter("value", named("int"))]),
            main_function(vec![Statement::Expression(call(
                "child_first",
                vec![
                    Expression::IntegerLiteral(1),
                    Expression::TupleLiteral(vec![Expression::IntegerLiteral(2)]),
                ],
            ))]),
        ],
        "flat Copy tuples require at least two elements",
        "surplus child precedence",
    );

    assert_admission(
        vec![
            int_function("shadowed", vec![parameter("value", named("int"))]),
            main_function(vec![
                Statement::Let {
                    name: "shadowed".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::Closure {
                        params: Vec::new(),
                        body: Box::new(Expression::IntegerLiteral(9)),
                        location: SourceLocation::new(8, 17),
                    }),
                },
                Statement::Expression(call("shadowed", Vec::new())),
            ]),
        ],
        "Error: closure expressions are parsed but unsupported in executable code at 8:17.",
        "local closure containment precedence",
    );

    assert_admission(
        vec![
            function(
                "consume",
                vec![parameter("value", named("int"))],
                None,
                Vec::new(),
                Vec::new(),
            ),
            main_function(vec![Statement::Let {
                name: "captured".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(call("consume", Vec::new())),
            }]),
        ],
        "Void function calls cannot be used as values",
        "Void-as-value precedence",
    );

    assert_checked(
        vec![
            main_function(vec![Statement::Expression(call(
                "forward",
                vec![Expression::IntegerLiteral(1)],
            ))]),
            int_function("forward", vec![parameter("value", named("int"))]),
        ],
        "forward exact-arity call",
    );
    assert_checked(
        vec![
            function(
                "recur",
                vec![parameter("value", named("int"))],
                Some(named("int")),
                Vec::new(),
                vec![Statement::Return(Some(call(
                    "recur",
                    vec![Expression::Identifier("value".to_string())],
                )))],
            ),
            main_function(vec![Statement::Expression(call(
                "recur",
                vec![Expression::IntegerLiteral(1)],
            ))]),
        ],
        "recursive exact-arity call",
    );
    assert_checked(
        vec![
            function(
                "identity_bool",
                vec![parameter("flag", named("bool"))],
                Some(named("bool")),
                Vec::new(),
                vec![Statement::Return(Some(Expression::Identifier(
                    "flag".to_string(),
                )))],
            ),
            main_function(vec![Statement::Expression(call(
                "identity_bool",
                vec![Expression::Comparison {
                    op: ComparisonOp::Equal,
                    left: Box::new(Expression::IntegerLiteral(1)),
                    right: Box::new(Expression::IntegerLiteral(1)),
                }],
            ))]),
        ],
        "Boolean exact-arity call",
    );
    assert_checked(
        vec![
            function(
                "identity_float",
                vec![parameter("value", named("float"))],
                Some(named("float")),
                Vec::new(),
                vec![Statement::Return(Some(Expression::Identifier(
                    "value".to_string(),
                )))],
            ),
            main_function(vec![Statement::Expression(call(
                "identity_float",
                vec![Expression::FloatLiteral(1.5)],
            ))]),
        ],
        "Float exact-arity call",
    );
    assert_checked(
        vec![
            function(
                "discard_void",
                vec![parameter("value", named("int"))],
                None,
                Vec::new(),
                Vec::new(),
            ),
            main_function(vec![Statement::Expression(call(
                "discard_void",
                vec![Expression::IntegerLiteral(1)],
            ))]),
        ],
        "discarded Void exact-arity call",
    );

    assert_verification(
        vec![
            int_function("bad-name", vec![parameter("value", named("int"))]),
            main_function(vec![Statement::Expression(call("bad-name", Vec::new()))]),
        ],
        "function symbol `bad-name` is not admitted for LLVM emission",
        "invalid function symbol",
    );
    assert_verification(
        vec![
            int_function("printf", vec![parameter("value", named("int"))]),
            main_function(vec![Statement::Expression(call("printf", Vec::new()))]),
        ],
        "`printf` is reserved by the checked runtime ABI",
        "reserved function symbol",
    );
    assert_verification(
        vec![
            int_function("bad_parameter", vec![parameter("bad-name", named("int"))]),
            main_function(vec![Statement::Expression(call(
                "bad_parameter",
                Vec::new(),
            ))]),
        ],
        "parameter symbol `bad-name` is not admitted for LLVM emission",
        "invalid parameter symbol",
    );
    assert_verification(
        vec![
            int_function(
                "duplicate_parameter",
                vec![
                    parameter("value", named("int")),
                    parameter("value", named("int")),
                ],
            ),
            main_function(vec![Statement::Expression(call(
                "duplicate_parameter",
                Vec::new(),
            ))]),
        ],
        "function signature defines duplicate parameter `value`",
        "duplicate parameter symbol",
    );
    assert_verification(
        vec![
            int_function("duplicate", vec![parameter("first", named("int"))]),
            int_function(
                "duplicate",
                vec![
                    parameter("first", named("int")),
                    parameter("second", named("int")),
                ],
            ),
            main_function(vec![Statement::Expression(call("duplicate", Vec::new()))]),
        ],
        "duplicate result definition",
        "duplicate top-level declaration",
    );

    assert_verification(
        vec![
            int_function("entry_caller", Vec::new()),
            main_function(vec![Statement::Expression(call(
                "main",
                vec![Expression::IntegerLiteral(1)],
            ))]),
        ],
        "call to `main` has 1 arguments but its signature requires 0",
        "entry behavior remains ineligible",
    );
    assert_admission(
        vec![
            main_function(vec![Statement::Expression(call("generic", Vec::new()))]),
            function(
                "generic",
                vec![parameter("value", named("int"))],
                Some(named("int")),
                vec!["T"],
                vec![Statement::Return(Some(Expression::IntegerLiteral(1)))],
            ),
        ],
        "generic function IR is not admitted in CORE-010",
        "generic signature remains ineligible",
    );
    assert_admission(
        vec![
            main_function(vec![Statement::Expression(call("ineligible", Vec::new()))]),
            int_function(
                "ineligible",
                vec![parameter("value", Type::Array(Box::new(named("int")), 1))],
            ),
        ],
        "call to `ineligible` has 0 arguments but its signature requires 1",
        "fixed numeric array signature now participates in exact arity admission",
    );
    assert_admission(
        vec![
            main_function(vec![Statement::Expression(call("ineligible", Vec::new()))]),
            int_function(
                "ineligible",
                vec![parameter(
                    "value",
                    Type::Reference(Box::new(named("int")), false),
                )],
            ),
        ],
        "call to `ineligible` has 0 arguments but its signature requires 1",
        "reference signature now participates in exact arity admission",
    );
    assert_admission(
        vec![
            main_function(vec![Statement::Expression(call(
                "ineligible_result",
                vec![Expression::IntegerLiteral(1)],
            ))]),
            function(
                "ineligible_result",
                Vec::new(),
                Some(Type::Array(Box::new(named("int")), 1)),
                Vec::new(),
                vec![Statement::Return(Some(Expression::IntegerLiteral(1)))],
            ),
        ],
        "call to `ineligible_result` has 1 arguments but its signature requires 0",
        "fixed numeric array result now participates in exact arity admission",
    );
    assert_admission(
        vec![
            main_function(vec![Statement::Expression(call(
                "ineligible_result",
                vec![Expression::IntegerLiteral(1)],
            ))]),
            function(
                "ineligible_result",
                Vec::new(),
                Some(Type::Reference(Box::new(named("int")), false)),
                Vec::new(),
                vec![Statement::Return(Some(Expression::IntegerLiteral(1)))],
            ),
        ],
        "reference results require lifetime semantics and are not supported by CORE-053",
        "reference result remains ineligible",
    );

    let wrong_arity = [
        (
            "too few",
            vec![
                int_function(
                    "needs_two",
                    vec![
                        parameter("left", named("int")),
                        parameter("right", named("int")),
                    ],
                ),
                main_function(vec![Statement::Expression(call(
                    "needs_two",
                    vec![Expression::IntegerLiteral(1)],
                ))]),
            ],
            "call to `needs_two` has 1 arguments but its signature requires 2",
        ),
        (
            "too many",
            vec![
                int_function("needs_one", vec![parameter("value", named("int"))]),
                main_function(vec![Statement::Expression(call(
                    "needs_one",
                    vec![Expression::IntegerLiteral(1), Expression::IntegerLiteral(2)],
                ))]),
            ],
            "call to `needs_one` has 2 arguments but its signature requires 1",
        ),
    ];
    let observed = wrong_arity
        .into_iter()
        .map(|(label, ast, expected)| match checked_error(ast, label) {
            IrGenerationError::Admission(message) => {
                assert_eq!(message, expected, "{label}: wrong Admission diagnostic");
                format!("{label}: Admission")
            }
            IrGenerationError::Verification(error) => {
                assert!(
                    error.to_string().contains(expected),
                    "{label}: wrong pre-fix verifier diagnostic: {error}"
                );
                format!("{label}: Verification")
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        ["too few: Admission", "too many: Admission"],
        "wrong-arity calls reached raw IR verification instead of checked admission"
    );
}

#[test]
fn checked_ir_generation_returns_errors_instead_of_unwinding_or_partial_ir() {
    let cases = [
        (
            "array comparison",
            "fn main() { let compared = [1] == [1]; }",
        ),
        (
            "constant integer division by zero",
            "fn main() { let divided: int = 1 / 0; }",
        ),
    ];

    for (label, source) in cases {
        let ast = analyzed_ast(source);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let mut generator = IrGenerator::new();
            let result: Result<_, IrGenerationError> = generator.try_generate_ir(ast);
            result
        }));
        let result = outcome.unwrap_or_else(|_| panic!("{label}: checked IR generation unwound"));
        assert!(
            result.is_err(),
            "{label}: checked IR returned a partial map"
        );

        let public = catch_unwind(AssertUnwindSafe(|| {
            compile_program(source, CompilerOptions::default())
        }));
        let public = public.unwrap_or_else(|_| panic!("{label}: compile_program unwound"));
        let error = match public {
            Ok(llvm) => {
                panic!("{label}: compile_program returned LLVM instead of an error:\n{llvm}")
            }
            Err(error) => error,
        };
        assert!(
            error.starts_with("IR Generation Error:"),
            "{label}: expected exact IR Generation phase identity: {error}"
        );
    }
}

#[test]
fn checked_static_string_equality_returns_complete_ir_without_unwinding() {
    let source = "fn main() { let compared = \"left\" == \"right\"; }";
    let ast = analyzed_ast(source);
    let checked = catch_unwind(AssertUnwindSafe(|| {
        let mut generator = IrGenerator::new();
        generator.try_generate_ir(ast)
    }))
    .expect("static String equality checked IR generation unwound")
    .expect("static String equality should return a complete checked IR map");
    assert!(
        !checked.metadata().functions.is_empty(),
        "static String equality returned no checked IR"
    );

    let llvm = catch_unwind(AssertUnwindSafe(|| {
        compile_program(source, CompilerOptions::default())
    }))
    .expect("static String equality compile_program unwound")
    .expect("static String equality compile_program rejected checked source");
    assert!(
        llvm.contains("icmp ne i32 0, 0"),
        "false static String equality omitted complete Bool IR:\n{llvm}"
    );
}

#[derive(Clone, Copy)]
enum FakeVerifierBehavior {
    Accept,
    Reject,
}

fn write_fake_llvm_tool(
    workspace: &TestWorkspace,
    name: &str,
    behavior: FakeVerifierBehavior,
) -> PathBuf {
    #[cfg(windows)]
    let path = workspace.path(&format!("{name}.cmd"));
    #[cfg(not(windows))]
    let path = workspace.path(name);

    let exit_code = match behavior {
        FakeVerifierBehavior::Accept => 0,
        FakeVerifierBehavior::Reject => 42,
    };

    #[cfg(windows)]
    let script = format!(
        r#"@echo off
>>"%AERO_CHECK_FAKE_LOG%" echo invoked %*
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
exit /b {exit_code}
"#
    );
    #[cfg(not(windows))]
    let script = format!(
        r#"#!/bin/sh
printf '%s\n' "invoked $*" >> "$AERO_CHECK_FAKE_LOG"
if [ "${{1:-}}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
exit {exit_code}
"#
    );

    fs::write(&path, script).expect("write fake LLVM verifier");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("stat fake LLVM verifier")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake LLVM verifier executable");
    }
    path
}

fn workspace_snapshot(workspace: &TestWorkspace) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(&workspace.root)
        .expect("read checked-IR workspace")
        .map(|entry| entry.expect("read checked-IR workspace entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn assert_workspace_unchanged(workspace: &TestWorkspace, before: &[PathBuf], operation: &str) {
    let after = workspace_snapshot(workspace);
    assert_eq!(
        after, before,
        "{operation} created an artifact; before={before:?}, after={after:?}"
    );
}

fn run_check_with_verifier(
    workspace: &TestWorkspace,
    source_path: &Path,
    opt_path: &Path,
    llvm_as_path: &Path,
    invocation_log: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("check")
        .arg(source_path)
        .current_dir(&workspace.root)
        .env("AERO_REQUIRE_LLVM_VERIFIER", "true")
        .env("AERO_LLVM_OPT", opt_path)
        .env("AERO_LLVM_AS", llvm_as_path)
        .env("AERO_CHECK_FAKE_LOG", invocation_log)
        .env("PATH", &workspace.root)
        .output()
        .expect("run aero check")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .replace("\x1b[1;31m", "")
    .replace("\x1b[1;32m", "")
    .replace("\x1b[1;34m", "")
    .replace("\x1b[1;36m", "")
    .replace("\x1b[0m", "")
}

fn stable_check_diagnostics(output: &Output) -> String {
    diagnostics(output)
        .lines()
        .map(|line| {
            if line.contains("Checking") && line.ends_with(')') {
                line.rsplit_once(" (").map_or(line, |(stable, _)| stable)
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_only_source_file_exists(workspace: &TestWorkspace, source_path: &Path) {
    let mut unexpected = fs::read_dir(&workspace.root)
        .expect("read check workspace")
        .map(|entry| entry.expect("workspace entry").path())
        .filter(|path| path != source_path)
        .collect::<Vec<_>>();
    unexpected.sort();
    assert!(
        unexpected.is_empty(),
        "aero check created unexpected artifacts: {unexpected:?}"
    );
}

#[test]
fn check_is_identical_with_missing_accepting_or_rejecting_llvm_tools() {
    for (label, source, expected_success) in [
        ("valid", "fn main() { let value: int = 1 + 2; }", true),
        (
            "string-comparison",
            "fn main() { let compared = \"left\" == \"right\"; }",
            true,
        ),
        (
            "integer-divide-by-zero",
            "fn main() { let divided: int = 1 / 0; }",
            false,
        ),
    ] {
        let workspace = TestWorkspace::new(&format!("check-tool-independent-{label}"));
        let source_path = workspace.path("case.aero");
        fs::write(&source_path, source).expect("write check source");
        let accepting =
            write_fake_llvm_tool(&workspace, "accepting-opt", FakeVerifierBehavior::Accept);
        let rejecting =
            write_fake_llvm_tool(&workspace, "rejecting-opt", FakeVerifierBehavior::Reject);
        let missing = workspace.path("missing-opt-22");

        let configurations = [
            ("missing", missing.as_path(), missing.as_path()),
            ("accepting", accepting.as_path(), accepting.as_path()),
            ("rejecting", rejecting.as_path(), rejecting.as_path()),
        ];
        let mut observations = Vec::new();

        for (configuration, opt, llvm_as) in configurations {
            let invocation_log = workspace.path(&format!("{configuration}-invocations.log"));
            let before = workspace_snapshot(&workspace);
            let output =
                run_check_with_verifier(&workspace, &source_path, opt, llvm_as, &invocation_log);
            let diagnostic = stable_check_diagnostics(&output);
            assert_eq!(
                output.status.success(),
                expected_success,
                "{label}/{configuration}: unexpected check status: {diagnostic}"
            );
            assert!(
                !invocation_log.exists(),
                "{label}/{configuration}: check invoked an external LLVM tool"
            );
            assert_workspace_unchanged(
                &workspace,
                &before,
                &format!("{label}/{configuration} check"),
            );
            assert!(
                !diagnostic.to_ascii_lowercase().contains("panicked at"),
                "{label}/{configuration}: check unwound: {diagnostic}"
            );
            observations.push((
                configuration,
                output.status.code(),
                output.status.success(),
                diagnostic,
            ));
        }

        let reference = &observations[0];
        for observation in &observations[1..] {
            assert_eq!(
                (observation.1, observation.2, &observation.3),
                (reference.1, reference.2, &reference.3),
                "{label}: check changed with LLVM configuration; reference={reference:?}, observed={observation:?}"
            );
        }
        if !expected_success {
            assert!(
                reference.3.contains("error: IR Generation Error:"),
                "{label}: missing exact IR Generation phase identity: {}",
                reference.3
            );
        }
    }
}

#[test]
fn invalid_ir_generation_stops_build_before_backend_or_publication() {
    let workspace = TestWorkspace::new("build-ir-ordering");
    let source_path = workspace.path("invalid.aero");
    let artifact = workspace.path("must-not-exist.ll");
    fs::write(&source_path, "fn main() { let compared = [1] == [1]; }")
        .expect("write invalid build source");
    let rejecting = write_fake_llvm_tool(&workspace, "rejecting-opt", FakeVerifierBehavior::Reject);
    let invocation_log = workspace.path("build-verifier-invocations.log");
    let before = workspace_snapshot(&workspace);

    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .args([OsStr::new("build"), source_path.as_os_str()])
        .arg("-o")
        .arg(&artifact)
        .current_dir(&workspace.root)
        .env("AERO_REQUIRE_LLVM_VERIFIER", "true")
        .env("AERO_LLVM_OPT", &rejecting)
        .env("AERO_LLVM_AS", &rejecting)
        .env("AERO_CHECK_FAKE_LOG", &invocation_log)
        .env("PATH", &workspace.root)
        .output()
        .expect("run invalid aero build");
    let diagnostic = diagnostics(&output);

    assert!(
        !output.status.success(),
        "invalid checked IR build succeeded: {diagnostic}"
    );
    assert!(
        diagnostic.contains("error: IR Generation Error:"),
        "invalid build lost exact IR Generation identity: {diagnostic}"
    );
    assert!(
        !invocation_log.exists(),
        "invalid build reached the external LLVM verifier"
    );
    assert!(
        !diagnostic.contains("Advanced graph compilation")
            && !diagnostic.contains("Optimized code generation"),
        "invalid build reached graph/backend processing: {diagnostic}"
    );
    assert!(!artifact.exists(), "invalid build published LLVM output");
    assert_workspace_unchanged(&workspace, &before, "invalid checked-IR build");
}

#[test]
fn check_preserves_existing_raw_semantic_diagnostic_text() {
    let workspace = TestWorkspace::new("check-semantic-text");
    let source_path = workspace.path("semantic.aero");
    fs::write(
        &source_path,
        "struct Point { x: String } fn main() { let point = Point { x: \"text\" }; }",
    )
    .expect("write semantic check source");
    let missing = workspace.path("missing-opt-22");
    let invocation_log = workspace.path("semantic-check-invocations.log");
    let output = run_check_with_verifier(
        &workspace,
        &source_path,
        &missing,
        &missing,
        &invocation_log,
    );
    let diagnostic = diagnostics(&output);
    assert!(
        !output.status.success(),
        "semantic check succeeded: {diagnostic}"
    );
    assert!(
        diagnostic
            .lines()
            .any(|line| line.trim() == "error: Struct construction expressions are not supported."),
        "check changed its existing semantic text: {diagnostic}"
    );
    assert!(
        !diagnostic.contains("Semantic Analysis Error:"),
        "check gained the build-only semantic prefix: {diagnostic}"
    );
    assert_only_source_file_exists(&workspace, &source_path);
}
