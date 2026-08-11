use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn source_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(name)
}

fn declared_modules(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let declaration = line
                .strip_prefix("pub mod ")
                .or_else(|| line.strip_prefix("mod "))?;
            let name = declaration
                .split(|character: char| {
                    character == ';' || character.is_whitespace() || character == '{'
                })
                .next()?;
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

#[test]
fn binary_uses_one_canonical_library_compiler_graph() {
    let library = fs::read_to_string(source_path("lib.rs")).expect("read library crate root");
    let binary = fs::read_to_string(source_path("main.rs")).expect("read binary crate root");
    let profiler = fs::read_to_string(source_path("profiler.rs")).expect("read profiler module");
    let library_modules = declared_modules(&library);
    let binary_modules = declared_modules(&binary);
    let overlap = library_modules
        .intersection(&binary_modules)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        overlap.is_empty(),
        "binary duplicates {} library compiler modules instead of consuming one canonical graph: {}",
        overlap.len(),
        overlap.join(", ")
    );

    let expected_binary_modules = BTreeSet::from([
        "conformance_checked_ir_tests".to_string(),
        "doc_generator".to_string(),
        "llvm_verifier_cache_tests".to_string(),
        "lsp".to_string(),
        "profiler".to_string(),
        "project_init".to_string(),
        "tests".to_string(),
    ]);
    assert_eq!(
        binary_modules, expected_binary_modules,
        "only frozen CLI-specific modules may remain owned by the binary"
    );
    assert!(
        binary.contains("use compiler::"),
        "binary must consume the compiler library facade"
    );
    assert!(
        !binary.contains("module_resolver::"),
        "binary must not bypass the library-owned direct-module service"
    );

    for declaration in [
        "mod compatibility;",
        "mod optimizations;",
        "mod performance_optimizations;",
        "pub use performance_optimizations::PerformanceOptimizer;",
        "pub fn collect_direct_modules_for_compiler_service(",
        "pub fn prepare_checked_program_with_module_observer(",
    ] {
        assert!(
            library.contains(declaration),
            "library must own `{declaration}` after graph convergence"
        );
    }

    for (name, source) in [("binary", &binary), ("profiler", &profiler)] {
        assert!(
            source.contains("prepare_checked_program_with_module_observer"),
            "{name} must consume the library-owned checked-program authority"
        );
        for forbidden in [
            "SemanticAnalyzer::new()",
            "IrGenerator::new()",
            "lexer::try_tokenize_with_locations",
            "parser::parse_with_locations",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} duplicates checked-program phase orchestration through `{forbidden}`"
            );
        }
    }
}
