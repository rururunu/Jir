use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::commands::install;

const REPO: &str = "rururunu/Jir";
/// GitHub answers API requests that carry no User-Agent with 403.
const USER_AGENT: &str = "jir-self-update";
/// Staging files sit next to the executable so the final rename stays on one
/// volume; the leading dot keeps them out of the way in the install directory.
const STAGING_PREFIX: &str = ".jir-update-";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

pub fn run(force: bool) -> Result<()> {
    if !cfg!(windows) {
        bail!(
            "`jir update` ships prebuilt Windows x64 binaries only — \
             use your package manager to update jir on this platform"
        );
    }

    let exe = std::env::current_exe().context("cannot locate the running jir executable")?;
    let dir = exe
        .parent()
        .context("the running executable has no directory")?
        .to_path_buf();

    // A previous run could not delete its own backup: Windows keeps the old
    // image mapped for as long as that process lived.
    sweep_leftovers(&exe);

    println!();
    println!("  {:<10} jir {}", "current".dimmed(), CURRENT);

    let latest = latest_version()?;
    if !force && !is_newer(&latest, CURRENT) {
        println!(
            "  {}  {} is the newest release",
            "✔ up to date".green().bold(),
            latest
        );
        println!();
        return Ok(());
    }

    let asset = format!("jir-{}-windows-x64.zip", latest);
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        REPO, latest, asset
    );

    if is_newer(&latest, CURRENT) {
        println!("  {}  {} → {}", "◆ updating".cyan().bold(), CURRENT, latest);
    } else {
        println!("  {}  reinstalling {}", "◆ updating".cyan().bold(), latest);
    }
    println!("  {:<10} {}", "asset".dimmed(), asset);
    println!();

    let archive = install::download(&url, &asset, &format!("jir-update-{}", latest))?;
    let staged = dir.join(format!("{}{}.exe", STAGING_PREFIX, latest));
    let extracted = extract_exe(&archive, &staged);
    fs::remove_file(&archive).ok();
    extracted?;

    // A truncated download or an unexpected asset still unpacks *something*, so
    // only a binary reporting the requested version may replace this one.
    match reported_version(&staged) {
        Ok(reported) if reported == latest => {}
        Ok(reported) => {
            fs::remove_file(&staged).ok();
            bail!(
                "downloaded binary reports {} instead of {} — keeping the installed version",
                reported,
                latest
            );
        }
        Err(err) => {
            fs::remove_file(&staged).ok();
            return Err(err).context("downloaded binary does not run — keeping the installed version");
        }
    }

    swap_in(&exe, &staged)?;

    println!();
    println!("  {}  jir {}", "✔ updated".green().bold(), latest);
    println!(
        "  {:<10} {}",
        "path".dimmed(),
        exe.display().to_string().green()
    );
    println!(
        "  {:<10} {}",
        "home".dimmed(),
        crate::jdk::jdks_base().display()
    );
    println!();
    Ok(())
}

/// Newest published release, read from the API rather than guessed from the
/// asset name so a renamed asset cannot silently point at the wrong file.
fn latest_version() -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(USER_AGENT)
        .build()?;

    // reqwest is built without the `json` feature here, so parse the body by hand
    let body = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            REPO
        ))
        .send()
        .context("failed to reach the release feed")?
        .error_for_status()
        .context("no published release found")?
        .text()?;

    let value: serde_json::Value = serde_json::from_str(&body)?;
    let tag = value["tag_name"]
        .as_str()
        .context("the release feed carried no tag_name")?;
    Ok(tag.trim().trim_start_matches('v').to_string())
}

/// Component-wise numeric compare, so 0.10 beats 0.9 — a string compare would not.
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .split('.')
            .map(|p| p.trim().parse().unwrap_or(0))
            .collect()
    }

    let (left, right) = (parts(candidate), parts(current));
    for i in 0..left.len().max(right.len()) {
        let (a, b) = (
            left.get(i).copied().unwrap_or(0),
            right.get(i).copied().unwrap_or(0),
        );
        if a != b {
            return a > b;
        }
    }
    false
}

/// Pulls `jir.exe` out of the release archive, next to the running binary so the
/// rename that follows stays on one volume.
fn extract_exe(archive: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).context("release archive is not a zip")?;

    let mut found = None;
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if entry_file_name(&name).eq_ignore_ascii_case("jir.exe") {
            found = Some(i);
            break;
        }
    }
    let index = found.context("release archive contains no jir.exe")?;

    let mut entry = zip.by_index(index)?;
    let mut out = fs::File::create(dest).with_context(|| {
        format!(
            "cannot write {} — the install directory is not writable by this user",
            dest.display()
        )
    })?;
    io::copy(&mut entry, &mut out)?;
    Ok(())
}

/// Archive entries may use either separator; only the last component matters.
fn entry_file_name(name: &str) -> &str {
    name.rsplit(|c| c == '/' || c == '\\').next().unwrap_or(name)
}

/// Runs the staged binary once. A corrupt download or the wrong asset would
/// otherwise be swapped in silently.
fn reported_version(staged: &Path) -> Result<String> {
    let output = std::process::Command::new(staged)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to run {}", staged.display()))?;

    anyhow::ensure!(
        output.status.success(),
        "the downloaded binary exited with {}",
        output.status
    );

    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string())
}

/// Windows refuses to overwrite a running image but allows renaming it, so move
/// the live binary aside, put the new one in place, and roll back on failure.
fn swap_in(exe: &Path, staged: &Path) -> Result<()> {
    let backup = backup_path(exe);

    fs::rename(exe, &backup).with_context(|| {
        format!(
            "cannot replace {} — check that the directory is writable",
            exe.display()
        )
    })?;

    if let Err(err) = fs::rename(staged, exe) {
        // restore the working binary before surfacing the error
        fs::rename(&backup, exe).ok();
        return Err(err).context("failed to move the new binary into place");
    }

    // the old image stays mapped until this process exits
    fs::remove_file(&backup).ok();
    Ok(())
}

fn backup_path(exe: &Path) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(".old");
    exe.with_file_name(name)
}

/// Removes a backup from an earlier run and any staging file an interrupted
/// download left behind. Best effort: a file still in use is simply skipped.
fn sweep_leftovers(exe: &Path) {
    fs::remove_file(backup_path(exe)).ok();

    let Some(dir) = exe.parent() else { return };
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(STAGING_PREFIX) && name.ends_with(".exe") {
            fs::remove_file(entry.path()).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare_is_numeric_not_lexicographic() {
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(is_newer("0.2.3", "0.2.2"));
        assert!(is_newer("1.0", "0.9.9"));
        assert!(!is_newer("0.2.2", "0.2.2"));
        assert!(!is_newer("0.2.1", "0.2.2"));
        // a shorter version is zero-padded, not treated as older
        assert!(!is_newer("0.2", "0.2.0"));
    }

    #[test]
    fn entry_file_name_ignores_archive_separators() {
        assert_eq!(entry_file_name("jir.exe"), "jir.exe");
        assert_eq!(entry_file_name("bin\\jir.exe"), "jir.exe");
        assert_eq!(entry_file_name("bin/jir.exe"), "jir.exe");
    }

    #[test]
    fn backup_sits_next_to_the_binary() {
        let exe = Path::new("some").join("dir").join("jir.exe");
        assert_eq!(
            backup_path(&exe),
            Path::new("some").join("dir").join("jir.exe.old")
        );
    }
}
