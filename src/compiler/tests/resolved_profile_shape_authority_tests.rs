use compiler::{
    CompilerOptions, LanguageProfile, check_file, check_program, compile_file, compile_program,
    prepare_checked_program_for_compiler_service,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

const EXCLUDED_STRUCT_SOURCE: &str = r#"
struct Pair {
    left: int,
    right: int,
}

fn main() -> int {
    return 0;
}
"#;

const SEMANTIC_ERROR_SOURCE: &str = r#"
fn main() -> int {
    return missing;
}
"#;

const SEMANTIC_ERROR: &str =
    "Semantic Analysis Error: Error: Use of undeclared variable `missing`.";

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

struct TempWorkspace(PathBuf);

impl TempWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aero-cap028-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).expect("create CAP-028 characterization workspace");
        Self(root)
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write CAP-028 characterization source");
        path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn accepted_behavior_is_frozen_before_resolved_profile_authority() {
    let experimental = llvm(EXPERIMENTAL_RECURSIVE_SOURCE, LanguageProfile::Experimental);
    let stable = llvm(STABLE_SCALAR_SOURCE, LanguageProfile::StableScalarV0);
    let exact = llvm(EXACT_CAP023_SOURCE, LanguageProfile::ExactI32ArrayV0);

    assert_eq!(
        [
            md5_hex(experimental.as_bytes()),
            md5_hex(stable.as_bytes()),
            md5_hex(exact.as_bytes()),
        ],
        [
            "724bac62708812d4302224fec1047be6",
            "cbb7a6446d27119d50f70868bc2b6a96",
            "54bbfe8dc403ba00ff0587fd3b99e14a",
        ],
        "freeze accepted-head LLVM bytes before descriptor construction"
    );
    assert_eq!(
        experimental,
        llvm(EXPERIMENTAL_RECURSIVE_SOURCE, LanguageProfile::Experimental),
        "experimental descriptor specimen became nondeterministic"
    );
    assert_eq!(
        stable,
        llvm(STABLE_SCALAR_SOURCE, LanguageProfile::StableScalarV0),
        "stable descriptor control became nondeterministic"
    );
    assert_eq!(
        exact,
        llvm(EXACT_CAP023_SOURCE, LanguageProfile::ExactI32ArrayV0),
        "exact descriptor control became nondeterministic"
    );

    let checked_debug = format!(
        "{:?}",
        prepare_checked_program_for_compiler_service(STABLE_SCALAR_SOURCE, None, None)
            .expect("canonical checked preparation must remain available")
    );
    assert!(
        !checked_debug.contains("resolved_profile"),
        "the out-of-band prerequisite changed CheckedProgram's compatibility Debug surface"
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
        let file = compile_file(&path, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} file route failed: {error}"));
        assert_eq!(
            file, *expected,
            "{profile:?} source/file LLVM bytes drifted"
        );
        assert_eq!(
            md5_hex(file.as_bytes()),
            md5_hex(expected.as_bytes()),
            "{profile:?} source/file LLVM digest drifted"
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
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic == SEMANTIC_ERROR),
            "{profile:?} semantic diagnostic route drifted: {diagnostics:?}"
        );
    }

    let path = workspace.source("excluded_struct.aero", EXCLUDED_STRUCT_SOURCE);
    for profile in [
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
    ] {
        let source_compile = compile_program(EXCLUDED_STRUCT_SOURCE, options(profile))
            .expect_err("excluded source route must reject the struct");
        let file_compile = compile_file(&path, options(profile))
            .expect_err("excluded file route must reject the struct");
        assert_eq!(
            source_compile, file_compile,
            "{profile:?} source/file compile diagnostics drifted"
        );

        let source_check = check_program(EXCLUDED_STRUCT_SOURCE, options(profile))
            .expect_err("excluded source check must reject the struct");
        let file_check = check_file(&path, options(profile))
            .expect_err("excluded file check must reject the struct");
        assert_eq!(
            source_check, file_check,
            "{profile:?} source/file check diagnostics drifted"
        );
        assert_eq!(
            source_compile, source_check,
            "{profile:?} compile/check diagnostic precedence drifted"
        );
        let expected = match profile {
            LanguageProfile::StableScalarV0 => {
                "Language Profile Error: stable-scalar-v0 rejects struct definitions"
            }
            LanguageProfile::ExactI32ArrayV0 => {
                "Language Profile Error: exact-i32-array-v0 rejects struct definitions"
            }
            LanguageProfile::Experimental => unreachable!("loop excludes Experimental"),
        };
        assert_eq!(source_compile, expected, "{profile:?} profile text drifted");
    }
}

