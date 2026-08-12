use compiler::{CompilerOptions, LanguageProfile, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

const FIXED_INT_ARRAY_PROGRAM: &str =
    include_str!("../../../examples/fixed_int_array_v0/main.aero");

fn reference_kernel() -> i32 {
    let left: [i32; 8] = [127, 1_073_741_824, -128, 64, -64, 7, -3, 11];
    let right: [i32; 8] = [8, 2, -7, 6, -5, 4, -3, 2];
    left.into_iter()
        .zip(right)
        .fold(2_147_483_000_i32, |accumulator, (left, right)| {
            accumulator.wrapping_add(left.wrapping_mul(right))
        })
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-fixed-int-array-profile-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh fixed-int-array profile workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-fixed-int-array-profile-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn write_program(workspace: &TestWorkspace) -> PathBuf {
    let source = workspace.path("main.aero");
    fs::write(&source, FIXED_INT_ARRAY_PROGRAM).expect("write fixed-int-array source");
    source
}

fn run_cli(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for argument in args {
        command.arg(argument);
    }
    command
        .output()
        .expect("run Aero fixed-int-array profile route")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_check() {
    assert_eq!(reference_kernel(), 2027);
    let workspace = TestWorkspace::new("check-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("check"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success(),
        "exact-i32-array-v0 check is absent or rejected its complete kernel:\n{}",
        combined_output(&output)
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_build() {
    let workspace = TestWorkspace::new("build-red");
    let source = write_program(&workspace);
    let llvm = workspace.path("kernel.ll");
    let output = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &source,
            Path::new("-o"),
            &llvm,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success() && llvm.is_file(),
        "exact-i32-array-v0 build is absent or omitted LLVM:\n{}",
        combined_output(&output)
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_run() {
    let workspace = TestWorkspace::new("run-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(91),
        "exact-i32-array-v0 run is absent or diverged from the oracle:\n{}",
        combined_output(&output)
    );
}

#[test]
fn stable_scalar_profile_still_rejects_the_array_kernel() {
    let error = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::StableScalarV0,
            ..CompilerOptions::default()
        },
    )
    .expect_err("stable-scalar-v0 must retain its frozen array exclusion");
    assert!(error.contains("Language Profile Error: stable-scalar-v0 rejects"));
}

#[test]
fn experimental_profile_retains_the_legacy_double_array_lane() {
    let implicit = compile_program(FIXED_INT_ARRAY_PROGRAM, CompilerOptions::default())
        .expect("experimental array kernel control should compile");
    let explicit = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental array kernel control should compile");

    assert_eq!(implicit, explicit);
    assert!(implicit.contains("[8 x double]"));
    assert!(!implicit.contains("[8 x i32]"));
}
