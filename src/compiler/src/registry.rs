use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_REGISTRY_URL: &str = "https://registry.aero/api/v1";
pub const DEFAULT_LOCAL_INDEX_PATH: &str = "registry/index.json";
pub const DEFAULT_TOKEN_ENV: &str = "AERO_REGISTRY_TOKEN";

#[derive(Debug, Clone)]
pub struct RegistryClient {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl RegistryClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url.unwrap_or(DEFAULT_REGISTRY_URL).to_string(),
            timeout_secs: 20,
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
    pub manifest_sha256: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishResult {
    pub endpoint: String,
    pub package_name: String,
    pub version: String,
    pub uploaded_files: usize,
    pub manifest_sha256: String,
    pub accepted: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallPlan {
    pub endpoint: String,
    pub package_name: String,
    pub version: String,
    pub target_dir: String,
    pub trust: PackageTrustPolicy,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallReceipt {
    pub package_name: String,
    pub version: String,
    pub target_file: String,
    pub expected_sha256: Option<String>,
    pub actual_sha256: String,
    pub trusted: bool,
    pub downloaded_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTrustPolicy {
    pub require_sha256: bool,
    pub allow_untrusted: bool,
}

impl Default for PackageTrustPolicy {
    fn default() -> Self {
        Self {
            require_sha256: true,
            allow_untrusted: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAuth {
    pub token: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResolveMetadata {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub sha256: Option<String>,
}

pub fn resolve_registry_auth(
    explicit_token: Option<&str>,
    explicit_token_file: Option<&Path>,
) -> Result<Option<RegistryAuth>, String> {
    if let Some(token) = explicit_token {
        let clean = token.trim();
        if clean.is_empty() {
            return Err("registry token is empty".to_string());
        }
        return Ok(Some(RegistryAuth {
            token: clean.to_string(),
            source: "cli".to_string(),
        }));
    }

    if let Ok(token) = env::var(DEFAULT_TOKEN_ENV) {
        let clean = token.trim();
        if !clean.is_empty() {
            return Ok(Some(RegistryAuth {
                token: clean.to_string(),
                source: format!("env:{}", DEFAULT_TOKEN_ENV),
            }));
        }
    }

    let token_file = if let Some(path) = explicit_token_file {
        Some(path.to_path_buf())
    } else {
        default_token_file_path()
    };

    if let Some(path) = token_file {
        if path.exists() {
            let token = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read token file {}: {}", path.display(), err))?;
            let clean = token.trim();
            if clean.is_empty() {
                return Err(format!("token file {} is empty", path.display()));
            }
            return Ok(Some(RegistryAuth {
                token: clean.to_string(),
                source: format!("file:{}", path.display()),
            }));
        }
    }

    Ok(None)
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

pub fn search_live_registry(
    client: &RegistryClient,
    query: &str,
    auth: Option<&RegistryAuth>,
) -> Result<Vec<RegistryPackage>, String> {
    let endpoint = format!(
        "{}/packages/search?q={}",
        client.base_url,
        url_encode(query)
    );
    let body = run_curl(
        "GET",
        &endpoint,
        auth.map(|a| a.token.as_str()),
        None,
        client.timeout_secs,
    )?;
    let mut packages = parse_registry_index(&body)?;
    packages.sort_by(|a, b| {
        b.downloads
            .cmp(&a.downloads)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(packages)
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
    let manifest_sha256 = sha256_hex(manifest.as_bytes());

    Ok(PublishPreview {
        endpoint: format!("{}/packages/publish", client.base_url),
        package_name,
        version,
        manifest_path: manifest_path.display().to_string(),
        manifest_sha256,
        dry_run: true,
    })
}

pub fn publish_live(
    client: &RegistryClient,
    package_dir: &Path,
    auth: Option<&RegistryAuth>,
    dry_run: bool,
) -> Result<PublishResult, String> {
    let preview = build_publish_preview(client, package_dir)?;
    let files = collect_package_files(package_dir)?;

    if dry_run {
        return Ok(PublishResult {
            endpoint: preview.endpoint,
            package_name: preview.package_name,
            version: preview.version,
            uploaded_files: files.len(),
            manifest_sha256: preview.manifest_sha256,
            accepted: false,
            dry_run: true,
        });
    }

    let Some(auth) = auth else {
        return Err("live publish requires registry auth token".to_string());
    };

    let payload = json!({
        "name": preview.package_name,
        "version": preview.version,
        "manifest_sha256": preview.manifest_sha256,
        "files": files,
    });
    let endpoint = format!("{}/packages/publish", client.base_url);
    let _response = run_curl(
        "POST",
        &endpoint,
        Some(auth.token.as_str()),
        Some(payload.to_string()),
        client.timeout_secs,
    )?;

    Ok(PublishResult {
        endpoint,
        package_name: preview.package_name,
        version: preview.version,
        uploaded_files: payload
            .get("files")
            .and_then(|f| f.as_array())
            .map(|f| f.len())
            .unwrap_or(0),
        manifest_sha256: preview.manifest_sha256,
        accepted: true,
        dry_run: false,
    })
}

pub fn build_install_plan(
    client: &RegistryClient,
    package_name: &str,
    version: Option<&str>,
    target_dir: &Path,
    trust: PackageTrustPolicy,
) -> InstallPlan {
    InstallPlan {
        endpoint: format!("{}/packages/{}/resolve", client.base_url, package_name),
        package_name: package_name.to_string(),
        version: version.unwrap_or("latest").to_string(),
        target_dir: target_dir.display().to_string(),
        trust,
        dry_run: true,
    }
}

pub fn install_live(
    client: &RegistryClient,
    package_name: &str,
    version: Option<&str>,
    target_dir: &Path,
    auth: Option<&RegistryAuth>,
    trust: &PackageTrustPolicy,
    expected_sha256_override: Option<&str>,
    dry_run: bool,
) -> Result<InstallReceipt, String> {
    let resolved = resolve_live_package(client, package_name, version, auth)?;
    let target_file = target_dir.join(format!("{}-{}.aero.pkg", resolved.name, resolved.version));

    if dry_run {
        return Ok(InstallReceipt {
            package_name: resolved.name,
            version: resolved.version,
            target_file: target_file.display().to_string(),
            expected_sha256: expected_sha256_override
                .map(|s| s.to_string())
                .or(resolved.sha256),
            actual_sha256: "<dry-run>".to_string(),
            trusted: false,
            downloaded_bytes: 0,
        });
    }

    if !target_dir.exists() {
        fs::create_dir_all(target_dir).map_err(|err| {
            format!(
                "failed to create install target directory {}: {}",
                target_dir.display(),
                err
            )
        })?;
    }

    let bytes = run_curl_bytes(
        "GET",
        &resolved.download_url,
        auth.map(|a| a.token.as_str()),
        client.timeout_secs,
    )?;
    let actual_sha = sha256_hex(bytes.as_slice());
    let expected_sha = expected_sha256_override
        .map(|s| s.trim().to_string())
        .or(resolved.sha256);
    let trusted = verify_trust(trust, expected_sha.as_deref(), &actual_sha)?;

    fs::write(&target_file, bytes.as_slice()).map_err(|err| {
        format!(
            "failed to write package file {}: {}",
            target_file.display(),
            err
        )
    })?;

    Ok(InstallReceipt {
        package_name: resolved.name,
        version: resolved.version,
        target_file: target_file.display().to_string(),
        expected_sha256: expected_sha,
        actual_sha256: actual_sha,
        trusted,
        downloaded_bytes: bytes.len(),
    })
}

fn resolve_live_package(
    client: &RegistryClient,
    package_name: &str,
    version: Option<&str>,
    auth: Option<&RegistryAuth>,
) -> Result<PackageResolveMetadata, String> {
    let endpoint = format!(
        "{}/packages/{}/resolve?version={}",
        client.base_url,
        package_name,
        url_encode(version.unwrap_or("latest"))
    );
    let body = run_curl(
        "GET",
        &endpoint,
        auth.map(|a| a.token.as_str()),
        None,
        client.timeout_secs,
    )?;
    serde_json::from_str(&body).map_err(|err| format!("failed to decode package metadata: {}", err))
}

fn run_curl(
    method: &str,
    url: &str,
    token: Option<&str>,
    body: Option<String>,
    timeout_secs: u64,
) -> Result<String, String> {
    let mut cmd = Command::new("curl");
    cmd.arg("-sS")
        .arg("-f")
        .arg("-L")
        .arg("-X")
        .arg(method)
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg(url);

    if let Some(token) = token {
        cmd.arg("-H")
            .arg(format!("Authorization: Bearer {}", token));
    }
    if let Some(body) = body {
        cmd.arg("-H").arg("Content-Type: application/json");
        cmd.arg("--data").arg(body);
    }

    let output = cmd.output().map_err(|err| {
        format!(
            "failed to run curl for {} {}: {} (is curl installed?)",
            method, url, err
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "curl request failed for {} {}: {}",
            method,
            url,
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| format!("registry response was not valid UTF-8: {}", err))
}

fn run_curl_bytes(
    method: &str,
    url: &str,
    token: Option<&str>,
    timeout_secs: u64,
) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("curl");
    cmd.arg("-sS")
        .arg("-f")
        .arg("-L")
        .arg("-X")
        .arg(method)
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg(url);

    if let Some(token) = token {
        cmd.arg("-H")
            .arg(format!("Authorization: Bearer {}", token));
    }

    let output = cmd.output().map_err(|err| {
        format!(
            "failed to run curl for {} {}: {} (is curl installed?)",
            method, url, err
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "curl request failed for {} {}: {}",
            method,
            url,
            stderr.trim()
        ));
    }
    Ok(output.stdout)
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

fn collect_package_files(package_dir: &Path) -> Result<Vec<Value>, String> {
    let src_dir = package_dir.join("src");
    let mut files = Vec::new();
    if !src_dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(&src_dir).map_err(|err| {
        format!(
            "failed to read package src directory {}: {}",
            src_dir.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| format!("failed to read package entry: {}", err))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let data = fs::read(&path)
            .map_err(|err| format!("failed to read package file {}: {}", path.display(), err))?;
        files.push(json!({
            "path": path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown"),
            "bytes": data.len(),
            "sha256": sha256_hex(data.as_slice()),
        }));
    }
    Ok(files)
}

fn verify_trust(
    trust: &PackageTrustPolicy,
    expected_sha: Option<&str>,
    actual_sha: &str,
) -> Result<bool, String> {
    if let Some(expected_sha) = expected_sha {
        if expected_sha.eq_ignore_ascii_case(actual_sha) {
            return Ok(true);
        }
        if trust.allow_untrusted {
            return Ok(false);
        }
        return Err(format!(
            "package digest mismatch: expected {}, got {}",
            expected_sha, actual_sha
        ));
    }

    if trust.require_sha256 && !trust.allow_untrusted {
        return Err(
            "package digest is missing and trust policy requires SHA-256 verification".to_string(),
        );
    }
    Ok(false)
}

fn default_token_file_path() -> Option<PathBuf> {
    let home = env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())?;
    Some(Path::new(&home).join(".aero").join("registry_token"))
}

fn url_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

fn sha256_hex(input: &[u8]) -> String {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h = H0;
    let bit_len = (input.len() as u64) * 8;
    let mut data = input.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in data.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let j = i * 4;
            *word = u32::from_be_bytes([chunk[j], chunk[j + 1], chunk[j + 2], chunk[j + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = String::new();
    for part in h {
        out.push_str(&format!("{:08x}", part));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str, ext: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}.{}", name, nanos, ext))
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
        let path = unique_temp_path("aero_registry_index", "json");
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

    #[test]
    fn resolve_registry_auth_reads_cli_token_first() {
        let auth = resolve_registry_auth(Some("token-abc"), None).expect("auth should resolve");
        let auth = auth.expect("auth should be present");
        assert_eq!(auth.token, "token-abc");
        assert_eq!(auth.source, "cli");
    }

    #[test]
    fn resolve_registry_auth_reads_file_when_present() {
        let token_file = unique_temp_path("aero_registry_token", "txt");
        fs::write(&token_file, "file-token\n").expect("should write token file");
        let auth = resolve_registry_auth(None, Some(&token_file)).expect("auth should resolve");
        let auth = auth.expect("auth should exist");
        assert_eq!(auth.token, "file-token");
        assert!(auth.source.contains("file:"));
        let _ = fs::remove_file(&token_file);
    }

    #[test]
    fn trust_policy_rejects_mismatched_digest() {
        let trust = PackageTrustPolicy::default();
        let err = verify_trust(&trust, Some("deadbeef"), "cafebabe")
            .expect_err("mismatch should be rejected");
        assert!(err.contains("digest mismatch"));
    }

    #[test]
    fn trust_policy_allows_untrusted_when_enabled() {
        let trust = PackageTrustPolicy {
            require_sha256: true,
            allow_untrusted: true,
        };
        let trusted = verify_trust(&trust, None, "cafebabe").expect("should be allowed");
        assert!(!trusted);
    }

    #[test]
    fn sha256_matches_known_vector() {
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
