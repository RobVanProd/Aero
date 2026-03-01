use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InitResult {
    pub root_dir: PathBuf,
    pub created_files: Vec<PathBuf>,
}

pub fn init_project(target_dir: &Path) -> Result<InitResult, String> {
    let root_dir = target_dir.to_path_buf();
    let src_dir = root_dir.join("src");
    let manifest_path = root_dir.join("aero.toml");
    let main_path = src_dir.join("main.aero");

    if manifest_path.exists() {
        return Err(format!(
            "refusing to overwrite existing manifest: {}",
            manifest_path.display()
        ));
    }
    if main_path.exists() {
        return Err(format!(
            "refusing to overwrite existing source file: {}",
            main_path.display()
        ));
    }

    fs::create_dir_all(&src_dir).map_err(|err| {
        format!(
            "failed to create project directory {}: {}",
            src_dir.display(),
            err
        )
    })?;

    let package_name = infer_package_name(&root_dir);
    let manifest = render_manifest(&package_name);
    let main_source = render_main_source();

    fs::write(&manifest_path, manifest).map_err(|err| {
        format!(
            "failed to write manifest {}: {}",
            manifest_path.display(),
            err
        )
    })?;
    fs::write(&main_path, main_source)
        .map_err(|err| format!("failed to write source {}: {}", main_path.display(), err))?;

    Ok(InitResult {
        root_dir,
        created_files: vec![manifest_path, main_path],
    })
}

fn render_manifest(package_name: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
        package_name
    )
}

fn render_main_source() -> &'static str {
    "fn main() {\n    println!(\"Hello, Aero!\");\n}\n"
}

fn infer_package_name(path: &Path) -> String {
    let raw = path
        .file_name()
        .and_then(OsStr::to_str)
        .map_or_else(|| "aero_project".to_string(), |name| name.to_string());

    let mut name = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch.to_ascii_lowercase());
        } else {
            name.push('_');
        }
    }

    if name.is_empty() {
        name.push_str("aero_project");
    }

    if name.as_bytes()[0].is_ascii_digit() {
        name.insert(0, '_');
    }

    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn infer_package_name_sanitizes_path_component() {
        let path = PathBuf::from("My-Cool App");
        assert_eq!(infer_package_name(&path), "my_cool_app");
    }

    #[test]
    fn init_project_creates_manifest_and_main() {
        let temp_root = unique_temp_dir("aero_init_ok");
        fs::create_dir_all(&temp_root).expect("create temp root");

        let result = init_project(&temp_root).expect("init should succeed");
        assert_eq!(result.root_dir, temp_root);
        assert_eq!(result.created_files.len(), 2);

        let manifest = fs::read_to_string(temp_root.join("aero.toml")).expect("read aero.toml");
        assert!(manifest.contains("[package]"));
        assert!(manifest.contains("name = \"aero_init_ok_"));

        let main_source =
            fs::read_to_string(temp_root.join("src").join("main.aero")).expect("read main.aero");
        assert!(main_source.contains("fn main()"));
        assert!(main_source.contains("println!(\"Hello, Aero!\")"));

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn init_project_refuses_to_overwrite_existing_manifest() {
        let temp_root = unique_temp_dir("aero_init_existing");
        fs::create_dir_all(temp_root.join("src")).expect("create src");
        fs::write(
            temp_root.join("aero.toml"),
            "[package]\nname = \"existing\"\n",
        )
        .expect("write existing manifest");

        let err = init_project(&temp_root).expect_err("init should fail");
        assert!(err.contains("refusing to overwrite existing manifest"));

        let _ = fs::remove_dir_all(temp_root);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", prefix, now))
    }
}
