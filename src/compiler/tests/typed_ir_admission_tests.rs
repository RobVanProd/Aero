use compiler::{CompilerOptions, compile_program};
use std::collections::HashMap;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "aero-typed-ir-admission-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create typed-IR admission workspace");
        Self { root }
    }

    fn source_path(&self) -> PathBuf {
        self.root.join("case.aero")
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-typed-ir-admission-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[derive(Clone, Copy)]
struct RejectionCase {
    name: &'static str,
    source: &'static str,
    expected_prefix: &'static str,
    check_cli: bool,
}

fn combined_output(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn rejection_failures(label: &str, cases: &[RejectionCase]) -> Vec<String> {
    let workspace = TestWorkspace::new(label);
    let source_path = workspace.source_path();
    let mut failures = Vec::new();

    for case in cases {
        match catch_unwind(AssertUnwindSafe(|| {
            compile_program(case.source, CompilerOptions::default())
        })) {
            Err(_) => failures.push(format!(
                "{}: compile_program unwound instead of returning {}",
                case.name, case.expected_prefix
            )),
            Ok(Ok(llvm)) => failures.push(format!(
                "{}: compile_program unexpectedly accepted source:\n{}",
                case.name, llvm
            )),
            Ok(Err(error)) if !error.starts_with(case.expected_prefix) => failures.push(format!(
                "{}: expected error prefix {:?}, got {:?}",
                case.name, case.expected_prefix, error
            )),
            Ok(Err(_)) => {}
        }

        if case.check_cli {
            fs::write(&source_path, case.source).expect("write CLI admission source");
            let output = Command::new(env!("CARGO_BIN_EXE_aero"))
                .arg("check")
                .arg(&source_path)
                .current_dir(&workspace.root)
                .output()
                .expect("run aero check");
            let diagnostics = combined_output(&output);
            if output.status.success() {
                failures.push(format!(
                    "{}: `aero check` returned success; output:\n{}",
                    case.name, diagnostics
                ));
            }
            let expected_cli = case
                .expected_prefix
                .strip_prefix("Semantic Analysis Error: ")
                .unwrap_or(case.expected_prefix);
            if !diagnostics.contains(expected_cli) {
                failures.push(format!(
                    "{}: `aero check` omitted {:?}; output:\n{}",
                    case.name, expected_cli, diagnostics
                ));
            }
            if diagnostics.contains("panicked at") {
                failures.push(format!(
                    "{}: `aero check` exposed a panic; output:\n{}",
                    case.name, diagnostics
                ));
            }
        }
    }

    failures
}

fn compile_without_unwind(name: &str, source: &str, failures: &mut Vec<String>) -> Option<String> {
    match catch_unwind(AssertUnwindSafe(|| {
        compile_program(source, CompilerOptions::default())
    })) {
        Err(_) => {
            failures.push(format!("{name}: compile_program unwound"));
            None
        }
        Ok(Err(error)) => {
            failures.push(format!("{name}: unexpectedly rejected source: {error}"));
            None
        }
        Ok(Ok(llvm)) => Some(llvm),
    }
}

fn llvm_function_body<'a>(llvm: &'a str, signature: &str) -> Option<&'a str> {
    let start = llvm.find(signature)?;
    let remainder = &llvm[start..];
    let open = remainder.find('{')?;
    let body = &remainder[open + 1..];
    let close = body.find('}')?;
    Some(&body[..close])
}

#[derive(Debug)]
struct LlvmFunctionSignature {
    name: String,
    result: String,
    parameters: Vec<String>,
}

fn llvm_function_signatures(llvm: &str) -> Vec<LlvmFunctionSignature> {
    llvm.lines()
        .filter_map(|line| {
            let header = line.trim().strip_prefix("define ")?;
            let (before_name, after_at) = header.split_once('@')?;
            let result = before_name.split_whitespace().last()?.to_string();
            let (name, after_name) = after_at.split_once('(')?;
            let (parameters, _) = after_name.split_once(')')?;
            let parameters = parameters
                .split(',')
                .filter_map(|parameter| parameter.split_whitespace().next())
                .map(str::to_string)
                .collect();
            Some(LlvmFunctionSignature {
                name: name.to_string(),
                result,
                parameters,
            })
        })
        .collect()
}

