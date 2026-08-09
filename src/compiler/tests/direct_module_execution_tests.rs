use compiler::{CompilerOptions, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
            "aero-direct-module-execution-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create direct-module execution workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create module fixture directory");
        }
        fs::write(&path, contents).expect("write module fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_aero(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for argument in args {
        command.arg(argument);
    }
    command.output().expect("run Aero CLI")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn expect_success(
    failures: &mut Vec<String>,
    label: &str,
    workspace: &TestWorkspace,
    args: &[&Path],
) -> Option<Output> {
    let output = run_aero(workspace, args);
    if output.status.success() {
        Some(output)
    } else {
        failures.push(format!(
            "{label}: unexpectedly failed with status {:?}:\n{}",
            output.status.code(),
            diagnostics(&output)
        ));
        None
    }
}

fn expect_failure(
    failures: &mut Vec<String>,
    label: &str,
    workspace: &TestWorkspace,
    args: &[&Path],
    required: &[&str],
    forbidden_artifact: Option<&Path>,
) {
    let output = run_aero(workspace, args);
    let text = diagnostics(&output);
    if output.status.success() {
        failures.push(format!("{label}: unexpectedly succeeded:\n{text}"));
    }
    for marker in required {
        if !text.contains(marker) {
            failures.push(format!(
                "{label}: diagnostic omitted required marker {marker:?}:\n{text}"
            ));
        }
    }
    if forbidden_artifact.is_some_and(Path::exists) {
        failures.push(format!(
            "{label}: failure published requested artifact {}",
            forbidden_artifact.expect("checked above").display()
        ));
    }
}

fn expect_failure_without(
    failures: &mut Vec<String>,
    label: &str,
    workspace: &TestWorkspace,
    args: &[&Path],
    required: &[&str],
    forbidden: &[&str],
    forbidden_artifact: Option<&Path>,
) {
    let output = run_aero(workspace, args);
    let text = diagnostics(&output);
    if output.status.success() {
        failures.push(format!("{label}: unexpectedly succeeded:\n{text}"));
    }
    for marker in required {
        if !text.contains(marker) {
            failures.push(format!(
                "{label}: diagnostic omitted required marker {marker:?}:\n{text}"
            ));
        }
    }
    for marker in forbidden {
        if text.contains(marker) {
            failures.push(format!(
                "{label}: diagnostic included forbidden later marker {marker:?}:\n{text}"
            ));
        }
    }
    if forbidden_artifact.is_some_and(Path::exists) {
        failures.push(format!(
            "{label}: failure published requested artifact {}",
            forbidden_artifact.expect("checked above").display()
        ));
    }
}

fn expect_definition_order(failures: &mut Vec<String>, label: &str, llvm: &str, expected: &[&str]) {
    let actual = llvm
        .lines()
        .filter_map(|line| {
            line.strip_prefix("define ")?
                .split_once('@')?
                .1
                .split_once('(')
                .map(|(name, _)| name)
        })
        .collect::<Vec<_>>();
    if actual != expected {
        failures.push(format!(
            "{label}: emitted definition order {actual:?} did not equal exact-name order {expected:?}"
        ));
    }
}

fn build_twice(
    failures: &mut Vec<String>,
    label: &str,
    workspace: &TestWorkspace,
    root: &Path,
) -> Option<String> {
    let first_path = workspace.path("first.ll");
    let second_path = workspace.path("second.ll");
    expect_success(
        failures,
        &format!("{label} first checked build"),
        workspace,
        &[Path::new("build"), root, Path::new("-o"), &first_path],
    )?;
    expect_success(
        failures,
        &format!("{label} second checked build"),
        workspace,
        &[Path::new("build"), root, Path::new("-o"), &second_path],
    )?;

    let first = match fs::read_to_string(&first_path) {
        Ok(llvm) => llvm,
        Err(error) => {
            failures.push(format!(
                "{label}: first LLVM artifact {} could not be read: {error}",
                first_path.display()
            ));
            return None;
        }
    };
    let second = match fs::read_to_string(&second_path) {
        Ok(llvm) => llvm,
        Err(error) => {
            failures.push(format!(
                "{label}: second LLVM artifact {} could not be read: {error}",
                second_path.display()
            ));
            return None;
        }
    };
    if first != second {
        let first_order = first
            .lines()
            .filter(|line| line.starts_with("define "))
            .collect::<Vec<_>>();
        let second_order = second
            .lines()
            .filter(|line| line.starts_with("define "))
            .collect::<Vec<_>>();
        let first_difference = first
            .lines()
            .zip(second.lines())
            .enumerate()
            .find(|(_, (left, right))| left != right)
            .map(|(index, (left, right))| {
                format!("line {}: first={left:?}, second={right:?}", index + 1)
            })
            .unwrap_or_else(|| {
                format!(
                    "line counts differ: first={}, second={}",
                    first.lines().count(),
                    second.lines().count()
                )
            });
        failures.push(format!(
            "{label}: repeated flattened direct-module builds were not byte-identical; \
             first order={first_order:?}; second order={second_order:?}; {first_difference}"
        ));
    }
    Some(first)
}

