use compiler::ast::{AstNode, ComparisonOp, Expression};
use compiler::{
    CheckedIr, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::any::Any;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

const EXISTING_COMPARISON_REJECTION: &str =
    "comparison operand types are not admitted; expected numeric operands or Bool with Bool";
const PUBLIC_EXISTING_COMPARISON_REJECTION: &str = "IR Generation Error: comparison operand types are not admitted; expected numeric operands or Bool with Bool";

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

fn panic_text(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else {
        "non-string panic payload".to_string()
    }
}

fn expect_same_raw_panic(failures: &mut Vec<String>, label: &str, source: &str) {
    match catch_unwind(AssertUnwindSafe(|| {
        IrGenerator::new().generate_ir(parsed(source))
    })) {
        Ok(_) => failures.push(format!(
            "{label}: raw String comparison unexpectedly stopped panicking"
        )),
        Err(payload) => {
            let actual = panic_text(payload);
            let expected = "Unsupported comparison between String and String";
            if actual != expected {
                failures.push(format!(
                    "{label}: raw panic {actual:?} did not equal {expected:?}"
                ));
            }
        }
    }
}

#[test]
fn static_string_equality_class_is_complete_and_ci_executable() {
    const EXAMPLE: &str = include_str!("../../../examples/static_string_equality.aero");
    const CONTENT_AND_PROVENANCE: &str = r#"
        fn main() -> int {
            let empty = "";
            let ascii = "Aero";
            let same_ascii = "Aero";
            let direct_annotated: String = "Aero";
            let annotated_source = "Aero";
            let alias: String = annotated_source;
            let chain_root = "Aero";
            let chain_one = chain_root;
            let chain_two = chain_one;
            let escapes = "\n\t\r\\\"\0";
            let same_escapes = "\n\t\r\\\"\0";
            let unknown = "\q";
            let unicode = "é🚀中";
            let same_unicode = "é🚀中";
            let composed = "é";
            let combining = "é";
            let joined = "👩‍🚀";
            let same_joined = "👩‍🚀";
            let normalized_fstring = f"raw {text}";
            let shadow = "outer";
            {
                let nested_root = "Aero";
                let nested = nested_root;
                let shadow = "inner";
                nested == "Aero";
                shadow != "outer";
            }
            if empty == ""
                && empty != "x"
                && ascii == same_ascii
                && "Aero" == alias
                && alias == "Aero"
                && direct_annotated == "Aero"
                && chain_two == "Aero"
                && ascii != "aero"
                && "prefix" != "prefix-more"
                && "abc" != "adc"
                && escapes == same_escapes
                && unknown == "\q"
                && unicode == same_unicode
                && unicode != "é🚀文"
                && composed != combining
                && joined == same_joined
                && normalized_fstring == "raw {text}"
                && shadow == "outer"
            {
                return 41;
            }
            return 2;
        }
    "#;
    const BOOL_POSITIONS: &str = r#"
        fn identity(value: bool) -> bool { return value; }
        fn implicit() -> bool { "implicit" == "implicit" }
        fn main() -> int {
            let equal: bool = "a" == "a";
            let unequal = "a" != "b";
            let inverted = !("a" != "a");
            let conjunction = ("a" == "a") && ("b" != "c");
            let disjunction = ("a" != "a") || ("b" == "b");
            let bool_equality = ("a" == "a") == ("b" != "c");
            { let nested = "nested" == "nested"; nested; }
            while "loop" != "loop" {
                "body" == "body";
                break;
            }
            println!("{}", "print" == "print");
            "discard" == "discard";
            if identity(equal) && implicit() && unequal && inverted
                && conjunction && disjunction && bool_equality
            {
                return 41;
            }
            return 3;
        }
    "#;
    const LEGACY_TOP_LEVEL: &str =
        "let left = \"a\"; let same = left == \"a\"; if same { return 41; } return 1;";

    let mut failures = Vec::new();

    for (label, source) in [
        ("complete executable example", EXAMPLE),
        ("content/provenance product", CONTENT_AND_PROVENANCE),
        ("Bool placement/traversal product", BOOL_POSITIONS),
        (
            "String equality result stored in recursive Bool array",
            "fn main() -> int { let values = [\"a\" == \"a\"]; if values[0] { return 41; } 2 }",
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
                Ok(_) => failures.push(
                    "single-function static equality LLVM was not byte-identical".to_string(),
                ),
                Err(error) => failures.push(format!("second example compilation failed: {error}")),
            }
            if !first.contains("ret i32 41") {
                failures.push(format!("example omitted exact static exit 41:\n{first}"));
            }
            if !first.contains("icmp eq i32 0, 0") {
                failures.push(format!(
                    "example omitted Bool-producing constant comparison:\n{first}"
                ));
            }
            for forbidden in ["strcmp", "string_eq", "StringCompare"] {
                if first.contains(forbidden) {
                    failures.push(format!(
                        "example activated forbidden runtime marker {forbidden}:\n{first}"
                    ));
                }
            }
        }
    }

    match checked_source("fn main() { let same = \"é🚀中\" == \"é🚀中\"; }") {
        Err(error) => failures.push(format!("checked Unicode equality failed: {error}")),
        Ok(ir) => {
            let debug = format!("{ir:?}");
            if !debug.contains("ICmp") || !debug.contains("op: \"eq\"") {
                failures.push(format!("checked equality omitted Bool ICmp:\n{debug}"));
            }
            for forbidden in ["ImmString", "StringCompare", "string_eq"] {
                if debug.contains(forbidden) {
                    failures.push(format!(
                        "checked IR retained forbidden String marker {forbidden}:\n{debug}"
                    ));
                }
            }
        }
    }

    for (label, op) in [
        ("less-than ordering remains excluded", "<"),
        ("greater-than ordering remains excluded", ">"),
        ("less-equal ordering remains excluded", "<="),
        ("greater-equal ordering remains excluded", ">="),
    ] {
        let source = format!("fn main() {{ let ordered = \"a\" {op} \"b\"; }}");
        expect_acceptance(
            &mut failures,
            &format!("{label} semantic preservation"),
            analyzed(&source),
        );
        expect_exact_rejection(
            &mut failures,
            label,
            checked_source(&source),
            EXISTING_COMPARISON_REJECTION,
        );
    }

    for (label, source, expected) in [
        (
            "mutable literal binding",
            "fn main() { let mut text = \"a\"; let same = text == \"a\"; }",
            "mutable string bindings are not admitted; checked strings are immutable compile-time aliases",
        ),
        (
            "String parameter",
            "fn probe(text: String) -> bool { return text == \"a\"; }",
            "function parameter `text` is not an admitted scalar type",
        ),
        (
            "String return",
            "fn probe() -> String { return \"a\"; }",
            "function return type is not an admitted scalar or Void type",
        ),
        (
            "generic function outer boundary",
            "fn probe<T>() -> bool { return \"a\" == \"a\"; }",
            "generic function IR is not admitted in CORE-010",
        ),
        (
            "non-generic impl boundary",
            "impl Widget { fn probe() { let same = \"a\" == \"a\"; } }",
            EXISTING_COMPARISON_REJECTION,
        ),
        (
            "generic impl boundary",
            "impl<T> Widget { fn probe() { let same = \"a\" == \"a\"; } }",
            EXISTING_COMPARISON_REJECTION,
        ),
        (
            "closure boundary fails closed before equality classification",
            "fn main() { let probe = |ignored: int| \"a\" == \"a\"; let same = probe(0); }",
            "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
        ),
        (
            "closure parameter topology does not activate equality semantics",
            "fn main() { let probe = |text: String| \"a\" == \"a\"; }",
            "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
        ),
        (
            "closure parent precedes nested equality children",
            "fn main() { let probe = |ignored: int| missing + (\"a\" == \"a\"); let same = probe(0); }",
            "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
        ),
        (
            "left child precedes classification",
            "fn main() { let same = missing == \"a\"; }",
            "checked IR has no binding for `missing`",
        ),
        (
            "right child precedes classification",
            "fn main() { let same = \"a\" == missing; }",
            "checked IR has no binding for `missing`",
        ),
        (
            "admitted tuple left child reaches comparison classification",
            "fn main() { let same = (1, 2) == \"a\"; }",
            "comparison operand types are not admitted; expected numeric operands or Bool with Bool",
        ),
        (
            "admitted tuple right child reaches comparison classification",
            "fn main() { let same = \"a\" == (1, 2); }",
            "comparison operand types are not admitted; expected numeric operands or Bool with Bool",
        ),
        (
            "indexed String remains excluded",
            "fn main() { let same = \"abc\"[0] == \"a\"; }",
            "indexing requires an admitted fixed array",
        ),
        (
            "borrowed String remains excluded",
            "fn main() { let same = (&\"a\") == \"a\"; }",
            "a local immutable Copy-data borrow requires an identifier place",
        ),
        (
            "dereferenced String remains excluded",
            "fn main() { let text = \"a\"; let same = (*text) == \"a\"; }",
            "cannot dereference a non-reference value",
        ),
        (
            "field value remains excluded",
            "fn main() { let same = missing.field == \"a\"; }",
            "aggregate expression is not admitted in checked IR",
        ),
        (
            "struct value remains excluded",
            "fn main() { let same = Widget { field: 1 } == \"a\"; }",
            "aggregate expression is not admitted in checked IR",
        ),
        (
            "concatenation remains excluded",
            "fn main() { let same = (\"a\" + \"b\") == \"ab\"; }",
            "binary expression is not an admitted scalar",
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
            "fn main() { let text: string = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "custom named annotation",
            "fn main() { let text: Widget = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "custom annotated alias",
            "fn main() { let base = \"a\"; let text: Widget = base; let same = text == \"a\"; }",
        ),
        (
            "immutable reference annotation",
            "fn main() { let text: &String = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "mutable reference annotation",
            "fn main() { let text: &mut String = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "one-argument generic annotation",
            "fn main() { let text: Vec<int> = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "two-argument generic annotation",
            "fn main() { let text: HashMap<String, int> = \"a\"; let same = text == \"a\"; }",
        ),
        (
            "zero-count array annotation",
            "fn main() { let text: [int; 0] = \"a\"; let same = text == \"a\"; }",
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
            EXISTING_COMPARISON_REJECTION,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} public provenance quarantine"),
            compile_program(source, CompilerOptions::default()),
            PUBLIC_EXISTING_COMPARISON_REJECTION,
        );
    }

    for (label, source, semantic, public) in [
        (
            "nested array annotation",
            "fn main() { let text: [[int; 1]; 1] = \"a\"; let same = text == \"a\"; }",
            "fixed Copy-data array annotation mismatch: expected [[int; 1]; 1], actual String",
            "Semantic Analysis Error: fixed Copy-data array annotation mismatch: expected [[int; 1]; 1], actual String",
        ),
        (
            "array-around-reference annotation",
            "fn main() { let text: [&String; 0] = \"a\"; let same = text == \"a\"; }",
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

    let top_level_expression = vec![AstNode::Expression(Expression::Comparison {
        op: ComparisonOp::Equal,
        left: Box::new(Expression::StringLiteral("a".to_string())),
        right: Box::new(Expression::StringLiteral("a".to_string())),
    })];
    expect_exact_rejection(
        &mut failures,
        "top-level AstNode::Expression boundary",
        checked_ast(top_level_expression),
        "top-level expressions are not admitted in checked IR",
    );

    const TRAIT_DEFAULT: &str = "trait Contract<T> { fn probe() { let same = \"a\" == \"a\"; } }";
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

    expect_same_raw_panic(
        &mut failures,
        "ordinary raw route",
        "return \"a\" == \"a\";",
    );
    expect_same_raw_panic(
        &mut failures,
        "named-function raw route",
        "fn probe() -> bool { return \"a\" == \"a\"; }",
    );
    match catch_unwind(AssertUnwindSafe(|| {
        IrGenerator::new().generate_ir(parsed(
            "let probe = |ignored: int| \"a\" == \"a\"; return 0;",
        ))
    })) {
        Err(payload) => failures.push(format!(
            "closure raw quarantine unwound: {:?}",
            panic_text(payload)
        )),
        Ok(ir) if format!("{ir:#?}").contains("__closure_") => {
            failures.push("closure raw quarantine manufactured a closure symbol".to_string())
        }
        Ok(_) => {}
    }

    let workflow_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/rust.yml");
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not inspect Rust CI workflow {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test static string equality example",
                "cargo run -- build ../../examples/static_string_equality.aero",
                "opt-22 -passes=verify -disable-output ../../static_string_equality.ll",
                "llc-22 -verify-machineinstrs ../../static_string_equality.ll -o /dev/null",
                "llc-22 -filetype=obj ../../static_string_equality.ll -o ../../static_string_equality.o",
                "clang-22 ../../static_string_equality.o -o ../../static_string_equality",
                "../../static_string_equality",
                "if [ $exit_code -ne 41 ]; then",
                "static_string_equality test passed with exit code $exit_code",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("Rust CI omitted static-equality anchor {anchor:?}"));
                }
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
