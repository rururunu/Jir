use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::jdk::{current_version, jdks_base, occupy_dir, set_current};

pub fn run(spec: Option<&str>) -> Result<()> {
    // no spec → pick among every installed JDK; version only → pick a vendor
    let full_spec = match spec {
        None => match crate::prompt::pick_any_installed()? {
            Some(spec) => spec,
            None => { println!("{}", "Cancelled.".dimmed()); return Ok(()); }
        },
        Some(spec) if !spec.contains(':') => {
            let version = spec.trim();
            match crate::prompt::pick_installed(version)? {
                Some(distro) => format!("{}:{}", version, distro),
                None => { println!("{}", "Cancelled.".dimmed()); return Ok(()); }
            }
        }
        Some(spec) => {
            let (version, distro) = parse_spec(spec)?;
            format!("{}:{}", version, distro)
        }
    };

    let (version, distro) = full_spec.split_once(':').context("invalid spec")?;
    let src  = jdks_base().join(version).join(distro);
    let dest = occupy_dir();

    anyhow::ensure!(
        src.exists(),
        "not installed: {}  (run `jir ls` to see installed versions)",
        full_spec
    );

    // remember the current target so a failed re-link can be rolled back
    let previous = current_target();

    remove_occupy()?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // use junction on Windows, symlink on Unix
    if let Err(err) = link(&src, &dest) {
        // restoring the previous junction keeps JAVA_HOME from going dangling
        if let Some(prev) = previous {
            link(&prev, &dest).ok();
        }
        return Err(err);
    }

    let firm = resolve_firm(version.parse().unwrap_or(0), distro);
    set_current(&full_spec, &firm)?;

    println!();
    println!("  {}  {}", "✔ Active".green().bold(), full_spec);
    println!("  {:<10} {}", "JAVA_HOME".dimmed(), dest.display().to_string().green());
    println!();

    Ok(())
}

/// Vendor for a spec: local metadata first, so `jir use` stays offline for
/// anything this build installed. Only a pre-existing JDK without metadata
/// falls back to the index, and the result is then cached locally.
fn resolve_firm(version: u64, distro: &str) -> String {
    if let Some(firm) = crate::jdk::read_meta(version, distro).firm {
        return firm;
    }
    match crate::jdk::index_meta(version, distro) {
        Some((firm, java_version)) => {
            crate::jdk::write_meta(version, distro, Some(&firm), java_version.as_deref());
            firm
        }
        None => distro.to_string(),
    }
}

/// Path the active JDK currently points at, if it still exists.
fn current_target() -> Option<PathBuf> {
    let spec = current_version()?;
    let (version, distro) = spec.split_once(':')?;
    let path = jdks_base().join(version).join(distro);
    path.exists().then_some(path)
}

/// Remove the occupy junction/directory if present.
pub fn remove_occupy() -> Result<()> {
    let dest = occupy_dir();
    if dest.exists() || dest.symlink_metadata().is_ok() {
        remove_link_or_dir(&dest)?;
    }
    Ok(())
}

fn parse_spec(spec: &str) -> Result<(u64, &str)> {
    let mut parts = spec.splitn(2, ':');
    let ver = parts.next().unwrap_or("").trim()
        .parse::<u64>()
        .context("version must be a number, e.g. 21:temurin")?;
    let distro = parts.next()
        .context("missing distro, format: version:distro")?
        .trim();
    Ok((ver, distro))
}

/// Remove a junction point or an empty/full directory safely.
fn remove_link_or_dir(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        if junction::exists(path).unwrap_or(false) {
            // remove only the junction point, not the target contents
            return fs::remove_dir(path)
                .context("failed to remove junction");
        }
    }
    // real directory (legacy from old copy-based approach)
    fs::remove_dir_all(path).context("failed to remove occupy directory")
}

/// Create a directory junction (Windows) or symlink (Unix).
fn link(src: &Path, dest: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        junction::create(src, dest)
            .context("failed to create junction")?;
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(src, dest)
            .context("failed to create symlink")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_requires_distro() {
        assert_eq!(parse_spec("21:temurin").unwrap(), (21, "temurin"));
        assert_eq!(parse_spec(" 17 : corretto ").unwrap(), (17, "corretto"));
        assert!(parse_spec("21").is_err());
        assert!(parse_spec("abc:temurin").is_err());
    }
}
