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

    let mut items: Vec<Item> = packages
        .iter()
        .filter(|pkg| filter.map_or(true, |v| pkg["version"].as_u64() == Some(v)))
        .map(|pkg| {
            let ver = pkg["version"].as_u64().unwrap_or(0);
            let distro = pkg["distro"].as_str().unwrap_or("");
            let key = installed_key(distro, ver);
            let spec = format!("{}:{}", ver, distro);

            // `*` = active, `+` = installed but not active, blank = not installed.
            // Distinct glyphs so the state is readable without color.
            let marker = if current.as_deref() == Some(&spec) {
                Some('*')
            } else if installed.contains(&key) {
                Some('+')
            } else {
                None
            };

            Item {
                version: ver,
                distro: distro.to_string(),
                build: pkg["java_version"].as_str().map(str::to_string),
                marker,
            }
        })
        .collect();

    if items.is_empty() {
        println!("{}", "No matching versions in the index.".yellow());
        return Ok(());
    }

    render(&mut items);
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

    let mut items: Vec<Item> = specs
        .into_iter()
        .map(|(ver_name, dist_name)| {
            let spec = format!("{}:{}", ver_name, dist_name);
            let meta = read_meta(ver_name, &dist_name);
            let marker = if current.as_deref() == Some(&spec) {
                Some('*')
            } else {
                None
            };

            Item { version: ver_name, distro: dist_name, build: meta.java_version, marker }
        })
        .collect();

    render(&mut items);
    Ok(())
}

// ── rendering ────────────────────────────────────────────────────────────────

/// One JDK as listed: feature version, vendor, build, and how it relates to the
/// local install.
struct Item {
    version: u64,
    distro: String,
    build: Option<String>,
    marker: Option<char>,
}

#[derive(Clone, Copy)]
enum Style {
    Normal,
    Installed, // downloaded, not active
    Current,   // currently active via `jir use`
}

impl Style {
    fn of(marker: Option<char>) -> Self {
        match marker {
            Some('*') => Style::Current,
            Some('+') => Style::Installed,
            _ => Style::Normal,
        }
    }

    fn paint(self, text: &str) -> String {
        match self {
            Style::Current => text.blue().bold().to_string(),
            Style::Installed => text.green().to_string(),
            Style::Normal => text.to_string(),
        }
    }
}

/// A color-free table row: `gutter` names the Java version, `cells` hold the
/// vendors sharing it. Cells are padded, so aligning by concatenation is safe.
struct Line {
    gutter: String,
    cells: Vec<(String, Style)>,
}

/// Vendors of one feature version share a row prefix instead of repeating the
/// version in every cell. Padding each cell to the longest vendor in its group
/// keeps the build and state columns from drifting right on long names.
fn layout(items: &[Item], term_width: usize) -> Vec<Line> {
    /// Breathing room between cells, kept inside the cell so the longest row
    /// still separates its columns instead of running two vendors together.
    const CELL_GAP: usize = 2;

    let version_width = items.iter().map(|i| i.version.to_string().len()).max().unwrap_or(1);
    // Only spend a column on the state glyph when something is installed.
    let marker_width = if items.iter().any(|i| i.marker.is_some()) { 2 } else { 0 };
    let gutter_width = 2 + version_width + 2;

    let mut lines = Vec::new();
    let mut start = 0;
    while start < items.len() {
        let version = items[start].version;
        let len = items[start..].iter().take_while(|i| i.version == version).count();
        let group = &items[start..start + len];

        let distro_width = group.iter().map(|i| i.distro.chars().count()).max().unwrap_or(0);
        let build_width = group
            .iter()
            .filter_map(|i| i.build.as_deref())
            .map(str::len)
            .max()
            .unwrap_or(0);
        let trailing = if build_width == 0 { 0 } else { build_width + 1 };
        let cell_width = marker_width + distro_width + trailing + CELL_GAP;
        let per_line = (term_width.saturating_sub(gutter_width) / cell_width).max(1);

        for (row, chunk) in group.chunks(per_line).enumerate() {
            let label = if row == 0 { version.to_string() } else { String::new() };
            let cells = chunk
                .iter()
                .map(|item| (item_cell(item, marker_width, distro_width, build_width, cell_width), Style::of(item.marker)))
                .collect();
            lines.push(Line {
                gutter: format!("  {:<width$}  ", label, width = version_width),
                cells,
            });
        }
        start += len;
    }
    lines
}