fn mixed_root_source() -> &'static str {
    r#"mod numeric;
mod text;

fn root_bias(value: int) -> int {
    return value + 2;
}

fn main() -> int {
    return module_score();
}
"#
}

fn mixed_numeric_source() -> &'static str {
    r#"fn consume(value: int) {
    let observed = value;
}

fn float_gate(value: float) -> bool {
    return value > 2.5;
}

fn module_score() -> int {
    let values = [10, 20, 30];
    consume(values.len());
    if values.len() == 3 && float_gate(3.5) && string_capabilities() {
        return root_bias(45);
    }
    return 1;
}
"#
}

fn mixed_text_source() -> &'static str {
    r#"fn string_capabilities() -> bool {
    let language: String = "Aero 🚀";
    let alias = language;
    return alias.len() == 6
        && alias == "Aero 🚀"
        && !alias.is_empty()
        && alias.contains("ero")
        && alias.starts_with("Aero")
        && alias.ends_with("🚀");
}
"#
}

fn instantiate_mixed_program(workspace: &TestWorkspace) -> PathBuf {
    let root = workspace.write("main.aero", mixed_root_source());
    workspace.write("numeric/mod.aero", mixed_numeric_source());
    workspace.write("text.aero", mixed_text_source());
    root
}

fn inspect_mixed_llvm(failures: &mut Vec<String>, label: &str, llvm: &str) {
    for required in [
        "define i32 @root_bias(i32 %aero.arg.value)",
        "define i32 @module_score()",
        "define i1 @string_capabilities()",
        "call i32 @module_score()",
        "call i1 @string_capabilities()",
        "call i32 @root_bias(i32 45)",
        "call void @consume(i32 3)",
        "call i1 @float_gate(double 0x400C000000000000)",
        "icmp",
    ] {
        if !llvm.contains(required) {
            failures.push(format!(
                "{label}: LLVM omitted cross-capability marker {required:?}:\n{llvm}"
            ));
        }
    }
    for forbidden in [
        "ArrayLength",
        "aero_string_contains",
        "aero_string_starts_with",
        "aero_string_ends_with",
    ] {
        if llvm.contains(forbidden) {
            failures.push(format!(
                "{label}: LLVM activated forbidden compatibility/runtime marker {forbidden:?}:\n{llvm}"
            ));
        }
    }
}

