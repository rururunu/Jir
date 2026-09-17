use anyhow::{Context, Result};
use colored::Colorize;
use terminal_size::{terminal_size, Width};

use crate::jdk::{
    current_version, installed_key, installed_keys, installed_specs, load_version_json,
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
    let lts = lts_releases(&data);
    let packages = data["packages"].as_array().context("invalid version index")?;
    let installed = installed_keys();
    let current = current_version();

    let mut specs: Vec<(u64, String, Style)> = packages
        .iter()
        .filter(|pkg| filter.map_or(true, |v| pkg["version"].as_u64() == Some(v)))
        .map(|pkg| {
            let ver = pkg["version"].as_u64().unwrap_or(0);
            let distro = pkg["distro"].as_str().unwrap_or("").to_string();
            let key = installed_key(&distro, ver);
            let spec = format!("{}:{}", ver, distro);

            // `*` = active, `+` = installed but not active, blank = not installed.
            // Distinct glyphs so the state is readable without color.
            let style = if current.as_deref() == Some(&spec) {
                Style::Current
            } else if installed.contains(&key) {
                Style::Installed
            } else {
                Style::Normal
            };
            (ver, distro, style)
        })
        .collect();

    if specs.is_empty() {
        println!("{}", "No matching versions in the index.".yellow());
        return Ok(());
    }
    specs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let any_lts = specs.iter().any(|(ver, _, _)| lts.contains(ver));
    let items: Vec<(String, Style)> = specs
        .into_iter()
        .map(|(ver, distro, style)| {
            (installable_label(ver, &distro, style, lts.contains(&ver)), style)
        })
        .collect();

    print_columns(&items);
    if items.iter().any(|(_, style)| !matches!(style, Style::Normal)) {
        println!("{}", "  * active    + installed".dimmed());
    }
    if any_lts {
        println!("{}", "  ᴸᵀˢ long-term support".dimmed());
    }
    Ok(())
}

/// Long-term-support feature versions, as published by the index itself.
///
/// `bat/update_version_json.py` writes `lts_releases` from Adoptium's
/// `available_releases` endpoint. Reading it beats both alternatives: Foojay cannot
/// answer the question at all — its `term_of_support` is a per-package field, so 14
/// vendors report Java 21 as `sts` while 7 report Java 22 as `lts` — and a list
/// compiled into this binary would silently rot the next time Oracle names an LTS.
///
/// An index predating the field yields an empty list: the grid then carries no
/// marks rather than inventing them.
fn lts_releases(data: &serde_json::Value) -> Vec<u64> {
    data["lts_releases"]
        .as_array()
        .map(|releases| releases.iter().filter_map(|one| one.as_u64()).collect())
        .unwrap_or_default()
}

/// ` 21ᴸᵀˢ:temurin *` — the whole spec is the cell, so the grid stays flat and one
/// column always means one version. The mark is a superscript on the feature
/// version: it flags the long-term-support cadence without spending a column the
/// way a spelled-out badge did. The state glyph stays last, so what `jir use`
/// controls is still read last.
fn installable_label(ver: u64, distro: &str, style: Style, lts: bool) -> String {
    let marker = match style {
        Style::Current => " *",
        Style::Installed => " +",
        Style::Normal => "",
    };
    let mark = if lts { "ᴸᵀˢ" } else { "" };
    format!("  {}{}:{}{}", ver, mark, distro, marker)
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

    // Everything listed here is installed, so the only state left to show is which
    // one `jir use` has activated: a flat `version:distro` grid, as in 0.1.0.
    let items: Vec<(String, Style)> = specs
        .into_iter()
        .map(|(version, distro)| {
            let spec = format!("{}:{}", version, distro);
            let active = current.as_deref() == Some(spec.as_str());
            let style = if active { Style::Current } else { Style::Normal };
            (installed_label(version, &distro, active), style)
        })
        .collect();

    print_columns(&items);
    if items.iter().any(|(_, style)| matches!(style, Style::Current)) {
        println!("{}", "  * active".dimmed());
    }
    Ok(())
}

/// `  21:temurin *` — the whole spec is the cell, as in 0.1.0.
fn installed_label(version: u64, distro: &str, active: bool) -> String {
    format!("  {}:{}{}", version, distro, if active { " *" } else { "" })
}

// ── rendering ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Style {
    Normal,
    Installed, // downloaded, not active
    Current,   // currently active via `jir use`
}

impl Style {
    fn paint(self, text: &str) -> String {
        match self {
            Style::Current => text.blue().bold().to_string(),
            Style::Installed => text.green().to_string(),
            Style::Normal => text.to_string(),
        }
    }
}

/// Terminal width, with the floor the grids rely on.
fn term_width() -> usize {
    terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80).max(24)
}

/// Cells padded to one width and packed into as many columns as the terminal has
/// room for — the flat `version:distro` grid both listings use now.
fn columns(items: &[(String, Style)], term_width: usize) -> Vec<Vec<(String, Style)>> {
    if items.is_empty() {
        return Vec::new();
    }
    let col_width = items.iter().map(|(text, _)| text.chars().count()).max().unwrap_or(0) + 2;
    let num_cols = (term_width / col_width).max(1);

    items
        .chunks(num_cols)
        .map(|chunk| {
            chunk
                .iter()
                .map(|(text, style)| (format!("{:<width$}", text, width = col_width), *style))
                .collect()
        })
        .collect()
}

