# Build Guide

**Language:** English | [中文](BUILD.zh-CN.md)

This document explains how to build, run, test, and package `jir`.

## Requirements

- Windows 10/11
- Rust and Cargo
- PowerShell
- .NET Framework C# compiler (`csc.exe`) for building the GUI installer

Check the Rust toolchain:

```powershell
rustc --version
cargo --version
```

## Development Build

Build the debug binary:

```powershell
cargo build
```

Run commands during development:

```powershell
cargo run -- -h
cargo run -- ls -i
cargo run -- i 21
cargo run -- use 21
cargo run -- current
```

## Release Build

Build the optimized binary:

```powershell
cargo build --release
```

Find Cargo's actual output directory:

```powershell
cargo metadata --no-deps --format-version 1
```

The release binary is named `jir-cli.exe` before packaging. The installer renames it to `jir.exe`.

## Version Index

`jir` loads the Java version index from:

```text
https://rururunu.github.io/Jir/bat/version.json
```

The repository still keeps `bat/version.json` as the source file used for publishing that hosted index.

## GUI Installer

Build the standalone Windows GUI installer:

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1 -Version 0.2.3
```

Output:

```text
dist/jir-0.2.3-windows-x64-gui-setup.exe
```

If the output file is locked, the script writes a timestamped installer instead:

```text
dist/jir-0.2.3-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

The installer embeds:

- `jir.exe`
- `uninstall.exe`

`-Version` is compiled into the installer, so the version it records on disk always
matches the binary it installs. `packaging\windows\` is tracked by git, so a clean
checkout can build the installer.

## Release Pipeline

Two channels install `jir`: a PowerShell one-liner (curl/wget) and the GUI
installer. Only the one-liner installs from the portable archive; the GUI
installer does not produce one.

```powershell
# 1. Portable archive + checksum file. The one-liner installs from this.
powershell -ExecutionPolicy Bypass -File .\release\build-portable.ps1 -Version 0.2.3

# 2. Tag the release and let CI publish the assets the one-liner installs from.
git tag v0.2.3
git push origin v0.2.3
```

Output:

```text
dist/jir-0.2.3-windows-x64.zip      portable archive (jir.exe, LICENSE, README.md)
dist/SHA256SUMS.txt                 SHA-256 of the archive
```

`Cargo.toml` stays the single source of truth for the version.
`.github/workflows/release.yml` publishes on any `v*` tag and aborts when the tag
disagrees with `Cargo.toml`, so a release can never ship a binary reporting a
different version.

The GUI installer is not part of this pipeline: CI publishes the portable archive
only. Build it locally with `packaging\windows\build-installer.ps1` and attach the
resulting exe to the release by hand.

## Installer Behavior

The installer can:

- choose the install directory, prefilled with the location of an existing installation
- add `jir` to user `PATH`
- set `JAVA_HOME` to `<install>\home\occupy`
- add `%JAVA_HOME%\bin` to `PATH`
- elevate to administrator when system Java variables must be fixed

It records itself under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\jir`, so `jir` appears in
"Apps & features" with its version and can be uninstalled from there.

Installing into a different directory removes the old directory from `PATH` and leaves
the old copy on disk; the completion page names the path whose uninstaller removes it.
Installing over a running `jir` fails with a message asking you to close the terminals
that use it, instead of a raw sharing violation.

Installed layout:

```text
<install>/
├── jir.exe
├── uninstall.exe
└── home/
    └── occupy/
```

## Uninstaller

The generated `uninstall.exe` removes:

- the installation directory
- all installed JDKs under `home/`
- user `PATH` entries added by `jir`
- user `JAVA_HOME` when it points to `home\occupy`
- the `HKCU` uninstall record, but only while it still points at this directory

`%JAVA_HOME%\bin` is removed from `PATH` only when `JAVA_HOME` still points at this
installation. If you repointed `JAVA_HOME` at your own JDK, that entry stays.

The confirmation prompt names `<install>\home` and the size of the JDKs stored there
before deleting them.

If system environment variables point to `jir`, the uninstaller can restart as administrator to clean them.

## Project Structure

```text
jir/
├── .github/
│   └── workflows/
│       └── release.yml
├── bat/
│   └── version.json
├── packaging/
│   └── windows/
│       ├── build-installer.ps1
│       ├── JirSetup.cs
│       └── JirUninstall.cs
├── release/
│   ├── build-portable.ps1
│   └── install.ps1
├── src/
│   ├── commands/
│   ├── cli.rs
│   ├── jdk.rs
│   ├── main.rs
│   └── prompt.rs
├── Cargo.toml
├── README.md
├── README.zh-CN.md
├── BUILD.md
└── BUILD.zh-CN.md
```

## Clean

Clean Rust build artifacts:

```powershell
cargo clean
```

Remove generated installer output:

```powershell
Remove-Item .\dist -Recurse -Force
```

Only remove `dist/` when no installer window is open.

