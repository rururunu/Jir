use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::jdk::load_version_json;

pub fn run(spec: &str) -> Result<()> {
    // version-only → interactive picker
    let (version, owned_distro);
    if !spec.contains(':') {
        let ver: u64 = spec.trim().parse()
            .context("expected  version  or  version:distro  e.g.  21  or  21:temurin")?;
        match crate::prompt::pick_installable(ver)? {
            Some((d, _)) => { version = ver; owned_distro = d; }
            None => { println!("{}", "Cancelled.".dimmed()); return Ok(()); }
        }
    } else {
        let (v, d) = parse_spec(spec)?;
        version = v;
        owned_distro = d.to_string();
    }
    let distro = owned_distro.as_str();
    let pkg = find_package(version, distro)?;

    let firm         = pkg["firm"].as_str().unwrap_or(distro);
    let url          = pkg["url"].as_str().unwrap_or("");
    let filename     = pkg["filename"].as_str().unwrap_or("jdk.zip");
    let archive_type = pkg["archive_type"].as_str().unwrap_or("zip");
    let dest_dir     = install_dir(version, distro);

    if dest_dir.exists() {
        println!("{} {}  ({})", "already installed:".green().bold(), spec, dest_dir.display());
        return Ok(());
    }

    // ── header ────────────────────────────────────────────────────────────
    println!();
    println!("  {}  {}:{}", "◆ Installing".cyan().bold(), version, distro);
    println!("  {:<10} {}", "vendor".dimmed(), firm);
    println!("  {:<10} {}", "file".dimmed(), filename);
    println!("  {:<10} {}", "url".dimmed(), url.dimmed());
    println!();

    // ── download ──────────────────────────────────────────────────────────
    let archive_path = download(url, filename)?;

    // ── extract / move ────────────────────────────────────────────────────
    if archive_type == "zip" {
        extract_zip(&archive_path, &dest_dir)?;
        fs::remove_file(&archive_path).ok();
    } else {
        fs::create_dir_all(&dest_dir)?;
        let final_path = dest_dir.join(filename);
        fs::rename(&archive_path, &final_path)?;
        println!("  saved installer → {}", final_path.display());
    }

    // ── summary ───────────────────────────────────────────────────────────
    println!();
    println!("  {}  {}:{}", "✔ Installed".green().bold(), version, distro);
    println!("  {:<10} {}", "path".dimmed(), dest_dir.display().to_string().green());
    println!("  {:<10} {}", "JAVA_HOME".dimmed(), dest_dir.display());
    println!("  {:<10} {}", "hint".dimmed(), format!("run `jir use {}` to activate", spec).dimmed());
    println!();

    Ok(())
}

fn parse_spec(spec: &str) -> Result<(u64, &str)> {
    let mut parts = spec.splitn(2, ':');
    let ver_str = parts.next().unwrap_or("").trim();
    let distro  = parts.next()
        .context("invalid format — expected  version:distro  e.g.  21:temurin")?
        .trim();
    let version: u64 = ver_str.parse().context("version must be a number")?;
    Ok((version, distro))
}

fn find_package(version: u64, distro: &str) -> Result<serde_json::Value> {
    let data = load_version_json()?;
    let packages = data["packages"]
        .as_array()
        .context("invalid version index")?
        .clone();

    packages
        .into_iter()
        .find(|p| {
            p["version"].as_u64() == Some(version)
                && p["distro"].as_str()
                    .map(|d| d.eq_ignore_ascii_case(distro))
                    .unwrap_or(false)
        })
        .context(format!(
            "no package found for {}:{}\n  run `jir ls -i` to see available distro names",
            version, distro
        ))
}

fn install_dir(version: u64, distro: &str) -> PathBuf {
    crate::jdk::jdks_base().join(version.to_string()).join(distro)
}

fn download(url: &str, filename: &str) -> Result<PathBuf> {
    let tmp_dir = std::env::temp_dir().join("jir_download");
    fs::create_dir_all(&tmp_dir)?;
    let dest = tmp_dir.join(filename);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let mut resp = client.get(url).send()?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  {bar:38.cyan/white.dim}  {bytes:>9}/{total_bytes:<9}  {binary_bytes_per_sec}  eta {eta}",
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("━━╸─"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut file = fs::File::create(&dest)?;
    let mut buf  = [0u8; 65536];
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        file.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }
    pb.finish_and_clear();

    Ok(dest)
}

fn extract_zip(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    fs::create_dir_all(dest)?;

    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let total = zip.len();

    // detect the sole top-level wrapper dir (e.g. jdk-21.0.11+10/)
    let strip_prefix: Option<String> = {
        let first = zip.by_index(0)?;
        let name = first.name().replace('\\', "/");
        name.split('/').next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  {bar:38.cyan/white.dim}  {pos:>6}/{len:<6} files  {wide_msg:.dim}",
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("━━╸─"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    for i in 0..total {
        let mut entry = zip.by_index(i)?;
        let raw = entry.name().replace('\\', "/");
        if raw.contains("..") { continue; }

        let rel_str = if let Some(ref prefix) = strip_prefix {
            raw.strip_prefix(&format!("{}/", prefix))
                .or_else(|| if raw == *prefix { Some("") } else { None })
                .unwrap_or(&raw)
                .to_string()
        } else {
            raw.clone()
        };

        let rel = PathBuf::from(&rel_str);
        if rel.as_os_str().is_empty() { pb.inc(1); continue; }

        let out_path = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            pb.set_message(rel.display().to_string());
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    let java_bin = dest.join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
    if java_bin.exists() {
        println!("  {:<10} {}", "binary".dimmed(), java_bin.display().to_string().green());
    } else {
        println!("  {} bin/java not found — check archive structure", "warn".yellow());
    }

    Ok(())
}
