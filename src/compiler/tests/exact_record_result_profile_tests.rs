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
    check_program(&source, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 source check rejected the product: {error}"));
    check_file(&application_path, options(profile))
        .unwrap_or_else(|error| panic!("CAP-032 file check rejected the product: {error}"));

    for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
        assert!(
            !source_llvm.contains(forbidden),
            "CAP-032 exact product emitted forbidden LLVM token `{forbidden}`"
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("run")
        .arg(&application_path)
        .arg("--language-profile")
        .arg(PROFILE_NAME)
        .current_dir(root)
        .output()
        .expect("run CAP-032 product");
    assert_eq!(
        output.status.code(),
        Some(91),
        "CAP-032 product must return 91 (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "CAP-032 product must remain silent"
    );
}
