use compiler::{
    CompilerOptions, IrGenerator, LanguageProfile, LogicalType, SemanticAnalyzer, check_file,
    check_program, compile_file, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

const FIXED_INT_ARRAY_PROGRAM: &str =
    include_str!("../../../examples/fixed_int_array_v0/main.aero");
const FIXED_INT_ARRAY_WRAPPING_EDGES: &str =
    include_str!("../../../examples/fixed_int_array_v0/wrapping_edges.aero");
const NEGATIVE_RUNTIME_INDEX: &str =
    include_str!("../../../examples/fixed_int_array_v0/runtime_fail/negative_index.aero");
const EQUAL_TO_COUNT_RUNTIME_INDEX: &str =
    include_str!("../../../examples/fixed_int_array_v0/runtime_fail/equal_to_count_index.aero");

const IMMUTABLE_ARRAY_VALUE_COMPOSITION: &str = r#"
fn transform(values: [int; 3]) -> [i32; 3] {
    return [values[0] * 2, values[1] + 3, values[2] - 1];
}

fn identity(values: [i32; 3]) -> [int; 3] {
    return values;
}

fn return_call(values: [int; 3]) -> [i32; 3] {
    return identity(values);
}

fn score(values: [int; 3]) -> int {
    return values[0] + values[1] + values[2];
}

fn main() -> int {
    let source = [2, 3, 4];
    let computed = transform(source);
    let annotated: [i32; 3] = return_call(computed);
    let alias: [int; 3] = annotated;
    let copied = alias;
    let nested_score: int = score(identity(transform(source)));
    let literal_score: int = score([7, 8, 9]);
    let call_index: int = transform(source)[0];
    let literal_index: int = [10, 11, 12][1];
    if source[0] == 2 && copied[1] == 6 && annotated[2] == 3
        && nested_score == 13 && literal_score == 24
        && call_index == 4 && literal_index == 11 {
        return 91;
    }
    return 1;
}
"#;

const MUTABLE_ARRAY_RESULT_PRODUCTION: &str = r#"
fn seed() -> [int; 4] {
    return [3, 5, 7, 9];
}

fn from_literal() -> [i32; 2] {
    let mut output: [int; 2] = [10, 20];
    output[0] = output[0] + 1;
    return output;
}

fn from_identifier(source: [int; 4]) -> [i32; 4] {
    let mut output: [i32; 4] = source;
    let mut index: int = 0;
    let mut delta: int = 0;
    while index < 4 {
        output[index] = source[index] * 2 + delta;
        index = index + 1;
        delta = delta + 1;
    }
    return output;
}

fn from_call() -> [int; 4] {
    let mut output = seed();
    output[3] = output[3] + 1;
    return output;
}

fn score(values: [int; 4]) -> int {
    return values[0] + values[1] + values[2] + values[3];
}

fn main() -> int {
    let source: [i32; 4] = seed();
    let looped: [int; 4] = from_identifier(source);
    let literal = from_literal();
    let called = from_call();
    let consumed: int = score(looped);
    if source[0] == 3 && source[3] == 9
        && looped[0] == 6 && looped[1] == 11
        && looped[2] == 16 && looped[3] == 21
        && literal[0] == 11 && literal[1] == 20
        && called[3] == 10 && consumed == 54 {
        return 91;
    }
    return 1;
}
"#;

fn exact_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ArrayV0,
        ..CompilerOptions::default()
    }
}

fn reference_kernel() -> i32 {
    let source: [i32; 8] = [127, 1_073_741_824, -128, 64, -64, 7, -3, 11];
    let mut left = source;
    left[0] = left[0].wrapping_add(1);
    let right: [i32; 8] = [8, 2, -7, 6, -5, 4, -3, 2];
    let result = left
        .into_iter()
        .zip(right)
        .fold(2_147_483_000_i32, |accumulator, (left, right)| {
            accumulator.wrapping_add(left.wrapping_mul(right))
        });
    assert_eq!(source[0], 127, "Copy-array source must remain readable");
    assert_eq!(left[0], 128, "returned transform must change one lane");
    result
}

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
            "aero-fixed-int-array-profile-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh fixed-int-array profile workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-fixed-int-array-profile-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn write_program(workspace: &TestWorkspace) -> PathBuf {
    let source = workspace.path("main.aero");
    fs::write(&source, FIXED_INT_ARRAY_PROGRAM).expect("write fixed-int-array source");
    source
}

