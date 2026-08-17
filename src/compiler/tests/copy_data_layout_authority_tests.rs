use compiler::{CompilerOptions, LanguageProfile, compile_program};
use std::fs;
use std::path::{Path, PathBuf};

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

fn read_int(value: &int) -> int {
    return *value;
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
    let seed: int = 9;
    let success: Result<Frame, int> = make_frame(1 < 2);
    let failure: Result<Frame, int> = make_frame(2 < 1);
    if read_int(&seed) == 9
        && result_score(success) == 8
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

#[test]
fn accepted_profile_llvm_bytes_are_frozen_before_layout_consolidation() {
    let experimental = llvm(EXPERIMENTAL_RECURSIVE_SOURCE, LanguageProfile::Experimental);
    let default = compile_program(EXPERIMENTAL_RECURSIVE_SOURCE, CompilerOptions::default())
        .expect("default recursive characterization must compile");
    assert_eq!(default, experimental, "default/experimental LLVM drifted");

    let stable = llvm(STABLE_SCALAR_SOURCE, LanguageProfile::StableScalarV0);
    let exact = llvm(EXACT_CAP023_SOURCE, LanguageProfile::ExactI32ArrayV0);
    assert_eq!(
        stable,
        llvm(STABLE_SCALAR_SOURCE, LanguageProfile::StableScalarV0),
        "stable-scalar LLVM became nondeterministic"
    );
    assert_eq!(
        exact,
        llvm(EXACT_CAP023_SOURCE, LanguageProfile::ExactI32ArrayV0),
        "exact CAP-023 LLVM became nondeterministic"
    );
    let actual = [
        md5_hex(experimental.as_bytes()),
        md5_hex(stable.as_bytes()),
        md5_hex(exact.as_bytes()),
    ];

    // Re-frozen by CORE-093, which moves every static `alloca` into the entry
    // block so a loop body cannot grow the stack once per iteration. For each of
    // these three programs the change was verified to move nothing else: the
    // line multiset, the alloca line multiset, and the relative order of every
    // non-alloca line are all identical to the previous digests' modules.
    assert_eq!(
        actual,
        [
            "e36e43b4d14b99332a47cf23b1de1784",
            "9aa9981631c60de5058c928bc8ac060f",
            "f74b8b67d5c10c9ef18cd67a07c90e23",
        ],
        "freeze these accepted-head LLVM byte digests before production mutation"
    );

    for anchor in [
        "%aero.struct.Frame = type { %aero.struct.Leaf, [2 x double], { double, i1 } }",
        "%aero.struct.Leaf = type { double, i1 }",
        "{ i32, %aero.struct.Frame, double }",
        "{ i32, double, i1 }",
        "double* %aero.arg.value",
    ] {
        assert!(
            experimental.contains(anchor),
            "experimental recursive LLVM omitted `{anchor}`:\n{experimental}"
        );
    }
    assert!(
        stable.contains("define i32 @mix(i32 %aero.arg.left, i32 %aero.arg.right)"),
        "stable scalar LLVM lane drifted:\n{stable}"
    );
    assert!(
        exact.contains("define [8 x i32] @infer_record([20 x i32] %aero.arg.record)"),
        "exact CAP-023 flat-array lane drifted:\n{exact}"
    );
}

#[test]
fn recursive_copydata_physical_layout_has_one_shared_authority() {
    let root = repository_root();
    let shared_path = root.join("src/compiler/src/copy_data_layout.rs");
    assert!(
        shared_path.is_file(),
        "CAP-026 intentional structural red: shared copy_data_layout.rs is absent"
    );

    let shared = read(&shared_path);
    let library = read(&root.join("src/compiler/src/lib.rs"));
    let backend = read(&root.join("src/compiler/src/code_generator.rs"));
    let verifier = read(&root.join("src/compiler/src/ir_verifier.rs"));
    let backend_production = backend
        .split("\n#[cfg(test)]")
        .next()
        .expect("backend source has a production prefix");

    assert!(
        library.contains("mod copy_data_layout;"),
        "crate root does not own the shared CopyData layout module"
    );
    for anchor in [
        "CopyDataLayout",
        "CopyDataLayoutPolicy",
        "llvm_type",
        "zero_value",
        "alignment",
        "enum_llvm_type",
    ] {
        assert!(
            shared.contains(anchor),
            "shared CopyData layout authority omitted `{anchor}`"
        );
    }
    for duplicate in [
        "fn copy_data_type_to_llvm",
        "fn enum_schema_to_llvm",
        "fn enum_schema_is_scalar_only",
        "fn enum_payload_lane",
        "fn copy_data_zero_value",
        "fn struct_field_type_to_llvm",
    ] {
        assert!(
            !backend_production.contains(duplicate),
            "backend retained duplicate physical authority `{duplicate}`"
        );
    }
    assert!(
        !verifier.contains("fn physical_copy_type_hint"),
        "verifier retained its duplicate recursive physical hint renderer"
    );
    for primitive_duplicate in [".copy_data_llvm_type()", ".copy_data_zero()"] {
        assert!(
            !backend_production.contains(primitive_duplicate),
            "backend bypasses shared primitive-lane authority via `{primitive_duplicate}`"
        );
        assert!(
            !verifier.contains(primitive_duplicate),
            "verifier bypasses shared primitive-lane authority via `{primitive_duplicate}`"
        );
        assert!(
            shared.contains(primitive_duplicate),
            "shared authority does not delegate primitive identity via `{primitive_duplicate}`"
        );
    }
    assert!(
        !backend_production.contains("\"{ i32, double, i1 }\""),
        "backend retained the hardcoded compact-enum physical schema"
    );
    assert!(
        !backend_production.contains("PrimitiveKind::alignment"),
        "backend bypasses shared primitive alignment authority"
    );
    for compact_lane_duplicate in [
        "insertvalue {enum_type} %{with_tag}, double",
        "insertvalue {enum_type} %{numeric}, i1",
        ".lane_llvm_type(1,",
        ".lane_llvm_type(2,",
        ".lane_zero_value(1)",
        ".lane_zero_value(2)",
        "insertvalue {enum_type} poison, i32",
        "extractvalue {enum_type} %{parameter}, 0",
        "extractvalue {enum_type} %reg{value}, 0",
    ] {
        assert!(
            !backend_production.contains(compact_lane_duplicate),
            "backend retained hardcoded compact-enum lane `{compact_lane_duplicate}`"
        );
    }
    for shared_consumer in [
        ".tag_lane()",
        ".compact_numeric_lane()",
        ".compact_boolean_lane()",
        ".payload_variants()",
        "checked_place_storage",
    ] {
        assert!(
            backend_production.contains(shared_consumer),
            "backend does not delegate `{shared_consumer}` to the shared layout authority"
        );
    }
    for checked_array_duplicate in [
        "alloca [{count} x {element}]",
        "format!(\"[{count} x {element}]\")",
    ] {
        assert!(
            !backend_production.contains(checked_array_duplicate),
            "backend retained checked-array topology `{checked_array_duplicate}` outside the shared descriptor"
        );
    }
    assert!(
        backend_production.contains("CopyDataLayout::legacy")
            && backend_production.contains("CopyDataLayout::with_policy"),
        "backend does not consume the shared physical descriptor"
    );
    assert!(
        verifier.contains("CopyDataLayout::legacy")
            && verifier.contains("EnumStorageLayout::legacy"),
        "verifier does not consume the shared physical descriptor"
    );
    assert!(
        !verifier.contains("let _physical_layout = EnumStorageLayout"),
        "verifier computes and discards the shared enum layout"
    );
    assert!(
        verifier.contains("existing_physical_layout != &physical_layout"),
        "verifier does not compare repeated checked enum physical layouts"
    );
    for raw_legacy_anchor in [
        "\"  %{} = alloca [{} x {}], align 8\\n\"",
        "\"  %{} = getelementptr inbounds {}, {}* %{}, i64 0, i64 {}\\n\"",
    ] {
        assert!(
            backend_production.contains(raw_legacy_anchor),
            "raw legacy array/GEP emission anchor `{raw_legacy_anchor}` changed"
        );
    }
}
