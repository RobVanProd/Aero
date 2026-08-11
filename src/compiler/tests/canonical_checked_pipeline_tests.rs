use compiler::{CompilerOptions, check_file, check_program, compile_file, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-cap-007-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-007 workspace");
        Self { root }
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, source).expect("write CAP-007 fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        clear_readonly_recursively(&self.root);
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn clear_readonly_recursively(root: &Path) {
    let Ok(metadata) = fs::symlink_metadata(root) else {
        return;
    };
    if metadata.is_dir()
        && let Ok(entries) = fs::read_dir(root)
    {
        for entry in entries.flatten() {
            clear_readonly_recursively(&entry.path());
        }
    }

    #[cfg(windows)]
    {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = fs::set_permissions(root, permissions);
        }
    }
}

fn run_aero(workspace: &TestWorkspace, arguments: &[String]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command
        .current_dir(&workspace.root)
        .args(arguments)
        .stdin(Stdio::null())
        .env("AERO_REQUIRE_LLVM_VERIFIER", "true")
        .env("AERO_LLVM_OPT", workspace.root.join("must-not-run-opt"));
    command.output().expect("run Aero CLI")
}

fn visible_output(output: &Output) -> String {
    let raw = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut visible = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            visible.push(ch);
        }
    }
    visible
}

fn contains_generated_artifact(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            contains_generated_artifact(&path)
        } else {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ll" | "bc" | "o" | "obj" | "exe")
            )
        }
    })
}

fn command_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn file_command_args(command: &str, root: &Path, output: Option<&Path>) -> Vec<String> {
    let mut arguments = vec![command.to_string(), root.to_string_lossy().into_owned()];
    if let Some(output) = output {
        arguments.push("-o".to_string());
        arguments.push(output.to_string_lossy().into_owned());
    }
    arguments
}

#[test]
fn discovered_sources_must_pass_checked_admission_before_completion_is_reported() {
    let workspace = TestWorkspace::new("source-test-admission");
    workspace.write(
        "admission_test.aero",
        "fn main() { let quotient: int = 1 / 0; }",
    );

    let output = run_aero(&workspace, &command_args(&["test"]));
    let rendered = visible_output(&output);
    assert!(
        !output.status.success(),
        "semantic-only test discovery falsely succeeded on a checked-admission rejection:\n{rendered}"
    );
    assert!(
        rendered.contains("IR Generation Error:")
            && rendered.contains("constant integer division by zero"),
        "canonical checked-admission diagnostic was not reported:\n{rendered}"
    );
    assert!(
        !rendered.contains("admission_test.aero analysis completed"),
        "source rejected by checked admission was reported completed:\n{rendered}"
    );
}

#[test]
fn public_check_apis_validate_source_and_direct_modules_without_artifacts() {
    check_program(
        "fn main() -> int { return 17; }",
        CompilerOptions::default(),
    )
    .expect("source-only checked API must accept the scalar subset");

    let workspace = TestWorkspace::new("public-check-api");
    let root = workspace.write(
        "main.aero",
        "mod helper; fn main() -> int { return helper(); }",
    );
    workspace.write("helper.aero", "fn helper() -> int { return 19; }");

    check_file(&root, CompilerOptions::default())
        .expect("file-aware checked API must accept direct modules");
    assert!(
        !contains_generated_artifact(&workspace.root),
        "public check APIs generated a backend artifact"
    );

    let missing_context = check_program("mod helper; fn main() {}", CompilerOptions::default())
        .expect_err("source-only checking must not consult the process working directory");
    assert!(
        missing_context.contains("module declarations require an entry-file context"),
        "source-only module diagnostic lost canonical boundary: {missing_context}"
    );
}