fn run_cli(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for argument in args {
        command.arg(argument);
    }
    command
        .output()
        .expect("run Aero fixed-int-array profile route")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn llvm_body_without_public_route_headers(llvm: &str) -> Vec<&str> {
    llvm.lines()
        .filter(|line| {
            !line.starts_with("target datalayout = ")
                && !line.starts_with("target triple = ")
                && !line.starts_with("; aero.graph_compilation")
        })
        .collect()
}

fn assert_dynamic_guard_sequences(llvm: &str, aggregate: &str, expected: usize) {
    for anchor in ["icmp sge i32", "call void @llvm.trap()", "sext i32"] {
        assert_eq!(
            occurrences(llvm, anchor),
            expected,
            "expected {expected} exact dynamic-array occurrences of `{anchor}`:\n{llvm}"
        );
    }

    let mut cursor = 0;
    for _ in 0..expected {
        let lower = llvm[cursor..]
            .find("icmp sge i32")
            .map(|offset| cursor + offset)
            .expect("signed lower guard");
        let upper = llvm[lower..]
            .find("icmp slt i32")
            .map(|offset| lower + offset)
            .expect("signed upper guard");
        let conjunction = llvm[upper..]
            .find("and i1")
            .map(|offset| upper + offset)
            .expect("combined bounds predicate");
        let branch = llvm[conjunction..]
            .find("br i1")
            .map(|offset| conjunction + offset)
            .expect("guard branch");
        let trap = llvm[branch..]
            .find("call void @llvm.trap()")
            .map(|offset| branch + offset)
            .expect("trap branch");
        let safe = llvm[trap..]
            .find("aero.bounds.safe.")
            .map(|offset| trap + offset)
            .expect("safe label");
        let extension = llvm[safe..]
            .find("sext i32")
            .map(|offset| safe + offset)
            .expect("post-guard sign extension");
        let address_anchor = format!("getelementptr inbounds {aggregate}");
        let address = llvm[extension..]
            .find(&address_anchor)
            .map(|offset| extension + offset)
            .expect("post-guard array address");
        assert!(
            lower < upper
                && upper < conjunction
                && conjunction < branch
                && branch < trap
                && trap < safe
                && safe < extension
                && extension < address
        );
        cursor = address + address_anchor.len();
    }
}

fn assert_guarded_dynamic_array_read_and_write(llvm: &str, aggregate: &str) {
    let dynamic_geps = llvm
        .lines()
        .filter_map(|line| {
            (line.contains(&format!("getelementptr inbounds {aggregate}"))
                && line.contains(", i64 %reg"))
            .then(|| line.split_once(" = ").map(|(pointer, _)| pointer.trim()))
            .flatten()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        dynamic_geps.len(),
        2,
        "expected one dynamic read GEP and one dynamic write GEP for {aggregate}:\n{llvm}"
    );

    let mut loads = 0;
    let mut stores = 0;
    for pointer in dynamic_geps {
        let definition = llvm
            .find(&format!("{pointer} = getelementptr inbounds {aggregate}"))
            .expect("dynamic array pointer definition");
        let load = format!("load i32, i32* {pointer}, align 4");
        let store_prefix = "store i32 ";
        let store_target = format!("i32* {pointer}, align 4");
        let load_positions = llvm
            .match_indices(&load)
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        let store_positions = llvm
            .lines()
            .filter(|line| line.contains(store_prefix) && line.contains(&store_target))
            .filter_map(|line| llvm.find(line))
            .collect::<Vec<_>>();
        assert_eq!(
            load_positions.len() + store_positions.len(),
            1,
            "guarded pointer {pointer} must feed exactly one scalar read or write:\n{llvm}"
        );
        assert!(
            load_positions
                .iter()
                .chain(store_positions.iter())
                .all(|position| *position > definition),
            "guarded pointer {pointer} was consumed before its GEP"
        );
        loads += load_positions.len();
        stores += store_positions.len();
    }
    assert_eq!((loads, stores), (1, 1));
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_check() {
    assert_eq!(reference_kernel(), 2035);
    let workspace = TestWorkspace::new("check-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("check"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success(),
        "exact-i32-array-v0 check is absent or rejected its complete kernel:\n{}",
        combined_output(&output)
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_build() {
    let workspace = TestWorkspace::new("build-red");
    let source = write_program(&workspace);
    let llvm = workspace.path("kernel.ll");
    let output = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &source,
            Path::new("-o"),
            &llvm,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success() && llvm.is_file(),
        "exact-i32-array-v0 build is absent or omitted LLVM:\n{}",
        combined_output(&output)
    );
    let public_llvm = fs::read_to_string(&llvm).expect("public build must write readable LLVM");
    for route_header in [
        "; aero.graph_compilation=enabled",
        "; aero.graph_compilation.execution_scope=internal-scalar-helper",
        "; aero.graph_compilation.device_execution=false",
        "; aero.graph_compilation.backend=cpu",
        "; aero.graph_compilation.executable_fusion=true",
    ] {
        assert!(
            public_llvm.lines().any(|line| line == route_header),
            "public build omitted route header `{route_header}`"
        );
    }
    let library_llvm = compile_file(&source, exact_options())
        .expect("file library route should emit the same exact LLVM");
    assert_eq!(
        llvm_body_without_public_route_headers(&public_llvm),
        llvm_body_without_public_route_headers(&library_llvm),
        "public and library routes must have byte-identical LLVM bodies after the public route applies its documented graph and host-target framing"
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_run() {
    let workspace = TestWorkspace::new("run-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(91),
        "exact-i32-array-v0 run is absent or diverged from the oracle:\n{}",
        combined_output(&output)
    );
}

#[test]
fn stable_scalar_profile_still_rejects_the_array_kernel() {
    let error = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::StableScalarV0,
            ..CompilerOptions::default()
        },
    )
    .expect_err("stable-scalar-v0 must retain its frozen array exclusion");
    assert!(error.contains("Language Profile Error: stable-scalar-v0 rejects"));

    let result_only = "fn make() -> [int; 1] { return [1]; } fn main() -> int { return 0; }";
    let error = compile_program(
        result_only,
        CompilerOptions {
            language_profile: LanguageProfile::StableScalarV0,
            ..CompilerOptions::default()
        },
    )
    .expect_err("stable-scalar-v0 must reject array results independently of parameters");
    assert_eq!(
        error,
        "Language Profile Error: stable-scalar-v0 rejects function result types"
    );
}

#[test]
fn experimental_profile_retains_the_legacy_double_array_lane() {
    let implicit = compile_program(FIXED_INT_ARRAY_PROGRAM, CompilerOptions::default())
        .expect("experimental array kernel control should compile");
    let explicit = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental array kernel control should compile");

    assert_eq!(implicit, explicit);
    assert!(implicit.contains("[8 x double]"));
    assert!(!implicit.contains("[8 x i32]"));
}

#[test]
fn exact_profile_is_shared_by_source_and_file_library_routes() {
    check_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("source check should admit the exact fixed-array kernel");
    let source_llvm = compile_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("source compile should emit the exact fixed-array kernel");

    let workspace = TestWorkspace::new("library-route-parity");
    let source = write_program(&workspace);
    check_file(&source, exact_options()).expect("file check should share exact admission");
    let file_llvm = compile_file(&source, exact_options())
        .expect("file compile should share exact physical lowering");
    assert_eq!(source_llvm, file_llvm);

    fs::write(&source, "mod missing; fn main() -> int { return 0; }")
        .expect("write unresolved module attempt");
    assert_eq!(
        check_file(&source, exact_options()).expect_err("profile must precede module resolution"),
        "Language Profile Error: exact-i32-array-v0 rejects module declarations"
    );
}

#[test]
fn general_checked_pipeline_already_owns_the_complete_immutable_array_value_class() {
    let tokens = try_tokenize_with_locations(IMMUTABLE_ARRAY_VALUE_COMPOSITION, None)
        .expect("immutable array composition control should lex");
    let ast =
        parse_with_locations(tokens).expect("immutable array composition control should parse");
    IrGenerator::new()
        .try_generate_ir(ast.clone())
        .expect("independent raw checked admission should already own immutable array composition");
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer
        .analyze(ast)
        .expect("general semantics should already own immutable array composition");
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .expect("checked IR should already own immutable array composition");
    let metadata = format!("{:#?}", checked.metadata());
    for anchor in [
        "\"transform\": FunctionMetadata",
        "\"identity\": FunctionMetadata",
        "element: Int",
        "count: 3",
    ] {
        assert!(
            metadata.contains(anchor),
            "checked metadata omitted array-composition identity `{anchor}`:\n{metadata}"
        );
    }

    let llvm = compile_program(
        IMMUTABLE_ARRAY_VALUE_COMPOSITION,
        CompilerOptions::default(),
    )
    .expect("experimental control should retain general immutable array composition");
    assert!(llvm.contains("define [3 x double] @transform("));
    assert!(llvm.contains("call [3 x double] @identity("));
}

#[test]
fn general_pipeline_already_owns_initialized_mutable_array_result_production() {
    let tokens = try_tokenize_with_locations(MUTABLE_ARRAY_RESULT_PRODUCTION, None)
        .expect("mutable array-result production control should lex");
    let ast =
        parse_with_locations(tokens).expect("mutable array-result production control should parse");

    let checked = IrGenerator::new()
        .try_generate_ir(ast.clone())
        .expect("raw-AST checked admission should already own initialized array mutation");
    let exact_array_four = LogicalType::Array {
        element: Box::new(LogicalType::Int),
        count: 4,
    };
    let from_identifier = &checked.metadata().functions["from_identifier"];
    assert_eq!(
        from_identifier.signature.parameters,
        vec![("source".to_string(), exact_array_four.clone())],
        "raw checked metadata must retain the exact Array<Int, 4> input identity"
    );
    assert_eq!(
        from_identifier.signature.result, exact_array_four,
        "raw checked metadata must retain the exact Array<Int, 4> result identity"
    );
    let output_place = from_identifier
        .places
        .values()
        .find(|place| place.name.as_deref() == Some("output"))
        .expect("raw checked metadata must retain the mutable output place");
    assert_eq!(
        output_place.pointee, from_identifier.signature.result,
        "mutable output place and returned array must share one logical identity"
    );
    assert!(
        from_identifier
            .results
            .values()
            .any(|result| result == &from_identifier.signature.result),
        "raw checked metadata must retain an Array<Int, 4> produced value"
    );
    let checked_debug = format!("{checked:#?}");
    for marker in [
        "CheckedMutableOwnedPlaceAlloca",
        "CheckedOwnedPlaceAssignment",
        "element: Int",
        "count: 4",
    ] {
        assert!(
            checked_debug.contains(marker),
            "raw checked IR omitted mutable array-result marker `{marker}`:\n{checked_debug}"
        );
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(ast)
        .expect("general semantics should already own initialized array mutation and return");

    let implicit = compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, CompilerOptions::default())
        .expect("experimental control should compile mutable array-result production");
    let explicit = compile_program(
        MUTABLE_ARRAY_RESULT_PRODUCTION,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental control should compile mutable array-result production");
    assert_eq!(implicit, explicit);
    for anchor in [
        "define [2 x double] @from_literal()",
        "define [4 x double] @from_identifier([4 x double]",
        "define [4 x double] @from_call()",
        "alloca [2 x double], align 8",
        "alloca [4 x double], align 8",
        "store [2 x double]",
        "load [2 x double]",
        "store [4 x double]",
        "load [4 x double]",
        "store double",
        "ret [4 x double]",
        "call i32 @score([4 x double]",
    ] {
        assert!(
            implicit.contains(anchor),
            "experimental control omitted `{anchor}`:\n{implicit}"
        );
    }
    for forbidden in ["[2 x i32]", "[4 x i32]"] {
        assert!(
            !implicit.contains(forbidden),
            "experimental mutable-owner storage leaked `{forbidden}`:\n{implicit}"
        );
    }
}

#[test]
fn exact_profile_admits_initialized_mutable_array_result_production() {
    let mut failures = Vec::new();

    if let Err(error) = check_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        if !error.starts_with("Language Profile Error: exact-i32-array-v0 rejects ") {
            failures.push(format!(
                "exact source check crossed the pre-semantic boundary: {error}"
            ));
        }
        failures.push(format!(
            "exact source check rejected mutable array-result production: {error}"
        ));
    }

    match compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        Err(error) => failures.push(format!(
            "exact source compilation rejected mutable array-result production: {error}"
        )),
        Ok(first) => {
            let second = compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options())
                .expect("a second exact mutable array-result compilation should remain admitted");
            assert_eq!(
                first, second,
                "exact mutable array-result LLVM must be deterministic"
            );
            for anchor in [
                "define [2 x i32] @from_literal()",
                "define [4 x i32] @from_identifier([4 x i32] %aero.arg.source)",
                "define [4 x i32] @from_call()",
                "store [4 x i32]",
                "store i32",
                "ret [4 x i32]",
                "call i32 @score([4 x i32]",
            ] {
                assert!(
                    first.contains(anchor),
                    "exact mutable array-result LLVM omitted `{anchor}`:\n{first}"
                );
            }
            for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
                assert!(
                    !first.contains(forbidden),
                    "exact mutable array-result LLVM leaked `{forbidden}`:\n{first}"
                );
            }
            assert_dynamic_guard_sequences(&first, "[4 x i32]", 2);
            assert_guarded_dynamic_array_read_and_write(&first, "[4 x i32]");
        }
    }

    let workspace = TestWorkspace::new("mutable-array-result-production-red");
    let source = workspace.path("main.aero");
    fs::write(&source, MUTABLE_ARRAY_RESULT_PRODUCTION)
        .expect("write mutable array-result production source");
    let llvm = workspace.path("mutable-array-result.ll");

    let check = run_cli(
        &workspace,
        &[
            Path::new("check"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if !check.status.success() {
        failures.push(format!(
            "public check rejected mutable array-result production: {}",
            combined_output(&check)
        ));
    }

    let build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &source,
            Path::new("-o"),
            &llvm,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if !build.status.success() {
        assert!(
            !llvm.exists(),
            "a rejected exact mutable array-result build must not leave an LLVM artifact"
        );
        failures.push(format!(
            "public build rejected mutable array-result production: {}",
            combined_output(&build)
        ));
    } else if !llvm.is_file() {
        failures.push("public build admitted the source but omitted its LLVM artifact".to_string());
    }

    let run = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if run.status.code() != Some(91) {
        failures.push(format!(
            "public run did not preserve the source Copy and consume the returned result (status={:?}): {}",
            run.status.code(),
            combined_output(&run)
        ));
    }

    assert!(
        failures.is_empty(),
        "CAP-019 mutable exact-array result production is not complete:\n{}",
        failures.join("\n")
    );
}

#[test]
fn mutable_array_result_production_retains_profile_separation_boundaries() {
    let exact_rejections = [
        (
            "immutable assignment root",
            "fn main() -> int { let values: [int; 2] = [1, 2]; values[0] = 3; return values[0]; }",
            "projected assignment targets rooted in immutable exact-array bindings",
        ),
        (
            "wrong-count initializer",
            "fn seed() -> [int; 2] { return [1, 2]; } fn bad() -> [i32; 3] { let mut values: [int; 3] = seed(); return values; } fn main() -> int { return 0; }",
            "array literal counts that differ from their annotations",
        ),
        (
            "non-Int initializer",
            "fn bad() -> [int; 2] { let mut values: [i32; 2] = [1 < 2, 2 < 3]; return values; } fn main() -> int { return 0; }",
            "array literal elements other than exact Int expressions",
        ),
        (
            "nested initializer",
            "fn main() -> int { let mut values: [[int; 1]; 1] = [[1]]; return 0; }",
            "binding annotation types",
        ),
        (
            "mutable alias initializer",
            "fn main() -> int { let mut source: [int; 2] = [1, 2]; let mut alias: [i32; 2] = source; return alias[0]; }",
            "mutable exact-array values as initializer sources",
        ),
        (
            "inferred mutable alias initializer",
            "fn main() -> int { let mut source: [int; 2] = [1, 2]; let mut alias = source; return alias[0]; }",
            "mutable exact-array values as initializer sources",
        ),
        (
            "uninitialized mutable array",
            "fn main() -> int { let mut values: [int; 2]; return 0; }",
            "uninitialized bindings",
        ),
        (
            "array-valued element store",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; values[0] = [3, 4]; return values[0]; }",
            "array values in exact integer expressions",
        ),
        (
            "non-Int runtime selector",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let selector: bool = 1 < 2; values[selector] = 3; return values[0]; }",
            "non-Int values in exact integer expressions",
        ),
        (
            "non-Int stored value",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let flag: bool = 1 < 2; values[0] = flag; return values[0]; }",
            "non-Int values in exact integer expressions",
        ),
        (
            "array reference escape",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let view = &mut values; values[0] = 3; return values[0]; }",
            "reference expressions",
        ),
        (
            "whole-array reassignment",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; values = [3, 4]; return values[0]; }",
            "array writes",
        ),
        (
            "recursive array transform",
            "fn bad(source: [int; 1]) -> [i32; 1] { let mut output: [int; 1] = source; output[0] = output[0] + 1; return bad(output); } fn main() -> int { return 0; }",
            "recursive function call cycles",
        ),
    ];

    match check_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        Err(error) => assert_eq!(
            error, "Language Profile Error: exact-i32-array-v0 rejects mutable array bindings",
            "only the known CAP-019 red barrier may defer the prospective exclusion matrix"
        ),
        Ok(()) => {
            for (label, source, expected) in exact_rejections {
                let error = check_program(source, exact_options())
                    .expect_err("excluded mutable exact-array topology must fail closed");
                assert!(
                    error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
                    "{label} crossed the profile boundary: {error}"
                );
                assert!(
                    error.contains(expected),
                    "{label}: expected `{expected}`, got: {error}"
                );
                assert!(
                    !error.contains("Semantic Analysis Error")
                        && !error.contains("IR Generation Error"),
                    "{label} reached a later compiler phase: {error}"
                );
            }
        }
    }

    let result_only = "fn make() -> [int; 1] { return [1]; } fn main() -> int { return 0; }";
    assert_eq!(
        check_program(
            result_only,
            CompilerOptions {
                language_profile: LanguageProfile::StableScalarV0,
                ..CompilerOptions::default()
            }
        ),
        Err("Language Profile Error: stable-scalar-v0 rejects function result types".to_string())
    );

    let mutation_only =
        "fn main() -> int { let mut values: [int; 2] = [1, 2]; values[0] = 3; return values[0]; }";
    assert_eq!(
        check_program(
            mutation_only,
            CompilerOptions {
                language_profile: LanguageProfile::StableScalarV0,
                ..CompilerOptions::default()
            }
        ),
        Err(
            "Language Profile Error: stable-scalar-v0 rejects binding annotation types".to_string()
        )
    );
}