#[test]
fn flattened_direct_module_composition_is_complete_and_ci_executable() {
    let mut failures = Vec::new();

    let source_only = compile_program(
        "mod absent; fn main() -> int { return 1; }",
        CompilerOptions::default(),
    );
    match source_only {
        Err(error)
            if error
                == "Module resolution failed for `absent`: module declarations require an entry-file context" =>
            {}
        other => failures.push(format!(
            "source-only library module boundary changed: {other:?}"
        )),
    }

    {
        let workspace = TestWorkspace::new("zero-module-definition-order");
        let root = workspace.write(
            "main.aero",
            "fn zebra() -> int { return 2; } fn main() -> int { return alpha() + zebra(); } fn alpha() -> int { return 1; }",
        );
        if let Some(llvm) = build_twice(
            &mut failures,
            "zero-module checked definition order",
            &workspace,
            &root,
        ) {
            expect_definition_order(
                &mut failures,
                "zero-module checked definition order",
                &llvm,
                &["alpha", "main", "zebra"],
            );
        }
    }

    for (label, module_path) in [
        ("single file layout", "helper.aero"),
        ("single directory layout", "helper/mod.aero"),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write(
            "main.aero",
            "mod helper; fn main() -> int { return helper(); }",
        );
        workspace.write(module_path, "fn helper() -> int { return 47; }");
        expect_success(
            &mut failures,
            &format!("{label} checked analysis"),
            &workspace,
            &[Path::new("check"), &root],
        );
        if let Some(llvm) = build_twice(&mut failures, label, &workspace, &root) {
            for marker in ["define i32 @helper()", "call i32 @helper()", "ret i32 47"] {
                if !llvm.contains(marker) {
                    failures.push(format!("{label}: LLVM omitted marker {marker:?}:\n{llvm}"));
                }
            }
        }
    }

    {
        let workspace = TestWorkspace::new("file-first-precedence");
        let root = workspace.write(
            "main.aero",
            "mod helper; fn main() -> int { return selected(); }",
        );
        workspace.write("helper.aero", "fn selected() -> int { return 47; }");
        workspace.write("helper/mod.aero", "fn ignored() -> int { return 1; }");
        if let Some(llvm) = build_twice(
            &mut failures,
            "file-first module precedence",
            &workspace,
            &root,
        ) {
            if !llvm.contains("define i32 @selected()") || llvm.contains("define i32 @ignored()") {
                failures.push(format!(
                    "file-first module precedence selected the wrong candidate:\n{llvm}"
                ));
            }
        }
    }

    {
        let workspace = TestWorkspace::new("mixed-call-graph");
        let root = instantiate_mixed_program(&workspace);
        expect_success(
            &mut failures,
            "mixed-layout call graph checked analysis",
            &workspace,
            &[Path::new("check"), &root],
        );
        if let Some(llvm) = build_twice(&mut failures, "mixed-layout call graph", &workspace, &root)
        {
            inspect_mixed_llvm(&mut failures, "mixed-layout call graph", &llvm);
            expect_definition_order(
                &mut failures,
                "mixed-layout call graph",
                &llvm,
                &[
                    "consume",
                    "float_gate",
                    "main",
                    "module_score",
                    "root_bias",
                    "string_capabilities",
                ],
            );
        }
    }

    {
        let workspace = TestWorkspace::new("missing-module");
        let root = workspace.write("main.aero", "mod absent; fn main() -> int { return 1; }");
        let artifact = workspace.path("missing.ll");
        expect_failure(
            &mut failures,
            "missing direct module",
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &[
                "Module resolution failed for `absent`",
                "Cannot find module `absent`",
            ],
            Some(&artifact),
        );
    }

    {
        let workspace = TestWorkspace::new("malformed-module");
        let root = workspace.write("main.aero", "mod helper; fn main() -> int { return 1; }");
        workspace.write("helper.aero", "fn helper() { @ }");
        let artifact = workspace.path("malformed.ll");
        expect_failure(
            &mut failures,
            "malformed direct module",
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &["Lex error", "Unexpected character '@'"],
            Some(&artifact),
        );
    }

    {
        let workspace = TestWorkspace::new("unreadable-file-candidate");
        let root = workspace.write("main.aero", "mod helper; fn main() -> int { return 1; }");
        fs::create_dir_all(workspace.path("helper.aero"))
            .expect("create unreadable direct-module file candidate");
        let artifact = workspace.path("unreadable.ll");
        expect_failure(
            &mut failures,
            "unreadable direct-module candidate",
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &[
                "Module resolution failed for `helper`",
                "Could not read module file",
                "helper.aero",
            ],
            Some(&artifact),
        );
    }

    {
        let workspace = TestWorkspace::new("nested-module");
        let root = workspace.write("main.aero", "mod outer; fn main() -> int { return 1; }");
        workspace.write("outer.aero", "mod inner; fn outer() {}");
        workspace.write("inner.aero", "fn inner() {}");
        let artifact = workspace.path("nested.ll");
        expect_failure(
            &mut failures,
            "nested direct module",
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &[
                "Module resolution failed for `inner`",
                "nested module declarations are not supported",
            ],
            Some(&artifact),
        );
    }

    for (label, root_source, module_source, required) in [
        (
            "duplicate direct declaration",
            "mod helper; mod helper; fn main() -> int { return helper(); }",
            "fn helper() -> int { return 47; }",
            vec!["helper"],
        ),
        (
            "root/module symbol collision",
            "mod helper; fn helper() -> int { return 1; } fn main() -> int { return helper(); }",
            "fn helper() -> int { return 47; }",
            vec!["helper"],
        ),
        (
            "module entrypoint collision",
            "mod helper; fn main() -> int { return 47; }",
            "fn main() -> int { return 1; }",
            vec!["main"],
        ),
        (
            "cross-file undefined call",
            "mod helper; fn main() -> int { return missing(); }",
            "fn helper() -> int { return 47; }",
            vec!["missing"],
        ),
        (
            "cross-file wrong arity",
            "mod helper; fn main() -> int { return helper(); }",
            "fn helper(value: int) -> int { return value; }",
            vec!["helper", "expected 1", "actual 0"],
        ),
        (
            "cross-file wrong scalar type",
            "mod helper; fn main() -> int { return helper(1.5); }",
            "fn helper(value: int) -> int { return value; }",
            vec!["helper", "int", "float"],
        ),
        (
            "cross-file wrong return",
            "mod helper; fn main() -> int { return helper(); }",
            "fn helper() -> int { return 1 < 2; }",
            vec!["helper", "expected int", "actual bool"],
        ),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write("main.aero", root_source);
        workspace.write("helper.aero", module_source);
        let artifact = workspace.path("rejected.ll");
        expect_failure(
            &mut failures,
            label,
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &required,
            Some(&artifact),
        );
    }

    {
        let workspace = TestWorkspace::new("cross-file-child-precedence");
        let root = workspace.write(
            "main.aero",
            "mod helper; fn main() -> int { return helper(missing_value, 2); }",
        );
        workspace.write(
            "helper.aero",
            "fn helper(value: int) -> int { return value; }",
        );
        let artifact = workspace.path("child-precedence.ll");
        expect_failure_without(
            &mut failures,
            "cross-file child diagnostic precedes call arity",
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &["missing_value"],
            &["expected 1", "actual 2"],
            Some(&artifact),
        );
    }

    for (label, declarations, required, forbidden) in [
        (
            "alpha-first direct-module diagnostic order",
            "mod alpha; mod beta; fn main() -> int { return 1; }",
            "Unexpected character '@'",
            "Unexpected character '$'",
        ),
        (
            "beta-first direct-module diagnostic order",
            "mod beta; mod alpha; fn main() -> int { return 1; }",
            "Unexpected character '$'",
            "Unexpected character '@'",
        ),
    ] {
        let workspace = TestWorkspace::new(label);
        let root = workspace.write("main.aero", declarations);
        workspace.write("alpha.aero", "fn alpha() { @ }");
        workspace.write("beta.aero", "fn beta() { $ }");
        let artifact = workspace.path("ordered-module-diagnostic.ll");
        expect_failure_without(
            &mut failures,
            label,
            &workspace,
            &[Path::new("build"), &root, Path::new("-o"), &artifact],
            &[required],
            &[forbidden],
            Some(&artifact),
        );
    }

    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example_root = repository_root.join("examples/direct_module_integration/main.aero");
    let example_numeric =
        repository_root.join("examples/direct_module_integration/numeric/mod.aero");
    let example_text = repository_root.join("examples/direct_module_integration/text.aero");

    let tracked_sources = [
        (
            "tracked integration root",
            &example_root,
            mixed_root_source(),
        ),
        (
            "tracked directory-layout numeric module",
            &example_numeric,
            mixed_numeric_source(),
        ),
        (
            "tracked file-layout text module",
            &example_text,
            mixed_text_source(),
        ),
    ];
    let mut tracked_complete = true;
    for (label, path, expected) in tracked_sources {
        match fs::read_to_string(path) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => {
                tracked_complete = false;
                failures.push(format!(
                    "{label} {} does not equal the frozen source:\n{actual}",
                    path.display()
                ));
            }
            Err(error) => {
                tracked_complete = false;
                failures.push(format!(
                    "{label} {} is unavailable: {error}",
                    path.display()
                ));
            }
        }
    }

    if tracked_complete {
        let workspace = TestWorkspace::new("tracked-example");
        let root = workspace.write("main.aero", mixed_root_source());
        workspace.write("numeric/mod.aero", mixed_numeric_source());
        workspace.write("text.aero", mixed_text_source());
        if let Some(llvm) = build_twice(
            &mut failures,
            "tracked mixed-layout example",
            &workspace,
            &root,
        ) {
            inspect_mixed_llvm(&mut failures, "tracked mixed-layout example", &llvm);
        }
    }

    let workflow_path = repository_root.join(".github/workflows/rust.yml");
    match fs::read_to_string(&workflow_path) {
        Ok(workflow) => {
            for anchor in [
                "Test direct module integration example",
                "examples/direct_module_integration/main.aero",
                "direct_module_integration.ll",
                "opt-22 -passes=verify -disable-output ../../direct_module_integration.ll",
                "llc-22 -verify-machineinstrs ../../direct_module_integration.ll -o /dev/null",
                "llc-22 -filetype=obj ../../direct_module_integration.ll -o ../../direct_module_integration.o",
                "clang-22 ../../direct_module_integration.o -o ../../direct_module_integration",
                "../../direct_module_integration",
                "if [ $exit_code -ne 47 ]; then",
                "direct_module_integration test passed with exit code $exit_code",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "native workflow {} omitted anchor {anchor:?}",
                        workflow_path.display()
                    ));
                }
            }
        }
        Err(error) => failures.push(format!(
            "native workflow {} is unavailable: {error}",
            workflow_path.display()
        )),
    }

    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}