#[test]
fn every_trusted_validation_entrypoint_preserves_failure_stage_and_artifact_boundary() {
    struct FailureCase {
        label: &'static str,
        source: &'static str,
        marker: &'static str,
    }

    let cases = [
        FailureCase {
            label: "lex",
            source: "fn main() { @ }",
            marker: "Lex error:",
        },
        FailureCase {
            label: "parse",
            source: "fn main() { let = ; }",
            marker: "Parse error:",
        },
        FailureCase {
            label: "semantic",
            source: "fn main() { missing_value; }",
            marker: "Semantic Analysis Error:",
        },
        FailureCase {
            label: "checked-admission",
            source: "fn main() { let quotient: int = 1 / 0; }",
            marker: "IR Generation Error:",
        },
    ];

    for case in cases {
        let source_check = check_program(case.source, CompilerOptions::default())
            .expect_err("invalid source must fail source-only checking");
        let source_compile = compile_program(case.source, CompilerOptions::default())
            .expect_err("invalid source must fail source-only compilation");
        assert_eq!(
            source_check, source_compile,
            "{} source-only check/compile diagnostic drift",
            case.label
        );
        assert!(
            source_check.contains(case.marker),
            "{} source-only diagnostic lost stage marker: {source_check}",
            case.label
        );

        let workspace = TestWorkspace::new(case.label);
        let root = workspace.write("case_test.aero", case.source);
        let artifact = workspace.root.join("forbidden.ll");
        let trace = workspace.root.join("forbidden-profile.json");
        let file_check = check_file(&root, CompilerOptions::default())
            .expect_err("invalid source must fail file-aware checking");
        let file_compile = compile_file(&root, CompilerOptions::default())
            .expect_err("invalid source must fail file-aware compilation");
        assert_eq!(
            file_check, file_compile,
            "{} file-aware check/compile diagnostic drift",
            case.label
        );

        for (command, arguments) in [
            ("check", file_command_args("check", &root, None)),
            ("build", file_command_args("build", &root, Some(&artifact))),
            ("run", file_command_args("run", &root, None)),
            ("profile", file_command_args("profile", &root, Some(&trace))),
            ("test", command_args(&["test"])),
        ] {
            let output = run_aero(&workspace, &arguments);
            let rendered = visible_output(&output);
            assert!(
                !output.status.success(),
                "{} public {command} falsely succeeded:\n{rendered}",
                case.label
            );
            assert!(
                rendered.contains(case.marker),
                "{} public {command} lost stage marker {:?}:\n{rendered}",
                case.label,
                case.marker
            );
            assert!(
                !rendered.contains("must-not-run-opt"),
                "{} public {command} reached the external verifier after a source failure:\n{rendered}",
                case.label
            );
            assert!(
                !artifact.exists()
                    && !trace.exists()
                    && !contains_generated_artifact(&workspace.root),
                "{} public {command} left a generated artifact",
                case.label
            );
        }
    }

    let workspace = TestWorkspace::new("module");
    let root = workspace.write("module_test.aero", "mod absent; fn main() {}");
    let artifact = workspace.root.join("forbidden.ll");
    let trace = workspace.root.join("forbidden-profile.json");
    let file_check = check_file(&root, CompilerOptions::default())
        .expect_err("missing module must fail file checking");
    let file_compile = compile_file(&root, CompilerOptions::default())
        .expect_err("missing module must fail file compilation");
    assert_eq!(file_check, file_compile, "module check/compile drift");
    assert!(file_check.contains("Module resolution failed for `absent`"));

    for (command, arguments) in [
        ("check", file_command_args("check", &root, None)),
        ("build", file_command_args("build", &root, Some(&artifact))),
        ("run", file_command_args("run", &root, None)),
        ("profile", file_command_args("profile", &root, Some(&trace))),
        ("test", command_args(&["test"])),
    ] {
        let output = run_aero(&workspace, &arguments);
        let rendered = visible_output(&output);
        assert!(
            !output.status.success() && rendered.contains("Module resolution failed for `absent`"),
            "module public {command} did not preserve the canonical failure:\n{rendered}"
        );
        assert!(
            !artifact.exists() && !trace.exists() && !contains_generated_artifact(&workspace.root)
        );
    }
}
