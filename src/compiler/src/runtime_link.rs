use std::fs;
use std::path::Path;
use std::process::Command;

const PRODUCTION_RUNTIME_SOURCE: &[u8] = include_bytes!("../runtime/aero_runtime.c");

pub(crate) fn compile_production_runtime(
    clang_bin: &str,
    source_file: &Path,
    object_file: &Path,
) -> Result<(), String> {
    fs::write(source_file, PRODUCTION_RUNTIME_SOURCE).map_err(|error| {
        format!(
            "Error writing embedded Aero runtime source to {}: {error}",
            source_file.display()
        )
    })?;

    let output = Command::new(clang_bin)
        .arg("-std=c11")
        .arg("-O2")
        .args(["-Wall", "-Wextra", "-Werror", "-c"])
        .arg(source_file)
        .arg("-o")
        .arg(object_file)
        .output()
        .map_err(|error| {
            format!("Error executing clang for embedded Aero runtime ({clang_bin}): {error}")
        })?;

    if !output.status.success() {
        return Err(format!(
            "Error compiling embedded Aero runtime: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !object_file.is_file() {
        return Err(format!(
            "embedded Aero runtime compilation reported success without producing {}",
            object_file.display()
        ));
    }

    Ok(())
}