#[test]
fn exact_profile_admits_the_complete_immutable_array_value_composition_class() {
    check_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should admit every frozen immutable array value placement");
    let first = compile_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should lower immutable array value composition");
    let second = compile_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should lower deterministically");
    assert_eq!(
        first, second,
        "exact immutable-array LLVM must be deterministic"
    );

    for anchor in [
        "define [3 x i32] @transform([3 x i32] %aero.arg.values)",
        "define [3 x i32] @identity([3 x i32] %aero.arg.values)",
        "define [3 x i32] @return_call([3 x i32] %aero.arg.values)",
        "call [3 x i32] @transform([3 x i32]",
        "call [3 x i32] @identity([3 x i32]",
        "call [3 x i32] @return_call([3 x i32]",
        "store [3 x i32]",
        "load [3 x i32]",
        "ret [3 x i32]",
        "getelementptr inbounds [3 x i32]",
    ] {
        assert!(
            first.contains(anchor),
            "missing exact composition anchor `{anchor}`:\n{first}"
        );
    }
    for forbidden in ["[3 x double]", "fptosi", "sitofp", " nsw ", " nuw "] {
        assert!(
            !first.contains(forbidden),
            "exact array value composition leaked `{forbidden}`:\n{first}"
        );
    }
}

#[test]
fn exact_array_value_composition_retains_topology_and_mutability_separation() {
    let cases = [
        (
            "zero result",
            "fn bad() -> [int; 0] { return []; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "nested result",
            "fn bad() -> [[int; 1]; 1] { return [[1]]; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "non-int result",
            "fn bad() -> [bool; 1] { return [1 < 2]; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "repeat source",
            "fn bad() -> [int; 2] { return [1; 2]; } fn main() -> int { return 0; }",
            "array bindings without direct literal initializers",
        ),
        (
            "wrong result count",
            "fn bad() -> [int; 3] { return [1, 2]; } fn main() -> int { return 0; }",
            "array value source count mismatch",
        ),
        (
            "non-int computed element",
            "fn bad() -> [int; 1] { return [1 < 2]; } fn main() -> int { return 0; }",
            "array literal elements other than exact Int expressions",
        ),
        (
            "aggregate process result",
            "fn main() -> [int; 1] { return [1]; }",
            "entrypoints other than exact `fn main() -> int`",
        ),
    ];

    for (label, source, expected) in cases {
        let error = check_program(source, exact_options())
            .expect_err("excluded exact-array value topology must fail closed");
        assert!(
            error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
            "{label} escaped profile admission: {error}"
        );
        assert!(
            error.contains(expected),
            "{label}: wrong diagnostic: {error}"
        );
        assert!(
            !error.contains("Semantic Analysis Error") && !error.contains("IR Generation Error")
        );
    }
}

