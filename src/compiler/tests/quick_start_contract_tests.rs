use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
            "aero-quick-start-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create Quick Start test workspace");
        Self { root }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CodeBlock {
    language: String,
    body: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root")
}

fn unique_h2_section<'a>(document: &'a str, heading_suffix: &str) -> Result<&'a str, String> {
    let mut headings = Vec::new();
    let mut offset = 0;
    for line in document.split_inclusive('\n') {
        let heading = line.trim_end_matches(['\r', '\n']);
        if let Some(text) = heading.strip_prefix("## ") {
            headings.push((offset, text.trim()));
        }
        offset += line.len();
    }

    let matches = headings
        .iter()
        .enumerate()
        .filter(|(_, (_, text))| text.ends_with(heading_suffix))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "expected exactly one level-two heading ending in {heading_suffix:?}, found {}",
            matches.len()
        ));
    }

    let heading_index = matches[0];
    let start = headings[heading_index].0;
    let end = headings
        .get(heading_index + 1)
        .map_or(document.len(), |(next, _)| *next);
    Ok(&document[start..end])
}

fn fenced_code_blocks(section: &str) -> Result<Vec<CodeBlock>, String> {
    let mut blocks = Vec::new();
    let mut lines = section.lines().enumerate();
    while let Some((line_number, line)) = lines.next() {
        let Some(language) = line.strip_prefix("```") else {
            continue;
        };
        if language.contains('`') {
            return Err(format!(
                "invalid code fence at section line {}",
                line_number + 1
            ));
        }

        let mut body = Vec::new();
        let mut closed = false;
        for (_, body_line) in lines.by_ref() {
            if body_line == "```" {
                closed = true;
                break;
            }
            body.push(body_line);
        }
        if !closed {
            return Err(format!(
                "unclosed {language:?} code fence at section line {}",
                line_number + 1
            ));
        }
        blocks.push(CodeBlock {
            language: language.to_string(),
            body: body.join("\n"),
        });
    }
    Ok(blocks)
}

fn nonempty_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn unique_yaml_mapping_section<'a>(
    document: &'a str,
    key: &str,
    indent: usize,
) -> Result<&'a str, String> {
    let mut matches = Vec::new();
    let mut offset = 0;
    for line in document.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        let line_indent = content.len() - content.trim_start().len();
        if line_indent == indent && content.trim() == key {
            matches.push(offset);
        }
        offset += line.len();
    }
    if matches.len() != 1 {
        return Err(format!(
            "expected exactly one YAML mapping key {key:?} at indent {indent}, found {}",
            matches.len()
        ));
    }

    let start = matches[0];
    let remainder = &document[start..];
    let mut end = remainder.len();
    let mut relative = 0;
    for (line_index, line) in remainder.split_inclusive('\n').enumerate() {
        if line_index > 0 {
            let content = line.trim_end_matches(['\r', '\n']);
            if !content.trim().is_empty() && !content.trim_start().starts_with('#') {
                let line_indent = content.len() - content.trim_start().len();
                if line_indent <= indent {
                    end = relative;
                    break;
                }
            }
        }
        relative += line.len();
    }
    Ok(&remainder[..end])
}

fn unique_yaml_step<'a>(
    workflow: &'a str,
    step_name: &str,
    expected_indent: usize,
) -> Result<&'a str, String> {
    let marker = format!("- name: {step_name}");
    let mut matches = Vec::new();
    let mut offset = 0;
    for line in workflow.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        let indent = content.len() - content.trim_start().len();
        if indent == expected_indent && content.trim_start() == marker {
            matches.push((offset, indent));
        }
        offset += line.len();
    }
    if matches.len() != 1 {
        return Err(format!(
            "expected exactly one workflow step {step_name:?}, found {}",
            matches.len()
        ));
    }

    let (start, indent) = matches[0];
    let remainder = &workflow[start..];
    let mut end = remainder.len();
    let mut relative = 0;
    for (line_index, line) in remainder.split_inclusive('\n').enumerate() {
        if line_index > 0 {
            let content = line.trim_end_matches(['\r', '\n']);
            let line_indent = content.len() - content.trim_start().len();
            if line_indent == indent && content.trim_start().starts_with("- ") {
                end = relative;
                break;
            }
        }
        relative += line.len();
    }
    Ok(&remainder[..end])
}

