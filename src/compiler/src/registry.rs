use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub const DEFAULT_REGISTRY_URL: &str = "https://registry.aero/api/v1";
pub const DEFAULT_LOCAL_INDEX_PATH: &str = "registry/index.json";

#[derive(Debug, Clone)]
pub struct RegistryClient {
    pub base_url: String,
}

impl RegistryClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url.unwrap_or(DEFAULT_REGISTRY_URL).to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishPreview {
    pub endpoint: String,
    pub package_name: String,
    pub version: String,
    pub manifest_path: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallPlan {
    pub endpoint: String,
    pub package_name: String,
    pub version: String,
    pub target_dir: String,
    pub dry_run: bool,
}

pub fn search_local_index(index_path: &Path, query: &str) -> Result<Vec<RegistryPackage>, String> {
    let index_text = fs::read_to_string(index_path).map_err(|err| {
        format!(
            "failed to read registry index {}: {}",
            index_path.display(),
            err
        )
    })?;
    let packages = parse_registry_index(&index_text)?;
    let q = query.to_ascii_lowercase();

    let mut matched = packages
        .into_iter()
        .filter(|pkg| {
            pkg.name.to_ascii_lowercase().contains(&q)
                || pkg
                    .description
                    .as_ref()
                    .is_some_and(|d| d.to_ascii_lowercase().contains(&q))
        })
        .collect::<Vec<_>>();

    matched.sort_by(|a, b| {
        b.downloads
            .cmp(&a.downloads)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(matched)
}

pub fn build_publish_preview(
    client: &RegistryClient,
    package_dir: &Path,
) -> Result<PublishPreview, String> {
    let manifest_path = package_dir.join("aero.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(|err| {
        format!(
            "failed to read manifest {}: {}",
            manifest_path.display(),
            err
        )
    })?;

    let package_name = parse_manifest_field(&manifest, "name").ok_or_else(|| {
        format!(
            "manifest {} is missing `name = \"...\"`",
            manifest_path.display()
        )
    })?;
    let version = parse_manifest_field(&manifest, "version").unwrap_or_else(|| "0.1.0".to_string());

    Ok(PublishPreview {
        endpoint: format!("{}/packages/publish", client.base_url),
        package_name,
        version,
        manifest_path: manifest_path.display().to_string(),
        dry_run: true,
    })
}

pub fn build_install_plan(
    client: &RegistryClient,
    package_name: &str,
    version: Option<&str>,
    target_dir: &Path,
) -> InstallPlan {
    InstallPlan {
        endpoint: format!("{}/packages/{}/download", client.base_url, package_name),
        package_name: package_name.to_string(),
        version: version.unwrap_or("latest").to_string(),
        target_dir: target_dir.display().to_string(),
        dry_run: true,
    }
}

fn parse_registry_index(text: &str) -> Result<Vec<RegistryPackage>, String> {
    let json: Value = serde_json::from_str(text)
        .map_err(|err| format!("failed to parse registry index JSON: {}", err))?;

    if json.is_array() {
        return serde_json::from_value(json)
            .map_err(|err| format!("registry index array decode failed: {}", err));
    }

    if let Some(packages) = json.get("packages") {
        return serde_json::from_value(packages.clone())
            .map_err(|err| format!("registry `packages` decode failed: {}", err));
    }

    Err("registry index must be either an array or an object with `packages`".to_string())
}

fn parse_manifest_field(manifest: &str, field_name: &str) -> Option<String> {
    for raw_line in manifest.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        if key.trim() != field_name {
            continue;
        }
        let clean = value.trim().trim_matches('"').to_string();
        if !clean.is_empty() {
            return Some(clean);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}.json", name, nanos))
    }

    #[test]
    fn parse_manifest_field_extracts_string_values() {
        let manifest = r#"
name = "vision"
version = "1.2.3"
"#;
        assert_eq!(
            parse_manifest_field(manifest, "name"),
            Some("vision".to_string())
        );
        assert_eq!(
            parse_manifest_field(manifest, "version"),
            Some("1.2.3".to_string())
        );
        assert_eq!(parse_manifest_field(manifest, "missing"), None);
    }

    #[test]
    fn search_local_index_filters_and_sorts_results() {
        let path = unique_temp_path("aero_registry_index");
        let body = r#"{
  "packages": [
    {"name":"vision-core","version":"0.2.0","description":"vision kernels","downloads":120},
    {"name":"audio-kit","version":"0.1.0","description":"audio dsp","downloads":5},
    {"name":"vision-ops","version":"0.3.1","description":"tensor ops","downloads":280}
  ]
}"#;
        fs::write(&path, body).expect("should write test index");

        let results = search_local_index(&path, "vision").expect("search should succeed");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "vision-ops");
        assert_eq!(results[1].name, "vision-core");

        let _ = fs::remove_file(&path);
    }
}