fn llvm_call_parameter_types(body: &str, result: &str, function: &str) -> Option<Vec<String>> {
    let needle = format!("call {result} @{function}(");
    let line = body
        .lines()
        .map(str::trim)
        .find(|line| line.contains(&needle))?;
    let arguments = line.split_once(&needle)?.1.split_once(')')?.0;
    Some(
        arguments
            .split(',')
            .filter_map(|argument| argument.split_whitespace().next())
            .map(str::to_string)
            .collect(),
    )
}

fn llvm_double_literal_matches(literal: &str, expected: f64) -> bool {
    literal
        .strip_prefix("0x")
        .and_then(|bits| u64::from_str_radix(bits, 16).ok())
        .map(f64::from_bits)
        .or_else(|| literal.parse::<f64>().ok())
        == Some(expected)
}

fn overflowing_fold_failures(llvm: &str) -> Vec<String> {
    let Some(body) = llvm_function_body(llvm, "define i32 @overflow_candidate(") else {
        return vec![format!(
            "overflowing fold: missing candidate function:\n{llvm}"
        )];
    };
    let folded = format!("0x{:016X}", (2_147_483_648_f64).to_bits());
    let operation = body.lines().map(str::trim).find(|line| {
        let Some((_, operands)) = line.split_once(" = fadd double ") else {
            return false;
        };
        let Some((left, right)) = operands.split_once(',') else {
            return false;
        };
        (llvm_double_literal_matches(left.trim(), 2_147_483_647_f64)
            && llvm_double_literal_matches(right.trim(), 1_f64))
            || (llvm_double_literal_matches(left.trim(), 1_f64)
                && llvm_double_literal_matches(right.trim(), 2_147_483_647_f64))
    });
    let Some(operation) = operation else {
        return vec![format!(
            "overflowing fold did not retain the original logical add operands:\n{body}"
        )];
    };
    let Some(result) = operation
        .strip_prefix('%')
        .and_then(|line| line.split_once(" = "))
        .map(|(result, _)| result)
    else {
        return vec![format!(
            "overflowing fold had malformed add output: {operation}"
        )];
    };
    let Some(conversion) = body
        .lines()
        .map(str::trim)
        .find(|line| line.contains(&format!(" = fptosi double %{result} to i32")))
    else {
        return vec![format!(
            "overflowing fold operation result %{result} did not feed the i32 return ABI:\n{body}"
        )];
    };
    let converted_result = conversion
        .strip_prefix('%')
        .and_then(|line| line.split_once(" = "))
        .map(|(result, _)| result);
    let mut failures = Vec::new();
    if converted_result.is_none_or(|result| !body.contains(&format!("ret i32 %{result}"))) {
        failures.push(format!(
            "overflowing fold conversion was dead instead of supplying the return:\n{body}"
        ));
    }
    if body.contains(&folded) || body.contains("ret i32 2147483648") {
        failures.push(format!(
            "overflowing fold materialized the out-of-range result instead of using the add:\n{body}"
        ));
    }
    failures
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScalarKind {
    I1,
    Double,
}

fn llvm_scalar_definitions(llvm: &str) -> HashMap<String, ScalarKind> {
    let mut definitions = HashMap::new();
    for line in llvm.lines().map(str::trim) {
        if let Some(parameters) = line
            .strip_prefix("define ")
            .and_then(|line| line.split_once('('))
            .and_then(|(_, parameters)| parameters.split_once(')'))
            .map(|(parameters, _)| parameters)
        {
            for parameter in parameters.split(',').map(str::trim) {
                if let Some(name) = parameter.strip_prefix("i1 %") {
                    let name = name
                        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.')
                        .next()
                        .unwrap_or(name);
                    definitions.insert(name.to_string(), ScalarKind::I1);
                }
            }
        }
        let Some((name, rhs)) = line
            .strip_prefix('%')
            .and_then(|line| line.split_once(" = "))
        else {
            continue;
        };
        let kind = if rhs.starts_with("icmp ")
            || rhs.starts_with("fcmp ")
            || rhs.starts_with("call i1 ")
            || rhs.starts_with("load i1,")
        {
            Some(ScalarKind::I1)
        } else if rhs.starts_with("fadd double ")
            || rhs.starts_with("fsub double ")
            || rhs.starts_with("fmul double ")
            || rhs.starts_with("fdiv double ")
            || rhs.starts_with("call double ")
            || rhs.starts_with("load double,")
            || rhs.starts_with("sitofp ")
            || rhs.starts_with("uitofp ")
        {
            Some(ScalarKind::Double)
        } else {
            None
        };
        if let Some(kind) = kind {
            definitions.insert(name.to_string(), kind);
        }
    }
    definitions
}

fn has_typed_register_use(line: &str, ty: &str, register: &str) -> bool {
    let needle = format!("{ty} %{register}");
    let mut remainder = line;
    while let Some(index) = remainder.find(&needle) {
        let after = &remainder[index + needle.len()..];
        if after
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.')
        {
            return true;
        }
        remainder = &after[1..];
    }
    false
}

fn obvious_bool_type_mismatches(llvm: &str) -> Vec<String> {
    let definitions = llvm_scalar_definitions(llvm);
    let mut failures = Vec::new();
    for line in llvm.lines().map(str::trim) {
        for (register, kind) in &definitions {
            match kind {
                ScalarKind::I1 if has_typed_register_use(line, "double", register) => failures
                    .push(format!(
                        "i1 register %{register} is consumed as double: {line}"
                    )),
                ScalarKind::Double if has_typed_register_use(line, "i1", register) => failures
                    .push(format!(
                        "double register %{register} is consumed as i1: {line}"
                    )),
                _ => {}
            }
        }
    }
    failures.sort();
    failures.dedup();
    failures
}

#[test]
fn checked_admission_reports_panicking_scalar_sources_without_unwind() {
    let cases = [
        RejectionCase {
            name: "array comparison",
            source: "fn main() { let equal = [1] == [1]; }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "Void print returned as scalar",
            source: "fn value() -> int { return println!(\"x\"); } fn main() { value(); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "Void print passed as argument",
            source: "fn id(value: int) -> int { return value; } fn main() { id(println!(\"x\")); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "constant integer division by zero",
            source: "fn main() { let quotient: int = 1 / 0; }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "folded constant integer division by zero",
            source: "fn main() { let quotient: int = 1 / (1 - 1); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "integer literal above i32 range",
            source: "fn main() { let too_large: int = 2147483648; }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "integer literal below i32 range",
            source: "fn main() { let too_small: int = -2147483649; }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
    ];

    let failures = rejection_failures("scalar-errors", &cases);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn checked_admission_accepts_static_string_equality_without_unwind() {
    let llvm = compile_program(
        r#"fn main() { let equal = "left" == "right"; }"#,
        CompilerOptions::default(),
    )
    .expect("static literal String equality should pass checked admission");

    assert!(
        llvm.contains("icmp ne i32 0, 0"),
        "false static equality must retain executable Bool IR:\n{llvm}"
    );
}

#[test]
fn checked_admission_rejects_fabricated_scalar_fallbacks_and_accepts_verified_borrows() {
    let borrow_llvm = compile_program(
        "fn main() { let value = 7; let borrowed = &value; }",
        CompilerOptions::default(),
    )
    .expect("an immutable local Int borrow should pass checked admission");
    assert!(
        borrow_llvm.contains("getelementptr inbounds double, double* %ptr"),
        "a verified immutable local Int borrow must retain alias-place LLVM:\n{borrow_llvm}"
    );

    let deref_llvm = compile_program(
        "fn main() { let value = 7; let borrowed = &value; let copied = *borrowed; }",
        CompilerOptions::default(),
    )
    .expect("a dereferenced immutable local Int borrow should pass checked admission");
    assert!(
        deref_llvm.contains("load double, double* %ptr"),
        "a verified immutable local Int dereference must retain exact load LLVM:\n{deref_llvm}"
    );

    let cases = [
        RejectionCase {
            name: "Some construction",
            source: "fn main() { let option = Some(7); }",
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "Ok construction",
            source: "fn main() { let result = Ok(9); }",
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "empty array without an element type",
            source: "fn main() { let values = []; }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "mutable compile-time string alias",
            source: "fn main() { let mut message = \"x\"; println!(\"{}\", message); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
    ];

    let failures = rejection_failures("fallback-errors", &cases);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn overflowing_constant_fold_remains_an_unfurled_logical_operation() {
    let source = r#"
fn overflow_candidate() -> int { return 2147483647 + 1; }
fn main() { overflow_candidate(); }
"#;
    let mut failures = Vec::new();
    if let Some(llvm) = compile_without_unwind("overflowing fold", source, &mut failures) {
        failures.extend(overflowing_fold_failures(&llvm));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn top_level_float_return_is_explicitly_converted_to_an_i32_exit_code() {
    let source = include_str!("../../../examples/mixed.aero");
    let mut failures = Vec::new();
    if let Some(llvm) =
        compile_without_unwind("mixed arithmetic entry point", source, &mut failures)
    {
        for marker in ["define i32 @main()", "fptosi double", "ret i32"] {
            if !llvm.contains(marker) {
                failures.push(format!(
                    "mixed arithmetic entry point missing `{marker}`:\n{llvm}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn signed_minimum_parameter_names_and_nested_parameter_shadowing_remain_admitted() {
    let cases = [
        (
            "signed i32 minimum",
            "fn minimum() -> int { return -2147483648; } fn main() { let value = minimum(); }",
            &["define i32 @minimum()", "ret i32 -2147483648"][..],
        ),
        (
            "collision-prone parameter names",
            "fn collide(reg0: int, ptr0: int) -> int { return reg0 + ptr0; } fn main() { let value = collide(1, 2); }",
            &[
                "define i32 @collide(i32 %aero.arg.reg0, i32 %aero.arg.ptr0)",
                "%aero.arg.reg0",
                "%aero.arg.ptr0",
            ][..],
        ),
        (
            "nested parameter shadowing",
            "fn shadow(value: int) -> float { { let value: float = 1.5; } return 2.5; } fn main() { let value = shadow(1); }",
            &["define double @shadow(i32 %aero.arg.value)", "store double"][..],
        ),
    ];
    let mut failures = Vec::new();
    for (name, source, markers) in cases {
        if let Some(llvm) = compile_without_unwind(name, source, &mut failures) {
            for marker in markers {
                if !llvm.contains(marker) {
                    failures.push(format!("{name} missing `{marker}`:\n{llvm}"));
                }
            }
            if name == "collision-prone parameter names"
                && (llvm.contains("i32 %reg0") || llvm.contains("i32 %ptr0"))
            {
                failures.push(format!(
                    "collision-prone parameter retained a generated local name:\n{llvm}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn boolean_binding_slots_use_i1_consistently() {
    let source = r#"
fn main() {
    let flag = 1 < 2;
    if flag {
        println!("true");
    }
}
"#;
    let mut failures = Vec::new();
    if let Some(llvm) = compile_without_unwind("Boolean slot", source, &mut failures) {
        for mismatch in obvious_bool_type_mismatches(&llvm) {
            failures.push(format!("Boolean slot: {mismatch}"));
        }
        for marker in ["alloca i1", "store i1", "load i1"] {
            if !llvm.contains(marker) {
                failures.push(format!("Boolean slot missing `{marker}`:\n{llvm}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn boolean_returns_and_calls_use_i1_consistently() {
    let cases: [(&str, &str, &[&str]); 3] = [
        (
            "Boolean return",
            r#"
fn truth() -> bool { return 1 < 2; }
fn main() { let observed = truth(); }
"#,
            &["define i1 @truth(", "ret i1", "call i1 @truth(", "store i1"],
        ),
        (
            "Boolean call",
            r#"
fn identity_bool(flag: bool) -> bool { return flag; }
fn main() { let selected = identity_bool(1 < 2); }
"#,
            &[
                "define i1 @identity_bool(i1 %aero.arg.flag)",
                "call i1 @identity_bool(i1",
                "store i1",
            ],
        ),
        (
            "Boolean equality",
            r#"
fn same(left: bool, right: bool) -> bool { return left == right; }
fn main() { let selected = same(1 < 2, 2 < 3); }
"#,
            &[
                "define i1 @same(i1 %aero.arg.left, i1 %aero.arg.right)",
                "icmp eq i1",
            ],
        ),
    ];
    let mut failures = Vec::new();
    for (name, source, markers) in cases {
        if let Some(llvm) = compile_without_unwind(name, source, &mut failures) {
            for mismatch in obvious_bool_type_mismatches(&llvm) {
                failures.push(format!("{name}: {mismatch}"));
            }
            for &marker in markers {
                if !llvm.contains(marker) {
                    failures.push(format!("{name} missing `{marker}`:\n{llvm}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn exact_zero_argument_array_iter_remains_admitted() {
    let integer_source = r#"
fn main() {
    let values = [11, 22];
    for item in values.iter() {
        let copied: int = item;
    }
}
"#;
    let mut failures = Vec::new();
    if let Some(llvm) = compile_without_unwind("integer array iter", integer_source, &mut failures)
    {
        for marker in [
            "alloca [2 x double]",
            "getelementptr inbounds [2 x double]",
            "icmp slt i32",
        ] {
            if !llvm.contains(marker) {
                failures.push(format!("array iter missing `{marker}`:\n{llvm}"));
            }
        }
    }
    let float_source = r#"
fn main() {
    let values = [1.5, 2.5];
    for item in values.iter() {
        let copied: float = item;
    }
}
"#;
    if let Some(llvm) = compile_without_unwind("float array iter", float_source, &mut failures) {
        for marker in [
            "alloca [2 x double]",
            "getelementptr inbounds [2 x double]",
            "fptosi double",
            "icmp slt i32",
            "fadd double",
        ] {
            if !llvm.contains(marker) {
                failures.push(format!("float array iter missing `{marker}`:\n{llvm}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn zero_length_numeric_repeats_preserve_explicit_element_types() {
    let cases = [
        ("zero Int repeat", "fn main() { let values = [1; 0]; }"),
        ("zero Float repeat", "fn main() { let values = [1.5; 0]; }"),
    ];
    let mut failures = Vec::new();
    for (name, source) in cases {
        if let Some(llvm) = compile_without_unwind(name, source, &mut failures) {
            if !llvm.contains("alloca [0 x double]") {
                failures.push(format!("{name} missed zero-length allocation:\n{llvm}"));
            }
            if llvm.contains("getelementptr inbounds [0 x double]") {
                failures.push(format!(
                    "{name} emitted an out-of-bounds initializer:\n{llvm}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn generated_closure_symbols_skip_user_function_names() {
    let source = r#"
fn main() { let callback = | | 1; let value = callback(); }
fn __closure_0() -> int { return 99; }
"#;
    let mut failures = Vec::new();
    if let Some(llvm) = compile_without_unwind("closure symbol collision", source, &mut failures) {
        for marker in ["define i32 @__closure_0()", "define i32 @__closure_1()"] {
            if !llvm.contains(marker) {
                failures.push(format!(
                    "closure symbol collision missed `{marker}`:\n{llvm}"
                ));
            }
        }
        let main = llvm_function_body(&llvm, "define i32 @main(").unwrap_or_default();
        if !main.contains("call i32 @__closure_1()") || main.contains("call i32 @__closure_0()") {
            failures.push(format!(
                "closure alias did not target the collision-free generated symbol:\n{main}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn scalar_closure_bindings_are_compile_time_aliases_without_runtime_values() {
    let cases = [
        (
            "shadowing Int closure",
            r#"
fn compute(value: int) -> int { return value; }
fn main() { let compute = |value: int| value + 1; }
"#,
            r#"
fn compute(value: int) -> int { return value; }
fn main() { let compute = |value: int| value + 1; compute(7); }
"#,
            "i32",
            &["i32"][..],
            Some("compute"),
        ),
        (
            "Float closure",
            "fn main() { let project = |value: float| value + 0.5; }",
            "fn main() { let project = |value: float| value + 0.5; project(1.5); }",
            "double",
            &["double"][..],
            None,
        ),
        (
            "Bool closure",
            "fn main() { let predicate = |value: bool| value; }",
            "fn main() { let predicate = |value: bool| value; predicate(1 < 2); }",
            "i1",
            &["i1"][..],
            None,
        ),
    ];
    let mut failures = Vec::new();
    for (
        name,
        binding_source,
        call_source,
        expected_result,
        expected_parameters,
        shadowed_function,
    ) in cases
    {
        if let Some(binding_llvm) =
            compile_without_unwind(&format!("{name} binding"), binding_source, &mut failures)
        {
            let candidates = llvm_function_signatures(&binding_llvm)
                .into_iter()
                .filter(|signature| {
                    signature.name != "main"
                        && shadowed_function.is_none_or(|name| signature.name != name)
                        && signature.result == expected_result
                        && signature.parameters == expected_parameters
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                failures.push(format!(
                    "{name}: binding-only source expected exactly one generated callable with signature {expected_result}({expected_parameters:?}), got {candidates:?}:\n{binding_llvm}"
                ));
            }
            if let Some(main) = llvm_function_body(&binding_llvm, "define i32 @main(") {
                let runtime_binding_instructions = main
                    .lines()
                    .map(str::trim)
                    .filter(|line| {
                        (line.starts_with('%') && line.contains(" = "))
                            || line.starts_with("store ")
                            || line.starts_with("call ")
                    })
                    .collect::<Vec<_>>();
                if !runtime_binding_instructions.is_empty() {
                    failures.push(format!(
                        "{name}: binding-only source emitted a runtime place/store/fabricated value: {runtime_binding_instructions:?}\n{main}"
                    ));
                }
            } else {
                failures.push(format!(
                    "{name}: binding-only source missed synthesized main:\n{binding_llvm}"
                ));
            }
            for mismatch in obvious_bool_type_mismatches(&binding_llvm) {
                failures.push(format!("{name} binding: {mismatch}"));
            }
        }

        if let Some(call_llvm) =
            compile_without_unwind(&format!("{name} call"), call_source, &mut failures)
        {
            let Some(main) = llvm_function_body(&call_llvm, "define i32 @main(") else {
                failures.push(format!(
                    "{name}: call source missed synthesized main:\n{call_llvm}"
                ));
                continue;
            };
            let candidates = llvm_function_signatures(&call_llvm)
                .into_iter()
                .filter(|signature| {
                    signature.name != "main"
                        && shadowed_function.is_none_or(|name| signature.name != name)
                        && signature.result == expected_result
                        && signature.parameters == expected_parameters
                })
                .collect::<Vec<_>>();
            let [callable] = candidates.as_slice() else {
                failures.push(format!(
                    "{name}: call source expected exactly one generated callable with signature {expected_result}({expected_parameters:?}), got {candidates:?}:\n{call_llvm}"
                ));
                continue;
            };
            let call_parameters = llvm_call_parameter_types(main, expected_result, &callable.name);
            let call_signature_matches = call_parameters.as_ref().is_some_and(|actual| {
                actual
                    .iter()
                    .map(String::as_str)
                    .eq(expected_parameters.iter().copied())
            });
            if !call_signature_matches {
                failures.push(format!(
                    "{name}: main did not call @{} with signature {expected_result}({expected_parameters:?}); observed {call_parameters:?}:\n{main}",
                    callable.name
                ));
            }
            if shadowed_function.is_some_and(|shadowed| {
                main.contains(&format!("call {expected_result} @{shadowed}("))
            }) {
                failures.push(format!(
                    "{name}: lexical closure shadowing called the top-level function:\n{main}"
                ));
            }
            for mismatch in obvious_bool_type_mismatches(&call_llvm) {
                failures.push(format!("{name}: {mismatch}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn checked_admission_rejects_closure_capture_escape_and_rich_signatures() {
    let cases = [
        RejectionCase {
            name: "closure capture",
            source: "fn main() { let outer = 7; let callback = | | outer; callback(); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "closure passed as value",
            source: "fn consume(value: int) -> int { return value; } fn main() { let callback = | | 1; let result = consume(callback); }",
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "closure returned as value",
            source: "fn make() -> int { let callback = | | 1; return callback; } fn main() { let value = make(); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "closure string result",
            source: r#"fn main() { let callback = | | "ready"; callback(); }"#,
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "closure string parameter",
            source: r#"fn main() { let callback = |value: string| 1; callback("ready"); }"#,
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "closure composite parameter",
            source: "struct Point { x: int } fn main() { let callback = |value: Point| 1; }",
            expected_prefix: "IR Generation Error:",
            check_cli: false,
        },
        RejectionCase {
            name: "closure array index fallback",
            source: "fn main() { let callback = | | [7][0]; let value = callback(); }",
            expected_prefix: "IR Generation Error:",
            check_cli: true,
        },
        RejectionCase {
            name: "closure assignment",
            source: "fn main() { let mut callback = | | 1; callback = | | 2; }",
            expected_prefix: "Parse error:",
            check_cli: true,
        },
    ];

    let failures = rejection_failures("closure-errors", &cases);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
