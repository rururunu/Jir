use anyhow::Result;
use colored::Colorize;

use crate::jdk::{current_version, jdks_base, load_version_json, occupy_dir};

pub fn run() -> Result<()> {
    let Some(spec) = current_version() else {
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
    let vendor = vendor_name(version, distro)?;

    println!();
    println!("  {}  {}", "◆ Current".blue().bold(), spec.blue().bold());
    println!("  {:<10} {}", "vendor".dimmed(), vendor);
    println!("  {:<10} {}", "source".dimmed(), source.display());
    println!("  {:<10} {}", "JAVA_HOME".dimmed(), occupy_dir().display().to_string().green());
    println!("  {:<10} {}", "binary".dimmed(), format_binary(&java_bin));
    println!();

    Ok(())
}

fn vendor_name(version: &str, distro: &str) -> Result<String> {
    let version = version.parse::<u64>().ok();
    let data = load_version_json()?;
    let packages = data["packages"].as_array();

    Ok(packages
        .and_then(|packages| {
            packages.iter().find(|p| {
                p["version"].as_u64() == version
                    && p["distro"]
                        .as_str()
                        .map(|d| d.eq_ignore_ascii_case(distro))
                        .unwrap_or(false)
            })
        })
        .and_then(|p| p["firm"].as_str())
        .unwrap_or(distro)
        .to_string())
}

fn format_binary(path: &std::path::Path) -> String {
    if path.exists() {
        path.display().to_string().green().to_string()
    } else {
        format!("{} {}", path.display(), "(missing)").yellow().to_string()
    }
}