/// `"* temurin        21.0.11+11 "` — padded to `cell_width` so columns hold.
fn item_cell(
    item: &Item,
    marker_width: usize,
    distro_width: usize,
    build_width: usize,
    cell_width: usize,
) -> String {
    let mut cell = String::new();
    if marker_width > 0 {
        cell.push(item.marker.unwrap_or(' '));
        cell.push(' ');
    }
    cell.push_str(&format!("{:<width$}", item.distro, width = distro_width));
    if let Some(build) = &item.build {
        cell.push_str(&format!(" {:<width$}", build, width = build_width));
    }
    format!("{:<width$}", cell, width = cell_width)
}

fn render(items: &mut [Item]) {
    if items.is_empty() {
        return;
    }
    items.sort_by(|a, b| a.version.cmp(&b.version).then_with(|| a.distro.cmp(&b.distro)));

    let term_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80).max(24);
    let marker = items.iter().any(|i| i.marker.is_some());

    for line in &layout(items, term_width) {
        print!("{}", line.gutter.cyan().bold());
        let last = line.cells.len().saturating_sub(1);
        for (i, (text, style)) in line.cells.iter().enumerate() {
            // the padding that keeps columns apart is noise at the end of a row
            let text = if i == last { text.trim_end() } else { text.as_str() };
            print!("{}", style.paint(text));
        }
        println!();
    }

    if marker {
        println!("{}", "  * active    + installed".dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(version: u64, distro: &str, build: Option<&str>, marker: Option<char>) -> Item {
        Item { version, distro: distro.to_string(), build: build.map(str::to_string), marker }
    }

    fn sorted(items: &mut [Item]) {
        items.sort_by(|a, b| a.version.cmp(&b.version).then_with(|| a.distro.cmp(&b.distro)));
    }

    #[test]
    fn every_cell_in_a_row_is_padded_to_the_same_width() {
        let mut items = vec![
            item(8, "aoj", Some("8.0.292"), None),
            item(8, "graalvm_community", Some("8.0.10"), None),
            item(8, "zulu", None, None),
            item(17, "temurin", Some("17.0.20+8"), Some('*')),
        ];
        sorted(&mut items);

        let lines = layout(&items, 78);
        assert!(lines.iter().all(|line| line
            .cells
            .iter()
            .all(|(text, _)| text.chars().count() == line.cells[0].0.chars().count())));
    }

    #[test]
    fn a_long_vendor_does_not_shift_the_build_column() {
        let mut items = vec![
            item(8, "aoj", Some("8.0.292"), None),
            item(8, "graalvm_community", Some("8.0.10"), None),
        ];
        sorted(&mut items);

        let lines = layout(&items, 200);
        let at = |text: &str| text.find("8.0.");
        assert_eq!(at(&lines[0].cells[0].0), at(&lines[0].cells[1].0));
    }

    #[test]
    fn a_full_width_row_still_leaves_a_gap_between_cells() {
        let mut items = vec![
            item(8, "oracle_open_jdk", Some("8.0.342+7"), None),
            item(8, "redhat", Some("8.0.345+1"), Some('+')),
        ];
        sorted(&mut items);

        // both vendors and both builds hit the group maximum, so nothing is left
        // over from padding — the gap has to be part of the cell itself
        let lines = layout(&items, 200);
        assert!(lines[0].cells[0].0.ends_with("  "), "cells ran together: {:?}", lines[0].cells[0].0);
    }

    #[test]
    fn versions_are_named_once_per_group_and_rows_fit_the_terminal() {
        let mut items = vec![
            item(8, "zulu", Some("8.0.502+7"), None),
            item(9, "aoj", Some("9.0.4"), None),
        ];
        sorted(&mut items);

        let lines = layout(&items, 40);
        assert_eq!(lines.len(), 2); // one vendor per version, one row each
        assert!(lines[0].gutter.starts_with("  8"));
        assert!(lines[1].gutter.starts_with("  9"));
        for line in &lines {
            let width = line.gutter.chars().count()
                + line.cells.iter().map(|(text, _)| text.chars().count()).sum::<usize>();
            assert!(width <= 40, "row overflowed the terminal: {width}");
        }
    }

    #[test]
    fn the_state_column_is_only_reserved_when_something_is_installed() {
        let plain = vec![item(21, "temurin", Some("21.0.11+11"), None)];
        assert!(layout(&plain, 200)[0].cells[0].0.starts_with("temurin"));

        let installed = vec![item(21, "temurin", Some("21.0.11+11"), Some('+'))];
        assert!(layout(&installed, 200)[0].cells[0].0.starts_with("+ temurin"));
    }
}
