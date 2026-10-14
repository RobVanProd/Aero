#[test]
fn embedded_runtime_links_the_diagnostic_channel() {
    let config = BuildConfig {
        language_profile: LanguageProfile::ExactI32ByteDiagnosticsV0,
        ..BuildConfig::default()
    };
    let artifacts = create_run_artifact_paths("h1e-embedded-diagnostic.aero", &config).unwrap();
    let clang = find_llvm_tool("clang").expect("LLVM 22 Clang is required");
    let objects = compile_run_runtimes(&clang, &artifacts, config.language_profile)
        .expect("embedded diagnostic profile runtimes compile");
    let harness = artifacts.directory.join("diagnostic-observer.c");
    fs::write(
        &harness,
        b"#include <stdint.h>\nextern int32_t aero_stderr_write_byte(int32_t);\nint main(void) { return aero_stderr_write_byte(0) == 0 ? 91 : 71; }\n",
    )
    .unwrap();
    let linked = Command::new(&clang)
        .arg(&harness)
        .args(&objects)
        .arg("-o")
        .arg(&artifacts.exe_file)
        .output()
        .expect("link CLI embedded runtime to independent diagnostic observer");
    let outcome = if linked.status.success() {
        Some(Command::new(&artifacts.exe_file).output().unwrap())
    } else {
        None
    };
    fs::remove_dir_all(&artifacts.directory).unwrap();
    assert!(
        linked.status.success(),
        "CLI embedded runtime lacks the diagnostic export: {}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let output = outcome.unwrap();
    assert_eq!(output.status.code(), Some(91));
    assert_eq!(output.stderr, [0]);
    assert!(output.stdout.is_empty());
}

#[test]
fn diagnostic_runtime_is_linked_in_both_native_paths() {
    let clang = find_llvm_tool("clang").expect("LLVM 22 Clang is required");
    let llc = find_llvm_tool("llc")
        .or_else(|| find_llvm_tool("llc-22"))
        .expect("LLVM 22 llc or llc-22 is required");
    for lowering in [Some(llc.as_str()), None] {
        let config = BuildConfig {
            language_profile: LanguageProfile::ExactI32ByteDiagnosticsV0,
            ..BuildConfig::default()
        };
        let artifacts = create_run_artifact_paths("h1e-link-path.aero", &config).unwrap();
        let objects = compile_run_runtimes(&clang, &artifacts, config.language_profile).unwrap();
        fs::write(&artifacts.ll_file, b"declare i32 @aero_stderr_write_byte(i32)\ndefine i32 @main() {\nentry:\n  %status = call i32 @aero_stderr_write_byte(i32 255)\n  %ok = icmp eq i32 %status, 0\n  %code = select i1 %ok, i32 91, i32 71\n  ret i32 %code\n}\n").unwrap();
        link_cpu_program(&clang, lowering, &artifacts, &objects).unwrap();
        let output = Command::new(&artifacts.exe_file).output().unwrap();
        fs::remove_dir_all(&artifacts.directory).unwrap();
        assert_eq!(output.status.code(), Some(91), "{output:?}");
        assert_eq!(output.stderr, [255]);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn earlier_profiles_compile_only_the_existing_runtime() {
    let clang = find_llvm_tool("clang").expect("LLVM 22 Clang is required");
    for profile in [
        LanguageProfile::Experimental,
        LanguageProfile::ExactI32ByteIoV0,
    ] {
        let config = BuildConfig {
            language_profile: profile,
            ..BuildConfig::default()
        };
        let artifacts = create_run_artifact_paths("h1e-prior-profile.aero", &config).unwrap();
        let objects = compile_run_runtimes(&clang, &artifacts, profile).unwrap();
        assert_eq!(objects, vec![artifacts.runtime_obj_file.clone()]);
        assert!(!artifacts.diagnostic_source_file.exists());
        assert!(!artifacts.diagnostic_obj_file.exists());
        fs::remove_dir_all(&artifacts.directory).unwrap();
    }
}
