use compiler::{
    CompilerOptions, LanguageProfile, check_file, check_program, compile_file, compile_program,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const PROFILE_NAME: &str = "exact-i32-record-result-v0";

const EXPERIMENTAL_RECURSIVE_SOURCE: &str = r#"
struct Leaf {
    value: int,
    ready: bool,
}

struct Frame {
    leaf: Leaf,
    values: [int; 2],
    meta: (int, bool),
}

enum Scalar {
    Empty,
    Number(int),
    Flag(bool),
}

fn make_frame(valid: bool) -> Result<Frame, int> {
    let frame: Frame = Frame {
        leaf: Leaf { value: 1, ready: 1 < 2 },
        values: [2, 3],
        meta: (4, 2 < 3),
    };
    if valid {
        return Ok(frame);
    }
    return Err(7);
}

fn flag_score(flag: bool) -> int {
    if flag {
        return 1;
    }
    return 0;
}

fn scalar_score(value: Scalar) -> int {
    return match value {
        Scalar::Empty => 0,
        Scalar::Number(number) => number,
        Scalar::Flag(flag) => flag_score(flag),
    };
}

fn result_score(value: Result<Frame, int>) -> int {
    return match value {
        Ok(frame) => frame.leaf.value + frame.values[1] + frame.meta.0,
        Err(code) => code,
    };
}

fn main() -> int {
    let success: Result<Frame, int> = make_frame(1 < 2);
    let failure: Result<Frame, int> = make_frame(2 < 1);
    if result_score(success) == 8
        && result_score(failure) == 7
        && scalar_score(Scalar::Number(5)) == 5
        && scalar_score(Scalar::Flag(2 < 1)) == 0 {
        return 91;
    }
    return 1;
}
"#;

const STABLE_SCALAR_SOURCE: &str = r#"
fn mix(left: int, right: int) -> int {
    return left * 3 + right;
}

fn main() -> int {
    if mix(7, -4) == 17 {
        return 91;
    }
    return 1;
}
"#;

const EXACT_CAP023_SOURCE: &str =
    include_str!("../../../examples/fixed_int_array_v0/relu_argmax_inference.aero");

const SEMANTIC_ERROR_SOURCE: &str = r#"
fn main() -> int {
    return missing;
}
"#;

const SEMANTIC_ERROR: &str =
    "Semantic Analysis Error: Error: Use of undeclared variable `missing`.";
const MAIN_MARKER: &str = "// CAP-032 GENERATED MAIN MARKER";

fn options(language_profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile,
        ..CompilerOptions::default()
    }
}

