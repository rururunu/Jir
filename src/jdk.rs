use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

/// Embedded fallback — compiled into the binary so jir works standalone.
const EMBEDDED_INDEX: &str = include_str!("../bat/version.json");

pub fn jdks_base() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("home")
}

pub fn occupy_dir() -> PathBuf {
    jdks_base().join("occupy")
}

/// Returns "version:distro" of the currently active JDK, or None.
pub fn current_version() -> Option<String> {
    let marker = occupy_dir().join(".jir-current");
    std::fs::read_to_string(marker).ok().map(|s| s.trim().to_string())
}

pub fn installed_key(distro: &str, version: u64) -> String {
    format!("{}-{}", distro, version)
}

pub fn installed_keys() -> HashSet<String> {
    let base = jdks_base();
    let mut keys = HashSet::new();
    let Ok(versions) = std::fs::read_dir(&base) else { return keys };
    for ver_entry in versions.filter_map(|e| e.ok()) {
        let ver_path = ver_entry.path();
        if !ver_path.is_dir() { continue; }
        let ver_name = ver_entry.file_name().into_string().unwrap_or_default();
        if ver_name.parse::<u64>().is_err() { continue; }
        let Ok(distros) = std::fs::read_dir(&ver_path) else { continue };
        for dist_entry in distros.filter_map(|e| e.ok()) {
            if !dist_entry.path().is_dir() { continue; }
            let dist_name = dist_entry.file_name().into_string().unwrap_or_default();
            keys.insert(installed_key(&dist_name, ver_name.parse().unwrap_or(0)));
        }
    }
    keys
}

pub fn load_version_json() -> Result<serde_json::Value> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let mut candidates = vec![
        PathBuf::from("bat/version.json"),
        PathBuf::from("../bat/version.json"),
    ];
    if let Some(dir) = exe_dir {
        candidates.push(dir.join("bat/version.json"));
    }

    // external file takes priority (allows updates without recompile)
    for path in &candidates {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            return Ok(serde_json::from_str(&content)?);
        }
    }

    // fall back to the embedded index
    Ok(serde_json::from_str(EMBEDDED_INDEX)?)
}
