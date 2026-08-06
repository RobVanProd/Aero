use std::fs;
use std::path::PathBuf;

const LLVM_VERSION: &str = "22.1.8";
const LLVM_INSTALLER: &str = "LLVM-22.1.8-win64.exe";
const LLVM_INSTALLER_SHA256: &str =
    "16e5709785fef73c854646241c4a92c5cd574318d1b33c63330dd7721903e55c";
const WINDOWS_TRIPLE: &str = "x86_64-pc-windows-msvc";
const WINDOWS_DATA_LAYOUT: &str =
    "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("compiler manifest must be nested under the repository root")
        .to_path_buf()
}

#[test]
fn pinned_windows_llvm_native_system_gate_is_complete_and_unique() {
    let workflow = fs::read_to_string(repository_root().join(".github/workflows/rust.yml"))
        .expect("read Rust workflow");
    let official_url = format!(
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-{LLVM_VERSION}/{LLVM_INSTALLER}"
    );
    let required_once = [
        "windows-native:",
        "name: Windows LLVM 22 native system gate",
        "runs-on: windows-latest",
        "timeout-minutes: 30",
        official_url.as_str(),
        LLVM_INSTALLER_SHA256,
        "Get-FileHash -LiteralPath $installer -Algorithm SHA256",
        "Install pinned LLVM 22.1.8 for Windows",
        "$llvmRoot = Join-Path $env:ProgramW6432 \"LLVM\"",
        "official LLVM installer did not populate $llvmRoot\\bin",
        "-ArgumentList @(\"/S\")",
        "Assert pinned Windows LLVM and Clang versions",
        "AERO_REQUIRE_LLVM_VERIFIER: \"true\"",
        "AERO_LLVM_OPT=$llvmBin\\opt.exe",
        "AERO_LLVM_AS=$llvmBin\\llvm-as.exe",
        "Confirm Windows LLVM 22 rejects known-invalid IR",
        "Windows opt rejected the known-invalid LLVM IR fixture as required",
        "Test invalid Windows build artifact hygiene",
        "Test Windows balanced loop enum ownership system specimen",
        "cargo run --locked --manifest-path src/compiler/Cargo.toml -- check examples/balanced_loop_enum_ownership/main.aero",
        "cargo run --locked --manifest-path src/compiler/Cargo.toml -- run examples/balanced_loop_enum_ownership/main.aero",
        WINDOWS_TRIPLE,
        WINDOWS_DATA_LAYOUT,
        "-passes=verify -disable-output $llvmPath",
        "-verify-machineinstrs $llvmPath",
        "-filetype=obj $llvmPath -o $objectPath",
        "Windows public run passed with exit code 227",
        "Windows manual native execution passed with exit code 227",
    ];

    let mut failures = Vec::new();
    for anchor in required_once {
        let count = workflow.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "expected exactly one Windows system-gate anchor {anchor:?}, found {count}"
            ));
        }
    }

    for preserved_linux_anchor in [
        "owned unit-enum match example passed with exit code 149",
        "unified CopyData Match result example passed with exit code 223",
        "balanced loop enum ownership example passed with exit code 227",
    ] {
        let count = workflow.matches(preserved_linux_anchor).count();
        if count != 1 {
            failures.push(format!(
                "Linux preservation anchor {preserved_linux_anchor:?} must remain unique, found {count}"
            ));
        }
    }

    for rejected_installer_anchor in ["/D=$llvmRoot"] {
        if workflow.contains(rejected_installer_anchor) {
            failures.push(format!(
                "rejected Windows installer anchor {rejected_installer_anchor:?} must remain absent"
            ));
        }
    }

    let expected_exit_reset_count = workflow.matches("$global:LASTEXITCODE = 0").count();
    if expected_exit_reset_count != 3 {
        failures.push(format!(
            "the three expected-nonzero Windows controls must reset LASTEXITCODE, found {expected_exit_reset_count} resets"
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-078 Windows native workflow contract failures:\n{}",
        failures.join("\n")
    );
}