fn llvm(source: &str, language_profile: LanguageProfile) -> String {
    compile_program(source, options(language_profile))
        .unwrap_or_else(|error| panic!("{language_profile:?} characterization failed: {error}"))
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn visible_output(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_silent_exit_91(output: &std::process::Output, context: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(91),
        "{context} must return 91 (stdout={stdout:?}, stderr={stderr:?})"
    );
    assert_eq!(
        stdout.matches("Exit code: 91").count(),
        1,
        "{context} must report the native sentinel exactly once"
    );
    assert!(
        !stdout
            .lines()
            .any(|line| { line.starts_with("Output:") || line.starts_with("Error output:") })
            && stderr.is_empty(),
        "{context} application must remain silent (stdout={stdout:?}, stderr={stderr:?})"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReferenceSuccess {
    output: [i32; 8],
    count: i32,
    checksum: i32,
    valid: bool,
}

fn reference_transform(values: [i32; 8], length: i32, bias: i32) -> Result<ReferenceSuccess, i32> {
    if length < 0 {
        return Err(101);
    }
    if length > 8 {
        return Err(102);
    }
    let mut output = [0; 8];
    let mut checksum = bias;
    for index in 0..length as usize {
        output[index] = values[index] * 3 + bias + index as i32;
        checksum += output[index];
    }
    Ok(ReferenceSuccess {
        output,
        count: length,
        checksum,
        valid: true,
    })
}

fn aero_array(values: [i32; 8]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn generated_vector_source(product: &str, values: [i32; 8], length: i32, bias: i32) -> String {
    let marker = product
        .find(MAIN_MARKER)
        .expect("tracked CAP-032 main marker is absent")
        + MAIN_MARKER.len();
    let expected = reference_transform(values, length, bias);
    let (success_body, error_body) = match expected {
        Ok(success) => {
            let output_checks = success
                .output
                .iter()
                .enumerate()
                .map(|(index, value)| format!("success.output[{index}] == {value}"))
                .collect::<Vec<_>>()
                .join("\n        && ");
            (
                format!(
                    "if success.valid\n        && success.count == {}\n        && success.checksum == {}\n        && {} {{\n        return 91;\n    }}\n    return 1;",
                    success.count, success.checksum, output_checks
                ),
                "return 1;".to_string(),
            )
        }
        Err(expected_error) => (
            "return 1;".to_string(),
            format!("if code == {expected_error} {{\n        return 91;\n    }}\n    return 1;"),
        ),
    };
    format!(
        "{}\n\nfn generated_success_score(success: Success) -> int {{\n    {success_body}\n}}\n\nfn generated_error_score(code: int) -> int {{\n    {error_body}\n}}\n\nfn generated_score(result: Result<Success, int>) -> int {{\n    return match result {{\n        Ok(success) => generated_success_score(success),\n        Err(code) => generated_error_score(code),\n    }};\n}}\n\nfn main() -> int {{\n    let request: Request = Request {{\n        input: Buffer {{\n            values: {},\n            length: {length},\n        }},\n        bias: {bias},\n    }};\n    return generated_score(transform(request));\n}}\n",
        &product[..marker],
        aero_array(values),
    )
}

struct TempWorkspace(PathBuf);

impl TempWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aero-cap032-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).expect("create CAP-032 characterization workspace");
        Self(root)
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write CAP-032 characterization source");
        path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn accepted_profiles_are_frozen_before_cap032() {
    let experimental = llvm(EXPERIMENTAL_RECURSIVE_SOURCE, LanguageProfile::Experimental);
    let stable = llvm(STABLE_SCALAR_SOURCE, LanguageProfile::StableScalarV0);
    let exact = llvm(EXACT_CAP023_SOURCE, LanguageProfile::ExactI32ArrayV0);

    // Re-frozen by CORE-093, which moves every static `alloca` into the entry
    // block so a loop body cannot grow the stack once per iteration. For each of
    // these three programs the change was verified to move nothing else: the
    // line multiset, the alloca line multiset, and the relative order of every
    // non-alloca line are all identical to the previous digests' modules.
    assert_eq!(
        [
            md5_hex(experimental.as_bytes()),
            md5_hex(stable.as_bytes()),
            md5_hex(exact.as_bytes()),
        ],
        [
            "d14b7acdaf81c4cab55cd100d4430201",
            "9aa9981631c60de5058c928bc8ac060f",
            "f74b8b67d5c10c9ef18cd67a07c90e23",
        ],
        "accepted profile LLVM bytes drifted before CAP-032"
    );

    let workspace = TempWorkspace::new();
    for (name, source, profile, expected) in [
        (
            "experimental.aero",
            EXPERIMENTAL_RECURSIVE_SOURCE,
            LanguageProfile::Experimental,
            &experimental,
        ),
        (
            "stable.aero",
            STABLE_SCALAR_SOURCE,
            LanguageProfile::StableScalarV0,
            &stable,
        ),
        (
            "exact.aero",
            EXACT_CAP023_SOURCE,
            LanguageProfile::ExactI32ArrayV0,
            &exact,
        ),
    ] {
        let path = workspace.source(name, source);
        assert_eq!(
            compile_file(&path, options(profile))
                .unwrap_or_else(|error| panic!("{profile:?} file route failed: {error}")),
            *expected,
            "{profile:?} source/file LLVM bytes drifted"
        );

        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .arg("run")
            .arg(&path)
            .arg("--language-profile")
            .arg(profile.as_str())
            .current_dir(&workspace.0)
            .output()
            .unwrap_or_else(|error| panic!("failed to run {profile:?} sentinel: {error}"));
        assert_eq!(
            output.status.code(),
            Some(91),
            "{profile:?} native sentinel drifted (stdout={:?}, stderr={:?})",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    let semantic_error_path = workspace.source("semantic_error.aero", SEMANTIC_ERROR_SOURCE);
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
    ] {
        let diagnostics = [
            compile_program(SEMANTIC_ERROR_SOURCE, options(profile))
                .expect_err("semantic-error source compile must fail"),
            compile_file(&semantic_error_path, options(profile))
                .expect_err("semantic-error file compile must fail"),
            check_program(SEMANTIC_ERROR_SOURCE, options(profile))
                .expect_err("semantic-error source check must fail"),
            check_file(&semantic_error_path, options(profile))
                .expect_err("semantic-error file check must fail"),
        ];
        assert_eq!(
            diagnostics,
            [SEMANTIC_ERROR; 4].map(str::to_string),
            "{profile:?} semantic diagnostic route drifted"
        );
    }

    let cli = read(&repository_root().join("src/compiler/src/main.rs"));
    let preparation = cli
        .find("prepare_checked_program_with_module_observer_and_profile")
        .expect("canonical checked preparation call is absent");
    let cache_lookup = cli[preparation..]
        .find("get_cached_llvm")
        .map(|index| preparation + index)
        .expect("verified compilation cache lookup is absent");
    assert!(
        preparation < cache_lookup,
        "a cache hit may not bypass checked preparation"
    );
    assert!(
        cli.contains("\"language-profile\"")
            && cli.contains("build_config.language_profile.as_str()"),
        "selected language profile identity must remain framed into cache keys"
    );
}

#[test]
fn exact_record_result_profile_is_selector_red_first() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .unwrap_or_else(|_| panic!("CAP-032 intentional selector red: {PROFILE_NAME} is absent"));
    assert_eq!(profile.as_str(), PROFILE_NAME);

    let root = repository_root();
    let application_path =
        root.join("examples/fixed_int_array_v0/exact_record_result_application.aero");
    let source = read(&application_path);
    let source_llvm = compile_program(&source, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 source route rejected the product: {error}"));
    let file_llvm = compile_file(&application_path, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 file route rejected the product: {error}"));
    assert_eq!(source_llvm, file_llvm, "CAP-032 source/file LLVM drifted");
    assert_eq!(
        source_llvm,
        compile_program(&source, options(profile))
            .expect("repeat CAP-032 source compilation must succeed"),
        "CAP-032 LLVM is nondeterministic"
    );
    check_program(&source, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 source check rejected the product: {error}"));
    check_file(&application_path, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 file check rejected the product: {error}"));

    let diagnostic_workspace = TempWorkspace::new();
    let semantic_error_path =
        diagnostic_workspace.source("semantic-error.aero", SEMANTIC_ERROR_SOURCE);
    assert_eq!(
        [
            check_program(SEMANTIC_ERROR_SOURCE, options(profile))
                .expect_err("CAP-032 semantic-error source check must fail"),
            compile_program(SEMANTIC_ERROR_SOURCE, options(profile))
                .expect_err("CAP-032 semantic-error source compile must fail"),
            check_file(&semantic_error_path, options(profile))
                .expect_err("CAP-032 semantic-error file check must fail"),
            compile_file(&semantic_error_path, options(profile))
                .expect_err("CAP-032 semantic-error file compile must fail"),
        ],
        [SEMANTIC_ERROR; 4].map(str::to_string),
        "CAP-032 changed semantic diagnostic precedence"
    );

    for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
        assert!(
            !source_llvm.contains(forbidden),
            "CAP-032 exact product emitted forbidden LLVM token `{forbidden}`"
        );
    }
    for required in [
        "%aero.struct.Buffer = type { [8 x i32], i32 }",
        "%aero.struct.Request = type { %aero.struct.Buffer, i32 }",
        "%aero.struct.Success = type { [8 x i32], i32, i32, i1 }",
        "define { i32, %aero.struct.Success, i32 } @transform(%aero.struct.Request %aero.arg.request)",
        "define i32 @score({ i32, %aero.struct.Success, i32 } %aero.arg.result)",
        "icmp sge i32",
        "icmp slt i32",
        "sext i32",
    ] {
        assert!(
            source_llvm.contains(required),
            "CAP-032 product LLVM omitted `{required}`"
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("run")
        .arg(&application_path)
        .arg("--language-profile")
        .arg(PROFILE_NAME)
        .current_dir(&diagnostic_workspace.0)
        .output()
        .expect("run CAP-032 product");
    assert_silent_exit_91(&output, "CAP-032 tracked product");
}

#[test]
fn exact_record_result_profile_executes_independent_valid_and_error_vectors() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("CAP-032 selector must be available after the red checkpoint");
    let root = repository_root();
    let product =
        read(&root.join("examples/fixed_int_array_v0/exact_record_result_application.aero"));
    let workspace = TempWorkspace::new();

    for (index, (values, length, bias)) in [
        ([1, 0, -2, 5, 0, 0, 0, 0], 4, 2),
        ([9, 8, 7, 6, 5, 4, 3, 2], 0, 7),
        ([3, -1, 4, -2, 5, -3, 6, -4], 8, -6),
        ([0; 8], -1, 0),
        ([0; 8], 9, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let source = generated_vector_source(&product, values, length, bias);
        let path = workspace.source(&format!("vector-{index}.aero"), &source);
        let source_llvm = compile_program(&source, options(profile))
            .unwrap_or_else(|error| panic!("generated vector {index} source failed: {error}"));
        let file_llvm = compile_file(&path, options(profile))
            .unwrap_or_else(|error| panic!("generated vector {index} file failed: {error}"));
        assert_eq!(
            source_llvm, file_llvm,
            "generated vector {index} LLVM drifted"
        );
        for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
            assert!(
                !source_llvm.contains(forbidden),
                "generated vector {index} emitted forbidden LLVM token `{forbidden}`"
            );
        }

        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .arg("run")
            .arg(&path)
            .arg("--language-profile")
            .arg(PROFILE_NAME)
            .current_dir(&workspace.0)
            .output()
            .unwrap_or_else(|error| panic!("run generated vector {index}: {error}"));
        assert_silent_exit_91(&output, &format!("CAP-032 generated vector {index}"));
    }
}

#[test]
fn exact_record_result_profile_rejects_closed_surface_context_shape_and_origin_boundaries() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("CAP-032 selector must be available after the red checkpoint");
    let workspace = TempWorkspace::new();
    let cases = [
        (
            "float-literal",
            "fn main() -> int { let hidden: float = 1.0; return 0; }",
            "surface expression `FloatLiteral`",
        ),
        (
            "output",
            "fn main() -> int { println!(\"hidden\"); return 0; }",
            "surface expression `Println`",
        ),
        (
            "unannotated-binding",
            "fn main() -> int { let hidden = 1; return hidden; }",
            "surface statement `Let { mutable: false, annotated: false, initialized: true }`",
        ),
        (
            "uninitialized-binding",
            "fn main() -> int { let hidden: int; return 0; }",
            "surface statement `Let { mutable: false, annotated: true, initialized: false }`",
        ),
        (
            "division",
            "fn main() -> int { let value: int = 8 / 2; return value; }",
            "surface expression `Binary(Divide)`",
        ),
        (
            "file-scope-execution",
            "1; fn main() -> int { return 0; }",
            "file-scope statement `Expression`",
        ),
        (
            "bool-array-shape",
            "fn consume(value: [bool; 2]) -> int { return 0; } fn main() -> int { return 0; }",
            "unavailable typed use",
        ),
        (
            "tuple-record-field",
            "struct Pair { values: (int, int), } fn main() -> int { return 0; }",
            "unavailable record declaration",
        ),
        (
            "user-enum-origin",
            "enum Choice { Value(int) } fn main() -> int { return 0; }",
            "nominal origin `Choice`",
        ),
        (
            "source-generic-context",
            "fn choose<T>(first: T, second: T, take_first: bool) -> T { if take_first { return first; } return second; } fn main() -> int { return choose(1, 2, 1 < 2); }",
            "non-source function context `choose<int>`",
        ),
        (
            "record-field-assignment",
            "struct Row { value: int, } fn main() -> int { let mut row: Row = Row { value: 1 }; row.value = 2; return row.value; }",
            "surface statement `Assignment { target: ResolvedProfileAssignmentTarget { root: Identifier, projections: [Field] } }`",
        ),
        (
            "whole-record-assignment",
            "struct Row { value: int, } fn main() -> int { let mut row: Row = Row { value: 1 }; row = Row { value: 2 }; return row.value; }",
            "OwnedAssignment logical type",
        ),
        (
            "wildcard-result-match",
            "struct Row { value: int, } fn inspect(value: Result<Row, int>) -> int { return match value { _ => 0 }; } fn main() -> int { return 0; }",
            "Match pattern `Wildcard`",
        ),
    ];

    for (name, source, expected_feature) in cases {
        let path = workspace.source(&format!("rejected-{name}.aero"), source);
        let diagnostics = [
            check_program(source, options(profile)).expect_err("source check must reject"),
            compile_program(source, options(profile)).expect_err("source compile must reject"),
            check_file(&path, options(profile)).expect_err("file check must reject"),
            compile_file(&path, options(profile)).expect_err("file compile must reject"),
        ];
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic == &diagnostics[0]),
            "{name} diagnostic routes drifted: {diagnostics:?}"
        );
        assert!(
            diagnostics[0]
                .starts_with("Language Profile Error: exact-i32-record-result-v0 rejects ")
                && diagnostics[0].contains(expected_feature),
            "{name} reached the wrong boundary: {}",
            diagnostics[0]
        );
    }
}

#[test]
fn exact_record_result_profile_is_cpu_only_and_emits_no_failure_artifact() {
    let root = repository_root();
    let application_path =
        root.join("examples/fixed_int_array_v0/exact_record_result_application.aero");
    let workspace = TempWorkspace::new();
    let output_path = workspace.0.join("forbidden-target.ll");
    let rejected_build = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(&application_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--language-profile")
        .arg(PROFILE_NAME)
        .arg("--target")
        .arg("rocm")
        .current_dir(&workspace.0)
        .output()
        .expect("run rejected CAP-032 ROCm build");
    assert_eq!(rejected_build.status.code(), Some(2));
    assert!(
        visible_output(&rejected_build).contains(
            "Language Profile Error: exact-i32-record-result-v0 requires --target cpu without --gpu"
        ),
        "CAP-032 ROCm rejection drifted: {}",
        visible_output(&rejected_build)
    );
    assert!(
        !output_path.exists(),
        "rejected CAP-032 target published an LLVM artifact"
    );

    let rejected_gpu = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("run")
        .arg(&application_path)
        .arg("--language-profile")
        .arg(PROFILE_NAME)
        .arg("--gpu")
        .arg("gfx1100")
        .current_dir(&workspace.0)
        .output()
        .expect("run rejected CAP-032 GPU selection");
    assert_eq!(rejected_gpu.status.code(), Some(2));
    assert!(
        visible_output(&rejected_gpu).contains(
            "Language Profile Error: exact-i32-record-result-v0 requires --target cpu without --gpu"
        ),
        "CAP-032 GPU rejection drifted: {}",
        visible_output(&rejected_gpu)
    );
}

#[test]
fn exact_record_result_profile_has_linux_and_windows_native_workflow_proof() {
    let workflow = read(&repository_root().join(".github/workflows/rust.yml"));
    for required in [
        "Test exact i32 record and Result CPU profile at O0 and O2",
        "Test exact i32 record and Result CPU profile on Windows at O0 and O2",
        "examples/fixed_int_array_v0/exact_record_result_application.aero",
        "--language-profile exact-i32-record-result-v0",
        "llvm-as-22 \"${llvm}\" -o \"${bitcode}\"",
        "opt-22 -passes=verify -disable-output \"${llvm}\"",
        "llc-22 -verify-machineinstrs \"${llvm}\" -o \"${machine}\"",
        "clang-22 -O0 \"${llvm}\" -o \"${executable_o0}\"",
        "clang-22 -O2 \"${llvm}\" -o \"${executable_o2}\"",
        "& \"$llvmBin\\llvm-as.exe\" $llvm -o $bitcode",
        "& \"$llvmBin\\opt.exe\" -passes=verify -disable-output $llvm",
        "& \"$llvmBin\\llc.exe\" -verify-machineinstrs $llvm -o $machine",
        "& \"$llvmBin\\clang.exe\" -O0 $llvm -o $executableO0",
        "& \"$llvmBin\\clang.exe\" -O2 $llvm -o $executableO2",
        "Exit code: 91",
    ] {
        assert!(
            workflow.contains(required),
            "CAP-032 workflow omitted `{required}`"
        );
    }
    assert!(
        workflow
            .matches("--language-profile exact-i32-record-result-v0")
            .count()
            >= 6,
        "CAP-032 workflow does not exercise check/build/repeat/run on both systems"
    );
    assert!(
        workflow.contains("test ! -s \"${stdout}\"")
            && workflow.contains("test ! -s \"${stderr}\"")
            && workflow.contains("$native.Stdout -cne \"\"")
            && workflow.contains("$native.Stderr -cne \"\""),
        "CAP-032 workflow does not require silent native execution"
    );
}
