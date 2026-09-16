use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::jdk::{jdks_base, load_version_json};

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
    let java_version = pkg["java_version"].as_str().unwrap_or("");
    let url          = pkg["url"].as_str().unwrap_or("");
    let filename     = pkg["filename"].as_str().unwrap_or("jdk.zip");
    let archive_type = pkg["archive_type"].as_str().unwrap_or("zip");
    let dest_dir     = install_dir(version, distro);
    let key          = work_key(version, distro);

    if dest_dir.exists() {
        println!("{} {}  ({})", "already installed:".green().bold(), spec, dest_dir.display());
        return Ok(());
    }

    // ── header ────────────────────────────────────────────────────────────
    println!();
    println!("  {}  {}:{}", "◆ Installing".cyan().bold(), version, distro);
    println!("  {:<10} {}", "vendor".dimmed(), firm);
    if !java_version.is_empty() {
        println!("  {:<10} {}", "build".dimmed(), java_version);
    }
    println!("  {:<10} {}", "file".dimmed(), filename);
    println!("  {:<10} {}", "url".dimmed(), url.dimmed());
    println!();

    // ── download ──────────────────────────────────────────────────────────
    let archive_path = download(url, filename, &key)?;

    // ── extract / move ────────────────────────────────────────────────────
    if archive_type != "zip" {
        // msi/exe archives cannot be unpacked here: keep the installer, but do
        // NOT create home/<version>/<distro>, otherwise `jir ls` and `jir use`
        // would report a JDK that is not actually installed.
        let out = jdks_base().join("installers").join(filename);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&archive_path, &out).or_else(|_| fs::copy(&archive_path, &out).map(|_| ()))?;
        println!();
        println!("  {}  {}:{}", "Downloaded".yellow().bold(), version, distro);
        println!("  {:<10} {}", "file".dimmed(), out.display());
        println!("  {:<10} {}", "note".dimmed(), "not a zip archive — run the installer manually".dimmed());
        println!();
        return Ok(());
    }

    install_zip(&archive_path, &dest_dir, &key)?;
    fs::remove_file(&archive_path).ok();
    // record vendor/build locally so `use`, `ls` and `current` stay offline
    crate::jdk::write_meta(version, distro, Some(firm), Some(java_version));

    let java_bin = dest_dir.join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
    if java_bin.exists() {
        println!("  {:<10} {}", "binary".dimmed(), java_bin.display().to_string().green());
    } else {
        println!("  {} bin/java not found — check archive structure", "warn".yellow());
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
    crate::jdk::find_package(&data, version, distro)
        .cloned()
        .context(format!(
            "no package found for {}:{}\n  run `jir ls -i` to see available distro names",
            version, distro
        ))
}

fn install_dir(version: u64, distro: &str) -> PathBuf {
    crate::jdk::jdks_base().join(version.to_string()).join(distro)
}

/// Names the download and extract temp dirs. Keyed on version *and* distro so
/// installing 17:temurin and 21:temurin at once cannot overwrite each other,
/// and leftovers from an interrupted run stay identifiable.
fn work_key(version: u64, distro: &str) -> String {
    format!("{}-{}", version, distro)
}

/// JDK archives are 100–200 MB: a dropped connection used to mean starting over,
/// so retry a few times, dropping the partial file between attempts.
fn download(url: &str, filename: &str, key: &str) -> Result<PathBuf> {
    let tmp_dir = std::env::temp_dir().join("jir_download").join(key);
    fs::create_dir_all(&tmp_dir)?;
    let dest = tmp_dir.join(filename);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let mut last_err = None;
    for attempt in 1..=3 {
        match fetch_body(&client, url, &dest) {
            Ok(()) => return Ok(dest),
            Err(err) => {
                last_err = Some(err);
                fs::remove_file(&dest).ok();
                if attempt < 3 {
                    println!("  {} retrying ({}/3)", "!".yellow(), attempt);
                }
            }
        }
    }
    Err(last_err.unwrap())
}

fn fetch_body(client: &reqwest::blocking::Client, url: &str, dest: &Path) -> Result<()> {
    let mut resp = client.get(url).send()?.error_for_status()?;
    let pb = progress_bar(
        resp.content_length().unwrap_or(0),
        "  {spinner:.cyan}  {bar:38.cyan/white.dim}  {bytes:>9}/{total_bytes:<9}  {binary_bytes_per_sec}  eta {eta}",
    );
    // clear the bar on every path so a failure never leaves a half-drawn row
    let result = write_body(&mut resp, dest, &pb);
    pb.finish_and_clear();
    result
}

fn write_body(resp: &mut impl Read, dest: &Path, pb: &ProgressBar) -> Result<()> {
    let mut file = fs::File::create(dest)?;
    let mut buf = [0u8; 65536];
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }
    Ok(())
}