fn print_columns(items: &[(String, Style)]) {
    for row in columns(items, term_width()) {
        let last = row.len().saturating_sub(1);
        for (i, (text, style)) in row.iter().enumerate() {
            // the padding that keeps columns apart is noise at the end of a row
            let text = if i == last { text.trim_end() } else { text.as_str() };
            print!("{}", style.paint(text));
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── the flat `version:distro` grids ──────────────────────────────────────

    #[test]
    fn an_active_installed_version_keeps_the_0_1_0_cell() {
        assert_eq!(installed_label(21, "temurin", true), "  21:temurin *");
        assert_eq!(installed_label(21, "corretto", false), "  21:corretto");
    }

    #[test]
    fn the_installed_grid_is_flat_and_fits_the_terminal() {
        let items: Vec<(String, Style)> = vec![
            (installed_label(17, "temurin", false), Style::Normal),
            (installed_label(21, "corretto", false), Style::Normal),
            (installed_label(21, "temurin", true), Style::Current),
        ];

        let rows = columns(&items, 60);
        let width = rows[0][0].0.chars().count();
        assert!(rows.iter().flatten().all(|(text, _)| text.chars().count() == width));
        assert_eq!(rows.iter().flatten().count(), items.len());
        for row in &rows {
            let used: usize = row.iter().map(|(text, _)| text.chars().count()).sum();
            assert!(used <= 60, "row overflowed the terminal: {used}");
        }
    }

    fn cell(version: u64, distro: &str, style: Style) -> (String, Style) {
        (installable_label(version, distro, style, false), style)
    }

    #[test]
    fn an_installed_version_is_marked_in_its_own_cell() {
        assert_eq!(installable_label(21, "temurin", Style::Normal, false), "  21:temurin");
        assert_eq!(installable_label(21, "temurin", Style::Installed, false), "  21:temurin +");
        assert_eq!(installable_label(21, "temurin", Style::Current, false), "  21:temurin *");
    }

    #[test]
    fn an_lts_version_carries_a_superscript_mark_on_its_version() {
        assert_eq!(installable_label(21, "temurin", Style::Normal, true), "  21ᴸᵀˢ:temurin");
        assert_eq!(installable_label(21, "temurin", Style::Current, true), "  21ᴸᵀˢ:temurin *");
        assert_eq!(installable_label(8, "aoj", Style::Installed, true), "  8ᴸᵀˢ:aoj +");
        assert_eq!(installable_label(22, "temurin", Style::Normal, false), "  22:temurin");
    }

    #[test]
    fn the_superscript_mark_is_shorter_than_the_old_badge() {
        // the point of the mark is that it costs less of the cell than " LTS" did
        let marked = installable_label(21, "temurin", Style::Normal, true);
        let plain = installable_label(21, "temurin", Style::Normal, false);
        assert_eq!(marked.chars().count() - plain.chars().count(), 3);
    }

    #[test]
    fn the_lts_set_is_read_from_the_index() {
        let index = serde_json::json!({ "lts_releases": [8, 11, 17, 21, 25] });
        let lts = lts_releases(&index);

        for ver in [8, 11, 17, 21, 25] {
            assert!(lts.contains(&ver), "Java {ver} is an LTS release");
        }
        // the rest of 6..27, including the releases some vendors mislabel as lts
        for ver in [6, 7, 9, 10, 12, 13, 14, 15, 16, 18, 19, 20, 22, 23, 24, 26, 27] {
            assert!(!lts.contains(&ver), "Java {ver} is not an LTS release");
        }
    }

    #[test]
    fn an_index_without_the_lts_field_marks_nothing() {
        // a cached index from before the field existed must not invent marks
        let index = serde_json::json!({ "packages": [] });
        assert!(lts_releases(&index).is_empty());
    }

    #[test]
    fn a_row_mixing_lts_and_plain_cells_still_pads_to_one_width() {
        let items = vec![
            (installable_label(21, "temurin", Style::Normal, true), Style::Normal),
            (installable_label(22, "zulu", Style::Normal, false), Style::Normal),
        ];
        let rows = columns(&items, 200);
        let width = rows[0][0].0.chars().count();
        assert!(rows[0].iter().all(|(text, _)| text.chars().count() == width));
        assert!(rows[0][0].0.contains("ᴸᵀˢ"), "the mark was dropped: {:?}", rows[0][0].0);
    }

    #[test]
    fn installable_cells_are_padded_to_one_width_and_packed_to_fit() {
        let items = vec![
            cell(8, "aoj", Style::Normal),
            cell(8, "graalvm_community", Style::Current),
            cell(21, "temurin", Style::Installed),
        ];

        let rows = columns(&items, 60);
        let width = rows[0][0].0.chars().count();
        assert!(rows.iter().flatten().all(|(text, _)| text.chars().count() == width));
        assert_eq!(rows.iter().flatten().count(), items.len());
        assert!(rows.iter().any(|row| row.len() > 1), "nothing was packed into columns");
        for row in &rows {
            let used: usize = row.iter().map(|(text, _)| text.chars().count()).sum();
            assert!(used <= 60, "row overflowed the terminal: {used}");
        }
    }

    #[test]
    fn a_narrow_terminal_still_prints_one_version_per_row() {
        let items = vec![cell(8, "aoj", Style::Normal), cell(21, "temurin", Style::Normal)];
        let rows = columns(&items, 24);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.len() == 1));
    }
}
