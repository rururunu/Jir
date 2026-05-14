use anyhow::Result;
use colored::Colorize;
use terminal_size::{terminal_size, Width};

use crate::jdk::{current_version, installed_key, installed_keys, jdks_base, load_version_json};

pub fn run(installable: bool) -> Result<()> {
    if installable {
        list_installable()
    } else {
        list_installed()
    }
}

// ── jir ls -i ────────────────────────────────────────────────────────────────

fn list_installable() -> Result<()> {
    let data = load_version_json()?;
    let packages = data["packages"].as_array().unwrap();
    let installed = installed_keys();
    let current = current_version();

    let items: Vec<(String, Style)> = packages
        .iter()
        .map(|pkg| {
            let ver = pkg["version"].as_u64().unwrap_or(0);
            let distro = pkg["distro"].as_str().unwrap_or("");
            let key = installed_key(distro, ver);
            let spec = format!("{}:{}", ver, distro);

            let (label, style) = if current.as_deref() == Some(&spec) {
                (format!("  {}:{} *", ver, distro), Style::Current)
            } else if installed.contains(&key) {
                (format!("  {}:{} *", ver, distro), Style::Installed)
            } else {
                (format!("  {}:{}", ver, distro), Style::Normal)
            };
            (label, style)
        })
        .collect();

    print_columns(&items);
    Ok(())
}

// ── jir ls ───────────────────────────────────────────────────────────────────

fn list_installed() -> Result<()> {
    let base = jdks_base();
    let current = current_version();
    let keys = installed_keys();

    if keys.is_empty() {
        println!("{}", "No Java versions installed.".yellow());
        println!("{}", "  run `jir list -i` to see available versions".dimmed());
        return Ok(());
    }

    let Ok(ver_entries) = std::fs::read_dir(&base) else { return Ok(()) };

    let mut items: Vec<(String, Style)> = Vec::new();

    for ver_entry in ver_entries.filter_map(|e| e.ok()) {
        let ver_path = ver_entry.path();
        if !ver_path.is_dir() { continue; }
        let ver_name = ver_entry.file_name().into_string().unwrap_or_default();
        if ver_name.parse::<u64>().is_err() { continue; }

        let Ok(dist_entries) = std::fs::read_dir(&ver_path) else { continue };
        for dist_entry in dist_entries.filter_map(|e| e.ok()) {
            if !dist_entry.path().is_dir() { continue; }
            let dist_name = dist_entry.file_name().into_string().unwrap_or_default();
            let spec = format!("{}:{}", ver_name, dist_name);

            let (label, style) = if current.as_deref() == Some(&spec) {
                (format!("  {}:{} *", ver_name, dist_name), Style::Current)
            } else {
                (format!("  {}:{}", ver_name, dist_name), Style::Normal)
            };
            items.push((label, style));
        }
    }

    print_columns(&items);
    Ok(())
}

// ── rendering ────────────────────────────────────────────────────────────────

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
