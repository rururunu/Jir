use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

use crate::jdk::{jdks_base, occupy_dir};

pub fn run(spec: &str) -> Result<()> {
    // version-only → interactive picker among installed
    let owned_distro: String;
    let version_str: String;
    let distro: &str;

    if !spec.contains(':') {
        let ver = spec.trim();
        match crate::prompt::pick_installed(ver)? {
            Some(d) => { version_str = ver.to_string(); owned_distro = d; }
            None    => { println!("{}", "Cancelled.".dimmed()); return Ok(()); }
        }
        distro = &owned_distro;
    } else {
        let (v, d) = parse_spec(spec)?;
        version_str = v.to_string();
        owned_distro = d.to_string();
        distro = &owned_distro;
    }

    let full_spec = format!("{}:{}", version_str, distro);
    let src  = jdks_base().join(&version_str).join(distro);
    let dest = occupy_dir();

    anyhow::ensure!(
        src.exists(),
        "not installed: {}  (run `jir ls` to see installed versions)",
        full_spec
    );

    // remove existing occupy (junction or real dir)
    if dest.exists() || dest.symlink_metadata().is_ok() {
        remove_link_or_dir(&dest)?;
    }

    // create parent dir if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // use junction on Windows, symlink on Unix
    link(&src, &dest)?;

    // write marker inside the now-linked occupy dir
    fs::write(dest.join(".jir-current"), &full_spec)?;

    println!();
    println!("  {}  {}", "✔ Active".green().bold(), full_spec);
    println!("  {:<10} {}", "JAVA_HOME".dimmed(), dest.display().to_string().green());
    println!();

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
fn remove_link_or_dir(path: &std::path::Path) -> Result<()> {
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
fn link(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
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
