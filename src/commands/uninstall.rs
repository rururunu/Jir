use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::jdk::{current_version, jdks_base};

pub fn run(spec: &str) -> Result<()> {
    let (version, owned_distro): (&str, String);

    if !spec.contains(':') {
        version = spec.trim();
        match crate::prompt::pick_installed(version)? {
            Some(d) => owned_distro = d,
            None    => { println!("{}", "Cancelled.".dimmed()); return Ok(()); }
        }
    } else {
        let mut parts = spec.splitn(2, ':');
        version = parts.next().unwrap_or("").trim();
        owned_distro = parts.next()
            .context("invalid format — expected  version:distro  e.g.  21:temurin")?
            .trim()
            .to_string();
    }

    let distro = owned_distro.as_str();
    let full_spec = format!("{}:{}", version, distro);
    let target = jdks_base().join(version).join(distro);

    if !target.exists() {
        println!("{} {} is not installed", "error:".red().bold(), full_spec);
        return Ok(());
    }

    if current_version().as_deref() == Some(&full_spec) {
        println!("{} {} is currently active via `jir use`", "warn:".yellow().bold(), full_spec);
    }

    println!(
        "  {} {} ({})",
        "Remove".red().bold(),
        full_spec,
        target.display()
    );
    print!("  Confirm? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        std::fs::remove_dir_all(&target)
            .context(format!("failed to remove {}", target.display()))?;
        println!("{} {}", "Uninstalled".green().bold(), full_spec);
    } else {
        println!("{}", "Cancelled.".dimmed());
    }

    Ok(())
}
