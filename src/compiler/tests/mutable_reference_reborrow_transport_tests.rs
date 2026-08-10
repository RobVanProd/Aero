use compiler::ast::AstNode;
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/mutable_reference_reborrowing/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_reference_reborrowing/reborrows.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 253;

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
            "aero-mutable-reference-reborrow-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create mutable-reference reborrow test workspace");
        Self { root }
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
            .is_some_and(|name| name.starts_with("aero-mutable-reference-reborrow-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn parsed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed_ast(source)?)
        .map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed_ast(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
}

fn expect_success(label: &str, source: &str, required: &[&str]) -> Vec<String> {
    match compile_program(source, CompilerOptions::default()) {
        Err(error) => vec![format!("{label}: expected success, got {error}")],
        Ok(llvm) => required
            .iter()
            .filter(|fragment| !llvm.contains(**fragment))
            .map(|fragment| format!("{label}: LLVM missing {fragment:?}\n{llvm}"))
            .collect(),
    }
}

fn expect_rejection(label: &str, source: &str, expected: &str) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: unsupported mutable-reference reborrow form compiled:\n{llvm}"
        )),
        Err(error) if error.contains(expected) => None,
        Err(error) => Some(format!(
            "{label}: expected diagnostic containing {expected:?}, got {error:?}"
        )),
    }
}

fn run_cli(workspace: &TestWorkspace, arguments: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    for argument in arguments {
        command.arg(argument);
    }
    command
        .current_dir(&workspace.root)
        .output()
        .expect("run Aero CLI")
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn tracked_root_source() -> &'static str {
    r#"mod reborrows;

enum Mode { Off, On }
struct Packet { value: int, ready: bool }

fn read(value: &int) -> int { *value }
fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 7 } }

