use std::fs;
use std::path::PathBuf;

const LLVM_VERSION: &str = "22.1.8";
const LLVM_ARCHIVE: &str = "clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz";
const LLVM_ARCHIVE_SHA256: &str =
    "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234";
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
    let encoded_archive = LLVM_ARCHIVE.replace('+', "%2B");
    let official_url = format!(
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-{LLVM_VERSION}/{encoded_archive}"
    );
    let windows_target_header = format!("target triple = \"{WINDOWS_TRIPLE}\"");
    let required_once = [
        "windows-native:",
        "name: Windows LLVM 22 native system gate",
        "runs-on: windows-latest",
        "timeout-minutes: 30",
        official_url.as_str(),
        LLVM_ARCHIVE_SHA256,
        "Get-FileHash -LiteralPath $archive -Algorithm SHA256",
        "Extract pinned LLVM 22.1.8 for Windows",
        "$llvmRoot = Join-Path $env:RUNNER_TEMP \"llvm-22.1.8-archive\"",
        "tar.exe -xf $archive -C $llvmRoot --strip-components=1",
        "official LLVM archive did not provide $toolPath",
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
        windows_target_header.as_str(),
        WINDOWS_DATA_LAYOUT,
        "-passes=verify -disable-output $llvmPath",
        "-verify-machineinstrs $llvmPath",
        "-filetype=obj $llvmPath -o $objectPath",
        "Windows public run passed with exit code 227",
        "Windows manual native execution passed with exit code 227",
        "Test Windows primitive const system specimen",
        "Windows primitive const public run passed with exit code 81",
        "Windows primitive const manual native execution passed with exit code 81",
        "Test Windows mutable enum reference system specimen",
        "Windows mutable enum reference public run passed with exit code 83",
        "Windows mutable enum reference manual native execution passed with exit code 83",
        "Test Windows immutable enum reference system specimen",
        "Windows immutable enum reference public run passed with exit code 84",
        "Windows immutable enum reference manual native execution passed with exit code 84",
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

    for rejected_installer_anchor in [
        "LLVM-22.1.8-win64.exe",
        "Start-Process -FilePath $installer",
        "/D=$llvmRoot",
    ] {
        if workflow.contains(rejected_installer_anchor) {
            failures.push(format!(
                "rejected Windows installer anchor {rejected_installer_anchor:?} must remain absent"
            ));
        }
    }

    let expected_exit_reset_count = workflow.matches("$global:LASTEXITCODE = 0").count();
    if expected_exit_reset_count != 7 {
        failures.push(format!(
            "the seven expected-nonzero Windows control groups must reset LASTEXITCODE, found {expected_exit_reset_count} resets"
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-078 Windows native workflow contract failures:\n{}",
        failures.join("\n")
    );
}