fn yaml_mapping_lines_at_indent(
    document: &str,
    expected_indent: usize,
) -> Result<Vec<&str>, String> {
    let mut mappings = Vec::new();
    for line in document.lines() {
        let content = line.trim_end();
        let trimmed = content.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("- ") {
            continue;
        }
        let indent = content.len() - trimmed.len();
        if indent != expected_indent {
            continue;
        }
        if !trimmed.contains(':') {
            return Err(format!(
                "YAML content at indent {expected_indent} has no mapping key: {line:?}"
            ));
        }
        mappings.push(trimmed);
    }
    Ok(mappings)
}

fn yaml_step_keys(step: &str) -> Result<Vec<&str>, String> {
    let mut lines = step.lines();
    let first = lines.next().ok_or("empty workflow step")?;
    let step_indent = first.len() - first.trim_start().len();
    let child_indent = step_indent + 2;
    let mut keys = Vec::new();
    for line in lines {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent != child_indent {
            continue;
        }
        let key = line
            .trim()
            .split_once(':')
            .map(|(key, _)| key)
            .ok_or_else(|| format!("workflow step child has no mapping key: {line:?}"))?;
        keys.push(key);
    }
    Ok(keys)
}

fn yaml_run_commands(step: &str) -> Result<Vec<&str>, String> {
    let lines = step.lines().collect::<Vec<_>>();
    let run_indices = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == "run: |")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if run_indices.len() != 1 {
        return Err(format!(
            "expected exactly one run block, found {}",
            run_indices.len()
        ));
    }

    let run_index = run_indices[0];
    let run_indent = lines[run_index].len() - lines[run_index].trim_start().len();
    let command_indent = run_indent + 2;
    let mut commands = Vec::new();
    for line in &lines[run_index + 1..] {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent < command_indent {
            break;
        }
        commands.push(line[command_indent..].trim_end());
    }
    Ok(commands)
}

fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn readme_quick_start_has_exact_executable_blocks_and_links() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("read README");
    let quick_start = unique_h2_section(&readme, "Quick Start").unwrap_or_else(|error| {
        panic!("README Quick Start section error: {error}");
    });
    let blocks = fenced_code_blocks(quick_start).unwrap_or_else(|error| {
        panic!("README Quick Start fence error: {error}");
    });

    assert_eq!(
        blocks
            .iter()
            .map(|block| block.language.as_str())
            .collect::<Vec<_>>(),
        vec!["bash", "aero"],
        "README Quick Start must contain exactly one Bash block followed by one Aero block"
    );
    assert_eq!(
        nonempty_lines(&blocks[0].body),
        vec![
            "git clone https://github.com/RobVanProd/Aero.git",
            "cd Aero",
            "cargo build --release --manifest-path src/compiler/Cargo.toml",
            "export PATH=\"$PWD/src/compiler/target/release:$PATH\"",
            "aero --version",
            "aero init my_app",
            "cd my_app",
            "aero check src/main.aero",
            "aero run src/main.aero",
        ],
        "README Quick Start Bash commands must be exact and ordered"
    );
    assert_eq!(
        blocks[1].body.trim_end(),
        "fn main() {\n    println!(\"Hello, Aero!\");\n}",
        "README Quick Start Aero block must equal the generated scaffold"
    );
    for link in [
        "[Windows PowerShell instructions](BUILD.md)",
        "[backend capability status](BACKEND_STATUS.md)",
    ] {
        assert!(
            quick_start.contains(link),
            "README Quick Start omitted exact link {link:?}"
        );
    }
    for forbidden in [
        "aeronum",
        "aeronn",
        "Transformer",
        "distributed",
        "ROCm",
        "CUDA",
        "--target",
        "--backend",
        "graph-opt",
        "quantize",
        "registry",
        "GGUF",
        "benchmark",
        "aero lsp",
        "aero conformance",
        "aero doc",
        "aero profile",
    ] {
        assert!(
            !quick_start.contains(forbidden),
            "README Quick Start retained non-contract surface {forbidden:?}"
        );
    }
}

