use compiler::{CompilerOptions, LanguageProfile, compile_program};
use std::fs;
use std::path::{Path, PathBuf};

const PROFILE_NAME: &str = "exact-i32-byte-input-v0";

const CHARACTERIZATION_SOURCE: &str = r#"
fn main() -> int {
    return 91;
}
"#;

const SOURCE_BYTE_BUFFER_PRODUCT: &str =
    include_str!("../../../examples/owned_byte_buffer_v0/source_owned_byte_buffer.aero");

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

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

#[test]
fn accepted_profiles_and_byte_buffer_are_frozen_before_r2() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
        LanguageProfile::ExactI32ByteBufferV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            md5_hex(llvm.as_bytes()),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R2"
        );
        assert!(
            !llvm.contains("aero_stdin_read_byte"),
            "{profile:?} unexpectedly acquired the R2 runtime ABI"
        );
    }

    let byte_buffer_llvm = compile_program(
        SOURCE_BYTE_BUFFER_PRODUCT,
        options(LanguageProfile::ExactI32ByteBufferV0),
    )
    .expect("accepted R1 source-owned byte-buffer product compiles");
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
    ] {
        assert!(
            byte_buffer_llvm.contains(anchor),
            "accepted R1 LLVM omitted `{anchor}`"
        );
    }
    assert!(
        !byte_buffer_llvm.contains("aero_stdin_read_byte"),
        "accepted R1 product unexpectedly acquired R2 input"
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
        ("src/compiler/src/ir.rs", "cf45477b10aad24e9a4f0f769910ddfc"),
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
            "accepted authority `{relative}` drifted before R2"
        );
    }
}

#[test]
fn whole_stream_binary_stdin_profile_is_selector_red_first() {
    let profile = PROFILE_NAME
        .parse::<LanguageProfile>()
        .expect("R2 red: exact-i32-byte-input-v0 selector is absent");
    assert_eq!(profile.as_str(), PROFILE_NAME);
}
