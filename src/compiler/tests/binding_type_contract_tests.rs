use compiler::ast::{AstNode, BinaryOp, Block, Expression, Statement, Type};
use compiler::types::Ty;
use compiler::{
    CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn parsed(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn semantic(source: &str) -> Result<Vec<AstNode>, String> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(parsed(source)).map(|(_, ast)| ast)
}

fn checked_ast(ast: Vec<AstNode>) -> Result<(), String> {
    match catch_unwind(AssertUnwindSafe(|| IrGenerator::new().try_generate_ir(ast))) {
        Err(_) => Err("checked IR unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error.to_string()),
    }
}

fn checked_source(source: &str) -> Result<(), String> {
    checked_ast(parsed(source))
}

fn expect_rejection(
    failures: &mut Vec<String>,
    label: &str,
    result: Result<(), String>,
    fragments: &[&str],
) {
    match result {
        Ok(()) => failures.push(format!("{label}: unexpectedly accepted")),
        Err(error) => {
            for fragment in fragments {
                if !error.contains(fragment) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} missing {fragment:?}"
                    ));
                }
            }
        }
    }
}

fn expect_exact_rejection(
    failures: &mut Vec<String>,
    label: &str,
    result: Result<(), String>,
    expected: &str,
) {
    match result {
        Ok(()) => failures.push(format!("{label}: unexpectedly accepted")),
        Err(error) if error != expected => failures.push(format!(
            "{label}: diagnostic {error:?} did not equal {expected:?}"
        )),
        Err(_) => {}
    }
}

fn expect_acceptance(failures: &mut Vec<String>, label: &str, result: Result<(), String>) {
    if let Err(error) = result {
        failures.push(format!("{label}: unexpectedly rejected: {error}"));
    }
}

fn semantic_result(source: &str) -> Result<(), String> {
    semantic(source).map(|_| ())
}

fn binding(name: &str, annotation: Option<Type>, value: Expression) -> Statement {
    Statement::Let {
        name: name.to_string(),
        mutable: false,
        type_annotation: annotation,
        value: Some(value),
    }
}

fn uninitialized_binding(name: &str, annotation: Type) -> Statement {
    Statement::Let {
        name: name.to_string(),
        mutable: false,
        type_annotation: Some(annotation),
        value: None,
    }
}

fn binary_with_metadata(ty: Option<Ty>) -> Expression {
    Expression::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expression::IntegerLiteral(1)),
        right: Box::new(Expression::IntegerLiteral(2)),
        ty,
    }
}

fn float_binary_with_metadata(ty: Option<Ty>) -> Expression {
    Expression::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expression::FloatLiteral(1.0)),
        right: Box::new(Expression::FloatLiteral(2.0)),
        ty,
    }
}

fn generic_impl_with(statement: Statement) -> Vec<AstNode> {
    vec![AstNode::Statement(Statement::ImplBlock {
        type_name: "Widget".to_string(),
        methods: vec![Statement::Function {
            name: "probe".to_string(),
            parameters: Vec::new(),
            return_type: None,
            body: Block {
                statements: vec![statement],
                expression: None,
            },
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
        }],
        type_params: vec!["T".to_string()],
        trait_name: None,
    })]
}

