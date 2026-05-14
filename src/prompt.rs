use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::jdk::{installed_key, installed_keys, jdks_base, load_version_json};

/// Interactive vendor picker for installable packages of a given version.
/// Returns (distro, firm).
pub fn pick_installable(version: u64) -> Result<Option<(String, String)>> {
    let data = load_version_json()?;
    let packages = data["packages"].as_array().unwrap();

    let installed = installed_keys();

    let candidates: Vec<(String, String)> = packages
        .iter()
        .filter(|p| p["version"].as_u64() == Some(version))
        .filter(|p| {
            let distro = p["distro"].as_str().unwrap_or("");
            !installed.contains(&installed_key(distro, version))
        })
        .map(|p| {
            let distro = p["distro"].as_str().unwrap_or("").to_string();
            let firm   = p["firm"].as_str().unwrap_or("").to_string();
            (distro, firm)
        })
        .collect();

    if candidates.is_empty() {
        anyhow::bail!("all available vendors for Java {} are already installed", version);
    }

    let labels: Vec<String> = candidates
        .iter()
        .map(|(distro, firm)| format!("{:<22} {}", distro, firm))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select vendor for Java {}", version))
        .items(&labels)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i| candidates[i].clone()))
}

/// Interactive vendor picker among already-installed distros for a given version.
/// Returns distro string.
pub fn pick_installed(version: &str) -> Result<Option<String>> {
    let base = jdks_base().join(version);
    if !base.exists() {
        anyhow::bail!("no installed versions found for Java {}", version);
    }

    let mut distros: Vec<String> = std::fs::read_dir(&base)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    distros.sort();

    if distros.is_empty() {
        anyhow::bail!("no installed vendors found for Java {}", version);
    }

    if distros.len() == 1 {
        return Ok(Some(distros.remove(0)));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select vendor for Java {}", version))
        .items(&distros)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i| distros[i].clone()))
}
