use compiler::{CompilerOptions, LanguageProfile, check_program, compile_program};
use std::fs;
use std::path::{Path, PathBuf};

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

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn options(profile: LanguageProfile) -> CompilerOptions {
    CompilerOptions {
        language_profile: profile,
        ..CompilerOptions::default()
    }
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

#[test]
fn accepted_source_runtime_and_legacy_vec_boundaries_are_frozen_before_r1b() {
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::StableScalarV0,
        LanguageProfile::ExactI32ArrayV0,
        LanguageProfile::ExactI32RecordResultV0,
    ] {
        let llvm = compile_program(CHARACTERIZATION_SOURCE, options(profile))
            .unwrap_or_else(|error| panic!("{profile:?} characterization failed: {error}"));
        assert_eq!(
            format!("{:x}", md5::compute(llvm.as_bytes())),
            "caf93783f729e0b040bb47170a92085f",
            "{profile:?} LLVM drifted before R1B"
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
            .expect_err("Vec::new must remain absent before R1C"),
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
            "src/compiler/src/main.rs",
            "445a1274d3ac083df46931b4a33fb07e",
        ),
    ] {
        let bytes = fs::read(root.join(relative))
            .unwrap_or_else(|error| panic!("read {relative}: {error}"));
        assert_eq!(
            format!("{:x}", md5::compute(bytes)),
            expected,
            "accepted R1A authority `{relative}` drifted during R1B"
        );
    }
}

#[test]
fn r1b_checked_resource_verifier_and_backend_authorities_are_required() {
    let root = repository_root();
    let ir = read(&root, "src/compiler/src/ir.rs");
    let verifier = read(&root, "src/compiler/src/ir_verifier.rs");
    let backend = read(&root, "src/compiler/src/code_generator.rs");

    for anchor in [
        "ByteBuffer,",
        "pub struct ByteBufferId",
        "pub enum ByteBufferPlaceRole",
        "pub struct ByteBufferPlaceMetadata",
        "pub byte_buffers: BTreeMap<PlaceId, ByteBufferPlaceMetadata>",
        "CheckedByteBufferNew",
        "CheckedByteBufferMove",
        "CheckedByteBufferImmutableBorrow",
        "CheckedByteBufferImmutableBorrowEnd",
        "CheckedByteBufferMutableBorrow",
        "CheckedByteBufferMutableBorrowEnd",
        "CheckedByteBufferPush",
        "CheckedByteBufferLength",
        "CheckedByteBufferCapacity",
        "CheckedByteBufferGet",
        "CheckedByteBufferDrop",
    ] {
        assert!(
            ir.contains(anchor),
            "R1B red: checked IR omitted `{anchor}`"
        );
    }

    for anchor in [
        "fn build_byte_buffer_metadata",
        "fn verify_byte_buffer_resource_flow",
        "byte-buffer resource state differs across a control-flow join or loop backedge",
        "byte-buffer resources and loans must be closed before every reachable return",
        "checked byte-buffer get",
        "CheckedByteBufferDrop",
    ] {
        assert!(
            verifier.contains(anchor),
            "R1B red: verifier omitted `{anchor}`"
        );
    }

    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
        "fn emit_checked_byte_buffer_push",
        "fn emit_checked_byte_buffer_get",
        "fn emit_checked_byte_buffer_drop",
        "byte_buffer_owner_place",
    ] {
        assert!(
            backend.contains(anchor),
            "R1B red: verified backend omitted `{anchor}`"
        );
    }

    for legacy in [
        "Inst::VecAlloca",
        "Inst::VecPush",
        "Inst::VecPop",
        "Inst::VecLength",
        "Inst::VecCapacity",
        "Inst::VecAccess",
        "Inst::VecInit",
    ] {
        assert!(
            verifier.contains(legacy) && backend.contains(legacy),
            "R1B must preserve independent legacy rejection for `{legacy}`"
        );
    }

    for relative in [
        "src/compiler/src/ast.rs",
        "src/compiler/src/types.rs",
        "src/compiler/src/parser.rs",
        "src/compiler/src/semantic_analyzer.rs",
        "src/compiler/src/ir_generator.rs",
    ] {
        assert!(
            !read(&root, relative).contains("ByteBuffer"),
            "R1B must not admit source behavior through `{relative}`"
        );
    }

    assert!(
        root.join("src/compiler/src/owned_byte_buffer_contract_test.rs")
            .is_file(),
        "R1B red: crate-private verifier corruption/native evidence is absent"
    );
}