fn main() -> int {
    let base = 14;
    let packet = Packet { value: 4, ready: 1 == 1 };
    let rows = [packet, packet];

    let mut total = 200;
    {
        let alias: &mut int = &mut total;
        forward_once(alias);
        recurse(alias);
    }

    let mut ratio = 2.0;
    { let ratio_alias = &mut ratio; adjust(ratio_alias); }
    let mut ready = 1 == 2;
    { let ready_alias = &mut ready; set_ready(ready_alias); }

    let mut ratio_score = 0;
    if ratio > 2.0 { ratio_score = 10; }
    let mut ready_score = 0;
    if ready { ready_score = 5; }
    if packet.ready && total == 207 && "aero".len() == 4 {
        return total + ratio_score + ready_score + mode_score(Mode::On) + rows.len() + packet.value + "aero".len() + read(&base);
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn add_one(value: &mut int) { *value = *value + 1; }

fn forward_once(value: &mut int) -> int {
    add_one(value);
    *value = *value + 2;
    add_one(value);
    *value
}

fn recurse(value: &mut int) -> int {
    if *value < 207 {
        *value = *value + 1;
        recurse(value);
    }
    *value
}

fn adjust(value: &mut float) { *value = *value + 0.5; }
fn set_ready(value: &mut bool) { *value = 1 == 1; }
"#
}

#[test]
fn mutable_scalar_reference_identifier_reborrow_class_is_complete() {
    let mut failures = Vec::new();

    let parser_source = r#"
fn inner(value: &mut int) { *value = *value + 1; }
fn outer(value: &mut int) { inner(value); }
fn main() -> int { let mut value = 1; let alias = &mut value; outer(alias); *alias }
"#;
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser reborrow retention failed: {error}")),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            if debug.matches("Reference(").count() < 2
                || debug.matches("mutable: true").count() < 1
                || !debug.contains("name: \"inner\"")
                || !debug.contains("name: \"outer\"")
                || debug.matches("\"value\"").count() < 5
                || debug.matches("\"alias\"").count() < 2
                || !debug.contains("FunctionCall")
                || !debug.contains("Deref")
            {
                failures.push(format!(
                    "parser lost local-alias/parameter reborrow topology:\n{debug}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "inferred local Int alias reborrow and reuse",
            "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut value = 4; { let alias = &mut value; bump(alias); *alias = *alias + 2; bump(alias); } value }",
            vec![
                "define void @bump(double*",
                "call void @bump(double*",
                "store double",
            ],
        ),
        (
            "annotated local Int alias reborrow",
            "fn bump(value: &mut int) -> int { *value = *value + 1; *value } fn main() -> int { let mut value = 4; let mut result = 0; { let alias: &mut int = &mut value; result = bump(alias); } result + value }",
            vec!["define i32 @bump(double*", "call i32 @bump(double*"],
        ),
        (
            "local Float alias reborrow",
            "fn adjust(value: &mut float) -> float { *value = *value + 0.5; *value } fn main() -> int { let mut value = 1.5; let mut result = 0.0; { let alias: &mut float = &mut value; result = adjust(alias); } if result == 2.0 { return 7; } 1 }",
            vec![
                "define double @adjust(double*",
                "call double @adjust(double*",
            ],
        ),
        (
            "local Bool alias Void reborrow",
            "fn set(value: &mut bool) { *value = 1 == 1; } fn main() -> int { let mut value = 1 == 2; { let alias: &mut bool = &mut value; set(alias); } if value { return 9; } 1 }",
            vec!["define void @set(i1*", "call void @set(i1*", "store i1"],
        ),
        (
            "independent local alias parent identities",
            "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut left = 1; let mut right = 2; let mut total = 0; { let left_alias = &mut left; let right_alias = &mut right; bump(left_alias); bump(right_alias); total = *left_alias + *right_alias; } total + left + right }",
            vec!["call void @bump(double*", "store double"],
        ),
        (
            "nearest shadowed local alias identity",
            "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut first = 1; let mut second = 10; let mut total = 0; { let alias = &mut first; bump(alias); { let alias = &mut second; bump(alias); total = *alias; } total = total + *alias; } total + first + second }",
            vec!["call void @bump(double*", "store double"],
        ),
        (
            "mutable parameter forwarding and parent reuse",
            "fn inner(value: &mut int) { *value = *value + 1; } fn outer(value: &mut int) -> int { inner(value); *value = *value + 2; inner(value); *value } fn main() -> int { let mut value = 1; outer(&mut value) + value }",
            vec![
                "define i32 @outer(double*",
                "call void @inner(double*",
                "store double",
            ],
        ),
        (
            "multi-hop forwarding",
            "fn leaf(value: &mut int) { *value = *value + 1; } fn middle(value: &mut int) { leaf(value); } fn outer(value: &mut int) -> int { middle(value); *value } fn main() -> int { let mut value = 3; outer(&mut value) }",
            vec!["call void @leaf(double*", "call void @middle(double*"],
        ),
        (
            "branch and loop forwarding",
            "fn bump(value: &mut int) { *value = *value + 1; } fn advance(value: &mut int) -> int { while *value < 3 { bump(value); } if *value == 3 { bump(value); } else { *value = 0; } *value } fn main() -> int { let mut value = 1; advance(&mut value) }",
            vec!["while_start", "if_then", "call void @bump(double*"],
        ),
        (
            "forward-declared parameter reborrow",
            "fn outer(value: &mut int) -> int { change(value); *value } fn main() -> int { let mut value = 3; outer(&mut value) } fn change(value: &mut int) { *value = 8; }",
            vec!["call void @change(double*"],
        ),
        (
            "terminating direct recursive forwarding",
            "fn climb(value: &mut int) -> int { if *value < 3 { *value = *value + 1; climb(value); } *value } fn main() -> int { let mut value = 0; climb(&mut value) }",
            vec!["call i32 @climb(double*", "if_then"],
        ),
        (
            "CORE-056 direct owner compatibility",
            "fn bump(value: &mut int) -> int { *value = *value + 1; *value } fn main() -> int { let mut value = 4; bump(&mut value) + value }",
            vec!["call i32 @bump(double*"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    let checked_source = "fn inner(value: &mut int) { *value = *value + 1; } fn outer(value: &mut int) -> int { inner(value); *value = *value + 2; inner(value); *value } fn main() -> int { let mut owner = 4; let mut result = 0; { let alias = &mut owner; result = outer(alias); result = result + outer(alias); } result + owner }";
    match checked_ir_and_llvm(checked_source) {
        Err(error) => failures.push(format!("checked reborrow case failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for identity in [
                "MutableReference",
                "CheckedMutableReferenceParameter",
                "CheckedMutableBorrow",
                "CheckedMutableDereferenceAssignment",
                "CheckedMutableBorrowEnd",
            ] {
                if !debug.contains(identity) {
                    failures.push(format!("checked IR missing {identity}:\n{debug}"));
                }
            }
            let borrow_count = debug.matches("CheckedMutableBorrow {").count();
            let end_count = debug.matches("CheckedMutableBorrowEnd {").count();
            if borrow_count != 5 || end_count != 5 {
                failures.push(format!(
                    "checked IR expected five root/child borrows and ends, got borrows={borrow_count}, ends={end_count}:\n{debug}"
                ));
            }
            for anchor in [
                "define void @inner(double*",
                "define i32 @outer(double*",
                "call void @inner(double*",
                "call i32 @outer(double*",
                "store double",
                "load double",
            ] {
                if !llvm.contains(anchor) {
                    failures.push(format!("mutable reborrow LLVM missing {anchor:?}:\n{llvm}"));
                }
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") {
                failures.push(format!(
                    "mutable reborrow lowering used pointer/integer conversion:\n{llvm}"
                ));
            }
        }
    }

    if let Err(error) = compile_program(
        "fn add(value: &mut int, amount: int) -> int { *value = *value + amount; *value } fn main() -> int { let mut value = 1; let alias = &mut value; add(alias, 2) }",
        CompilerOptions::default(),
    ) {
        failures.push(format!(
            "mixed mutable-reference reborrow and scalar parameter regressed: {error}"
        ));
    }

    for (label, source, expected) in [
        (
            "immutable alias argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let value = 1; let alias = &value; bump(alias) }",
            "requires a mutable-reference identifier",
        ),
        (
            "owned scalar identifier argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; bump(value) }",
            "requires a mutable-reference identifier",
        ),
        (
            "uninitialized mutable alias",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let alias: &mut int; bump(alias) }",
            "uninitialized variable `alias`",
        ),
        (
            "mismatched alias pointee",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut ratio = 1.0; let alias = &mut ratio; bump(alias) }",
            "mutable reference call pointee mismatch",
        ),
        (
            "root owner remains borrowed after alias call",
            "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut value = 1; let alias = &mut value; bump(alias); value + *alias }",
            "cannot read `value` while it is mutably borrowed",
        ),
        (
            "mutable alias relocation remains rejected",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; let alias = &mut value; let relocated = alias; bump(relocated) }",
            "mutable reference aliases cannot be copied or relocated",
        ),
        (
            "explicit dereference reborrow remains excluded",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; let alias = &mut value; bump(&mut *alias) }",
            "requires an identifier place",
        ),
        (
            "immutable parameter forwarded to mutable",
            "fn inner(value: &mut int) -> int { *value } fn outer(value: &int) -> int { inner(value) } fn main() -> int { let value = 1; outer(&value) }",
            "requires a mutable-reference identifier",
        ),
        (
            "two mutable parameters remain excluded",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { 0 }",
            "at most one mutable-reference parameter",
        ),
        (
            "mutable reference result remains excluded",
            "fn bad(value: &mut int) -> &mut int { value } fn main() -> int { 0 }",
            "reference results require lifetime semantics",
        ),
        (
            "String mutable parameter remains excluded",
            "fn bad(value: &mut String) -> int { 0 } fn main() -> int { 0 }",
            "mutable reference parameter pointee is not admitted Copy-data",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    for source in [
        "fn read(value: &int) -> int { *value } fn main() -> int { let value = 7; let alias = &value; read(alias) + *alias + value }",
        "fn main() -> int { let mut value = 1; let alias = &mut value; *alias = 2; *alias }",
        "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut value = 1; bump(&mut value); value }",
    ] {
        if let Err(error) = compile_program(source, CompilerOptions::default()) {
            failures.push(format!(
                "accepted reference compatibility regressed: {error}"
            ));
        }
    }

    match analyzed_ast(checked_source) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:#?}");
            if debug.contains("CheckedMutableReferenceParameter")
                || debug.contains("CheckedMutableBorrowEnd")
            {
                failures.push(format!(
                    "deprecated raw generation activated mutable reborrow identities:\n{debug}"
                ));
            }
        }
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_ROOT);
    let module = root.join(EXAMPLE_MODULE);
    match (fs::read_to_string(&example), fs::read_to_string(&module)) {
        (Ok(actual), Ok(module_actual)) => {
            if actual != tracked_root_source() {
                failures.push(format!(
                    "tracked root example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!("tracked module drifted at {}", module.display()));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked mutable reborrow example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for mutable reborrow integration anchor");
    for anchor in [
        "Test mutable scalar-reference identifier reborrow integration example",
        "examples/mutable_reference_reborrowing/main.aero",
        "cargo run -- build ../../examples/mutable_reference_reborrowing/main.aero -o ../../mutable_reference_reborrowing.ll",
        "if [ $exit_code -ne 253 ]; then",
        "mutable scalar-reference identifier reborrow example passed with exit code 253",
    ] {
        let count = workflow.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "Rust workflow anchor {anchor:?} occurs {count} times instead of once"
            ));
        }
    }

    if example.is_file() && module.is_file() {
        let workspace = TestWorkspace::new("tracked-example");
        let output_path = workspace.path("mutable_reference_reborrowing.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked mutable reborrow example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn bump(value: &mut int) -> int { *value } fn main() -> int { let value = 1; let alias = &value; bump(alias) }",
    )
    .expect("write invalid mutable reborrow source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("requires a mutable-reference identifier")
    {
        failures.push(format!(
            "invalid mutable reborrow CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-057 mutable scalar-reference reborrow failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