#[test]
fn semantic_rejects_selected_initialized_binding_mismatches() {
    let cases = [
        (
            "canonical String from int",
            "fn main() { let value: String = 1; }",
            "String",
            "int",
        ),
        (
            "bool from int",
            "fn main() { let value: bool = 1; }",
            "bool",
            "int",
        ),
        (
            "fixed numeric element mismatch",
            "fn main() { let value: [int; 2] = [1.0, 2.0]; }",
            "[int; 2]",
            "[float; 2]",
        ),
        (
            "fixed i32 element mismatch",
            "fn main() { let value: [i32; 2] = [1.0, 2.0]; }",
            "[int; 2]",
            "[float; 2]",
        ),
        (
            "fixed float element mismatch",
            "fn main() { let value: [float; 2] = [1, 2]; }",
            "[float; 2]",
            "[int; 2]",
        ),
        (
            "fixed f64 element mismatch",
            "fn main() { let value: [f64; 2] = [1, 2]; }",
            "[float; 2]",
            "[int; 2]",
        ),
        (
            "fixed numeric length mismatch",
            "fn main() { let value: [int; 3] = [1, 2]; }",
            "[int; 3]",
            "[int; 2]",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source, expected, actual) in cases {
        expect_rejection(
            &mut failures,
            label,
            semantic_result(source),
            &[
                "Variable `value` type annotation mismatch",
                &format!("expected {expected}"),
                &format!("actual {actual}"),
            ],
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn checked_ir_rejects_selected_and_numeric_scalar_binding_mismatches() {
    let cases = [
        (
            "canonical String from int",
            "fn main() { let value: String = 1; }",
            "String",
            "int",
        ),
        (
            "bool from int",
            "fn main() { let value: bool = 1; }",
            "bool",
            "int",
        ),
        (
            "fixed numeric element mismatch",
            "fn main() { let value: [int; 2] = [1.0, 2.0]; }",
            "[int; 2]",
            "[float; 2]",
        ),
        (
            "fixed i32 element mismatch",
            "fn main() { let value: [i32; 2] = [1.0, 2.0]; }",
            "[int; 2]",
            "[float; 2]",
        ),
        (
            "fixed float element mismatch",
            "fn main() { let value: [float; 2] = [1, 2]; }",
            "[float; 2]",
            "[int; 2]",
        ),
        (
            "fixed f64 element mismatch",
            "fn main() { let value: [f64; 2] = [1, 2]; }",
            "[float; 2]",
            "[int; 2]",
        ),
        (
            "fixed numeric length mismatch",
            "fn main() { let value: [int; 3] = [1, 2]; }",
            "[int; 3]",
            "[int; 2]",
        ),
        (
            "int from float",
            "fn main() { let value: int = 1.5; }",
            "int",
            "float",
        ),
        (
            "i32 from float",
            "fn main() { let value: i32 = 1.5; }",
            "int",
            "float",
        ),
        (
            "float from int",
            "fn main() { let value: float = 1; }",
            "float",
            "int",
        ),
        (
            "f64 from int",
            "fn main() { let value: f64 = 1; }",
            "float",
            "int",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source, expected, actual) in cases {
        expect_rejection(
            &mut failures,
            label,
            checked_source(source),
            &[
                "binding `value`",
                &format!("expected {expected}"),
                &format!("actual {actual}"),
            ],
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn semantic_enforces_numeric_array_elements_indexes_and_child_precedence() {
    let mut failures = Vec::new();
    expect_rejection(
        &mut failures,
        "mixed numeric literal",
        semantic_result("fn main() { let values = [1, 2, 3.5]; }"),
        &["array", "expected int", "actual float"],
    );
    expect_rejection(
        &mut failures,
        "float-first mixed numeric literal",
        semantic_result("fn main() { let values = [1.0, 2.0, 3]; }"),
        &["array", "expected float", "actual int"],
    );
    expect_rejection(
        &mut failures,
        "float numeric-array index",
        semantic_result("fn main() { let values = [1, 2]; let item = values[1.5]; }"),
        &["array index", "expected int", "actual float"],
    );
    expect_rejection(
        &mut failures,
        "later element child error",
        semantic_result("fn main() { let values = [1, 2.5, missing_value]; }"),
        &["Use of undeclared variable `missing_value`"],
    );
    match semantic_result("fn main() { let values = [1, missing_first, missing_second]; }") {
        Ok(()) => failures.push("left-to-right child error: unexpectedly accepted".to_string()),
        Err(error) => {
            if !error.contains("Use of undeclared variable `missing_first`") {
                failures.push(format!(
                    "left-to-right child error: diagnostic {error:?} missed the first failing child"
                ));
            }
            if error.contains("missing_second") {
                failures.push(format!(
                    "left-to-right child error: diagnostic {error:?} visited the later failing child first"
                ));
            }
        }
    }
    for (label, source, forbidden) in [
        (
            "inference error precedes later child inference error",
            "fn main() { let values = [1, missing_function(), missing_later]; }",
            "missing_later",
        ),
        (
            "inference error precedes later child preflight error",
            "fn main() { let values = [1, missing_function(), (2, 3)]; }",
            "Tuple expressions are not supported",
        ),
    ] {
        match semantic_result(source) {
            Ok(()) => failures.push(format!("{label}: unexpectedly accepted")),
            Err(error) => {
                if !error.contains("Function `missing_function` is not defined") {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} missed the earlier failing child"
                    ));
                }
                if error.contains(forbidden) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} reported the later child first"
                    ));
                }
            }
        }
    }
    match semantic_result("fn main() { let values = [missing_function(), (2, 3)]; }") {
        Ok(()) => failures
            .push("non-numeric-first preflight precedence: unexpectedly accepted".to_string()),
        Err(error) => {
            if !error.contains("Tuple expressions are not supported") {
                failures.push(format!(
                    "non-numeric-first preflight precedence: diagnostic {error:?} missed the preserved later preflight error"
                ));
            }
            if error.contains("Function `missing_function` is not defined") {
                failures.push(format!(
                    "non-numeric-first preflight precedence: diagnostic {error:?} incorrectly activated numeric-first ordering"
                ));
            }
        }
    }
    for (label, source, expected, forbidden) in [
        (
            "nested numeric array retains element-major ordering",
            "fn main() { let item = [1, missing_function(), (2, 3)][0]; }",
            "Function `missing_function` is not defined",
            "Tuple expressions are not supported",
        ),
        (
            "nested unselected array retains preflight precedence",
            "fn main() { let item = [missing_function(), (2, 3)][0]; }",
            "Tuple expressions are not supported",
            "Function `missing_function` is not defined",
        ),
    ] {
        match semantic_result(source) {
            Ok(()) => failures.push(format!("{label}: unexpectedly accepted")),
            Err(error) => {
                if !error.contains(expected) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} missed expected precedence {expected:?}"
                    ));
                }
                if error.contains(forbidden) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} reported forbidden precedence {forbidden:?}"
                    ));
                }
            }
        }
    }
    match semantic_result("fn main() { let point = 1; let item = [1, (2, 3)][point.field]; }") {
        Ok(()) => failures
            .push("nested array precedes later parent sibling: unexpectedly accepted".to_string()),
        Err(error) => {
            if !error.contains("Tuple expressions are not supported") {
                failures.push(format!(
                    "nested array precedes later parent sibling: diagnostic {error:?} missed the earlier array child"
                ));
            }
            if error.contains("Field access expressions are not supported") {
                failures.push(format!(
                    "nested array precedes later parent sibling: diagnostic {error:?} advanced to the later index first"
                ));
            }
        }
    }
    expect_acceptance(
        &mut failures,
        "stub-only method argument retains pre-task numeric-array quarantine",
        semantic_result("fn main() { let values = [1, 2]; let item = values.unknown([1, 2.5]); }"),
    );
    for (label, source) in [
        (
            "closure body retains strict unsupported-child preflight",
            "fn main() { let callback = |value: int| [1, (2, 3)]; }",
        ),
        (
            "method argument retains strict unsupported-child preflight",
            "fn main() { let text = \"x\"; let value = text.unknown([1, (2, 3)]); }",
        ),
    ] {
        expect_rejection(
            &mut failures,
            label,
            semantic_result(source),
            &["Tuple expressions are not supported"],
        );
    }
    match semantic_result("fn main() { println!(\"\", [1, 2.5]); }") {
        Ok(()) => failures
            .push("format count precedes argument inference: unexpectedly accepted".to_string()),
        Err(error) => {
            if !error.contains("Format string has 0 placeholders but 1 arguments were provided") {
                failures.push(format!(
                    "format count precedes argument inference: unexpected diagnostic {error:?}"
                ));
            }
            if error.contains("array element type mismatch") {
                failures.push(format!(
                    "format count precedes argument inference: array child incorrectly preempted format validation: {error:?}"
                ));
            }
        }
    }
    expect_acceptance(
        &mut failures,
        "custom enum payload retains pre-task numeric-array quarantine",
        semantic_result(
            "enum Choice { Number(int) } fn main() { let choice = Choice::Number([1, 2.5]); }",
        ),
    );
    let deeply_nested_array = format!(
        "fn main() {{ let values = {}1{}; }}",
        "[".repeat(18),
        "]".repeat(18)
    );
    expect_acceptance(
        &mut failures,
        "deep first-element array nesting remains accepted without duplicate traversal",
        semantic_result(&deeply_nested_array),
    );
    let alternating_array_index = (0..18).fold("1".to_string(), |expression, _| {
        format!("[{expression}][0]")
    });
    expect_acceptance(
        &mut failures,
        "alternating array/index nesting reuses interleaved array inference",
        semantic_result(&format!(
            "fn main() {{ let value = {alternating_array_index}; }}"
        )),
    );
    match semantic_result("fn main() { let point = 1; let values = [(1, 2), point.field]; }") {
        Ok(()) => failures
            .push("first preflight error remains immediate: unexpectedly accepted".to_string()),
        Err(error) => {
            if !error.contains("Tuple expressions are not supported") {
                failures.push(format!(
                    "first preflight error remains immediate: diagnostic {error:?} missed the first tuple"
                ));
            }
            if error.contains("Field access expressions are not supported") {
                failures.push(format!(
                    "first preflight error remains immediate: diagnostic {error:?} advanced to the later field"
                ));
            }
        }
    }
    expect_rejection(
        &mut failures,
        "custom enum payload retains strict unsupported-child preflight",
        semantic_result(
            "enum Choice { Number(int) } fn main() { let choice = Choice::Number([1, (2, 3)]); }",
        ),
        &["Tuple expressions are not supported"],
    );
    assert!(
        !failures
            .iter()
            .any(|failure| failure.starts_with("later element child error")),
        "later child diagnostic did not retain precedence:\n{}",
        failures.join("\n")
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn checked_ir_rejects_spoofed_binary_result_metadata_everywhere() {
    let direct_unannotated = vec![AstNode::Statement(binding(
        "value",
        None,
        binary_with_metadata(Some(Ty::Float)),
    ))];
    let direct_excluded = vec![AstNode::Statement(binding(
        "value",
        Some(Type::Named("Widget".to_string())),
        binary_with_metadata(Some(Ty::Float)),
    ))];
    let direct_selected = vec![AstNode::Statement(binding(
        "value",
        Some(Type::Named("bool".to_string())),
        binary_with_metadata(Some(Ty::Float)),
    ))];
    let generic_impl = generic_impl_with(binding(
        "value",
        Some(Type::Named("bool".to_string())),
        binary_with_metadata(Some(Ty::Float)),
    ));
    let float_derived = vec![AstNode::Statement(binding(
        "value",
        None,
        float_binary_with_metadata(Some(Ty::Int)),
    ))];

    let mut failures = Vec::new();
    for (label, ast, expected, actual) in [
        (
            "unannotated direct binding",
            direct_unannotated,
            "int",
            "float",
        ),
        ("excluded direct binding", direct_excluded, "int", "float"),
        (
            "selected direct binding metadata precedes annotation",
            direct_selected,
            "int",
            "float",
        ),
        ("generic impl binding", generic_impl, "int", "float"),
        (
            "float-derived direct binding",
            float_derived,
            "float",
            "int",
        ),
    ] {
        expect_rejection(
            &mut failures,
            label,
            checked_ast(ast),
            &[
                "binary",
                &format!("expected {expected}"),
                &format!("actual {actual}"),
            ],
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn non_generic_impl_rejects_selected_mismatches_at_both_frontend_boundaries() {
    let source = "impl Widget { fn probe() { let value: bool = 1; } }";
    let mut failures = Vec::new();
    expect_rejection(
        &mut failures,
        "semantic non-generic impl",
        semantic_result(source),
        &[
            "Variable `value` type annotation mismatch",
            "expected bool",
            "actual int",
        ],
    );
    expect_rejection(
        &mut failures,
        "checked-IR non-generic impl",
        checked_source(source),
        &["binding `value`", "expected bool", "actual int"],
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn exact_selected_and_unannotated_controls_pass_semantics_and_checked_ir() {
    let cases = [
        ("int scalar", "fn main() { let value: int = 1; }"),
        ("i32 scalar", "fn main() { let value: i32 = 1; }"),
        ("float scalar", "fn main() { let value: float = 1.5; }"),
        ("f64 scalar", "fn main() { let value: f64 = 1.5; }"),
        ("bool", "fn main() { let value: bool = 1 < 2; }"),
        ("String", "fn main() { let value: String = \"aero\"; }"),
        ("int array", "fn main() { let value: [int; 2] = [1, 2]; }"),
        ("i32 array", "fn main() { let value: [i32; 2] = [1, 2]; }"),
        (
            "float array",
            "fn main() { let value: [float; 2] = [1.0, 2.0]; }",
        ),
        (
            "f64 array",
            "fn main() { let value: [f64; 2] = [1.0, 2.0]; }",
        ),
        (
            "unannotated selected values",
            "fn main() { let flag = 1 < 2; let text = \"aero\"; let values = [1, 2]; }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in cases {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn tuple_annotations_fail_closed_before_binding_insertion_or_generation() {
    const INITIALIZED_SOURCE: &str = "fn main() { let value: (int, float) = 1; }";
    const INITIALIZED_SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation for an initialized binding.";
    const INITIALIZED_CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation for an initialized binding";
    const INITIALIZED_PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation for an initialized binding.";
    const UNINITIALIZED_SOURCE: &str = "fn main() { let value: (int, float); }";
    const UNINITIALIZED_SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation for an uninitialized binding.";
    const UNINITIALIZED_CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation for an uninitialized binding";
    const UNINITIALIZED_PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation for an uninitialized binding.";

    let mut failures = Vec::new();
    expect_exact_rejection(
        &mut failures,
        "initialized ordinary direct semantics",
        semantic_result(INITIALIZED_SOURCE),
        INITIALIZED_SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "initialized ordinary checked admission",
        checked_source(INITIALIZED_SOURCE),
        INITIALIZED_CHECKED_ERROR,
    );

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(INITIALIZED_SOURCE, CompilerOptions::default())
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "initialized public compile_program",
        public_result,
        INITIALIZED_PUBLIC_ERROR,
    );

    let generic_impl = generic_impl_with(binding(
        "value",
        Some(Type::Tuple(vec![
            Type::Named("int".to_string()),
            Type::Named("float".to_string()),
        ])),
        Expression::IntegerLiteral(1),
    ));
    let generic_semantics = {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(generic_impl.clone()).map(|_| ())
    };
    expect_exact_rejection(
        &mut failures,
        "generic impl direct semantics",
        generic_semantics,
        INITIALIZED_SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "generic impl checked admission",
        checked_ast(generic_impl),
        INITIALIZED_CHECKED_ERROR,
    );

    expect_exact_rejection(
        &mut failures,
        "tuple RHS semantic child precedence",
        semantic_result("fn main() { let value: (int, float) = (1, 2); }"),
        "Tuple expressions are not supported.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS checked child precedence",
        checked_source("fn main() { let value: (int, float) = (1, 2); }"),
        "aggregate expression is not admitted in checked IR",
    );
    expect_exact_rejection(
        &mut failures,
        "uninitialized tuple annotation semantics",
        semantic_result(UNINITIALIZED_SOURCE),
        UNINITIALIZED_SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "uninitialized tuple annotation checked admission",
        checked_source(UNINITIALIZED_SOURCE),
        UNINITIALIZED_CHECKED_ERROR,
    );

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(UNINITIALIZED_SOURCE, CompilerOptions::default())
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "uninitialized public compile_program",
        public_result,
        UNINITIALIZED_PUBLIC_ERROR,
    );

    let generic_impl = generic_impl_with(uninitialized_binding(
        "value",
        Type::Tuple(vec![
            Type::Named("int".to_string()),
            Type::Named("float".to_string()),
        ]),
    ));
    let generic_semantics = {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(generic_impl.clone()).map(|_| ())
    };
    expect_exact_rejection(
        &mut failures,
        "uninitialized generic impl direct semantics",
        generic_semantics,
        UNINITIALIZED_SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "uninitialized generic impl checked admission",
        checked_ast(generic_impl),
        UNINITIALIZED_CHECKED_ERROR,
    );

    expect_exact_rejection(
        &mut failures,
        "uninitialized tuple duplicate semantic precedence",
        semantic_result("fn main() { let value = 1; let value: (int, float); }"),
        "Error: Variable `value` is already defined in this scope.",
    );

    let outer_shape_controls = [
        ("scalar", "fn main() { let value: int; }"),
        (
            "generic containing tuple",
            "fn main() { let value: Vec<(int, float)>; }",
        ),
    ];
    for (label, source) in outer_shape_controls {
        expect_acceptance(
            &mut failures,
            &format!("uninitialized {label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("uninitialized {label} checked admission"),
            checked_source(source),
        );
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn immediate_reference_to_tuple_annotations_fail_closed_before_integer_fallback() {
    const IMMUTABLE_SOURCE: &str = "fn main() { let value: &(int, float); }";
    const MUTABLE_SOURCE: &str = "fn main() { let value: &mut (int, float); }";
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.";
    const DUPLICATE_ERROR: &str = "Error: Variable `value` is already defined in this scope.";

    let mut failures = Vec::new();
    expect_exact_rejection(
        &mut failures,
        "immutable immediate reference-to-tuple direct semantics",
        semantic_result(IMMUTABLE_SOURCE),
        SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "immutable immediate reference-to-tuple checked admission",
        checked_source(IMMUTABLE_SOURCE),
        CHECKED_ERROR,
    );
    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(IMMUTABLE_SOURCE, CompilerOptions::default())
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "immutable immediate reference-to-tuple public compilation",
        public_result,
        PUBLIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "mutable immediate reference-to-tuple direct semantics",
        semantic_result(MUTABLE_SOURCE),
        SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "mutable immediate reference-to-tuple checked admission",
        checked_source(MUTABLE_SOURCE),
        CHECKED_ERROR,
    );

    for source in [
        "fn main() { let value = 1; let value: &(int, float); }",
        "fn main() { let value = 1; let value: &mut (int, float); }",
    ] {
        expect_exact_rejection(
            &mut failures,
            "immediate reference-to-tuple duplicate semantic precedence",
            semantic_result(source),
            DUPLICATE_ERROR,
        );
    }

    expect_exact_rejection(
        &mut failures,
        "accepted outer-tuple semantic diagnostic",
        semantic_result("fn main() { let value: (int, float); }"),
        "Error: Variable `value` uses an unsupported tuple type annotation for an uninitialized binding.",
    );
    expect_exact_rejection(
        &mut failures,
        "accepted outer-tuple checked diagnostic",
        checked_source("fn main() { let value: (int, float); }"),
        "checked IR binding `value` uses an unsupported tuple type annotation for an uninitialized binding",
    );

    let preservation_cases = [
        ("scalar", "fn main() { let value: int; }"),
        ("scalar reference", "fn main() { let value: &int; }"),
        (
            "mutable scalar reference",
            "fn main() { let value: &mut int; }",
        ),
        (
            "generic containing tuple",
            "fn main() { let value: Vec<(int, float)>; }",
        ),
        (
            "second reference layer containing tuple",
            "fn main() { let value: & &(int, float); }",
        ),
        (
            "mutable second reference layer containing tuple",
            "fn main() { let value: &mut &(int, float); }",
        ),
    ];
    for (label, source) in preservation_cases {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
        );
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn initialized_immediate_reference_to_tuple_annotations_fail_closed_after_value_validation() {
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding.";
    const IMMUTABLE_SOURCE: &str = "fn main() { let value: &(int, float) = 1; }";
    const MUTABLE_SOURCE: &str = "fn main() { let value: &mut (int, float) = 1; }";

    let mut failures = Vec::new();

    for (label, source) in [
        ("immutable direct", IMMUTABLE_SOURCE),
        ("mutable direct", MUTABLE_SOURCE),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    for (label, source) in [
        ("immutable public", IMMUTABLE_SOURCE),
        ("mutable public", MUTABLE_SOURCE),
    ] {
        let result = match catch_unwind(AssertUnwindSafe(|| {
            compile_program(source, CompilerOptions::default())
        })) {
            Err(_) => Err("compile_program unwound".to_string()),
            Ok(Ok(_)) => Ok(()),
            Ok(Err(error)) => Err(error),
        };
        expect_exact_rejection(&mut failures, label, result, PUBLIC_ERROR);
    }

    for (label, source) in [
        ("immutable top-level", "let value: &(int, float) = 1;"),
        ("mutable top-level", "let value: &mut (int, float) = 1;"),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    for (label, source) in [
        (
            "immutable generic impl",
            "impl<T> Widget { fn probe() { let value: &(int, float) = 1; } }",
        ),
        (
            "mutable generic impl",
            "impl<T> Widget { fn probe() { let value: &mut (int, float) = 1; } }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    for (label, source) in [
        (
            "immutable generic function",
            "fn probe<T>() { let value: &(int, float) = 1; }",
        ),
        (
            "mutable generic function",
            "fn probe<T>() { let value: &mut (int, float) = 1; }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
    }
    expect_rejection(
        &mut failures,
        "generic function checked admission retains outer rejection",
        checked_source("fn probe<T>() { let value: &(int, float) = 1; }"),
        &["generic function IR is not admitted"],
    );

    for (label, source) in [
        (
            "explicit block",
            "fn main() { { let value: &(int, float) = 1; } }",
        ),
        (
            "if then",
            "fn main() { if 1 < 2 { let value: &(int, float) = 1; } }",
        ),
        (
            "if else",
            "fn main() { if 1 < 2 { let keep = 1; } else { let value: &(int, float) = 1; } }",
        ),
        (
            "while",
            "fn main() { while 1 < 2 { let value: &(int, float) = 1; break; } }",
        ),
        (
            "for",
            "fn main() { for item in [1] { let value: &(int, float) = 1; } }",
        ),
        (
            "loop",
            "fn main() { loop { let value: &(int, float) = 1; break; } }",
        ),
        (
            "non-generic impl",
            "impl Widget { fn probe() { let value: &(int, float) = 1; } }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    for source in [
        "fn main() { let value = 0; let value: &(int, float) = 1; }",
        "fn main() { let value = 0; let value: &mut (int, float) = 1; }",
    ] {
        expect_exact_rejection(
            &mut failures,
            "duplicate semantic precedence",
            semantic_result(source),
            "Error: Variable `value` is already defined in this scope.",
        );
    }
    expect_exact_rejection(
        &mut failures,
        "tuple RHS semantic child precedence",
        semantic_result("fn main() { let value: &(int, float) = (1, 2); }"),
        "Tuple expressions are not supported.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS checked child precedence",
        checked_source("fn main() { let value: &(int, float) = (1, 2); }"),
        "aggregate expression is not admitted in checked IR",
    );
    expect_rejection(
        &mut failures,
        "undefined RHS semantic child precedence",
        semantic_result("fn main() { let value: &(int, float) = missing_value; }"),
        &["Use of undeclared variable `missing_value`"],
    );
    expect_rejection(
        &mut failures,
        "undefined RHS checked child precedence",
        checked_source("fn main() { let value: &(int, float) = missing_value; }"),
        &["checked IR has no binding for `missing_value`"],
    );
    const VOID_SOURCE: &str = "fn notify() {} fn main() { let value: &(int, float) = notify(); }";
    expect_rejection(
        &mut failures,
        "Void RHS semantic child precedence",
        semantic_result(VOID_SOURCE),
        &["void function `notify` cannot be used as a value"],
    );
    expect_exact_rejection(
        &mut failures,
        "Void RHS checked child precedence",
        checked_source(VOID_SOURCE),
        "Void function calls cannot be used as values",
    );

    for (label, source, semantic_error, checked_error) in [
        (
            "initialized outer tuple",
            "fn main() { let value: (int, float) = 1; }",
            "Error: Variable `value` uses an unsupported tuple type annotation for an initialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation for an initialized binding",
        ),
        (
            "initialized immediate array-to-tuple",
            "fn main() { let value: [(int, float); 1] = 1; }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an initialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath an array for an initialized binding",
        ),
        (
            "initialized two-array-to-tuple",
            "fn main() { let value: [[(int, float); 1]; 1] = 1; }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} semantic diagnostic"),
            semantic_result(source),
            semantic_error,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked diagnostic"),
            checked_source(source),
            checked_error,
        );
    }

    for source in [
        "fn main() { let value: &(int, float); }",
        "fn main() { let value: &mut (int, float); }",
    ] {
        expect_exact_rejection(
            &mut failures,
            "valueless direct reference semantic diagnostic",
            semantic_result(source),
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.",
        );
        expect_exact_rejection(
            &mut failures,
            "valueless direct reference checked diagnostic",
            checked_source(source),
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding",
        );
    }

    for (label, source) in [
        ("scalar reference", "fn main() { let value: &int = 1; }"),
        (
            "mutable scalar reference",
            "fn main() { let value: &mut int = 1; }",
        ),
        (
            "double reference",
            "fn main() { let value: & &(int, float) = 1; }",
        ),
        (
            "mutable double reference",
            "fn main() { let value: &mut &(int, float) = 1; }",
        ),
        (
            "reference around array",
            "fn main() { let value: &[(int, float); 1] = 1; }",
        ),
        (
            "array around reference",
            "fn main() { let value: [&(int, float); 1] = 1; }",
        ),
        (
            "generic wrapper",
            "fn main() { let value: Vec<(int, float)> = 1; }",
        ),
        (
            "deeper reference",
            "fn main() { let value: & & &(int, float) = 1; }",
        ),
        (
            "reference around generic wrapper",
            "fn main() { let value: &Vec<(int, float)> = 1; }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
        );
    }

    const TRAIT_DEFAULT_SOURCE: &str =
        "trait Contract<T> { fn probe() { let value: &(int, float) = 1; } }";
    expect_acceptance(
        &mut failures,
        "generic trait default remains syntax-only in semantics",
        semantic_result(TRAIT_DEFAULT_SOURCE),
    );
    expect_acceptance(
        &mut failures,
        "generic trait default remains syntax-only in checked admission",
        checked_source(TRAIT_DEFAULT_SOURCE),
    );

    let valid_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let values: [int; 1] = [1]; let first = values[0]; println!(\"{}\", first); }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("valid compile_program unwound".to_string()),
        Ok(Ok(llvm)) if llvm.is_empty() => {
            Err("valid compile_program returned empty LLVM".to_string())
        }
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_acceptance(&mut failures, "valid numeric-array output", valid_result);

    assert!(
        failures.is_empty(),
        "{} binding contract failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn valueless_immediate_array_of_tuple_annotation_fails_closed_before_integer_fallback() {
    const NONZERO_SOURCE: &str = "fn main() { let value: [(int, float); 1]; }";
    const ZERO_SOURCE: &str = "fn main() { let value: [(int, float); 0]; }";
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding.";

    let mut failures = Vec::new();
    for (label, source) in [
        ("nonzero count", NONZERO_SOURCE),
        ("zero count", ZERO_SOURCE),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} direct semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(NONZERO_SOURCE, CompilerOptions::default())
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "public compilation",
        public_result,
        PUBLIC_ERROR,
    );

    expect_exact_rejection(
        &mut failures,
        "duplicate semantic precedence",
        semantic_result("fn main() { let value = 1; let value: [(int, float); 1]; }"),
        "Error: Variable `value` is already defined in this scope.",
    );

    for (label, source) in [
        ("scalar array", "fn main() { let value: [int; 1]; }"),
        (
            "generic containing tuple",
            "fn main() { let value: Vec<(int, float)>; }",
        ),
        (
            "reference containing array containing tuple",
            "fn main() { let value: &[(int, float); 1]; }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
        );
    }

    expect_exact_rejection(
        &mut failures,
        "accepted outer-tuple semantic diagnostic",
        semantic_result("fn main() { let value: (int, float); }"),
        "Error: Variable `value` uses an unsupported tuple type annotation for an uninitialized binding.",
    );
    expect_exact_rejection(
        &mut failures,
        "accepted outer-tuple checked diagnostic",
        checked_source("fn main() { let value: (int, float); }"),
        "checked IR binding `value` uses an unsupported tuple type annotation for an uninitialized binding",
    );
    expect_exact_rejection(
        &mut failures,
        "accepted immediate reference-to-tuple semantic diagnostic",
        semantic_result("fn main() { let value: &(int, float); }"),
        "Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.",
    );
    expect_exact_rejection(
        &mut failures,
        "accepted immediate reference-to-tuple checked diagnostic",
        checked_source("fn main() { let value: &(int, float); }"),
        "checked IR binding `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding",
    );

    let valid_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let values: [int; 1] = [1]; let first = values[0]; println!(\"{}\", first); }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("valid compile_program unwound".to_string()),
        Ok(Ok(llvm)) if llvm.is_empty() => {
            Err("valid compile_program returned empty LLVM".to_string())
        }
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_acceptance(&mut failures, "valid numeric-array output", valid_result);

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn initialized_immediate_array_of_tuple_annotation_fails_closed_after_value_validation() {
    const NONZERO_SOURCE: &str = "fn main() { let value: [(int, float); 1] = [1]; }";
    const ZERO_SOURCE: &str = "fn main() { let value: [(int, float); 0] = 1; }";
    const PUBLIC_SOURCE: &str = "fn main() { let value: [(int, float); 1] = 1; }";
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an initialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath an array for an initialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an initialized binding.";

    let mut failures = Vec::new();
    for (label, source) in [
        ("nonzero count", NONZERO_SOURCE),
        ("zero count", ZERO_SOURCE),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} direct semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(PUBLIC_SOURCE, CompilerOptions::default())
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "public compilation",
        public_result,
        PUBLIC_ERROR,
    );

    let generic_impl = generic_impl_with(binding(
        "value",
        Some(Type::Array(
            Box::new(Type::Tuple(vec![
                Type::Named("int".to_string()),
                Type::Named("float".to_string()),
            ])),
            1,
        )),
        Expression::IntegerLiteral(1),
    ));
    let generic_semantics = {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(generic_impl.clone()).map(|_| ())
    };
    expect_exact_rejection(
        &mut failures,
        "generic impl direct semantics",
        generic_semantics,
        SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "generic impl checked admission",
        checked_ast(generic_impl),
        CHECKED_ERROR,
    );

    const GENERIC_FUNCTION_SOURCE: &str = "fn probe<T>() { let value: [(int, float); 1] = 1; }";
    expect_exact_rejection(
        &mut failures,
        "generic function direct semantics",
        semantic_result(GENERIC_FUNCTION_SOURCE),
        SEMANTIC_ERROR,
    );
    expect_rejection(
        &mut failures,
        "generic function checked admission retains outer rejection",
        checked_source(GENERIC_FUNCTION_SOURCE),
        &["generic function IR is not admitted"],
    );

    expect_exact_rejection(
        &mut failures,
        "duplicate semantic precedence",
        semantic_result("fn main() { let value = 1; let value: [(int, float); 1] = 1; }"),
        "Error: Variable `value` is already defined in this scope.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS semantic child precedence",
        semantic_result("fn main() { let value: [(int, float); 1] = (1, 2); }"),
        "Tuple expressions are not supported.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS checked child precedence",
        checked_source("fn main() { let value: [(int, float); 1] = (1, 2); }"),
        "aggregate expression is not admitted in checked IR",
    );

    for (label, source, semantic_error, checked_error) in [
        (
            "initialized outer tuple",
            "fn main() { let value: (int, float) = 1; }",
            "Error: Variable `value` uses an unsupported tuple type annotation for an initialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation for an initialized binding",
        ),
        (
            "valueless immediate array-to-tuple",
            "fn main() { let value: [(int, float); 1]; }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding",
        ),
        (
            "valueless two-array-deep tuple",
            "fn main() { let value: [[(int, float); 1]; 1]; }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an uninitialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath two array layers for an uninitialized binding",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("accepted {label} semantic diagnostic"),
            semantic_result(source),
            semantic_error,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("accepted {label} checked diagnostic"),
            checked_source(source),
            checked_error,
        );
    }

    for (label, source) in [
        (
            "Candidate T valueless three-array depth",
            "fn main() { let value: [[[(int, float); 1]; 1]; 1]; }",
        ),
        (
            "Candidate B valueless reference-array-tuple",
            "fn main() { let value: &[(int, float); 1]; }",
        ),
        (
            "initialized reference-wrapped target",
            "fn main() { let value: &[(int, float); 1] = 1; }",
        ),
        (
            "initialized generic wrapper",
            "fn main() { let value: Vec<(int, float)> = 1; }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
        );
    }

    let valid_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let values: [int; 1] = [1]; let first = values[0]; println!(\"{}\", first); }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("valid compile_program unwound".to_string()),
        Ok(Ok(llvm)) if llvm.is_empty() => {
            Err("valid compile_program returned empty LLVM".to_string())
        }
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_acceptance(&mut failures, "valid numeric-array output", valid_result);

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn initialized_immediate_array_of_array_of_tuple_annotation_fails_closed_after_value_validation() {
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding.";

    let mut failures = Vec::new();
    for (label, source) in [
        (
            "zero outer and zero inner counts",
            "fn main() { let value: [[(int, float); 0]; 0] = 1; }",
        ),
        (
            "nonzero outer and zero inner counts",
            "fn main() { let value: [[(int, float); 0]; 1] = 1; }",
        ),
        (
            "zero outer and nonzero inner counts",
            "fn main() { let value: [[(int, float); 1]; 0] = 1; }",
        ),
        (
            "nonzero outer and nonzero inner counts",
            "fn main() { let value: [[(int, float); 1]; 1] = 1; }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} direct semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let value: [[(int, float); 1]; 1] = 1; }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "public compilation",
        public_result,
        PUBLIC_ERROR,
    );

    let target_annotation = || {
        Type::Array(
            Box::new(Type::Array(
                Box::new(Type::Tuple(vec![
                    Type::Named("int".to_string()),
                    Type::Named("float".to_string()),
                ])),
                1,
            )),
            1,
        )
    };
    let generic_impl = generic_impl_with(binding(
        "value",
        Some(target_annotation()),
        Expression::IntegerLiteral(1),
    ));
    let generic_semantics = {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(generic_impl.clone()).map(|_| ())
    };
    expect_exact_rejection(
        &mut failures,
        "generic impl direct semantics",
        generic_semantics,
        SEMANTIC_ERROR,
    );
    expect_exact_rejection(
        &mut failures,
        "generic impl checked admission",
        checked_ast(generic_impl),
        CHECKED_ERROR,
    );

    const GENERIC_FUNCTION_SOURCE: &str =
        "fn probe<T>() { let value: [[(int, float); 1]; 1] = 1; }";
    expect_exact_rejection(
        &mut failures,
        "generic function direct semantics",
        semantic_result(GENERIC_FUNCTION_SOURCE),
        SEMANTIC_ERROR,
    );
    expect_rejection(
        &mut failures,
        "generic function checked admission retains outer rejection",
        checked_source(GENERIC_FUNCTION_SOURCE),
        &["generic function IR is not admitted"],
    );

    expect_exact_rejection(
        &mut failures,
        "duplicate semantic precedence",
        semantic_result("fn main() { let value = 1; let value: [[(int, float); 1]; 1] = 1; }"),
        "Error: Variable `value` is already defined in this scope.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS semantic child precedence",
        semantic_result("fn main() { let value: [[(int, float); 1]; 1] = (1, 2); }"),
        "Tuple expressions are not supported.",
    );
    expect_exact_rejection(
        &mut failures,
        "tuple RHS checked child precedence",
        checked_source("fn main() { let value: [[(int, float); 1]; 1] = (1, 2); }"),
        "aggregate expression is not admitted in checked IR",
    );
    const INITIALIZED_THREE_ARRAY_SOURCE: &str =
        "fn main() { let value: [[[(int, float); 1]; 1]; 1] = 1; }";
    expect_acceptance(
        &mut failures,
        "initialized three-array-depth semantics",
        semantic_result(INITIALIZED_THREE_ARRAY_SOURCE),
    );
    expect_acceptance(
        &mut failures,
        "initialized three-array-depth checked admission",
        checked_source(INITIALIZED_THREE_ARRAY_SOURCE),
    );

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn valueless_immediate_array_of_array_of_tuple_annotation_fails_closed_before_integer_fallback() {
    const SEMANTIC_ERROR: &str = "Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an uninitialized binding.";
    const CHECKED_ERROR: &str = "checked IR binding `value` uses an unsupported tuple type annotation directly beneath two array layers for an uninitialized binding";
    const PUBLIC_ERROR: &str = "Semantic Analysis Error: Error: Variable `value` uses an unsupported tuple type annotation directly beneath two array layers for an uninitialized binding.";

    let mut failures = Vec::new();
    for (label, source) in [
        (
            "zero outer and zero inner counts",
            "fn main() { let value: [[(int, float); 0]; 0]; }",
        ),
        (
            "nonzero outer and zero inner counts",
            "fn main() { let value: [[(int, float); 0]; 1]; }",
        ),
        (
            "zero outer and nonzero inner counts",
            "fn main() { let value: [[(int, float); 1]; 0]; }",
        ),
        (
            "nonzero outer and nonzero inner counts",
            "fn main() { let value: [[(int, float); 1]; 1]; }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} direct semantics"),
            semantic_result(source),
            SEMANTIC_ERROR,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
            CHECKED_ERROR,
        );
    }

    let public_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let value: [[(int, float); 1]; 1]; }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("compile_program unwound".to_string()),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_exact_rejection(
        &mut failures,
        "public compilation",
        public_result,
        PUBLIC_ERROR,
    );

    expect_exact_rejection(
        &mut failures,
        "duplicate semantic precedence",
        semantic_result("fn main() { let value = 1; let value: [[(int, float); 1]; 1]; }"),
        "Error: Variable `value` is already defined in this scope.",
    );

    for (label, source) in [
        ("scalar array", "fn main() { let value: [int; 1]; }"),
        (
            "nested scalar array",
            "fn main() { let value: [[int; 1]; 1]; }",
        ),
        (
            "generic containing tuple",
            "fn main() { let value: Vec<(int, float)>; }",
        ),
        (
            "reference containing array containing tuple",
            "fn main() { let value: &[(int, float); 1]; }",
        ),
        (
            "reference containing target shape",
            "fn main() { let value: &[[(int, float); 1]; 1]; }",
        ),
        (
            "array containing reference to tuple",
            "fn main() { let value: [&(int, float); 1]; }",
        ),
        (
            "third array layer containing tuple",
            "fn main() { let value: [[[(int, float); 1]; 1]; 1]; }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked admission"),
            checked_source(source),
        );
    }

    for (label, source, semantic_error, checked_error) in [
        (
            "outer tuple",
            "fn main() { let value: (int, float); }",
            "Error: Variable `value` uses an unsupported tuple type annotation for an uninitialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation for an uninitialized binding",
        ),
        (
            "immediate reference-to-tuple",
            "fn main() { let value: &(int, float); }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding",
        ),
        (
            "immediate array-to-tuple",
            "fn main() { let value: [(int, float); 1]; }",
            "Error: Variable `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding.",
            "checked IR binding `value` uses an unsupported tuple type annotation directly beneath an array for an uninitialized binding",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("accepted {label} semantic diagnostic"),
            semantic_result(source),
            semantic_error,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("accepted {label} checked diagnostic"),
            checked_source(source),
            checked_error,
        );
    }

    let valid_result = match catch_unwind(AssertUnwindSafe(|| {
        compile_program(
            "fn main() { let values: [int; 1] = [1]; let first = values[0]; println!(\"{}\", first); }",
            CompilerOptions::default(),
        )
    })) {
        Err(_) => Err("valid compile_program unwound".to_string()),
        Ok(Ok(llvm)) if llvm.is_empty() => {
            Err("valid compile_program returned empty LLVM".to_string())
        }
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
    };
    expect_acceptance(&mut failures, "valid numeric-array output", valid_result);

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn excluded_annotations_remain_ignored_at_direct_frontend_boundaries() {
    let cases = [
        ("lowercase string", "fn main() { let value: string = 1; }"),
        ("custom named", "fn main() { let value: Widget = 1; }"),
        ("generic", "fn main() { let value: Vec<int> = 1; }"),
        ("reference", "fn main() { let value: &int = 1; }"),
        ("bool array", "fn main() { let value: [bool; 1] = [1]; }"),
        (
            "String array",
            "fn main() { let value: [String; 1] = [1]; }",
        ),
        (
            "string array",
            "fn main() { let value: [string; 1] = [1]; }",
        ),
        (
            "nested numeric array",
            "fn main() { let value: [[int; 1]; 1] = [1]; }",
        ),
        (
            "custom array",
            "fn main() { let value: [Widget; 1] = [1]; }",
        ),
        (
            "generic array",
            "fn main() { let value: [Vec<int>; 1] = [1]; }",
        ),
        (
            "reference array",
            "fn main() { let value: [&int; 1] = [1]; }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in cases {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn generic_semantic_quarantine_and_existing_numeric_contract_are_preserved() {
    let accepted = [
        (
            "generic function annotations",
            r#"
fn probe<T>() {
    let contextual: T = 1;
    let flag: bool = 1;
    let text: String = 1;
    let legacy: string = 1;
    let values: [int; 2] = [1.0, 2.0];
    let sized: [int; 3] = [1, 2];
    let mixed = [1, 2.5];
    let indexed = [1, 2][1.5];
}
"#,
        ),
        (
            "generic impl annotations",
            r#"
impl<T> Widget {
    fn probe() {
        let contextual: T = 1;
        let flag: bool = 1;
        let text: String = 1;
        let legacy: string = 1;
        let values: [int; 2] = [1.0, 2.0];
        let sized: [int; 3] = [1, 2];
        let mixed = [1, 2.5];
        let indexed = [1, 2][1.5];
    }
}
"#,
        ),
        (
            "generic trait default syntax preflight",
            r#"
trait Contract<T> {
    fn probe() {
        let contextual: T = 1;
        let flag: bool = 1;
        let numeric: int = 1.5;
        let mixed = [1, 2.5];
    }
}
"#,
        ),
    ];
    let rejected = [
        (
            "generic function numeric scalar",
            "fn probe<T>() { let value: int = 1.5; }",
        ),
        (
            "generic impl numeric scalar",
            "impl<T> Widget { fn probe() { let value: int = 1.5; } }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in accepted {
        expect_acceptance(&mut failures, label, semantic_result(source));
    }
    for (label, source) in rejected {
        expect_rejection(
            &mut failures,
            label,
            semantic_result(source),
            &[
                "Variable `value` type annotation mismatch",
                "expected int",
                "actual float",
            ],
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn checked_ir_generic_and_trait_quarantine_is_preserved() {
    let mut failures = Vec::new();
    expect_rejection(
        &mut failures,
        "generic function remains rejected before body admission",
        checked_source("fn probe<T>() { let value: bool = 1; }"),
        &["generic function IR is not admitted"],
    );
    for (label, source) in [
        (
            "generic impl ignores selected binding annotations",
            r#"
impl<T> Widget {
    fn probe() {
        let numeric: int = 1.5;
        let flag: bool = 1;
        let text: String = 1;
        let values: [int; 2] = [1.0, 2.0];
        let sized: [int; 3] = [1, 2];
    }
}
"#,
        ),
        (
            "generic impl admits mixed numeric array",
            "impl<T> Widget { fn probe() { let values = [1, 2.5]; } }",
        ),
        (
            "generic trait body remains syntax-only",
            "trait Contract<T> { fn probe() { let value: bool = 1; } }",
        ),
    ] {
        expect_acceptance(&mut failures, label, checked_source(source));
    }
    expect_rejection(
        &mut failures,
        "generic impl retains checked-IR index rejection",
        checked_source(
            "impl<T> Widget { fn probe() { let values = [1, 2]; let item = values[1.5]; } }",
        ),
        &["array index must be Int"],
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn nonnumeric_array_semantic_behavior_remains_quarantined() {
    let cases = [
        (
            "heterogeneous String-first array",
            "fn main() { let values = [\"aero\", 1]; }",
        ),
        (
            "float index on String array",
            "fn main() { let values = [\"aero\"]; let item = values[1.5]; }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in cases {
        expect_acceptance(&mut failures, label, semantic_result(source));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn empty_literals_and_typed_zero_repeats_keep_phase_specific_behavior() {
    let empty_cases = [
        ("unannotated empty", "fn main() { let values = []; }"),
        (
            "int-annotated empty",
            "fn main() { let values: [int; 0] = []; }",
        ),
        (
            "float-annotated empty",
            "fn main() { let values: [float; 0] = []; }",
        ),
    ];
    let repeat_cases = [
        (
            "float annotation with int zero repeat",
            "fn main() { let values: [float; 0] = [1; 0]; }",
        ),
        (
            "int annotation with float zero repeat",
            "fn main() { let values: [int; 0] = [1.5; 0]; }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in empty_cases {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_rejection(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
            &["empty array literals have no admitted logical element type"],
        );
    }
    for (label, source) in repeat_cases {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            semantic_result(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn matching_and_absent_binary_metadata_remain_admitted() {
    let cases = [
        (
            "matching metadata",
            vec![AstNode::Statement(binding(
                "value",
                None,
                binary_with_metadata(Some(Ty::Int)),
            ))],
        ),
        (
            "absent metadata",
            vec![AstNode::Statement(binding(
                "value",
                None,
                binary_with_metadata(None),
            ))],
        ),
        (
            "matching metadata in generic impl",
            generic_impl_with(binding(
                "value",
                Some(Type::Named("bool".to_string())),
                binary_with_metadata(Some(Ty::Int)),
            )),
        ),
        (
            "matching float metadata",
            vec![AstNode::Statement(binding(
                "value",
                None,
                float_binary_with_metadata(Some(Ty::Float)),
            ))],
        ),
    ];
    let mut failures = Vec::new();
    for (label, ast) in cases {
        expect_acceptance(&mut failures, label, checked_ast(ast));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn uninitialized_use_and_duplicate_diagnostic_precedence_remain_unchanged() {
    let mut failures = Vec::new();
    expect_rejection(
        &mut failures,
        "uninitialized selected binding use",
        semantic_result("fn main() { let value: bool; let observed = value; }"),
        &["Use of uninitialized variable `value`"],
    );
    expect_rejection(
        &mut failures,
        "duplicate binding",
        semantic_result("fn main() { let value = 1; let value: bool = 1; }"),
        &["Variable `value` is already defined in this scope"],
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn analyzer_reuse_cleans_generic_scope_after_error() {
    let mut analyzer = SemanticAnalyzer::new();
    let first = analyzer.analyze(parsed(
        "impl<T> Widget { fn probe() { let numeric: int = 1.5; } }",
    ));
    let first_error = first.expect_err("existing numeric mismatch must fail generic impl");
    assert!(
        first_error.contains("Variable `numeric` type annotation mismatch"),
        "unexpected first diagnostic: {first_error}"
    );

    let second = analyzer.analyze(parsed("fn main() { let value: bool = 1; }"));
    let second_error = second.expect_err(
        "reused analyzer must reject a non-generic selected mismatch after generic failure",
    );
    assert!(
        second_error.contains("Variable `value` type annotation mismatch")
            && second_error.contains("expected bool")
            && second_error.contains("actual int"),
        "unexpected second diagnostic: {second_error}"
    );
}

#[test]
fn public_library_rejects_selected_mismatches_without_unwinding() {
    let cases = [
        ("String", "fn main() { let value: String = 1; }"),
        ("bool", "fn main() { let value: bool = 1; }"),
        (
            "array element",
            "fn main() { let value: [int; 2] = [1.0, 2.0]; }",
        ),
        (
            "array length",
            "fn main() { let value: [int; 3] = [1, 2]; }",
        ),
    ];
    let mut failures = Vec::new();
    for (label, source) in cases {
        match catch_unwind(AssertUnwindSafe(|| {
            compile_program(source, CompilerOptions::default())
        })) {
            Err(_) => failures.push(format!("{label}: compile_program unwound")),
            Ok(Ok(_)) => failures.push(format!("{label}: compile_program unexpectedly accepted")),
            Ok(Err(error)) => {
                if !error.contains("Semantic Analysis Error:")
                    || !error.contains("type annotation mismatch")
                {
                    failures.push(format!("{label}: unexpected public error: {error}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-binding-contract-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create binding-contract workspace");
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture directory");
        }
        fs::write(&path, contents).expect("write fixture");
        path
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-binding-contract-"));
        if expected && self.root.starts_with(std::env::temp_dir()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn run_cli(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .args(args)
        .current_dir(&workspace.root)
        .output()
        .expect("run Aero CLI")
}

fn cli_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn root_and_direct_module_cli_routes_reject_without_artifacts() {
    let mut failures = Vec::new();
    for (label, root_source, module_source, binding_name) in [
        ("root", "fn main() { let value: bool = 1; }", None, "value"),
        (
            "direct-module",
            "mod invalid; fn main() {}",
            Some("fn helper() { let module_value: bool = 1; }"),
            "module_value",
        ),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write("main.aero", root_source);
        if let Some(module_source) = module_source {
            workspace.write("invalid.aero", module_source);
        }
        let check = run_cli(&workspace, &[Path::new("check"), &root]);
        let artifact = workspace.path("program.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
        );

        for (route, output) in [("check", check), ("build", build)] {
            let text = cli_text(&output);
            if output.status.success() {
                failures.push(format!("{label} {route}: exited zero: {text}"));
            }
            for fragment in [
                &format!("Variable `{binding_name}` type annotation mismatch"),
                "expected bool",
                "actual int",
            ] {
                if !text.contains(fragment) {
                    failures.push(format!(
                        "{label} {route}: diagnostic {text:?} missing {fragment:?}"
                    ));
                }
            }
        }
        if artifact.exists() {
            failures.push(format!(
                "{label} build published failed artifact {}",
                artifact.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
