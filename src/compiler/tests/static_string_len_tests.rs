use compiler::ast::{AstNode, Expression};
use compiler::{
    CheckedIr, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::PathBuf;

const EXISTING_METHOD_REJECTION: &str =
    "method calls other than exact zero-argument array/Vec .iter() are not admitted";
const PUBLIC_EXISTING_METHOD_REJECTION: &str = "IR Generation Error: method calls other than exact zero-argument array/Vec .iter() are not admitted";
const WRONG_ARITY_ONE: &str = "compile-time string .len() expects exactly 0 arguments, got 1";
const WRONG_ARITY_TWO: &str = "compile-time string .len() expects exactly 0 arguments, got 2";

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

fn raw_debug(source: &str) -> String {
    let mut functions = IrGenerator::new()
        .generate_ir(parsed(source))
        .into_iter()
        .collect::<Vec<_>>();
    functions.sort_by(|left, right| left.0.cmp(&right.0));
    format!("{functions:#?}")
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

fn expect_same_raw(failures: &mut Vec<String>, label: &str, candidate: &str, control: &str) {
    let candidate = raw_debug(candidate);
    let control = raw_debug(control);
    if candidate != control {
        failures.push(format!(
            "{label}: raw fabricated-zero compatibility changed\ncandidate:\n{candidate}\ncontrol:\n{control}"
        ));
    }
}

#[test]
fn static_string_character_len_class_is_complete_and_ci_executable() {
    const EXAMPLE: &str = include_str!("../../../examples/static_string_len.aero");
    const CONTENT_AND_PROVENANCE: &str = r#"
        fn main() -> int {
            let empty = "";
            let ascii = "Aero";
            let escapes = "\n\t\r\\\"\0\q";
            let unicode = "é🚀中";
            let annotated: String = "xy";
            let alias = unicode;
            let alias_two = alias;
            let annotated_source = "canonical";
            let annotated_alias: String = annotated_source;
            let normalized_fstring = f"raw {text}";
            let shadow = "outer";
            {
                let nested = alias_two;
                let shadow = "i";
                nested.len() + shadow.len();
            }
            return empty.len() + ascii.len() + escapes.len() + unicode.len()
                + annotated.len() + alias.len() + alias_two.len()
                + annotated_alias.len() + normalized_fstring.len()
                + shadow.len() + ("z").len();
        }
    "#;
    const TRAVERSAL_AND_INT_POSITIONS: &str = r#"
        fn identity(value: int) -> int { return value; }
        fn helper() -> int { return "help".len(); }
        fn implicit() -> int { "implicit".len() }
        fn main() -> int {
            let bound: int = "a".len();
            { let nested = "bb".len() + 1; nested; }
            if "é".len() == 1 {
                let then_value = "then".len();
                then_value;
            } else {
                let else_value = "else".len();
                else_value;
            }
            while "".len() > 0 {
                let while_value = "while".len();
                while_value;
                break;
            }
            for item in [1].iter() {
                let for_value = "for".len() + item;
                for_value;
            }
            loop {
                let loop_value = "loop".len();
                loop_value;
                break;
            }
            println!("{}", "print".len());
            "discard".len();
            let unary = -"u".len();
            let element = ["e".len()];
            let repeated = ["r".len(); 1];
            let indexed = [7, 9]["i".len()];
            return identity("call".len()) + helper() + implicit() + bound + unary
                + element[0] + repeated[0] + indexed;
        }
    "#;
    const LEGACY_TOP_LEVEL: &str =
        "let text = \"abc\"; let alias = text; let observed = alias.len(); return observed;";

    let mut failures = Vec::new();

    for (label, source) in [
        ("complete executable example", EXAMPLE),
        ("content/provenance product", CONTENT_AND_PROVENANCE),
        (
            "traversal/Int-position product",
            TRAVERSAL_AND_INT_POSITIONS,
        ),
        ("legacy top-level statement route", LEGACY_TOP_LEVEL),
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

    match compile_program(EXAMPLE, CompilerOptions::default()) {
        Err(error) => failures.push(format!("example compilation failed: {error}")),
        Ok(first) => {
            match compile_program(EXAMPLE, CompilerOptions::default()) {
                Ok(second) if second == first => {}
                Ok(_) => {
                    failures.push("single-function example LLVM was not byte-identical".to_string())
                }
                Err(error) => failures.push(format!("second example compilation failed: {error}")),
            }
            if !first.contains("ret i32 33") {
                failures.push(format!(
                    "example did not lower exact static exit 33:\n{first}"
                ));
            }
            for forbidden in ["StringLength", "string_len", "1.000000e+01"] {
                if first.contains(forbidden) {
                    failures.push(format!(
                        "example activated forbidden runtime marker {forbidden}:\n{first}"
                    ));
                }
            }
        }
    }

    match checked_source("fn main() -> int { let text = \"é🚀中\"; return text.len(); }") {
        Err(error) => failures.push(format!("checked Unicode specimen failed: {error}")),
        Ok(ir) => {
            let debug = format!("{ir:?}");
            if !debug.contains("ImmInt(3)") {
                failures.push(format!(
                    "checked Unicode length was not exact Int 3:\n{debug}"
                ));
            }
            for forbidden in ["StringLength", "string_len", "ImmFloat(10"] {
                if debug.contains(forbidden) {
                    failures.push(format!(
                        "checked IR activated forbidden marker {forbidden}:\n{debug}"
                    ));
                }
            }
        }
    }

    for (label, source, expected) in [
        (
            "mutable literal binding",
            "fn main() { let mut text = \"a\"; let observed = text.len(); }",
            "mutable string bindings are not admitted; checked strings are immutable compile-time aliases",
        ),
        (
            "String parameter",
            "fn probe(text: String) -> int { return text.len(); }",
            "function parameter `text` is not an admitted scalar type",
        ),
        (
            "String return",
            "fn probe() -> String { return \"a\"; }",
            "function return type is not an admitted scalar or Void type",
        ),
        (
            "generic function outer boundary",
            "fn probe<T>() { let observed = \"a\".len(); }",
            "generic function IR is not admitted in CORE-010",
        ),
        (
            "non-generic impl boundary",
            "impl Widget { fn probe() { let observed = \"a\".len(); } }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "generic impl boundary",
            "impl<T> Widget { fn probe() { let observed = \"a\".len(); } }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "closure boundary fails closed before string length classification",
            "fn main() { let probe = |ignored: int| \"a\".len(); let observed = probe(0); }",
            "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
        ),
        (
            "receiver child precedes classification",
            "fn main() { let observed = missing.len(); }",
            "checked IR has no binding for `missing`",
        ),
        (
            "admitted tuple argument reaches arity classification",
            "fn main() { let observed = \"a\".len((1, 2)); }",
            WRONG_ARITY_ONE,
        ),
        (
            "one-argument exact len arity",
            "fn main() { let observed = \"a\".len(1); }",
            WRONG_ARITY_ONE,
        ),
        (
            "two-argument exact len arity",
            "fn main() { let observed = \"a\".len(1, 2); }",
            WRONG_ARITY_TWO,
        ),
        (
            "unknown String method",
            "fn main() { let observed = \"a\".missing(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "case-variant String method",
            "fn main() { let observed = \"a\".Len(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "String-producing method chain remains excluded",
            "fn main() { let observed = \"a\".trim().len(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "field receiver remains excluded",
            "fn main() { let observed = missing.field.len(); }",
            "aggregate expression is not admitted in checked IR",
        ),
        (
            "indexed String receiver remains excluded",
            "fn main() { let observed = (\"abc\"[0]).len(); }",
            "indexing requires an admitted fixed array",
        ),
        (
            "borrowed String receiver remains excluded",
            "fn main() { let observed = (&\"a\").len(); }",
            "a local immutable Copy-data borrow requires an identifier place",
        ),
        (
            "dereferenced String receiver remains excluded",
            "fn main() { let text = \"a\"; let observed = (*text).len(); }",
            "cannot dereference a non-reference value",
        ),
        (
            "struct receiver remains excluded",
            "fn main() { let observed = Widget { field: 1 }.len(); }",
            "aggregate expression is not admitted in checked IR",
        ),
        (
            "len result used as receiver",
            "fn main() { let observed = \"a\".len().len(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "len result iterated",
            "fn main() { let observed = \"a\".len().iter(); }",
            EXISTING_METHOD_REJECTION,
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            label,
            checked_parsed_source(source),
            expected,
        );
    }

    for (label, source) in [
        (
            "lowercase string annotation",
            "fn main() { let text: string = \"a\"; let observed = text.len(); }",
        ),
        (
            "custom named annotation",
            "fn main() { let text: Widget = \"a\"; let observed = text.len(); }",
        ),
        (
            "custom annotated alias",
            "fn main() { let base = \"a\"; let text: Widget = base; let observed = text.len(); }",
        ),
        (
            "immutable reference annotation",
            "fn main() { let text: &String = \"a\"; let observed = text.len(); }",
        ),
        (
            "mutable reference annotation",
            "fn main() { let text: &mut String = \"a\"; let observed = text.len(); }",
        ),
        (
            "one-argument generic annotation",
            "fn main() { let text: Vec<int> = \"a\"; let observed = text.len(); }",
        ),
        (
            "two-argument generic annotation",
            "fn main() { let text: HashMap<String, int> = \"a\"; let observed = text.len(); }",
        ),
        (
            "zero-count array annotation",
            "fn main() { let text: [int; 0] = \"a\"; let observed = text.len(); }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} keeps prior semantic preservation"),
            analyzed(source),
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked provenance quarantine"),
            checked_source(source),
            EXISTING_METHOD_REJECTION,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} public provenance quarantine"),
            compile_program(source, CompilerOptions::default()),
            PUBLIC_EXISTING_METHOD_REJECTION,
        );
    }

    for (label, source, semantic, public) in [
        (
            "nested array annotation",
            "fn main() { let text: [[int; 1]; 1] = \"a\"; let observed = text.len(); }",
            "fixed Copy-data array annotation mismatch: expected [[int; 1]; 1], actual String",
            "Semantic Analysis Error: fixed Copy-data array annotation mismatch: expected [[int; 1]; 1], actual String",
        ),
        (
            "array-around-reference annotation",
            "fn main() { let text: [&String; 0] = \"a\"; let observed = text.len(); }",
            "fixed Copy-data array annotation mismatch: expected array[0], actual String",
            "Semantic Analysis Error: fixed Copy-data array annotation mismatch: expected array[0], actual String",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            &format!("{label} shared semantic mismatch"),
            analyzed(source),
            semantic,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked mismatch precedence"),
            checked_source(source),
            semantic,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} public mismatch precedence"),
            compile_program(source, CompilerOptions::default()),
            public,
        );
    }

    let top_level_expression = vec![AstNode::Expression(Expression::MethodCall {
        object: Box::new(Expression::StringLiteral("a".to_string())),
        method: "len".to_string(),
        arguments: Vec::new(),
    })];
    expect_exact_rejection(
        &mut failures,
        "top-level AstNode::Expression boundary",
        checked_ast(top_level_expression),
        "top-level expressions are not admitted in checked IR",
    );

    const TRAIT_DEFAULT: &str = "trait Contract<T> { fn probe() { let observed = \"a\".len(); } }";
    expect_acceptance(
        &mut failures,
        "trait default remains semantic syntax-only",
        analyzed(TRAIT_DEFAULT),
    );
    expect_acceptance(
        &mut failures,
        "trait default remains checked syntax-only",
        checked_source(TRAIT_DEFAULT),
    );

    expect_same_raw(
        &mut failures,
        "ordinary raw route",
        "return \"abc\".len();",
        "return 0;",
    );
    expect_same_raw(
        &mut failures,
        "named-function raw route",
        "fn probe() -> int { return \"abc\".len(); }",
        "fn probe() -> int { return 0; }",
    );
    expect_same_raw(
        &mut failures,
        "closure raw route",
        "let probe = |ignored: int| \"abc\".len(); return probe(0);",
        "let probe = |ignored: int| 0; return probe(0);",
    );

    let workflow_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/rust.yml");
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not inspect Rust CI workflow {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test static string character length example",
                "cargo run -- build ../../examples/static_string_len.aero",
                "opt-22 -passes=verify -disable-output ../../static_string_len.ll",
                "llc-22 -verify-machineinstrs ../../static_string_len.ll -o /dev/null",
                "llc-22 -filetype=obj ../../static_string_len.ll -o ../../static_string_len.o",
                "clang-22 ../../static_string_len.o -o ../../static_string_len",
                "../../static_string_len",
                "if [ $exit_code -ne 33 ]; then",
                "static_string_len test passed with exit code $exit_code",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("Rust CI omitted static-string anchor {anchor:?}"));
                }
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
