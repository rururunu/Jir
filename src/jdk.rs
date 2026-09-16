use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

const VERSION_INDEX_URL: &str = "https://rururunu.github.io/Jir/bat/version.json";
/// How long a locally cached index is trusted before hitting the network again.
const VERSION_CACHE_TTL: Duration = Duration::from_secs(3600);

/// Runtime home (holds `occupy`, `<version>/<distro>` and local metadata).
/// `JIR_HOME` overrides the default of a `home` dir next to the executable.
pub fn jdks_base() -> PathBuf {
    if let Some(dir) = std::env::var_os("JIR_HOME") {
        return PathBuf::from(dir);
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("home")
}

pub fn occupy_dir() -> PathBuf {
    jdks_base().join("occupy")
}

/// Marker recording the active JDK. Lives next to `occupy`, not inside it, so
/// activating a JDK never writes into the managed JDK directory itself.
pub fn current_marker() -> PathBuf {
    jdks_base().join(".jir-current")
}

/// Where older builds kept the marker (inside the JDK, reachable via the junction).
fn legacy_marker() -> PathBuf {
    occupy_dir().join(".jir-current")
}

/// Active JDK as (spec, vendor?) read from local state in a single pass.
/// Reads local state only — never the network, so it works offline.
pub fn current_info() -> Option<(String, Option<String>)> {
    let text = std::fs::read_to_string(current_marker())
        .or_else(|_| std::fs::read_to_string(legacy_marker()))
        .ok()?;
    parse_marker(&text)
}

fn parse_marker(text: &str) -> Option<(String, Option<String>)> {
    let mut lines = text.lines();
    let spec = lines.next()?.trim().to_string();
    if spec.is_empty() {
        return None;
    }
    let firm = lines
        .next()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty());
    Some((spec, firm))
}

/// Returns "version:distro" of the currently active JDK, or None.
pub fn current_version() -> Option<String> {
    current_info().map(|(spec, _)| spec)
}

pub fn set_current(spec: &str, firm: &str) -> Result<()> {
    let marker = current_marker();
    if let Some(parent) = marker.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&marker, format!("{}\n{}\n", spec, firm))
        .with_context(|| format!("failed to write {}", marker.display()))
}

/// Drop the active-JDK marker, used when the active JDK is uninstalled.
pub fn clear_current() {
    std::fs::remove_file(current_marker()).ok();
    std::fs::remove_file(legacy_marker()).ok();
}

pub fn installed_key(distro: &str, version: u64) -> String {
    format!("{}-{}", distro, version)
}

/// Every installed JDK as (version, distro), sorted by version then distro.
pub fn installed_specs() -> Vec<(u64, String)> {
    let base = jdks_base();
    let mut specs = Vec::new();
    let Ok(versions) = std::fs::read_dir(&base) else { return specs };
    for ver_entry in versions.filter_map(|e| e.ok()) {
        let ver_path = ver_entry.path();
        if !ver_path.is_dir() { continue; }
        let ver_name = ver_entry.file_name().into_string().unwrap_or_default();
        let Ok(version) = ver_name.parse::<u64>() else { continue };
        let Ok(distros) = std::fs::read_dir(&ver_path) else { continue };
        for dist_entry in distros.filter_map(|e| e.ok()) {
            if !dist_entry.path().is_dir() { continue; }
            let dist_name = dist_entry.file_name().into_string().unwrap_or_default();
            specs.push((version, dist_name));
        }
    }
    specs.sort();
    specs
}

pub fn installed_keys() -> HashSet<String> {
    installed_specs()
        .into_iter()
        .map(|(version, distro)| installed_key(&distro, version))
        .collect()
}

/// Per-JDK metadata recorded at install/activate time, so `use`, `ls` and
/// `current` can show the vendor and build number without touching the network.
#[derive(Default)]
pub struct JdkMeta {
    pub firm: Option<String>,
    pub java_version: Option<String>,
}

fn meta_path(version: u64, distro: &str) -> PathBuf {
    jdks_base()
        .join(".meta")
        .join(format!("{}.json", installed_key(distro, version)))
}

pub fn read_meta(version: u64, distro: &str) -> JdkMeta {
    let Some(text) = std::fs::read_to_string(meta_path(version, distro)).ok() else {
        return JdkMeta::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return JdkMeta::default();
    };
    JdkMeta {
        firm: value["firm"].as_str().map(str::to_string),
        java_version: value["java_version"].as_str().map(str::to_string),
    }
}

