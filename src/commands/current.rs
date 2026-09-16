use anyhow::Result;
use colored::Colorize;

use crate::jdk::{current_info, jdks_base, occupy_dir, read_meta};

pub fn run() -> Result<()> {
    let Some((spec, marker_firm)) = current_info() else {
        println!("{}", "No active JDK.".yellow());
        println!("{}", "  run `jir use <version:distro>` to activate one".dimmed());
        return Ok(());
    };

    let Some((version, distro)) = spec.split_once(':') else {
        println!("{} invalid current marker: {}", "error:".red().bold(), spec);
        return Ok(());
    };

    let source = jdks_base().join(version).join(distro);
    let java_bin = occupy_dir()
        .join("bin")
        .join(if cfg!(windows) { "java.exe" } else { "java" });
    // everything below is local state — `jir current` must work offline
    let meta = read_meta(version.parse().unwrap_or(0), distro);
    let vendor = marker_firm.or(meta.firm).unwrap_or_else(|| distro.to_string());

    println!();
    println!("  {}  {}", "◆ Current".blue().bold(), spec.blue().bold());
    println!("  {:<10} {}", "vendor".dimmed(), vendor);
    if let Some(build) = meta.java_version {
        println!("  {:<10} {}", "build".dimmed(), build);
    }
    println!("  {:<10} {}", "source".dimmed(), source.display());
    println!("  {:<10} {}", "JAVA_HOME".dimmed(), occupy_dir().display().to_string().green());
    println!("  {:<10} {}", "binary".dimmed(), format_binary(&java_bin));
    println!();

    Ok(())
}

fn format_binary(path: &std::path::Path) -> String {
    if path.exists() {
        path.display().to_string().green().to_string()
    } else {
        format!("{} {}", path.display(), "(missing)").yellow().to_string()
    }
}