#[test]
fn build_guide_has_exact_windows_powershell_block() {
    let build = fs::read_to_string(repo_root().join("BUILD.md")).expect("read BUILD guide");
    let windows = unique_h2_section(&build, "Windows PowerShell").unwrap_or_else(|error| {
        panic!("BUILD Windows PowerShell section error: {error}");
    });
    let blocks = fenced_code_blocks(windows).unwrap_or_else(|error| {
        panic!("BUILD Windows PowerShell fence error: {error}");
    });

    assert_eq!(
        blocks
            .iter()
            .map(|block| block.language.as_str())
            .collect::<Vec<_>>(),
        vec!["powershell"],
        "BUILD Windows PowerShell section must contain exactly one PowerShell block"
    );
    assert_eq!(
        nonempty_lines(&blocks[0].body),
        vec![
            "cargo build --release --manifest-path src/compiler/Cargo.toml",
            "$env:PATH = \"$PWD\\src\\compiler\\target\\release;$env:PATH\"",
            "aero.exe --version",
        ],
        "BUILD Windows PowerShell commands must be exact and ordered"
    );
    assert!(
        windows.contains(r"`src\compiler\target\release\aero.exe`"),
        "BUILD Windows section omitted exact compiler executable location"
    );
}

#[test]
fn rust_ci_has_exact_stable_quick_start_execution_step() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/rust.yml"))
        .expect("read Rust CI workflow");
    let jobs =
        unique_yaml_mapping_section(&workflow, "jobs:", 0).expect("resolve Rust CI jobs mapping");
    let build_job =
        unique_yaml_mapping_section(jobs, "build:", 2).expect("resolve Rust CI build job");
    assert_eq!(
        yaml_mapping_lines_at_indent(build_job, 4)
            .expect("parse immediate Rust CI build job mappings"),
        vec!["runs-on: ubuntu-latest", "strategy:", "steps:"],
        "Rust CI build job must remain a gating Ubuntu matrix job without job-level skips or failure suppression"
    );
    let strategy = unique_yaml_mapping_section(build_job, "strategy:", 4)
        .expect("resolve Rust CI build strategy");
    assert_eq!(
        yaml_mapping_lines_at_indent(strategy, 6)
            .expect("parse immediate Rust CI build strategy mappings"),
        vec!["matrix:"],
        "Rust CI build strategy must contain exactly one matrix mapping"
    );
    let matrix =
        unique_yaml_mapping_section(strategy, "matrix:", 6).expect("resolve Rust CI build matrix");
    assert_eq!(
        yaml_mapping_lines_at_indent(matrix, 8)
            .expect("parse immediate Rust CI build matrix mappings"),
        vec!["rust: [stable, nightly]"],
        "Rust CI build matrix must retain exactly the stable and nightly Rust entries"
    );
    unique_yaml_step(build_job, "Install pinned LLVM 22 + Clang 22", 4)
        .expect("resolve pinned LLVM/Clang install step in build job");
    let install_offset = build_job
        .find("- name: Install pinned LLVM 22 + Clang 22")
        .expect("build job omitted pinned LLVM/Clang install step");
    let quick_start_offset = build_job
        .find("- name: Verify documented Quick Start")
        .expect("build job omitted documented Quick Start step");
    assert!(
        install_offset < quick_start_offset,
        "Quick Start step must run after pinned LLVM/Clang installation"
    );

    let step = unique_yaml_step(build_job, "Verify documented Quick Start", 4)
        .unwrap_or_else(|error| panic!("Rust CI Quick Start step error: {error}"));
    assert_eq!(
        yaml_step_keys(step).expect("parse Quick Start workflow step keys"),
        vec!["if", "shell", "run"],
        "Quick Start workflow step must have exactly one gating if, shell, and run key"
    );
    assert!(
        step.lines()
            .any(|line| line.trim() == "if: matrix.rust == 'stable'"),
        "Quick Start workflow step must be stable-only"
    );
    assert!(
        step.lines().any(|line| line.trim() == "shell: bash"),
        "Quick Start workflow step must use Bash"
    );
    assert_eq!(
        yaml_run_commands(step).expect("parse Quick Start workflow run block"),
        vec![
            "export PATH=\"/usr/lib/llvm-22/bin:${PATH}\"",
            "test \"$(command -v clang)\" = \"/usr/lib/llvm-22/bin/clang\"",
            "test \"$(command -v llc)\" = \"/usr/lib/llvm-22/bin/llc\"",
            "clang --version | grep -Eq 'clang version 22\\.'",
            "llc --version | grep -Eq 'LLVM version 22\\.'",
            "test \"${AERO_LLVM_OPT}\" = \"/usr/bin/opt-22\"",
            "test \"${AERO_LLVM_AS}\" = \"/usr/bin/llvm-as-22\"",
            "cargo build --release --manifest-path src/compiler/Cargo.toml",
            "export PATH=\"${GITHUB_WORKSPACE}/src/compiler/target/release:${PATH}\"",
            "aero --version",
            "quick_start_root=\"$(mktemp -d)\"",
            "cd \"${quick_start_root}\"",
            "aero init my_app",
            "cd my_app",
            "aero check src/main.aero",
            "set +e",
            "run_output=\"$(aero run src/main.aero)\"",
            "run_status=$?",
            "set -e",
            "printf '%s\\n' \"${run_output}\"",
            "test \"${run_status}\" -eq 0",
            "output_count=\"$(printf '%s\\n' \"${run_output}\" | grep -c '^Output: Hello, Aero!$' || true)\"",
            "test \"${output_count}\" -eq 1",
        ],
        "Rust CI Quick Start commands must be exact and ordered"
    );
}