/// Best-effort: a failed metadata write only costs a future network lookup.
pub fn write_meta(version: u64, distro: &str, firm: Option<&str>, java_version: Option<&str>) {
    let mut obj = serde_json::Map::new();
    obj.insert("version".into(), serde_json::json!(version));
    obj.insert("distro".into(), serde_json::json!(distro));
    for (key, value) in [("firm", firm), ("java_version", java_version)] {
        if let Some(value) = value.filter(|s| !s.is_empty()) {
            obj.insert(key.into(), serde_json::json!(value));
        }
    }

    let path = meta_path(version, distro);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, serde_json::Value::Object(obj).to_string());
}

pub fn remove_meta(version: u64, distro: &str) {
    std::fs::remove_file(meta_path(version, distro)).ok();
}

/// Look up a package entry in the (already loaded) index.
pub fn find_package<'a>(
    index: &'a serde_json::Value,
    version: u64,
    distro: &str,
) -> Option<&'a serde_json::Value> {
    index["packages"].as_array()?.iter().find(|p| {
        p["version"].as_u64() == Some(version)
            && p["distro"]
                .as_str()
                .map(|d| d.eq_ignore_ascii_case(distro))
                .unwrap_or(false)
    })
}

/// Vendor and build number for a version:distro from the (cached) index.
/// Returns None when the index is unreachable so callers can fall back locally.
pub fn index_meta(version: u64, distro: &str) -> Option<(String, Option<String>)> {
    let index = load_version_json().ok()?;
    let package = find_package(&index, version, distro)?;
    let firm = package["firm"].as_str().unwrap_or(distro).to_string();
    let java_version = package["java_version"].as_str().map(str::to_string);
    Some((firm, java_version))
}

/// Machine-local index cache. Kept out of the runtime home so it still works
/// when `home` lives somewhere read-only (e.g. Program Files).
fn cache_dir() -> PathBuf {
    // `JIR_HOME` pins an isolated runtime: keep the cache with it
    if let Some(home) = std::env::var_os("JIR_HOME") {
        return PathBuf::from(home).join(".cache");
    }
    #[cfg(windows)]
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(dir).join("jir");
    }
    #[cfg(not(windows))]
    {
        if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
            return PathBuf::from(dir).join("jir");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".cache").join("jir");
        }
    }
    std::env::temp_dir().join("jir")
}

fn cache_path() -> PathBuf {
    cache_dir().join("version-cache.json")
}

pub fn load_version_json() -> Result<serde_json::Value> {
    let cache = cache_path();
    if let Some(text) = fresh_cache(&cache) {
        return Ok(serde_json::from_str(&text)?);
    }

    match fetch_index() {
        Ok(text) => {
            if let Some(parent) = cache.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&cache, &text);
            Ok(serde_json::from_str(&text)?)
        }
        // network failed: fall back to a stale cache so commands still work offline
        Err(err) => match std::fs::read_to_string(&cache) {
            Ok(text) if !text.is_empty() => Ok(serde_json::from_str(&text)?),
            _ => Err(err),
        },
    }
}

fn fresh_cache(path: &Path) -> Option<String> {
    let age = std::fs::metadata(path).ok()?.modified().ok()?.elapsed().ok()?;
    if age > VERSION_CACHE_TTL {
        return None;
    }
    std::fs::read_to_string(path).ok().filter(|s| !s.is_empty())
}

fn fetch_index() -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;

    Ok(client
        .get(VERSION_INDEX_URL)
        .send()
        .context("failed to reach the JDK index")?
        .error_for_status()?
        .text()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_key_is_distro_dash_version() {
        assert_eq!(installed_key("temurin", 21), "temurin-21");
    }

    #[test]
    fn parse_marker_reads_spec_and_optional_vendor() {
        assert_eq!(
            parse_marker("21:temurin\nTemurin\n"),
            Some(("21:temurin".to_string(), Some("Temurin".to_string())))
        );
        // no vendor line (marker written by an older build)
        assert_eq!(
            parse_marker("17:corretto\n"),
            Some(("17:corretto".to_string(), None))
        );
        assert_eq!(parse_marker(""), None);
        assert_eq!(parse_marker("  \nTemurin\n"), None);
    }

    #[test]
    fn fresh_cache_ignores_missing_and_empty_files() {
        let dir = std::env::temp_dir().join("jir-cache-test");
        std::fs::create_dir_all(&dir).unwrap();

        let missing = dir.join("missing.json");
        std::fs::remove_file(&missing).ok();
        assert_eq!(fresh_cache(&missing), None);

        let empty = dir.join("empty.json");
        std::fs::write(&empty, "").unwrap();
        assert_eq!(fresh_cache(&empty), None);

        let fresh = dir.join("fresh.json");
        std::fs::write(&fresh, "{}").unwrap();
        assert_eq!(fresh_cache(&fresh).as_deref(), Some("{}"));
    }
}
