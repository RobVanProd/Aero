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

const HIDDEN_SURFACE_SOURCE: &str = r#"
fn main() -> int {
    let hidden = 1.0;
    println!("{}", hidden);
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
        let root = std::env::temp_dir().join(format!("aero-cap030-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).expect("create CAP-030 characterization workspace");
        Self(root)
    }

    fn source(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write CAP-030 characterization source");
        path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn accepted_behavior_is_frozen_before_resolved_profile_surface_witness() {
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
        "freeze accepted CAP-029 LLVM bytes before surface observation"
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
        assert_eq!(
            llvm(source, profile),
            *expected,
            "{profile:?} repeated source compilation drifted"
        );
        let path = workspace.source(name, source);
        assert_eq!(
            compile_file(&path, options(profile))
                .unwrap_or_else(|error| panic!("{profile:?} file route failed: {error}")),
            *expected,
            "{profile:?} source/file LLVM bytes drifted"
        );
    }

    let hidden = llvm(HIDDEN_SURFACE_SOURCE, LanguageProfile::Experimental);
    let hidden_path = workspace.source("hidden_surface.aero", HIDDEN_SURFACE_SOURCE);
    assert_eq!(
        compile_file(&hidden_path, options(LanguageProfile::Experimental))
            .expect("accepted hidden-surface file must remain compilable"),
        hidden,
        "surface observation must not change accepted Experimental LLVM"
    );
    check_program(
        HIDDEN_SURFACE_SOURCE,
        options(LanguageProfile::Experimental),
    )
    .expect("surface observation must not reject accepted Experimental source");
    check_file(&hidden_path, options(LanguageProfile::Experimental))
        .expect("surface observation must not reject the accepted file route");

    let checked_debug = format!(
        "{:?}",
        prepare_checked_program_for_compiler_service(STABLE_SCALAR_SOURCE, None, None)
            .expect("canonical checked preparation must remain available")
    );
    let debug_fields = [
        "checked_ir",
        "language_profile",
        "semantic_message",
        "direct_module_cache_material",
        "timings",
    ];
    let mut prior = 0;
    for field in debug_fields {
        let index = checked_debug
            .find(field)
            .unwrap_or_else(|| panic!("CheckedProgram Debug omitted `{field}`"));
        assert!(index >= prior, "CheckedProgram Debug field order drifted");
        prior = index;
    }
    assert!(
        !checked_debug.contains("resolved_profile")
            && !checked_debug.contains("surface")
            && !checked_debug.contains("authentication"),
        "the out-of-band witness changed CheckedProgram's Debug surface"
    );

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
fn resolved_profile_surface_witness_extends_the_single_total_authority() {
    let root = repository_root();
    let authority_path = root.join("src/compiler/src/resolved_profile_shape.rs");
    let authority = read(&authority_path);
    let production = authority
        .split("\n#[cfg(test)]")
        .next()
        .expect("resolved profile-shape source has a production prefix");

    for anchor in [
        "ResolvedProfileSurfaceObservation",
        "ResolvedProfileStatementKind",
        "ResolvedProfileExpressionKind",
        "ResolvedProfileBinaryOperator",
        "ResolvedProfileComparisonOperator",
        "ResolvedProfileLogicalOperator",
        "ResolvedProfileUnaryOperator",
        "ResolvedProfilePatternKind",
        "ResolvedProfileAssignmentTarget",
        "ResolvedProfileAssignmentRoot",
        "ResolvedProfileAssignmentProjection",
        "surface: Vec<ResolvedProfileSurfaceObservation>",
        "fn walk_pattern",
        "fn assignment_target",
    ] {
        assert!(
            production.contains(anchor),
            "CAP-030 intentional structural red: surface authority omitted `{anchor}`"
        );
    }

    for forbidden in [
        "normalize_primitive_consts",
        "normalize_copydata_specializations",
        "normalize_builtin_carriers",
        "StructRegistry::from_top_level_ast",
        "EnumRegistry::from_top_level_ast",
        "infer_and_validate_expression",
        "LanguageProfile",
        "ResolvedProfileAuthentication",
        "authenticate_resolved_profile",
        "CopyDataLayout",
        "CodeGenerator",
        "IrGenerator",
        "CheckedIr",
        "RawIr",
        "Inst::",
        "Value::",
        "SourceLocation",
    ] {
        assert!(
            !production.contains(forbidden),
            "surface witness crossed or duplicated authority via `{forbidden}`"
        );
    }

    assert_eq!(
        production.matches("for node in ast").count(),
        1,
        "surface witnesses must reuse the one normalized-AST walk"
    );
    let build_body = production
        .split("fn build(")
        .nth(1)
        .and_then(|tail| tail.split("\n    fn record_declaration").next())
        .expect("descriptor authority must expose one isolated build body");
    assert_eq!(
        build_body.matches("self.record_declaration(node)").count(),
        1,
        "each normalized node must retain one declaration-recording call"
    );
    assert_eq!(
        build_body.matches("self.walk_node(node)").count(),
        1,
        "each normalized node must retain one combined use/operation/surface walk"
    );
    assert_eq!(
        production
            .matches("surface: Vec<ResolvedProfileSurfaceObservation>")
            .count(),
        2,
        "only the immutable program and its one builder may own surface vectors"
    );
    assert_eq!(
        build_body.matches("surface: self.surface").count(),
        1,
        "the builder must finalize its ordered surface stream exactly once"
    );
}
