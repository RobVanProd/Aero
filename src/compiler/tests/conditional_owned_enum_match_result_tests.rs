use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::{fs, path::PathBuf};

const CURRENT_FRESH_ONLY_DIAGNOSTIC: &str = "must produce a fresh constructor";
const MAYBE_MOVED_DIAGNOSTIC: &str = "may have been moved on another control-flow path";

fn parsed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source)?)
        .map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
}

fn checked_ir_without_semantic(source: &str) -> Result<compiler::CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source)?)
        .map_err(|error| error.to_string())
}

fn common_prelude() -> &'static str {
    r#"
struct Cell { value: int, flags: [bool; 2] }

enum Input {
    Left,
    Right
}

enum Output {
    Empty,
    Count(int),
    Pair(int, bool),
    Mark(char),
    CellValue(Cell),
    Cells([Cell; 2]),
    Matrix([[int; 2]; 2]),
    Mixed(int, bool, char)
}

fn score(value: Output) -> int {
    match value {
        Output::Empty => 0,
        Output::Count(number) => number,
        Output::Pair(number, flag) => number,
        Output::Mark(glyph) => 3,
        Output::CellValue(item) => item.value,
        Output::Cells(items) => items[1].value,
        Output::Matrix(matrix) => matrix[1][0],
        Output::Mixed(number, flag, glyph) => number
    }
}
"#
}

fn with_prelude(body: &str) -> String {
    format!("{}\n{body}\n", common_prelude())
}

fn complete_positive_source() -> String {
    with_prelude(
        r#"
fn same_owner(input: Input, value: Output) -> Output {
    match input {
        Input::Left => value,
        Input::Right => value
    }
}

fn different_owners(input: Input, left: Output, right: Output) -> Output {
    match input {
        Input::Left => left,
        Input::Right => right
    }
}

fn mixed_fresh_and_owned(input: Input, existing: Output) -> Output {
    match input {
        Input::Left => existing,
        Input::Right => Output::Count(7)
    }
}

fn nested_owned(input: Input, left: Output, right: Output) -> Output {
    match input {
        Input::Left => match Input::Right {
            Input::Left => left,
            Input::Right => right
        },
        Input::Right => Output::Empty
    }
}

fn exact_binding(input: Input, existing: Output) -> int {
    let result: Output = match input {
        Input::Left => existing,
        Input::Right => Output::Count(11)
    };
    score(result)
}

fn replacement(input: Input, existing: Output) -> int {
    let mut result = Output::Empty;
    result = match input {
        Input::Left => existing,
        Input::Right => Output::Count(13)
    };
    score(result)
}

fn maybe_moved_reinitialization(input: Input) -> int {
    let mut existing = Output::Count(1);
    let result = match input {
        Input::Left => existing,
        Input::Right => Output::Count(17)
    };
    existing = Output::Count(19);
    score(result) + score(existing)
}

fn all_shapes(input: Input) -> int {
    let cell = Cell { value: 23, flags: [1 < 2, 1 > 2] };
    let unit = same_owner(input, Output::Empty);
    let unary = different_owners(Input::Left, Output::Count(29), Output::Mark('x'));
    let multi = mixed_fresh_and_owned(Input::Left, Output::Mixed(31, 1 < 2, 'x'));
    let named = nested_owned(
        Input::Left,
        Output::CellValue(cell),
        Output::Cells([
            Cell { value: 37, flags: [1 < 2, 1 > 2] },
            Cell { value: 41, flags: [1 < 2, 1 > 2] }
        ])
    );
    let recursive = different_owners(
        Input::Right,
        Output::Pair(43, 1 < 2),
        Output::Matrix([[47, 49], [53, 59]])
    );
    score(unit) + score(unary) + score(multi) + score(named) + score(recursive)
}

fn main() -> int {
    let selected = different_owners(Input::Left, Output::Count(211), Output::Empty);
    let exercised = all_shapes(Input::Right)
        + exact_binding(Input::Right, Output::Count(1))
        + replacement(Input::Right, Output::Count(1))
        + maybe_moved_reinitialization(Input::Right);
    if exercised > 0 { return score(selected); }
    1
}
"#,
    )
}

