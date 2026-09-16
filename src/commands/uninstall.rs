use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::jdk::{clear_current, current_version, jdks_base, remove_meta};

pub fn run(spec: &str, force: bool) -> Result<()> {
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

    let was_active = current_version().as_deref() == Some(&full_spec);
    if was_active {
        println!("{} {} is currently active via `jir use`", "warn:".yellow().bold(), full_spec);
    }

    println!(
        "  {} {} ({})",
        "Remove".red().bold(),
        full_spec,
        target.display()
    );
    if !force {
        print!("  Confirm? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    std::fs::remove_dir_all(&target)
        .context(format!("failed to remove {}", target.display()))?;
    remove_meta(version.parse().unwrap_or(0), distro);
    if was_active {
        // the occupy junction now points at a removed directory: drop it and
        // the marker so JAVA_HOME is not left dangling.
        crate::commands::switch::remove_occupy()?;
        clear_current();
    }
    println!("{} {}", "Uninstalled".green().bold(), full_spec);
    if was_active {
        println!("{}", "  JAVA_HOME is now unset — run `jir use` to activate another JDK".dimmed());
    }

    Ok(())
}
