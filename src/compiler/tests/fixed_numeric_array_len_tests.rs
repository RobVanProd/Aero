use compiler::ast::{AstNode, Block, Expression, Statement, Type};
use compiler::{
    CheckedIr, CompilerOptions, IrGenerator, LogicalType, SemanticAnalyzer, compile_program,
    parse_with_locations, try_generate_code, try_tokenize_with_locations,
};
use std::fs;
use std::path::PathBuf;

const EXISTING_METHOD_REJECTION: &str =
    "method calls other than exact zero-argument array/Vec .iter() are not admitted";
const WRONG_ARITY_ONE: &str = "fixed numeric array .len() expects exactly 0 arguments, got 1";
const WRONG_ARITY_TWO: &str = "fixed numeric array .len() expects exactly 0 arguments, got 2";
const OUTSIDE_INT_RANGE: &str =
    "fixed numeric array .len() count 2147483648 is outside the admitted i32 range";

fn parsed(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn analyzed(source: &str) -> Result<Vec<AstNode>, String> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(parsed(source)).map(|(_, ast)| ast)
}

fn checked_ast(ast: Vec<AstNode>) -> Result<CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(ast)
        .map_err(|error| error.to_string())
}

fn checked_source(source: &str) -> Result<CheckedIr, String> {
    analyzed(source).and_then(checked_ast)
}

fn checked_parsed_source(source: &str) -> Result<CheckedIr, String> {
    checked_ast(parsed(source))
}

fn expect_acceptance<T>(failures: &mut Vec<String>, label: &str, result: Result<T, String>) {
    if let Err(error) = result {
        failures.push(format!("{label}: unexpectedly rejected: {error}"));
    }
}

fn expect_exact_rejection<T>(
    failures: &mut Vec<String>,
    label: &str,
    result: Result<T, String>,
    expected: &str,
) {
    match result {
        Ok(_) => failures.push(format!("{label}: unexpectedly accepted")),
        Err(error) if error != expected => failures.push(format!(
            "{label}: diagnostic {error:?} did not equal {expected:?}"
        )),
        Err(_) => {}
    }
}

fn expect_rejection_fragments<T>(
    failures: &mut Vec<String>,
    label: &str,
    result: Result<T, String>,
    expected: &[&str],
) {
    match result {
        Ok(_) => failures.push(format!("{label}: unexpectedly accepted")),
        Err(error) => {
            for fragment in expected {
                if !error.contains(fragment) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} omitted {fragment:?}"
                    ));
                }
            }
        }
    }
}

fn main_returning(expression: Expression) -> Vec<AstNode> {
    vec![AstNode::Statement(Statement::Function {
        name: "main".to_string(),
        parameters: Vec::new(),
        return_type: Some(Type::Named("int".to_string())),
        body: Block {
            statements: vec![Statement::Return(Some(expression))],
            expression: None,
        },
        type_params: Vec::new(),
        trait_bounds: Vec::new(),
    })]
}

fn static_len(object: Expression, arguments: Vec<Expression>) -> Expression {
    Expression::MethodCall {
        object: Box::new(object),
        method: "len".to_string(),
        arguments,
    }
}

