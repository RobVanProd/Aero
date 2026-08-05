use compiler::conformance::run_conformance_suite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn repository_file(path: &str) -> String {
    let full_path = repository_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", full_path.display()))
}

fn run_aero(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .args(args)
        .output()
        .expect("run Aero CLI")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_implementation_version_is_manifest_derived_on_every_existing_route() {
    let expected_version = format!("Aero compiler version {PACKAGE_VERSION}");
    for flag in ["--version", "-v"] {
        let output = run_aero(&[flag]);
        assert!(output.status.success(), "{flag}: {}", stderr(&output));
        assert_eq!(stdout(&output).trim(), expected_version);
    }

    let no_command = run_aero(&[]);
    assert_eq!(no_command.status.code(), Some(2));
    assert_eq!(
        stdout(&no_command).lines().next(),
        Some(format!("Aero Programming Language Compiler v{PACKAGE_VERSION}").as_str())
    );

    let bare_version = run_aero(&["version"]);
    assert_eq!(bare_version.status.code(), Some(2));
    assert!(stderr(&bare_version).contains("Unknown command: version"));

    let main = repository_file("src/compiler/src/main.rs");
    assert!(
        main.contains(r#"env!("CARGO_PKG_VERSION")"#),
        "main must interpolate Cargo's package version"
    );
    assert!(
        !main.contains(PACKAGE_VERSION),
        "main must not contain the current package-version literal"
    );
    assert!(!main.contains(&format!("Aero compiler version {PACKAGE_VERSION}")));
    assert!(!main.contains(&format!(
        "Aero Programming Language Compiler v{PACKAGE_VERSION}"
    )));
}

#[test]
fn conformance_counts_and_compatibility_schema_remain_unchanged() {
    let report = run_conformance_suite();
    assert_eq!((report.passed_cases, report.total_cases), (3, 3));
    assert_eq!(
        (
            report.passed_mechanized_checks,
            report.total_mechanized_checks
        ),
        (4, 4)
    );

    let json = serde_json::to_value(&report).expect("serialize conformance report");
    let object = json.as_object().expect("report JSON object");
    for key in [
        "total_mechanized_checks",
        "passed_mechanized_checks",
        "failed_mechanized_checks",
        "mechanized_checks",
    ] {
        assert!(object.contains_key(key), "missing compatibility key {key}");
    }
}

#[test]
fn conformance_is_presented_as_deterministic_regression_evidence() {
    let output = run_aero(&["conformance"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let output_text = stdout(&output);
    assert!(output_text.contains("Conformance cases: 3/3 passed | Determinism checks: 4/4 passed"));
    assert!(!output_text.contains("Mechanized checks"));

    let help = run_aero(&["--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let help_text = stdout(&help);
    assert!(help_text.contains("Run deterministic regression checks"));
    assert!(
        !help_text
            .to_ascii_lowercase()
            .contains("formal conformance")
    );
    assert!(!help_text.to_ascii_lowercase().contains("mechanized checks"));

    let build = repository_file("BUILD.md");
    assert!(build.contains("deterministic regression checks"));
    assert!(!build.contains("CLI command summary (v1.0.0)"));
    assert!(
        !build
            .to_ascii_lowercase()
            .contains("formal conformance suite")
    );
    assert!(!build.to_ascii_lowercase().contains("mechanized semantics"));
}

#[test]
fn current_repository_surfaces_state_only_evidenced_capabilities() {
    let readme = repository_file("README.md");
    assert!(readme.contains("Generic and trait syntax is parsed but quarantined"));
    assert!(readme.contains(
        "No general borrow checker, mutable references, lifetime analysis, drop model, or memory-safety guarantee."
    ));
    assert!(readme.contains("3 example cases and 4 deterministic regression checks"));
    assert!(!readme.contains("formal conformance + mechanized checks"));
    assert!(
        !readme
            .contains("| **Type System** | Static typing, generics, trait bounds, where clauses |")
    );
    assert!(!readme.contains(
        "| **Memory** | Ownership, move semantics, shared & mutable references, borrow checker |"
    ));

    let claude = repository_file("CLAUDE.md");
    assert!(claude.contains("## Current Evidence Status"));
    assert!(claude.contains("Minimal prototype / correctness recovery"));
    assert!(!claude.contains("## Current Phase: Phase 5 (Advanced Features) — COMPLETE"));
    assert!(!claude.contains("Ownership and move semantics (DONE)"));
    assert!(!claude.contains("References and borrowing syntax (DONE)"));
    assert!(!claude.contains("Borrow checker enforcement (DONE)"));
    assert!(!claude.contains("Generics: type params, trait bounds, where clauses (DONE)"));
    assert!(!claude.contains("Traits: registry, completeness checking, bound enforcement (DONE)"));
    assert!(
        !claude.contains("174 tests passing (63 unit + 52 optimizer + 59 frontend integration)")
    );
    assert!(!claude.contains("37/38 Phase 5 spec tests passing"));
    assert!(!claude.contains("## Completed Phases"));
    assert!(!claude.contains(
        "Phase 5: Advanced Features (ownership, borrowing, borrow checker, generics, traits)"
    ));

    let tutorial_one = repository_file("tutorials/01-getting-started.md");
    assert!(tutorial_one.contains("deterministic regression checks"));
    assert!(tutorial_one.contains("not a formal semantics proof"));
    assert!(tutorial_one.contains("conceptual ownership and borrowing design"));
    assert!(!tutorial_one.contains("conformance and mechanized checks"));
    assert!(!tutorial_one.contains("against the formal suite"));
    assert!(!tutorial_one.contains("Aero's key memory safety feature"));

    let tutorial_two = repository_file("tutorials/02-core-features.md");
    assert!(tutorial_two.contains("design-only ownership and borrowing model"));
    assert!(!tutorial_two.contains("powerful features for memory safety"));

    let tutorial_four = repository_file("tutorials/04-data-structures.md");
    assert!(tutorial_four.contains("**Current implementation boundary:**"));
}

#[test]
fn normative_safety_documents_are_visibly_design_targets() {
    for path in [
        "docs/language/aero_formal_language_specification.md",
        "docs/language/aero_type_system.md",
        "docs/language/aero_ownership_borrowing.md",
        "tutorials/03-ownership-borrowing.md",
    ] {
        let document = repository_file(path);
        let introduction = document.lines().take(12).collect::<Vec<_>>().join("\n");
        assert!(
            introduction.contains("**Design target — not current implementation evidence.**"),
            "missing leading design-target notice in {path}"
        );
    }

    let formal = repository_file("docs/language/aero_formal_language_specification.md");
    assert!(formal.contains("v1.0.0 remains a language design target"));
    assert!(formal.contains("not the compiler package version"));
    assert!(formal.contains("not a conformance or stability claim"));
}

#[test]
fn grammar_and_core_tutorial_are_visibly_design_targets() {
    let mut missing = Vec::new();
    for path in [
        "docs/language/aero_grammar.md",
        "tutorials/02-core-features.md",
    ] {
        let document = repository_file(path);
        let introduction = document.lines().take(12).collect::<Vec<_>>().join("\n");
        for (requirement, expected) in [
            (
                "leading design-target marker",
                "**Design target — not current implementation evidence.**",
            ),
            ("v1 design authority", "Aero v1.0.0 design target"),
            (
                "current compiler boundary",
                "not the currently implemented compiler subset",
            ),
            (
                "conformance and stability boundary",
                "not conformance or stability evidence",
            ),
            ("current capability audit", "CURRENT_CAPABILITY_AUDIT.md"),
            ("implementation matrix", "SPEC_IMPLEMENTATION_MATRIX.md"),
        ] {
            if !introduction.contains(expected) {
                missing.push(format!("{path}: missing {requirement}"));
            }
        }
    }

    let grammar = repository_file("docs/language/aero_grammar.md");
    let unqualified_authority =
        "definitive guide for implementing the lexer and parser components of the Aero compiler";
    if grammar.contains(unqualified_authority) {
        missing
            .push("docs/language/aero_grammar.md: unqualified compiler authority remains".into());
    }
    let normative_boundary = "Every EBNF production below is part of the normative Aero v1.0.0 design target, not a statement of current compiler conformance.";
    if !grammar.contains(normative_boundary) {
        missing.push("docs/language/aero_grammar.md: missing normative v1 boundary".into());
    }

    assert!(
        missing.is_empty(),
        "missing grammar/tutorial authority boundaries:\n{}",
        missing.join("\n")
    );
}

#[test]
fn historical_completion_records_are_visibly_archived() {
    for path in [
        "todo.md",
        "docs/demos/builtin_collections_demo.md",
        "docs/demos/collection_string_demo.md",
        "docs/demos/enum_pattern_demo.md",
        "docs/demos/struct_generation_demo.md",
        "docs/tasks/TASK_10_1_LLVM_STRUCT_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_10_2_ENUM_PATTERN_IR_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_10_3_COLLECTION_STRING_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_11_BUILTIN_COLLECTIONS_LIBRARY_SUMMARY.md",
    ] {
        let document = repository_file(path);
        let document = document
            .lines()
            .take(12)
            .flat_map(str::split_whitespace)
            .filter(|word| *word != ">")
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase();
        assert!(
            document.contains("historical"),
            "missing leading historical label in {path}"
        );
        assert!(
            document.contains("not current")
                || document.contains("not active")
                || document.contains("not evidence")
                || document.contains("does not describe the active"),
            "missing leading non-capability qualification in {path}"
        );
    }
}

#[test]
fn repository_remains_explicitly_experimental_without_stability_claims() {
    let readme = repository_file("README.md");
    assert!(readme.contains("Experimental systems language and compiler repository"));
    assert!(readme.contains("minimal prototype under correctness recovery"));

    let project_state = repository_file("PROJECT_STATE.md");
    assert!(project_state.contains("Repository stability: experimental"));
    assert!(
        project_state.contains("Formal conformance: three example cases plus four deterministic")
    );
}
