use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn rust_ci_declares_only_read_only_repository_permissions() {
    let workflow_path = repository_root().join(".github/workflows/rust.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", workflow_path.display()));
    let lines = workflow.lines().collect::<Vec<_>>();
    let jobs_index = lines
        .iter()
        .position(|line| *line == "jobs:")
        .expect("Rust CI must retain its top-level jobs mapping");
    let permissions_index = lines
        .iter()
        .position(|line| *line == "permissions:")
        .expect("Rust CI must declare top-level GITHUB_TOKEN permissions explicitly");

    assert!(
        permissions_index < jobs_index,
        "Rust CI permissions must be top-level workflow authority before jobs"
    );

    let permission_entries = lines[permissions_index + 1..]
        .iter()
        .take_while(|line| line.is_empty() || line.starts_with(' '))
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>();
    assert_eq!(
        permission_entries,
        ["contents: read"],
        "Rust CI must receive only read-only repository contents authority"
    );
}
