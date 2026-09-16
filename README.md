# jir

**Language:** English | [中文](README.zh-CN.md)

`jir` helps you manage Java versions without fighting `JAVA_HOME`.

Install a JDK, switch to it, and keep your active Java runtime behind one stable path: `home/occupy`.

## Why

If you often switch between Java 8, 17, 21, or different vendors like Temurin, Corretto, Zulu, Oracle, and Microsoft OpenJDK, `jir` keeps that workflow simple.

You can set `JAVA_HOME` to `home/occupy` once. After that, `jir use 21:temurin` switches Java without editing environment variables again.

On Windows, switching uses a directory junction, so it is fast and does not copy the whole JDK.

> **Platform:** the version index and the installer target Windows x64 only. The
> runtime home can be relocated with the `JIR_HOME` environment variable.

## Install

### PowerShell one-liner (curl / wget)

```powershell
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.2/install.ps1')
```

This downloads the portable archive, verifies its SHA-256 against the release's
`SHA256SUMS.txt`, extracts it to `%LOCALAPPDATA%\jir\bin`, adds that directory to
your user `PATH`, and sets `JIR_HOME`. It needs no administrator rights.

> Use this form rather than `irm ... | iex`. GitHub serves release assets as
> `application/octet-stream`, so `Invoke-WebRequest` hands back a byte array and
> piping that into `iex` fails with a parse error; `DownloadString` always returns
> text. Two Windows naming traps: in PowerShell `curl` is an alias for
> `Invoke-WebRequest`, so write `curl.exe` when you mean the real thing, and `wget`
> is not part of Windows at all (Git for Windows provides it).

To fetch and unpack it by hand instead:

```powershell
curl.exe -fL -o jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.2/jir-0.2.2-windows-x64.zip
curl.exe -fL -o SHA256SUMS.txt https://github.com/rururunu/Jir/releases/download/v0.2.2/SHA256SUMS.txt
Get-FileHash .\jir.zip -Algorithm SHA256   # compare with SHA256SUMS.txt
Expand-Archive .\jir.zip -DestinationPath "$env:LOCALAPPDATA\jir\bin"
```

With `wget` from Git for Windows, the download is:

```bash
wget -O jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.2/jir-0.2.2-windows-x64.zip
```

Either way, once you have installed a JDK, point `JAVA_HOME` at
`%LOCALAPPDATA%\jir\home\occupy` — `jir` keeps that path pointing at whichever JDK
is active.

### GUI installer

Download or build the Windows GUI installer:

```text
dist/jir-0.2.2-windows-x64-gui-setup.exe
```

The installer lets you choose where `jir` lives. It can also add `jir` to `PATH` and set `JAVA_HOME` for you.

After installing, open a new terminal and check:

```powershell
jir -h
```

## Quick Start

See what you can install:

```powershell
jir ls -i
```

Install Java 21. If there are multiple vendors, `jir` will let you choose one:

```powershell
jir i 21
```

Already know what you want?

```powershell
jir i 21:temurin
```

Switch to it:

```powershell
jir use 21:temurin
```

Check what is active:

```powershell
jir current
```

Remove something you no longer need:

```powershell
jir uni 21:temurin
```

## Commands

A target is either a feature version (`21`) or a full `version:distro` pair
(`21:temurin`). Give a version only, and `jir` asks which vendor you mean.

- `jir`, `jir -h`, `jir --help`, `jir help <command>`: show help.
- `jir ls`: show installed JDKs.
- `jir ls -i`: show installable JDKs.
- `jir ls -i 21`: show installable JDKs for one feature version.
- `jir i 21`: install Java 21 and choose a vendor.
- `jir i 21:temurin`: install a specific distro.
- `jir i 21:temurin 17:corretto`: install several at once.
- `jir use`: pick from every installed JDK and activate it.
- `jir use 21`: choose an installed Java 21 distro and activate it.
- `jir use 21:temurin`: activate a specific installed distro.
- `jir current`: show the active Java runtime.
- `jir uni 21`: choose an installed Java 21 distro and remove it.
- `jir uni 21:temurin`: uninstall a JDK after confirmation (`-y` to skip).

Aliases: `ls` = `list`, `i` = `install`, `u` = `use`, `uni` = `uninstall`,
`cur` = `current`. Show a command's own help with `jir help use`.

### Reading the lists

`*` marks the active JDK, `+` marks one that is installed but not active, and a
blank entry is not installed yet. The glyphs differ, so the state is readable
without colour.

`jir ls`, `jir current` and `jir use <spec>` work offline. The version index is
cached for an hour, and a stale copy is used when the network is unreachable.

## Directory Layout

Everything lives under the install directory:

```text
<install>/
├── jir.exe
├── uninstall.exe
└── home/
    ├── 21/
    │   └── temurin/
    ├── 17/
    │   └── zulu/
    ├── occupy/            points to the active JDK
    ├── .jir-current       records which JDK is active
    └── .meta/             cached vendor and build info per JDK
```

`home/occupy` points to the active JDK. The installer can set `JAVA_HOME` to this path for you.

The last two entries are managed for you: `.jir-current` is what lets `jir current` answer without a network call, and `.meta/` is what lets the listings show a vendor name and build number. Set `JIR_HOME` to move the whole `home/` tree somewhere else.

## If Java Still Shows the Wrong Version

Open a new terminal first. Environment variables do not update inside terminals that are already open.

Then check:

```powershell
where.exe java
java -version
```

If Windows still finds another JDK first, the installer can request administrator permission and clean up conflicting system-level Java settings.

## Build

See `BUILD.md`.

## Technical Documentation

How `jir` works internally — the on-disk state, the version index format and its
cache, and the execution path of every command — is documented in
`TECHNICAL.md`.