fn expect_success(label: &str, source: &str, failures: &mut Vec<String>) {
    match compile_program(source, CompilerOptions::default()) {
        Ok(_) => {}
        Err(error) => failures.push(format!("{label}: unexpectedly failed: {error}")),
    }
}

fn expect_rejection(
    label: &str,
    source: &str,
    required: &[&str],
    forbidden: &[&str],
    failures: &mut Vec<String>,
) {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => failures.push(format!("{label}: unexpectedly compiled:\n{llvm}")),
        Err(error) => {
            for marker in required {
                if !error.contains(marker) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} omitted required marker {marker:?}"
                    ));
                }
            }
            for marker in forbidden {
                if error.contains(marker) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} contained forbidden marker {marker:?}"
                    ));
                }
            }
        }
    }
}

#[test]
fn conditional_direct_owner_result_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let complete = complete_positive_source();

    if let Err(error) = parsed(&complete) {
        failures.push(format!(
            "complete direct-owner result syntax did not parse: {error}"
        ));
    }

    for (label, body) in [
        (
            "same owner on every path",
            r#"
fn choose(input: Input, value: Output) -> Output {
    match input { Input::Left => value, Input::Right => value }
}
fn main() -> int { score(choose(Input::Right, Output::Count(211))) }
"#,
        ),
        (
            "different owners by path",
            r#"
fn choose(input: Input, left: Output, right: Output) -> Output {
    match input { Input::Left => left, Input::Right => right }
}
fn main() -> int { score(choose(Input::Left, Output::Count(211), Output::Empty)) }
"#,
        ),
        (
            "mixed fresh and direct origins",
            r#"
fn choose(input: Input, value: Output) -> Output {
    match input { Input::Left => value, Input::Right => Output::Empty }
}
fn main() -> int { score(choose(Input::Left, Output::Count(211))) }
"#,
        ),
        (
            "recursive direct origins",
            r#"
fn choose(input: Input, left: Output, right: Output) -> Output {
    match input {
        Input::Left => match Input::Right {
            Input::Left => left,
            Input::Right => right
        },
        Input::Right => Output::Empty
    }
}
fn main() -> int { score(choose(Input::Left, Output::Empty, Output::Count(211))) }
"#,
        ),
        (
            "inferred and exact result bindings",
            r#"
fn choose(input: Input, value: Output) -> Output {
    let result: Output = match input {
        Input::Left => value,
        Input::Right => Output::Count(211)
    };
    result
}
fn main() -> int { score(choose(Input::Right, Output::Empty)) }
"#,
        ),
        (
            "recursive CopyData payload universe",
            r#"
fn choose(input: Input, left: Output, right: Output) -> Output {
    match input { Input::Left => left, Input::Right => right }
}
fn main() -> int {
    let cell = Cell { value: 211, flags: [1 < 2, 1 > 2] };
    score(choose(
        Input::Right,
        Output::Cells([cell, Cell { value: 1, flags: [1 < 2, 1 > 2] }]),
        Output::Matrix([[1, 2], [211, 4]])
    ))
}
"#,
        ),
        (
            "direct Match result as call argument",
            r#"
fn main() -> int {
    let existing = Output::Count(211);
    score(match Input::Left {
        Input::Left => existing,
        Input::Right => Output::Empty
    })
}
"#,
        ),
    ] {
        expect_success(label, &with_prelude(body), &mut failures);
    }

    match checked_ir_and_llvm(&complete) {
        Err(error) => failures.push(format!(
            "complete checked direct-owner result failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedMatchResultPlaceAlloca",
                "CheckedEnumParameter",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumDispatch",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!("checked IR omitted {marker:?}:\n{debug}"));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "direct-owner result LLVM contained forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(&complete) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "direct-owner result LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic direct-owner result compile failed: {error}"
                )),
            }
        }
    }

    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tracked_root =
        repository_root.join("examples/conditional_owned_enum_match_results/main.aero");
    let tracked_module =
        repository_root.join("examples/conditional_owned_enum_match_results/values.aero");
    match (
        fs::read_to_string(&tracked_root),
        fs::read_to_string(&tracked_module),
    ) {
        (Ok(root), Ok(module))
            if root.contains("if result == 211")
                && module.contains("fn same_owner")
                && module.contains("fn nested")
                && module.contains("Input::Left => partial") => {}
        (Ok(root), Ok(module)) => failures.push(format!(
            "tracked CORE-075 example drifted:\nROOT\n{root}\nMODULE\n{module}"
        )),
        (root, module) => failures.push(format!(
            "tracked CORE-075 example is missing: root={:?}, module={:?}",
            root.err(),
            module.err()
        )),
    }
    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("@choose(")
                && llvm.contains("@same_owner(")
                && llvm.contains("@nested(")
                && llvm.contains("ret i32 211") => {}
        Ok(llvm) => failures.push(format!(
            "tracked CORE-075 example LLVM omitted composed evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!("tracked CORE-075 example did not compile: {error}")),
    }
    let workflow = fs::read_to_string(repository_root.join(".github/workflows/rust.yml"))
        .expect("read tracked Rust workflow");
    for anchor in [
        "Test conditional direct-owner enum Match result integration example",
        "cargo run -- check ../../examples/conditional_owned_enum_match_results/main.aero",
        "cargo run -- run ../../examples/conditional_owned_enum_match_results/main.aero",
        "opt-22 -passes=verify -disable-output ../../conditional_owned_enum_match_results.ll",
        "llc-22 -verify-machineinstrs ../../conditional_owned_enum_match_results.ll",
        "clang-22 -no-pie ../../conditional_owned_enum_match_results.o -o ../../conditional_owned_enum_match_results",
        "conditional direct-owner enum Match result example passed with exit code 211",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "stable/nightly workflow must contain exactly one {anchor:?} anchor"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-075 positive direct-owner result failures:\n{}",
        failures.join("\n---\n")
    );
}