#[test]
fn generated_project_initializes_by_relative_name_and_checks_successfully() {
    let workspace = TestWorkspace::new("generated-project");
    let project = workspace.root.join("my_app");
    let init = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(["init", "my_app"])
        .output()
        .expect("run relative aero init my_app");
    let (init_stdout, init_stderr) = output_text(&init);
    assert!(
        init.status.success(),
        "aero init failed: stdout={init_stdout:?}; stderr={init_stderr:?}"
    );
    assert!(
        init_stderr.is_empty(),
        "aero init wrote unexpected stderr: {init_stderr:?}"
    );

    let manifest = project.join("aero.toml");
    let source = project.join("src/main.aero");
    assert!(manifest.is_file(), "aero init omitted aero.toml");
    assert!(source.is_file(), "aero init omitted src/main.aero");
    assert_eq!(
        fs::read_to_string(&source).expect("read generated source"),
        "fn main() {\n    println!(\"Hello, Aero!\");\n}\n"
    );

    let check = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&project)
        .args(["check", "src/main.aero"])
        .output()
        .expect("run aero check");
    let (check_stdout, check_stderr) = output_text(&check);
    assert!(
        check.status.success(),
        "aero check failed: stdout={check_stdout:?}; stderr={check_stderr:?}"
    );
    assert!(
        check_stderr.is_empty(),
        "aero check wrote unexpected stderr: {check_stderr:?}"
    );
    assert!(
        check_stdout.contains("Semantic analysis completed successfully"),
        "aero check omitted success evidence: {check_stdout:?}"
    );
}

#[test]
fn founding_ai_ml_direction_remains_conceptual_and_linked() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("README.md")).expect("read README");
    let vision = fs::read_to_string(root.join("LANGUAGE_VISION.md")).expect("read vision");
    let alignment =
        fs::read_to_string(root.join("FRAMEWORK_ALIGNMENT.md")).expect("read alignment");

    assert!(
        readme.contains("[Framework alignment](FRAMEWORK_ALIGNMENT.md)"),
        "README must retain the founding-framework alignment link"
    );
    for fragment in [
        "The lead adoption wedge is AI/ML and high-performance data infrastructure",
        "These are design goals, not claims about the current implementation.",
    ] {
        assert!(
            vision.contains(fragment),
            "LANGUAGE_VISION.md omitted preservation fragment {fragment:?}"
        );
    }
    for fragment in [
        "The first flagship must be an **Aero-native, reproducible infrastructure",
        "not Aero execution and cannot satisfy the flagship or backend gates",
    ] {
        assert!(
            alignment.contains(fragment),
            "FRAMEWORK_ALIGNMENT.md omitted preservation fragment {fragment:?}"
        );
    }
}