#[test]
fn representative_native_sentinels_remain_91() {
    let workspace = TempWorkspace::new();
    for (name, source, profile) in [
        (
            "experimental-native.aero",
            EXPERIMENTAL_RECURSIVE_SOURCE,
            LanguageProfile::Experimental,
        ),
        (
            "stable-native.aero",
            STABLE_SCALAR_SOURCE,
            LanguageProfile::StableScalarV0,
        ),
        (
            "exact-native.aero",
            EXACT_CAP023_SOURCE,
            LanguageProfile::ExactI32ArrayV0,
        ),
    ] {
        let path = workspace.source(name, source);
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
}

#[test]
fn resolved_profile_shape_has_one_post_semantic_authority() {
    let root = repository_root();
    let authority_path = root.join("src/compiler/src/resolved_profile_shape.rs");
    assert!(
        authority_path.is_file(),
        "CAP-028 intentional structural red: resolved_profile_shape.rs is absent"
    );

    let authority = read(&authority_path);
    let authority_production = authority
        .split("\n#[cfg(test)]")
        .next()
        .expect("resolved profile-shape source has a production prefix");
    let library = read(&root.join("src/compiler/src/lib.rs"));
    let semantics = read(&root.join("src/compiler/src/semantic_analyzer.rs"));

    assert_eq!(
        library.matches("mod resolved_profile_shape;").count(),
        1,
        "crate root must own exactly one resolved profile-shape authority"
    );
    for anchor in [
        "ResolvedProfileProgram",
        "ResolvedProfileShapeId",
        "ResolvedProfileOrigin",
        "ResolvedProfileResolution",
        "ResolvedProfileUse",
        "ProfileTypeUse",
        "ResolvedProfileOperation",
        "StructConstruction",
        "EnumConstruction",
        "ExhaustiveMatch",
    ] {
        assert!(
            authority_production.contains(anchor),
            "resolved profile-shape authority omitted `{anchor}`"
        );
    }
    for forbidden in [
        "normalize_primitive_consts",
        "normalize_copydata_specializations",
        "normalize_builtin_carriers",
        "StructRegistry::from_top_level_ast",
        "EnumRegistry::from_top_level_ast",
        "infer_and_validate_expression",
    ] {
        assert!(
            !authority_production.contains(forbidden),
            "descriptor authority duplicated semantic work via `{forbidden}`"
        );
    }
    for duplicate in ["enum ProfileTypeUse", "enum LogicalType"] {
        assert!(
            !authority_production.contains(duplicate),
            "descriptor authority declared a duplicate `{duplicate}`"
        );
    }
    for forbidden_dependency in [
        "LanguageProfile",
        "IrGenerator",
        "CheckedIr",
        "CopyDataLayout",
    ] {
        assert!(
            !authority_production.contains(forbidden_dependency),
            "descriptor production authority crossed into `{forbidden_dependency}`"
        );
    }
    let build_body = authority_production
        .split("fn build(")
        .nth(1)
        .and_then(|tail| tail.split("\n    fn record_declaration").next())
        .expect("descriptor authority must expose one isolated build body");
    assert_eq!(
        build_body.matches("for node in ast").count(),
        1,
        "semantic success must traverse the normalized AST exactly once"
    );
    assert_eq!(
        build_body.matches("self.record_declaration(node)").count(),
        1,
        "each normalized node must contribute declaration facts exactly once"
    );
    assert_eq!(
        build_body.matches("self.walk_node(node)").count(),
        1,
        "each normalized node must contribute use and operation facts exactly once"
    );
    assert_eq!(
        semantics
            .matches("ResolvedProfileProgram::from_semantic_success")
            .count(),
        1,
        "semantic success must finalize exactly one descriptor"
    );
    assert!(
        semantics.contains(
            "pub fn analyze(&mut self, ast: Vec<AstNode>) -> Result<(String, Vec<AstNode>), String>"
        ),
        "public SemanticAnalyzer::analyze signature changed"
    );
    assert_eq!(
        semantics
            .matches("pub(crate) fn analyze_with_resolved_profile")
            .count(),
        1,
        "rich semantic success must remain one crate-private entrypoint"
    );
    assert!(
        library.contains(".analyze_with_resolved_profile(ast)"),
        "canonical library preparation does not consume the rich semantic success"
    );
    assert!(
        library.contains("_resolved_profile:"),
        "CheckedProgram does not carry the immutable descriptor out of band"
    );
    let checked_debug = library
        .split("impl std::fmt::Debug for CheckedProgram")
        .nth(1)
        .and_then(|tail| tail.split("impl CheckedProgram").next())
        .expect("CheckedProgram must preserve its compatibility Debug surface");
    assert!(
        !checked_debug.contains("_resolved_profile"),
        "CheckedProgram Debug leaked the out-of-band prerequisite"
    );
}
