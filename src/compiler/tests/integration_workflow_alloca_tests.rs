//! Execute the platform workflow oracle against real CORE-093 output and mutations.
use compiler::{CompilerOptions, LanguageProfile, compile_program};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove this test's unique workspace");
    }
}

fn workflow_pattern(workflow: &str, prefix: &str) -> String {
    let patterns: Vec<_> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .collect();
    assert_eq!(patterns.len(), 1, "one platform scorer oracle");
    patterns[0]
        .strip_suffix(prefix.chars().last().unwrap())
        .expect("quoted workflow regex")
        .to_string()
}

#[test]
fn inference_workflow_oracles_follow_hoisted_storage_without_losing_dataflow() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workflow = fs::read_to_string(root.join(".github/workflows/rust.yml")).unwrap();
    let source =
        fs::read_to_string(root.join("examples/fixed_int_array_v0/relu_argmax_inference.aero"))
            .unwrap();
    let llvm = compile_program(
        &source,
        CompilerOptions {
            language_profile: LanguageProfile::ExactI32ArrayV0,
            ..CompilerOptions::default()
        },
    )
    .expect("compile accepted inference source");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = Workspace(std::env::temp_dir().join(format!(
        "aero-integration-inference-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&workspace.0).unwrap();
    let mut cases = Vec::new();
    for (offset, width) in [(3, 3), (6, 6), (12, 2), (14, 4), (18, 2)] {
        let linux = workflow_pattern(&workflow, "inference_payload_offset_pattern=\"")
            .replace("${inference_payload_width}", &width.to_string())
            .replace("${inference_payload_offset}", &offset.to_string());
        let windows = workflow_pattern(&workflow, "$inferencePayloadOffsetPattern = \"")
            .replace("$inferencePayloadWidth", &width.to_string())
            .replace("$inferencePayloadOffset", &offset.to_string());
        cases.push((format!("payload-{offset}"), linux, windows, "infer_record"));
    }
    for (linux_name, windows_name, function) in [
        ("header", "Header", "infer_record"),
        ("relu", "Relu", "infer_record"),
        ("argmax", "Argmax", "infer_record"),
        ("matvec_2x3", "Matvec2x3", "matvec_2x3"),
        ("matvec_2x2", "Matvec2x2", "matvec_2x2"),
    ] {
        cases.push((
            linux_name.to_string(),
            workflow_pattern(&workflow, &format!("inference_{linux_name}_pattern='")),
            workflow_pattern(&workflow, &format!("$inference{windows_name}Pattern = '")),
            function,
        ));
    }
    for (name, linux, windows, function_name) in cases {
        assert_eq!(linux, windows, "{name}: platform expressions");
        let start = llvm
            .lines()
            .find(|line| {
                line.starts_with("define ") && line.contains(&format!("@{function_name}("))
            })
            .unwrap();
        let offset = llvm.find(start).unwrap();
        let end = offset + llvm[offset..].find("\n}").unwrap() + 2;
        let body = &llvm[offset..end];
        assert_eq!(
            match_count(&workspace, &linux, body),
            1,
            "{name}: real emitted body"
        );
        let no_allocation = body.replace(" = alloca ", " = missing_alloca ");
        assert_eq!(
            match_count(&workspace, &linux, &no_allocation),
            0,
            "{name}: allocation required"
        );
        let non_entry = body.replace("\nentry:\n", "\nnot_entry:\n");
        assert_ne!(non_entry, body);
        assert_eq!(
            match_count(&workspace, &linux, &non_entry),
            0,
            "{name}: entry required"
        );
        let broken_stores = body.replace("  store ", "  broken_store ");
        assert_eq!(
            match_count(&workspace, &linux, &broken_stores),
            0,
            "{name}: dataflow stores required"
        );
    }
}

fn match_count(workspace: &Workspace, pattern: &str, input: &str) -> usize {
    let pattern_path = workspace.0.join("pattern.txt");
    let input_path = workspace.0.join("module.ll");
    fs::write(&pattern_path, pattern).unwrap();
    fs::write(&input_path, input).unwrap();
    if cfg!(windows) {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command"])
            .arg("[Console]::Write([regex]::Matches([IO.File]::ReadAllText($env:AERO_WORKFLOW_INPUT), [IO.File]::ReadAllText($env:AERO_WORKFLOW_PATTERN)).Count)")
            .env("AERO_WORKFLOW_PATTERN", pattern_path)
            .env("AERO_WORKFLOW_INPUT", input_path)
            .output()
            .expect("execute Windows workflow regex engine");
        assert!(output.status.success(), "{:?}", output);
        String::from_utf8(output.stdout)
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    } else {
        let output = Command::new("grep")
            .args(["-Pzo", "-f"])
            .arg(pattern_path)
            .arg(input_path)
            .output()
            .expect("execute Linux workflow regex engine");
        assert!(matches!(output.status.code(), Some(0 | 1)), "{:?}", output);
        output.stdout.iter().filter(|byte| **byte == 0).count()
    }
}

#[test]
fn platform_scorer_oracles_require_entry_allocation_and_unbroken_dependencies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workflow = fs::read_to_string(root.join(".github/workflows/rust.yml")).unwrap();
    let linux = workflow_pattern(&workflow, "scorer_affine_pattern='");
    let windows = workflow_pattern(&workflow, "$scorerAffinePattern = '");
    assert_eq!(linux, windows, "platforms must enforce the identical chain");

    let source =
        fs::read_to_string(root.join("examples/fixed_int_array_v0/tensor_record_scoring.aero"))
            .unwrap();
    let llvm = compile_program(
        &source,
        CompilerOptions {
            language_profile: LanguageProfile::ExactI32ArrayV0,
            ..CompilerOptions::default()
        },
    )
    .expect("compile real accepted scorer");
    let start = llvm.find("define i32 @affine_2_to_1(").unwrap();
    let end = start + llvm[start..].find("\n}").unwrap() + 2;
    let function = &llvm[start..end];
    let lines: Vec<_> = function.lines().collect();
    let bias_store = lines
        .iter()
        .position(|line| line.starts_with("  store i32 %aero.arg.bias,"))
        .unwrap();
    let initializer = lines[bias_store + 2];
    assert!(initializer.starts_with("  store i32 %reg"));
    let slot = initializer
        .split("i32* ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap();
    let allocation = format!("  {slot} = alloca i32, align 4\n");
    assert_eq!(function.matches(&allocation).count(), 1);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = Workspace(std::env::temp_dir().join(format!(
        "aero-integration-scorer-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&workspace.0).unwrap();
    assert_eq!(
        match_count(&workspace, &linux, function),
        1,
        "real hoisted scorer must match"
    );

    let missing = function.replacen(&allocation, "", 1);
    assert_eq!(
        match_count(&workspace, &linux, &missing),
        0,
        "missing allocation"
    );
    let body_label = lines
        .iter()
        .find(|line| line.starts_with("while_body_"))
        .unwrap();
    let moved = missing.replacen(
        &format!("{body_label}\n"),
        &format!("{body_label}\n{allocation}"),
        1,
    );
    assert_ne!(moved, missing);
    assert_eq!(
        match_count(&workspace, &linux, &moved),
        0,
        "non-entry allocation"
    );
    let wrong_slot = function.replacen(initializer, &initializer.replace(slot, "%ptr999999"), 1);
    assert_eq!(
        match_count(&workspace, &linux, &wrong_slot),
        0,
        "initializer identity"
    );
    let wrong_operation = function.replace(" = add i32 ", " = sub i32 ");
    assert_ne!(wrong_operation, function);
    assert_eq!(
        match_count(&workspace, &linux, &wrong_operation),
        0,
        "arithmetic dependency"
    );
}