#[test]
fn exact_profile_emits_one_guarded_i32_array_lane() {
    let llvm = compile_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("exact fixed-array kernel should compile");

    for anchor in [
        "define i32 @dot_with_bias([8 x i32] %aero.arg.left, [8 x i32] %aero.arg.right, i32 %aero.arg.bias)",
        "define [8 x i32] @offset_first_lane([8 x i32] %aero.arg.values)",
        "define [8 x i32] @forward_array([8 x i32] %aero.arg.values)",
        "alloca [8 x i32], align 8",
        "store [8 x i32]",
        "load [8 x i32]",
        "call i32 @dot_with_bias([8 x i32]",
        "call [8 x i32] @offset_first_lane([8 x i32]",
        "call [8 x i32] @forward_array([8 x i32]",
        "ret [8 x i32]",
        "load i32",
        "store i32",
        "mul i32",
        "add i32",
        "icmp sge i32",
        "icmp slt i32",
        "sext i32",
        "getelementptr inbounds [8 x i32]",
        "declare void @llvm.trap()",
        "ret i32",
    ] {
        assert!(
            llvm.contains(anchor),
            "missing exact-array anchor `{anchor}`:\n{llvm}"
        );
    }
    for forbidden in [
        "[8 x double]",
        "double",
        "fptosi",
        "sitofp",
        " nsw ",
        " nuw ",
        "[8 x i8]",
        "<8 x i32>",
    ] {
        assert!(
            !llvm.contains(forbidden),
            "exact fixed-array LLVM leaked forbidden representation `{forbidden}`:\n{llvm}"
        );
    }

    assert_dynamic_guard_sequences(&llvm, "[8 x i32]", 2);
}

