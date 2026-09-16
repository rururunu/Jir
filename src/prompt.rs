use anyhow::{Context, Result};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::jdk::{installed_key, installed_keys, installed_specs, load_version_json};

/// Interactive vendor picker for installable packages of a given version.
/// Returns (distro, firm).
pub fn pick_installable(version: u64) -> Result<Option<(String, String)>> {
    let data = load_version_json()?;
    let packages = data["packages"].as_array().context("invalid version index")?;

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
    let feature: u64 = version
        .trim()
        .parse()
        .context("expected a Java feature version, e.g. `jir use 21`")?;

    let mut distros: Vec<String> = installed_specs()
        .into_iter()
        .filter(|(v, _)| *v == feature)
        .map(|(_, distro)| distro)
        .collect();

    if distros.is_empty() {
        anyhow::bail!("no installed vendors found for Java {}", feature);
    }

    if distros.len() == 1 {
        return Ok(Some(distros.swap_remove(0)));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select vendor for Java {}", feature))
        .items(&distros)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i| distros[i].clone()))
}

/// Interactive picker across every installed JDK. Returns "version:distro".
pub fn pick_any_installed() -> Result<Option<String>> {
    let labels: Vec<String> = installed_specs()
        .into_iter()
        .map(|(version, distro)| format!("{}:{}", version, distro))
        .collect();

    if labels.is_empty() {
        anyhow::bail!("no Java versions installed — run `jir i <version>` first");
    }
    if labels.len() == 1 {
        return Ok(Some(labels[0].clone()));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a JDK to activate")
        .items(&labels)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i| labels[i].clone()))
}