fn progress_bar(len: u64, template: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(template)
            .unwrap()
            .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
            .progress_chars("━━╸─"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Extract to a sibling temp dir, then move it into place, so an interrupted
/// extract can never leave a half-written JDK that looks installed.
fn install_zip(archive: &Path, dest: &Path, key: &str) -> Result<()> {
    let parent = dest.parent().context("invalid install path")?;
    fs::create_dir_all(parent)?;

    let tmp = jdks_base().join(".tmp").join(key);
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }

    if let Err(err) = extract_zip(archive, &tmp).and_then(|()| fs::rename(&tmp, dest).map_err(Into::into)) {
        fs::remove_dir_all(&tmp).ok();
        return Err(err);
    }
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;

    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let total = zip.len();

    // detect the sole top-level wrapper dir (e.g. jdk-21.0.11+10/)
    let mut names = Vec::with_capacity(total);
    for i in 0..total {
        names.push(zip.by_index(i)?.name().replace('\\', "/"));
    }
    let strip_prefix = shared_wrapper(&names);

    let pb = progress_bar(
        total as u64,
        "  {spinner:.cyan}  {bar:38.cyan/white.dim}  {pos:>6}/{len:<6} files  {wide_msg:.dim}",
    );

    for i in 0..total {
        let mut entry = zip.by_index(i)?;
        let raw = entry.name().replace('\\', "/");

        let rel_str = if let Some(ref prefix) = strip_prefix {
            raw.strip_prefix(&format!("{}/", prefix))
                .or_else(|| if raw == *prefix { Some("") } else { None })
                .unwrap_or(&raw)
                .to_string()
        } else {
            raw.clone()
        };

        if let Some(rel) = safe_relative(&rel_str) {
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
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    Ok(())
}

/// The single top-level directory every entry lives under, if there is one.
/// Returns None unless all entries share the same first segment and at least one
/// entry is nested — otherwise stripping would misplace root-level files.
fn shared_wrapper(names: &[String]) -> Option<String> {
    let mut prefix: Option<&str> = None;
    let mut nested = false;

    for name in names {
        let name = name.trim_start_matches("./");
        let (first, rest) = match name.split_once('/') {
            Some((first, rest)) => (first, Some(rest)),
            None => (name, None),
        };
        if first.is_empty() {
            return None;
        }
        if rest.is_some_and(|rest| !rest.is_empty()) {
            nested = true;
        }
        match prefix {
            None => prefix = Some(first),
            Some(prefix) if prefix == first => {}
            Some(_) => return None,
        }
    }

    if nested { prefix.map(str::to_string) } else { None }
}

/// Turn an untrusted archive entry name into a safe relative path.
/// Rejects absolute paths, drive prefixes and `..` traversal.
fn safe_relative(name: &str) -> Option<PathBuf> {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in Path::new(name).components() {
        match component {
            Component::Normal(part) => {
                // drive-relative prefix such as "C:" — reject on every platform
                if part.to_string_lossy().ends_with(':') {
                    return None;
                }
                out.push(part);
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() { None } else { Some(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_splits_version_and_distro() {
        assert_eq!(parse_spec("21:temurin").unwrap(), (21, "temurin"));
        assert_eq!(parse_spec("17:corretto").unwrap(), (17, "corretto"));
        assert!(parse_spec("21").is_err());
        assert!(parse_spec("x:temurin").is_err());
    }

    #[test]
    fn safe_relative_blocks_traversal() {
        assert_eq!(safe_relative("bin/java.exe"), Some(PathBuf::from("bin/java.exe")));
        assert_eq!(safe_relative("foo..bar"), Some(PathBuf::from("foo..bar")));
        assert_eq!(safe_relative("./a/b"), Some(PathBuf::from("a/b")));
        assert_eq!(safe_relative("../evil"), None);
        assert_eq!(safe_relative("a/../../b"), None);
        assert_eq!(safe_relative("/etc/passwd"), None);
        assert_eq!(safe_relative("C:/Windows/system32"), None);
        assert_eq!(safe_relative(""), None);
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn work_key_is_unique_per_version_and_distro() {
        assert_eq!(work_key(21, "temurin"), "21-temurin");
        // a distro-only key would collide when two versions are installed
        assert_ne!(work_key(17, "temurin"), work_key(21, "temurin"));
        assert_ne!(work_key(21, "temurin"), work_key(21, "corretto"));
    }

    #[test]
    fn shared_wrapper_only_for_a_sole_top_level_dir() {
        // every entry under one wrapper dir → strip it
        assert_eq!(
            shared_wrapper(&names(&[
                "jdk-21.0.11+10/",
                "jdk-21.0.11+10/bin/java.exe",
                "jdk-21.0.11+10/lib/modules",
            ])),
            Some("jdk-21.0.11+10".to_string())
        );
        // root-level files mixed in → no strip, or the tree would be mangled
        assert_eq!(
            shared_wrapper(&names(&["bin/java.exe", "lib/modules"])),
            None
        );
        // a single root-level file is not a wrapper dir
        assert_eq!(shared_wrapper(&names(&["readme.txt"])), None);
        assert_eq!(shared_wrapper(&[]), None);
    }
}