#[test]
fn exact_array_kernel_and_wrapping_edges_match_independent_i32_oracles() {
    assert_eq!(reference_kernel(), 2035);

    let values = [i32::MAX, 1, 1_073_741_824, 2];
    let wrapped_add = values[0].wrapping_add(values[1]);
    let wrapped_sub = wrapped_add.wrapping_sub(values[1]);
    let wrapped_mul = values[2].wrapping_mul(values[3]);
    let wrapped_neg = wrapped_add.wrapping_neg();
    assert_eq!(
        if wrapped_add < 0 && wrapped_sub > 0 && wrapped_mul < 0 && wrapped_neg < 0 {
            93
        } else {
            1
        },
        93
    );

    for source in [FIXED_INT_ARRAY_PROGRAM, FIXED_INT_ARRAY_WRAPPING_EDGES] {
        let llvm = compile_program(source, exact_options())
            .expect("exact fixed-array arithmetic specimen should compile");
        for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
            assert!(
                !llvm.contains(forbidden),
                "wrapping proof leaked `{forbidden}`"
            );
        }
    }

    let wrapping_llvm = compile_program(FIXED_INT_ARRAY_WRAPPING_EDGES, exact_options())
        .expect("constant-index wrapping specimen should compile");
    assert!(
        wrapping_llvm.contains("icmp slt i32"),
        "wrapping control flow must retain its ordinary signed comparison"
    );
    assert_dynamic_guard_sequences(&wrapping_llvm, "[4 x i32]", 0);
}

