use compiler::{CompilerOptions, LanguageProfile, check_program, compile_program};
use std::fs;
use std::path::{Path, PathBuf};

const PROFILE_NAME: &str = "exact-i32-byte-buffer-v0";

const CHARACTERIZATION_SOURCE: &str = r#"
fn main() -> int {
    return 91;
}
"#;

const VEC_NEW_SOURCE: &str = r#"
fn main() -> int {
    let mut bytes: Vec<int> = Vec::new();
    return 0;
}
"#;

const EMPTY_VEC_SOURCE: &str = r#"
fn main() -> int {
    let bytes: Vec<int> = vec![];
    return 0;
}
"#;

const SOURCE_BYTE_BUFFER_PRODUCT: &str = r#"
fn result_value(result: Result<int, int>) -> int {
    return match result {
        Ok(value) => value,
        Err(code) => 0 - code,
    };
}

fn main() -> int {
    let mut source: ByteBuffer = bytes_new();
    let first: Result<int, int> = bytes_push(&mut source, 91);
    let mut bytes: ByteBuffer = source;
    let mut index: int = 0;
    while index < 2 {
        let step: Result<int, int> = bytes_push(&mut bytes, index);
        if result_value(step) < 0 {
            return 2;
        }
        index = index + 1;
    }
    let found: Result<int, int> = bytes_get(&bytes, 0);
    let missing: Result<int, int> = bytes_get(&bytes, 9);
    if result_value(first) == 1
        && bytes_len(&bytes) == 3
        && bytes_capacity(&bytes) == 8
        && result_value(found) == 91
        && result_value(missing) == -4 {
        return 91;
    }
    return 1;
}
"#;

fn options(language_profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile,
        ..CompilerOptions::default()
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

#[test]
fn accepted_profiles_and_owned_byte_substrates_are_frozen_before_r1c() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            md5_hex(llvm.as_bytes()),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R1C"
        );
        for forbidden in [
            "%aero.byte_buffer",
            "@aero_alloc",
            "@aero_realloc",
            "@aero_dealloc",
        ] {
            assert!(
                !llvm.contains(forbidden),
                "ordinary {profile:?} source unexpectedly emitted `{forbidden}`"
            );
        }
    }

    assert_eq!(
        check_program(VEC_NEW_SOURCE, CompilerOptions::default())
            .expect_err("Vec::new must remain absent during R1C"),
        "Semantic Analysis Error: enum `Vec` has no unique admitted definition"
    );
    assert_eq!(
        check_program(EMPTY_VEC_SOURCE, CompilerOptions::default())
            .expect_err("vec![] must remain a rejected fixed-array literal"),
        "IR Generation Error: empty array literals have no admitted logical element type"
    );

    let root = repository_root();
    for (relative, expected) in [
        (
            "src/compiler/runtime/aero_runtime.c",
            "2604780079240d54ebbda84bb205c39d",
        ),
        (
            "src/compiler/runtime/aero_test_runtime.c",
            "5f1db08f29355e78a1dda31747ec7055",
        ),
        (
            "src/compiler/src/ir.rs",
            "cf45477b10aad24e9a4f0f769910ddfc",
        ),
        (
            "src/compiler/src/ir_verifier.rs",
            "8f937004d4b2a3efb262dbacbe5fc474",
        ),
    ] {
        assert_eq!(
            md5_hex(
                &fs::read(root.join(relative))
                    .unwrap_or_else(|error| panic!("read {relative}: {error}"))
            ),
            expected,
            "accepted authority `{relative}` drifted during R1C"
        );
    }

    for relative in ["src/compiler/src/ast.rs", "src/compiler/src/parser.rs"] {
        assert!(
            !read(&root, relative).contains("ByteBuffer"),
            "R1C must not add parser/AST syntax through `{relative}`"
        );
    }
}

#[test]
fn source_byte_buffer_profile_is_selector_red_first() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("R1C red: exact-i32-byte-buffer-v0 selector is absent");
    assert_eq!(profile.as_str(), PROFILE_NAME);

    check_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must pass the public check route");
    let first = compile_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must compile");
    let second = compile_program(SOURCE_BYTE_BUFFER_PRODUCT, options(profile))
        .expect("R1C source product must compile deterministically");
    assert_eq!(first, second, "R1C LLVM must be deterministic");
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
    ] {
        assert!(first.contains(anchor), "R1C LLVM omitted `{anchor}`");
    }

    let root = repository_root();
    let profile_source = read(&root, "src/compiler/src/language_profile.rs");
    let types = read(&root, "src/compiler/src/types.rs");
    let semantics = read(&root, "src/compiler/src/semantic_analyzer.rs");
    let resolved = read(&root, "src/compiler/src/resolved_profile_shape.rs");
    let generator = read(&root, "src/compiler/src/ir_generator.rs");
    let library = read(&root, "src/compiler/src/lib.rs");
    let cli = read(&root, "src/compiler/src/main.rs");

    for anchor in [
        "EXACT_I32_BYTE_BUFFER_V0_NAME",
        "ExactI32ByteBufferV0",
        "exact-i32-byte-buffer-v0",
    ] {
        assert!(
            profile_source.contains(anchor),
            "R1C profile authority omitted `{anchor}`"
        );
    }
    assert!(types.contains("ByteBuffer,"), "R1C semantic type is absent");
    assert!(
        root.join("src/compiler/src/byte_buffer_source_contract.rs")
            .is_file(),
        "R1C shared source contract is absent"
    );
    for (source, anchor) in [
        (&semantics, "new_with_byte_buffer_source"),
        (&resolved, "ResolvedProfileCallArgumentKind"),
        (&generator, "new_with_byte_buffer_source"),
        (&generator, "CheckedByteBufferNew"),
        (&generator, "CheckedByteBufferDrop"),
        (&library, "mod byte_buffer_source_contract;"),
        (&cli, "exact-i32-byte-buffer-v0"),
    ] {
        assert!(source.contains(anchor), "R1C authority omitted `{anchor}`");
    }
}
