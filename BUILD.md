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
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1
```

Default output:

```text
dist/jir-0.1.0-windows-x64-gui-setup.exe
```

If the output file is locked, the script writes a timestamped installer instead:

```text
dist/jir-0.1.0-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

The installer embeds:

- `jir.exe`
- `uninstall.exe`

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
├── bat/
│   └── version.json
├── packaging/
│   └── windows/
│       ├── build-installer.ps1
│       ├── JirSetup.cs
│       └── JirUninstall.cs
├── src/
│   ├── commands/
│   ├── cli.rs
│   ├── help.rs
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