#[test]
fn exact_profile_rejects_every_neighboring_array_family_before_checked_ir() {
    let cases = [
        (
            "repeat array",
            "fn main() -> int { let values: [int; 2] = [1; 2]; return 0; }",
            "array bindings without direct literal initializers",
        ),
        (
            "empty array",
            "fn main() -> int { let values: [int; 0] = []; return 0; }",
            "binding annotation types",
        ),
        (
            "array count above the signed i32 profile boundary",
            "fn take(values: [int; 2147483648]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "nested array",
            "fn main() -> int { let values: [[int; 1]; 1] = [[1]]; return 0; }",
            "binding annotation types",
        ),
        (
            "non-int element",
            "fn main() -> int { let values: [bool; 1] = [true]; return 0; }",
            "binding annotation types",
        ),
        (
            "char element",
            "fn main() -> int { let values: [char; 1] = ['a']; return 0; }",
            "binding annotation types",
        ),
        (
            "user-defined element",
            "fn take(values: [Widget; 1]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "array write",
            "fn main() -> int { let values: [int; 1] = [1]; values[0] = 2; return 0; }",
            "projected assignment targets rooted in immutable exact-array bindings",
        ),
        (
            "array comparison",
            "fn main() -> int { let values: [int; 1] = [1]; if values == values { return 1; } return 0; }",
            "array identifiers outside direct call transport or index reads",
        ),
        (
            "wrong literal count",
            "fn main() -> int { let values: [int; 1] = [1, 2]; return 0; }",
            "array literal counts that differ from their annotations",
        ),
        (
            "wrong transport count",
            "fn take(values: [int; 2]) -> int { return values[0]; } fn main() -> int { let values: [int; 1] = [1]; return take(values); }",
            "array call arguments with mismatched counts",
        ),
        (
            "out-of-range lane literal",
            "fn main() -> int { let values: [int; 1] = [2147483648]; return 0; }",
            "array elements other than exact signed i32 literals",
        ),
    ];

    for (name, source, expected) in cases {
        let error = match check_program(source, exact_options()) {
            Ok(()) => panic!("{name} unexpectedly entered exact checked IR"),
            Err(error) => error,
        };
        assert!(
            error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
            "{name} escaped pre-semantic profile admission: {error}"
        );
        assert!(
            error.contains(expected),
            "{name}: wrong diagnostic: {error}"
        );
        assert!(
            !error.contains("Semantic Analysis Error") && !error.contains("IR Generation Error")
        );
    }
}

