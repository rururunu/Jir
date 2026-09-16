use anyhow::{Context, Result};
use colored::Colorize;
use terminal_size::{terminal_size, Width};

use crate::jdk::{
    current_version, installed_key, installed_keys, installed_specs, load_version_json, read_meta,
};

pub fn run(installable: bool, filter: Option<&str>) -> Result<()> {
    if installable {
        list_installable(filter)
    } else {
        list_installed()
    }
}

// ── jir ls -i ────────────────────────────────────────────────────────────────

fn list_installable(filter: Option<&str>) -> Result<()> {
    let filter = filter
        .map(|f| f.trim().parse::<u64>())
        .transpose()
        .context("filter must be a Java feature version, e.g. `jir ls -i 21`")?;

    let data = load_version_json()?;
    let packages = data["packages"].as_array().context("invalid version index")?;
    let installed = installed_keys();
    let current = current_version();

    let items: Vec<(String, Style)> = packages
        .iter()
        .filter(|pkg| filter.map_or(true, |v| pkg["version"].as_u64() == Some(v)))
        .map(|pkg| {
            let ver = pkg["version"].as_u64().unwrap_or(0);
            let distro = pkg["distro"].as_str().unwrap_or("");
            let key = installed_key(distro, ver);
            let spec = format!("{}:{}", ver, distro);
            let build = pkg["java_version"].as_str();

            // `*` = active, `+` = installed but not active, blank = not installed.
            // Distinct glyphs so the state is readable without color.
            let (marker, style) = if current.as_deref() == Some(&spec) {
                (Some('*'), Style::Current)
            } else if installed.contains(&key) {
                (Some('+'), Style::Installed)
            } else {
                (None, Style::Normal)
            };
            (spec_label(ver, distro, build, marker), style)
        })
        .collect();

    if items.is_empty() {
        println!("{}", "No matching versions in the index.".yellow());
        return Ok(());
    }

    print_columns(&items);
    Ok(())
}

// ── jir ls ───────────────────────────────────────────────────────────────────

fn list_installed() -> Result<()> {
    let current = current_version();
    let specs = installed_specs();

    if specs.is_empty() {
        println!("{}", "No Java versions installed.".yellow());
        println!("{}", "  run `jir list -i` to see available versions".dimmed());
        return Ok(());
    }

    let items: Vec<(String, Style)> = specs
        .iter()
        .map(|(ver_name, dist_name)| {
            let spec = format!("{}:{}", ver_name, dist_name);
            let meta = read_meta(*ver_name, dist_name);

            let (marker, style) = if current.as_deref() == Some(&spec) {
                (Some('*'), Style::Current)
            } else {
                (None, Style::Normal)
            };
            (spec_label(*ver_name, dist_name, meta.java_version.as_deref(), marker), style)
        })
        .collect();

    print_columns(&items);
    Ok(())
}

// ── rendering ────────────────────────────────────────────────────────────────

/// "  21:temurin     21.0.11+11 *" — build and marker are omitted when unknown.
fn spec_label(version: u64, distro: &str, build: Option<&str>, marker: Option<char>) -> String {
    let mut label = format!("  {}:{:<14}", version, distro);
    if let Some(build) = build.filter(|b| !b.is_empty()) {
        label.push_str(&format!(" {:<12}", build));
    }
    if let Some(marker) = marker {
        label.push_str(&format!(" {}", marker));
    }
    label.trim_end().to_string()
}

enum Style {
    Normal,
    Installed, // downloaded, not active
    Current,   // currently active via `jir use`
}

fn print_columns(items: &[(String, Style)]) {
    if items.is_empty() {
        return;
    }
    let term_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80);
    let col_width = items.iter().map(|(t, _)| t.len()).max().unwrap_or(40) + 2;
    let num_cols = (term_width / col_width).max(1);

    for (i, (text, style)) in items.iter().enumerate() {
        let cell = format!("{:<width$}", text, width = col_width);
        let colored = match style {
            Style::Current   => cell.blue().bold().to_string(),
            Style::Installed => cell.green().to_string(),
            Style::Normal    => cell,
        };
        print!("{}", colored);
        if (i + 1) % num_cols == 0 {
            println!();
        }
    }
    if items.len() % num_cols != 0 {
        println!();
    }
}
