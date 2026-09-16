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
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1 -Version 0.2.1
```

Output:

```text
dist/jir-0.2.1-windows-x64-gui-setup.exe
```

If the output file is locked, the script writes a timestamped installer instead:

```text
dist/jir-0.2.1-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

The installer embeds:

- `jir.exe`
- `uninstall.exe`

## Release Pipeline

Three channels install `jir`: Chocolatey, a PowerShell one-liner (curl/wget), and
the GUI installer. The first two share a single artifact — the portable archive —
which the GUI installer does not produce.

```powershell
# 1. Portable archive + checksum file. Everything else depends on this.
powershell -ExecutionPolicy Bypass -File .\release\build-portable.ps1 -Version 0.2.1

# 2. Chocolatey package. Reads the checksum produced by step 1.
powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.1

# 3. Push to the community feed (requires an API key).
$env:CHOCO_API_KEY = '<your key>'
powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.1 -Push

# 4. Tag the release and let CI publish the assets the one-liner installs from.
git tag v0.2.1
git push origin v0.2.1
```

Output:

```text
dist/jir-0.2.1-windows-x64.zip      portable archive (jir.exe, LICENSE, README.md)
dist/SHA256SUMS.txt                 SHA-256 of the archive
dist/jir.0.2.1.nupkg                Chocolatey package
```

`Cargo.toml` stays the single source of truth for the version.
`.github/workflows/release.yml` publishes on any `v*` tag and aborts when the tag
disagrees with `Cargo.toml`, so a release can never ship a binary reporting a
different version.

`chocolatey/jir.nuspec` and `chocolatey/tools/*.ps1` keep `__VERSION__` and
`__CHECKSUM__` placeholders; `release/build-chocolatey.ps1` substitutes them at
pack time. That keeps a packed version from ever pointing at an archive it was not
built from, without anyone copying hashes by hand.

Notes on the Chocolatey feed: `choco push` goes to community moderation, so the
package is not installable the moment it is pushed. Each release needs a new
version, and `choco pack` should be run locally first.

The GUI installer is deliberately not in this pipeline: `.gitignore` excludes
`/packaging`, so a clean checkout has no `build-installer.ps1` to run. Publish it
from a working tree that has it, or stop ignoring that directory.

## Installer Behavior

The installer can:

- choose the install directory
- add `jir` to user `PATH`
- set `JAVA_HOME` to `<install>\home\occupy`
- add `%JAVA_HOME%\bin` to `PATH`
- elevate to administrator when system Java variables must be fixed

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

If system environment variables point to `jir`, the uninstaller can restart as administrator to clean them.

## Project Structure

```text
jir/
├── .github/
│   └── workflows/
│       └── release.yml
├── bat/
│   └── version.json
├── chocolatey/
│   ├── jir.nuspec
│   └── tools/
│       ├── chocolateyinstall.ps1
│       └── chocolateyuninstall.ps1
├── packaging/
│   └── windows/
│       ├── build-installer.ps1
│       ├── JirSetup.cs
│       └── JirUninstall.cs
├── release/
│   ├── build-chocolatey.ps1
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