#[test]
fn exact_profile_preserves_compile_time_bounds_rejection_and_runtime_guards() {
    for (label, index) in [("negative", "-1"), ("equal", "2"), ("above", "3")] {
        let source = format!(
            "fn main() -> int {{ let values: [int; 2] = [10, 20]; return values[{index}]; }}"
        );
        let error = compile_program(&source, exact_options())
            .expect_err("constant out-of-bounds exact array index must reject");
        assert!(error.contains("outside 0..2"), "{label}: {error}");
    }

    for source in [NEGATIVE_RUNTIME_INDEX, EQUAL_TO_COUNT_RUNTIME_INDEX] {
        let llvm = compile_program(source, exact_options())
            .expect("runtime bounds-failure specimen should lower to a guard");
        assert_dynamic_guard_sequences(&llvm, "[2 x i32]", 1);
    }
}

#[test]
fn exact_i32_array_system_gate_is_anchored_on_linux_and_windows() {
    let workflow = include_str!("../../../.github/workflows/rust.yml");
    for step in [
        "Test exact i32 fixed-array CPU profile at O0 and O2",
        "Test exact i32 fixed-array CPU profile on Windows at O0 and O2",
    ] {
        assert_eq!(workflow.matches(step).count(), 1, "missing unique `{step}`");
    }
    for anchor in [
        "examples/fixed_int_array_v0/",
        "wrapping_edges.aero",
        "runtime_fail/",
        "negative_index.aero",
        "equal_to_count_index.aero",
        "--language-profile exact-i32-array-v0",
        "--require-llvm-verifier",
        "opt-22 -passes=verify",
        "llc-22 -verify-machineinstrs",
        "clang-22 -O0",
        "clang-22 -O2",
        "& \"$llvmBin\\opt.exe\" -passes=verify",
        "& \"$llvmBin\\llc.exe\" -verify-machineinstrs",
        "& \"$llvmBin\\clang.exe\" -O0",
        "& \"$llvmBin\\clang.exe\" -O2",
    ] {
        assert!(workflow.contains(anchor), "workflow omitted `{anchor}`");
    }

    assert!(
        !workflow
            .contains("upper_line=\"$(grep -n -m1 -F 'icmp slt i32' \"${llvm}\" | cut -d: -f1)\""),
        "Linux must search for an array upper guard only after its lower guard"
    );
    assert_eq!(
        workflow
            .matches("icmp sge i32|icmp slt i32|sext i32|llvm\\.trap")
            .count(),
        0,
        "ordinary signed comparisons are not sufficient evidence of dynamic bounds IR"
    );
    for identity_linked_anchor in [
        "guard_block_pattern='(?m)^  (?<lower>%reg[0-9]+) = icmp sge i32",
        "grep -Pzo -- \"${guard_block_pattern}\" \"${llvm}\"",
        "test \"${guard_count}\" -eq 2",
        "$guardPattern = '(?m)^  (?<lower>%reg[0-9]+) = icmp sge i32",
        "$guardMatches = [regex]::Matches($llvmText, $guardPattern)",
        "$guardMatches.Count -ne 2",
    ] {
        assert_eq!(
            workflow.matches(identity_linked_anchor).count(),
            1,
            "workflow must retain one identity-linked proof anchor `{identity_linked_anchor}`"
        );
    }
    for shared_identity_link in [
        r"\k<index>, [0-9]+",
        r"\k<lower>, \k<upper>",
        r"br i1 \k<inbounds>",
        r"%aero\.bounds\.trap\.\k<place>",
        r"sext i32 \k<index> to i64",
        r"\k<aggregate>\* %ptr",
        r"i64 \k<extended>\r?$",
    ] {
        assert_eq!(
            workflow.matches(shared_identity_link).count(),
            2,
            "Linux and Windows must both retain identity link `{shared_identity_link}`"
        );
    }
    for composition_anchor in [
        "define [8 x i32] @offset_first_lane([8 x i32] %aero.arg.values)",
        "define [8 x i32] @forward_array([8 x i32] %aero.arg.values)",
        "call [8 x i32] @offset_first_lane([8 x i32]",
        "call [8 x i32] @forward_array([8 x i32]",
        "ret [8 x i32]",
    ] {
        assert_eq!(
            workflow.matches(composition_anchor).count(),
            2,
            "Linux and Windows must both retain array-value composition anchor `{composition_anchor}`"
        );
    }
}