#[test]
fn path_sensitive_post_states_and_exclusions_fail_closed() {
    let mut failures = Vec::new();

    expect_rejection(
        "all-path source reuse",
        &with_prelude(
            r#"
fn illegal(input: Input) -> int {
    let value = Output::Count(1);
    let result = match input { Input::Left => value, Input::Right => value };
    score(value)
}
fn main() -> int { illegal(Input::Left) }
"#,
        ),
        &["moved value"],
        &[CURRENT_FRESH_ONLY_DIAGNOSTIC],
        &mut failures,
    );
    expect_rejection(
        "partial-path source reuse",
        &with_prelude(
            r#"
fn illegal(input: Input) -> int {
    let value = Output::Count(1);
    let result = match input { Input::Left => value, Input::Right => Output::Empty };
    score(value)
}
fn main() -> int { illegal(Input::Left) }
"#,
        ),
        &[MAYBE_MOVED_DIAGNOSTIC],
        &[CURRENT_FRESH_ONLY_DIAGNOSTIC],
        &mut failures,
    );
    expect_rejection(
        "consumed scrutinee as result",
        &with_prelude(
            r#"
fn illegal(value: Output) -> Output {
    match value {
        Output::Empty => value,
        Output::Count(number) => value,
        Output::Pair(number, flag) => value,
        Output::Mark(glyph) => value,
        Output::CellValue(item) => value,
        Output::Cells(items) => value,
        Output::Matrix(matrix) => value,
        Output::Mixed(number, flag, glyph) => value
    }
}
fn main() -> int { score(illegal(Output::Count(1))) }
"#,
        ),
        &["reuses consumed scrutinee"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "wrong result owner schema",
        r#"
enum Input { Left, Right }
enum Output { Empty }
enum Other { Empty }
fn main() {
    let output = Output::Empty;
    let other = Other::Empty;
    let result = match Input::Left { Input::Left => output, Input::Right => other };
}
"#,
        &["result mismatch", "expected"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "owned call argument remains excluded",
        &with_prelude(
            r#"
fn forward(value: Output) -> Output { value }
fn illegal(input: Input, value: Output) -> Output {
    match input { Input::Left => forward(value), Input::Right => Output::Empty }
}
fn main() -> int { score(illegal(Input::Left, Output::Count(1))) }
"#,
        ),
        &["owned enum match result arm 1"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "nested external owned scrutinee remains excluded",
        &with_prelude(
            r#"
fn illegal(input: Input, nested: Input, value: Output) -> Output {
    match input {
        Input::Left => match nested {
            Input::Left => value,
            Input::Right => Output::Empty
        },
        Input::Right => Output::Empty
    }
}
fn main() -> int { score(illegal(Input::Left, Input::Left, Output::Count(1))) }
"#,
        ),
        &["owned enum match result arm 1"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "direct owner result inside loop remains excluded",
        &with_prelude(
            r#"
fn illegal() -> int {
    loop {
        let value = Output::Count(1);
        let result = match Input::Left {
            Input::Left => value,
            Input::Right => Output::Empty
        };
        return score(result);
    }
    0
}
fn main() -> int { illegal() }
"#,
        ),
        &["loop", "ownership"],
        &[CURRENT_FRESH_ONLY_DIAGNOSTIC],
        &mut failures,
    );
    expect_rejection(
        "same owner consumed twice on one composed path",
        &with_prelude(
            r#"
fn combine(left: Output, right: Output) -> int { score(left) + score(right) }
fn illegal(input: Input) -> int {
    let value = Output::Count(1);
    combine(
        match input { Input::Left => value, Input::Right => Output::Empty },
        match Input::Left { Input::Left => value, Input::Right => Output::Empty }
    )
}
fn main() -> int { illegal(Input::Left) }
"#,
        ),
        &["more than once", "path"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "owner-consuming Match argument inside result call remains excluded",
        &with_prelude(
            r#"
fn forward(value: Output) -> Output { value }
fn illegal(input: Input, value: Output) -> Output {
    match input {
        Input::Left => forward(match Input::Left {
            Input::Left => value,
            Input::Right => Output::Empty
        }),
        Input::Right => Output::Empty
    }
}
fn main() -> int { score(illegal(Input::Left, Output::Count(1))) }
"#,
        ),
        &["owned enum match result arm 1"],
        &[],
        &mut failures,
    );
    expect_rejection(
        "owned Match result array storage remains excluded",
        &with_prelude(
            r#"
fn illegal(input: Input) {
    let value = Output::Count(1);
    let stored = [match input {
        Input::Left => value,
        Input::Right => Output::Empty
    }];
}
fn main() { illegal(Input::Left); }
"#,
        ),
        &["fixed arrays", "Copy"],
        &[],
        &mut failures,
    );

    for (label, source, marker) in [
        (
            "checked admission independently rejects full-path reuse",
            with_prelude(
                r#"
fn illegal(input: Input) -> int {
    let value = Output::Count(1);
    let result = match input { Input::Left => value, Input::Right => value };
    score(value)
}
fn main() -> int { illegal(Input::Left) }
"#,
            ),
            "moved",
        ),
        (
            "checked admission independently rejects partial-path reuse",
            with_prelude(
                r#"
fn illegal(input: Input) -> int {
    let value = Output::Count(1);
    let result = match input { Input::Left => value, Input::Right => Output::Empty };
    score(value)
}
fn main() -> int { illegal(Input::Left) }
"#,
            ),
            MAYBE_MOVED_DIAGNOSTIC,
        ),
    ] {
        match checked_ir_without_semantic(&source) {
            Ok(checked) => failures.push(format!(
                "{label}: unexpectedly reached checked IR:\n{checked:#?}"
            )),
            Err(error) if error.contains(marker) => {}
            Err(error) => {
                failures.push(format!("{label}: diagnostic {error:?} omitted {marker:?}"))
            }
        }
    }

    expect_success(
        "CORE-074 fresh result remains supported",
        &with_prelude(
            r#"
fn choose(input: Input) -> Output {
    match input { Input::Left => Output::Count(211), Input::Right => Output::Empty }
}
fn main() -> int { score(choose(Input::Left)) }
"#,
        ),
        &mut failures,
    );

    assert!(
        failures.is_empty(),
        "CORE-075 path-state/exclusion failures:\n{}",
        failures.join("\n---\n")
    );
}