#[test]
fn fixed_numeric_array_static_len_class_is_complete_and_ci_executable() {
    const EXAMPLE: &str = include_str!("../../../examples/fixed_numeric_array_len.aero");
    const NORMALIZED_RECEIVERS: &str = r#"
        fn main() -> int {
            let base = [1, 2, 3];
            let copied = base;
            return [1, 2].len()
                + [1.0; 3].len()
                + [1; 0].len()
                + vec![1.0; 0].len()
                + vec![1, 2, 3, 4].len()
                + (copied).len()
                + base.iter().iter().len();
        }
    "#;
    const TYPED_EMPTY_PRODUCT: &str = r#"
        fn main() -> int {
            let int_immutable: [int; 0] = [];
            let mut i32_mutable: [i32; 0] = [];
            let float_immutable: [float; 0] = vec![];
            let mut f64_mutable: [f64; 0] = [];
            return int_immutable.len()
                + i32_mutable.len()
                + float_immutable.len()
                + f64_mutable.len();
        }
    "#;
    const TRAVERSAL_AND_INT_POSITIONS: &str = r#"
        fn helper() -> int {
            let helper_values = [1, 2];
            return helper_values.len();
        }
        fn identity(value: int) -> int { return value; }
        fn main() -> int {
            let values = [1, 2, 3];
            let bound: int = values.len();
            { let nested = values.len() + 1; nested; }
            if values.len() == 3 {
                let then_value = values.len();
                then_value;
            } else {
                let else_value = values.len();
                else_value;
            }
            while values.len() < 0 {
                let while_value = values.len();
                while_value;
                break;
            }
            for item in values.iter() {
                let for_value = values.len() + item;
                for_value;
            }
            loop {
                let loop_value = values.len();
                loop_value;
                break;
            }
            println!("{}", values.len());
            values.len();
            let unary = -values.len();
            let element = [values.len()];
            let repeated = [values.len(); 1];
            let indexed = [1, 2, 3][values.len() - 1];
            return identity(values.len()) + helper() + bound + unary
                + element[0] + repeated[0] + indexed;
        }
    "#;
    const LEGACY_TOP_LEVEL: &str =
        "let values = [1, 2, 3]; let observed = values.len(); return observed;";
    const MISMATCH_PRESERVATION: &str = r#"
        fn main() -> int {
            let values: [int; 0] = [1, 2, 3];
            let observed: int = values.len();
            return observed;
        }
    "#;

    let mut failures = Vec::new();

    for (label, source) in [
        ("complete alias/mutability/provenance product", EXAMPLE),
        (
            "normalized immediate/bound/Copy/iter receiver product",
            NORMALIZED_RECEIVERS,
        ),
        ("typed-empty alias/mutability product", TYPED_EMPTY_PRODUCT),
        (
            "ordinary traversal and Int-position product",
            TRAVERSAL_AND_INT_POSITIONS,
        ),
        ("legacy top-level statement route", LEGACY_TOP_LEVEL),
        (
            "initializer-derived mismatch preservation",
            MISMATCH_PRESERVATION,
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            analyzed(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} public compile"),
            compile_program(source, CompilerOptions::default()),
        );
    }

    match compile_program(NORMALIZED_RECEIVERS, CompilerOptions::default()) {
        Err(error) => failures.push(format!("zero-repeat LLVM specimen failed: {error}")),
        Ok(llvm) => {
            if llvm.matches("alloca [0 x double]").count() != 2 {
                failures.push(format!(
                    "zero-repeat receivers did not preserve both Int/Float zero allocations:\n{llvm}"
                ));
            }
            if !llvm.contains("ret i32 15") {
                failures.push(format!(
                    "zero-repeat lengths did not remain exact static Int zero values:\n{llvm}"
                ));
            }
        }
    }

    match checked_source(MISMATCH_PRESERVATION) {
        Err(error) => failures.push(format!("metadata specimen failed: {error}")),
        Ok(ir) => {
            let Some(main) = ir.metadata().functions.get("main") else {
                failures.push("metadata specimen omitted main".to_string());
                assert!(failures.is_empty(), "{}", failures.join("\n"));
                return;
            };
            let observed = main
                .places
                .values()
                .filter_map(|place| place.name.as_deref().map(|name| (name, &place.pointee)))
                .collect::<Vec<_>>();
            if !main.places.values().any(|place| {
                matches!(
                    &place.pointee,
                    LogicalType::Array { element, count }
                        if **element == LogicalType::Int && *count == 3
                )
            }) {
                failures.push(format!(
                    "initializer-derived array metadata did not preserve Int count 3: {:?}",
                    main.places
                ));
            }
            if !observed
                .iter()
                .any(|(name, ty)| *name == "observed" && **ty == LogicalType::Int)
            {
                failures.push(format!(
                    "static length binding did not retain logical Int metadata: {observed:?}"
                ));
            }
            let debug = format!("{ir:#?}");
            if debug.contains("ArrayLength") {
                failures.push(format!(
                    "checked IR activated dormant ArrayLength:\n{debug}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "direct untyped empty",
            "fn main() { let observed = [].len(); }",
            "empty array literals have no admitted logical element type",
        ),
        (
            "Bool array receiver",
            "fn main() { let values = [true, false]; let observed = values.len(); }",
            "checked IR has no binding for `true`",
        ),
        (
            "String array receiver",
            "fn main() { let values = [\"a\", \"b\"]; let observed = values.len(); }",
            "only fixed numeric arrays are admitted",
        ),
        (
            "nested array receiver",
            "fn main() { let values = [[1], [2]]; let observed = values.len(); }",
            "only fixed numeric arrays are admitted",
        ),
    ] {
        expect_rejection_fragments(
            &mut failures,
            label,
            checked_parsed_source(source),
            &[expected],
        );
    }

    for (label, source) in [
        ("scalar receiver", "fn main() { let observed = 1.len(); }"),
        (
            "unknown array method",
            "fn main() { let values = [1, 2]; let observed = values.first(); }",
        ),
        (
            "case-variant method",
            "fn main() { let values = [1, 2]; let observed = values.Len(); }",
        ),
        (
            "len result used as receiver",
            "fn main() { let values = [1, 2]; let observed = values.len().len(); }",
        ),
        (
            "len result iterated",
            "fn main() { let values = [1, 2]; let observed = values.len().iter(); }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            label,
            checked_parsed_source(source),
            EXISTING_METHOD_REJECTION,
        );
    }

    expect_acceptance(
        &mut failures,
        "subsequently admitted compile-time String receiver",
        checked_parsed_source("fn main() { let observed = \"a\".len(); }"),
    );

    expect_exact_rejection(
        &mut failures,
        "one-argument exact len arity",
        checked_parsed_source("fn main() { let values = [1, 2]; let observed = values.len(1); }"),
        WRONG_ARITY_ONE,
    );
    expect_exact_rejection(
        &mut failures,
        "two-argument exact len arity",
        checked_parsed_source(
            "fn main() { let values = [1, 2]; let observed = values.len(1, 2); }",
        ),
        WRONG_ARITY_TWO,
    );
    expect_exact_rejection(
        &mut failures,
        "admitted tuple receiver reaches method classification",
        checked_parsed_source("fn main() { let observed = (1, 2).len(); }"),
        "method calls other than exact zero-argument array/Vec .iter() are not admitted",
    );
    expect_exact_rejection(
        &mut failures,
        "admitted tuple argument reaches arity classification",
        checked_parsed_source(
            "fn main() { let values = [1, 2]; let observed = values.len((1, 2)); }",
        ),
        WRONG_ARITY_ONE,
    );
    expect_exact_rejection(
        &mut failures,
        "direct checked heterogeneous receiver is not trusted",
        checked_parsed_source("fn main() { let values = [1, 2.0]; let observed = values.len(); }"),
        "fixed numeric arrays must have homogeneous element types in checked IR",
    );
    expect_exact_rejection(
        &mut failures,
        "homogeneity is enforced at the checked array boundary",
        checked_parsed_source("fn main() { let values = [1, 2.0]; }"),
        "fixed numeric arrays must have homogeneous element types in checked IR",
    );
    expect_exact_rejection(
        &mut failures,
        "later receiver child failure precedes homogeneity rejection",
        checked_parsed_source("fn main() { let observed = [1, 2.0, missing_value].len(); }"),
        "checked IR has no binding for `missing_value`",
    );
    expect_exact_rejection(
        &mut failures,
        "nonnumeric receiver child rejection keeps earlier precedence",
        checked_parsed_source("fn main() { let observed = [1, \"a\", missing_value].len(); }"),
        "only fixed numeric arrays are admitted",
    );

    let out_of_range = static_len(
        Expression::ArrayRepeat {
            value: Box::new(Expression::IntegerLiteral(1)),
            count: i32::MAX as usize + 1,
        },
        Vec::new(),
    );
    expect_exact_rejection(
        &mut failures,
        "direct AST count outside admitted Int",
        checked_ast(main_returning(out_of_range)),
        OUTSIDE_INT_RANGE,
    );
    expect_exact_rejection(
        &mut failures,
        "source count outside admitted Int",
        checked_parsed_source("fn main() -> int { return [1; 2147483648].len(); }"),
        OUTSIDE_INT_RANGE,
    );

    expect_exact_rejection(
        &mut failures,
        "generic function outer boundary",
        checked_parsed_source(
            "fn probe<T>() { let values = [1, 2]; let observed = values.len(); }",
        ),
        "generic function IR is not admitted in CORE-010",
    );
    for (label, source) in [
        (
            "non-generic impl boundary",
            "impl Widget { fn probe() { let values = [1, 2]; let observed = values.len(); } }",
        ),
        (
            "generic impl boundary",
            "impl<T> Widget { fn probe() { let values = [1, 2]; let observed = values.len(); } }",
        ),
        (
            "non-generic impl heterogeneous quarantine boundary",
            "impl Widget { fn probe() { let values = [1, 2.0]; let observed = values.len(); } }",
        ),
        (
            "generic impl heterogeneous quarantine boundary",
            "impl<T> Widget { fn probe() { let values = [1, 2.0]; let observed = values.len(); } }",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            label,
            checked_parsed_source(source),
            EXISTING_METHOD_REJECTION,
        );
    }
    expect_exact_rejection(
        &mut failures,
        "closure aggregate boundary",
        checked_parsed_source(
            "fn main() { let probe = |ignored: int| [1, 2].len(); let observed = probe(0); }",
        ),
        "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
    );

    const TRAIT_DEFAULT: &str =
        "trait Contract<T> { fn probe() { let values = [1, 2]; let observed = values.len(); } }";
    expect_acceptance(
        &mut failures,
        "trait default remains semantic syntax-only",
        analyzed(TRAIT_DEFAULT),
    );
    match checked_source(TRAIT_DEFAULT) {
        Err(error) => failures.push(format!("trait default checked route changed: {error}")),
        Ok(ir)
            if ir
                .metadata()
                .functions
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != vec!["main"] =>
        {
            failures.push(format!(
                "trait default changed the syntax-only checked-function set: {:?}",
                ir.metadata().functions.keys().collect::<Vec<_>>()
            ));
        }
        Ok(_) => {}
    }

    expect_exact_rejection(
        &mut failures,
        "top-level expression boundary",
        checked_ast(vec![AstNode::Expression(static_len(
            Expression::ArrayLiteral(vec![Expression::IntegerLiteral(1)]),
            Vec::new(),
        ))]),
        "top-level expressions are not admitted in checked IR",
    );

    for (label, len_source, preserved_control) in [
        (
            "raw legacy top-level",
            "let values = [1, 2]; let observed = values.len(); return observed;",
            "let values = [1, 2]; let observed = values.missing(); return observed;",
        ),
        (
            "raw named function",
            "fn main() -> int { let values = [1, 2]; return values.len(); }",
            "fn main() -> int { let values = [1, 2]; return values.missing(); }",
        ),
        (
            "raw iter chain",
            "fn main() -> int { let values = [1, 2]; return values.iter().len(); }",
            "fn main() -> int { let values = [1, 2]; return values.iter().missing(); }",
        ),
        (
            "raw closure helper",
            "fn main() { let probe = |ignored: int| [1, 2].len(); let observed = probe(0); }",
            "fn main() { let probe = |ignored: int| [1, 2].missing(); let observed = probe(0); }",
        ),
    ] {
        let len_ir = IrGenerator::new().generate_ir(parsed(len_source));
        let control_ir = IrGenerator::new().generate_ir(parsed(preserved_control));
        if len_ir != control_ir {
            failures.push(format!(
                "{label}: raw fabricated-zero compatibility changed:\nlen={len_ir:#?}\ncontrol={control_ir:#?}"
            ));
        }
        if format!("{len_ir:#?}").contains("ArrayLength") {
            failures.push(format!("{label}: raw path activated ArrayLength"));
        }
    }

    let first = compile_program(EXAMPLE, CompilerOptions::default());
    let second = compile_program(EXAMPLE, CompilerOptions::default());
    match (first, second) {
        (Ok(first), Ok(second)) if first != second => {
            failures.push("single-function example LLVM was not byte-identical".to_string())
        }
        (Ok(first), Ok(_)) => {
            if first.matches("alloca [").count() != 12 {
                failures.push(format!(
                    "example did not preserve the eight receiver initializers and four checked mutable-owner allocations:\n{first}"
                ));
            }
            for count in [0, 1, 2, 3, 4, 5, 6, 16] {
                if !first.contains(&format!("alloca [{count} x double]")) {
                    failures.push(format!(
                        "example omitted the count-{count} receiver allocation:\n{first}"
                    ));
                }
            }
            if !first.contains("ret i32 37") {
                failures.push(format!(
                    "example did not lower the exact static-length sentinel:\n{first}"
                ));
            }
            if first.contains("ArrayLength") {
                failures.push(format!("example LLVM mentions ArrayLength:\n{first}"));
            }
        }
        (Err(error), _) => failures.push(format!("first example compile failed: {error}")),
        (_, Err(error)) => failures.push(format!("second example compile failed: {error}")),
    }

    if let Ok(ir) = checked_source(EXAMPLE) {
        match try_generate_code(ir.clone()) {
            Ok(llvm) if llvm.contains("ArrayLength") => {
                failures.push(format!("checked codegen emitted ArrayLength:\n{llvm}"))
            }
            Err(error) => failures.push(format!("checked codegen rejected example: {error}")),
            Ok(_) => {}
        }
        let debug = format!("{ir:#?}");
        if debug.contains("ArrayLength") {
            failures.push(format!(
                "checked example IR activated ArrayLength:\n{debug}"
            ));
        }
    }

    let workflow = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/rust.yml"),
    )
    .expect("read Rust CI workflow");
    for required in [
        "- name: Test fixed numeric array static length example",
        "cargo run -- build ../../examples/fixed_numeric_array_len.aero -o ../../fixed_numeric_array_len.ll",
        "opt-22 -passes=verify -disable-output ../../fixed_numeric_array_len.ll",
        "llc-22 -verify-machineinstrs ../../fixed_numeric_array_len.ll -o /dev/null",
        "llc-22 -filetype=obj ../../fixed_numeric_array_len.ll -o ../../fixed_numeric_array_len.o",
        "clang-22 ../../fixed_numeric_array_len.o -o ../../fixed_numeric_array_len",
        "../../fixed_numeric_array_len",
        "fixed_numeric_array_len test passed with exit code $exit_code",
    ] {
        if !workflow.contains(required) {
            failures.push(format!(
                "Rust CI executable example gate omitted {required:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} fixed-array-length class failures:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
