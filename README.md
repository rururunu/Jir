<div align="center">

<img src="assets/jir.png" alt="jir" width="200">

**Manage Java runtimes fast.**

![version](https://img.shields.io/github/v/release/rururunu/Jir?style=flat-square&label=version&color=CE2029)
![platform](https://img.shields.io/badge/platform-Windows%20x64-586173?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-3A3030?style=flat-square)

**Language:** English | [中文](README.zh-CN.md) · **Website:** [rururunu.github.io/Jir](https://rururunu.github.io/Jir/site/index.html)

</div>

`jir` installs JDKs and switches between them with a single command — Java 8, 17,
21 and vendors like Temurin, Corretto, Zulu, Oracle and Microsoft all live behind
one stable path, so you never edit `JAVA_HOME` by hand again.

## Quick Start

```powershell
# install (Windows x64, no admin rights needed)
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.4/install.ps1')

jir ls -i        # see what you can install
jir i 21         # install Java 21 (asks which vendor you mean)
jir use 21       # switch to it — one command, nothing else to change
jir current      # confirm what is active
```

Once `JAVA_HOME` points at `home\occupy` inside your install (the GUI installer
can set it for you), every later switch is a single `jir use`. Prefer a normal
Windows installer? Grab the
[GUI installer](https://github.com/rururunu/Jir/releases/latest) instead — both
channels are described under [Install](#install).

## What It Does

- **One stable path.** Every JDK lives under `home/`; `jir use` re-points
  `home/occupy` with a directory junction — instant, and nothing gets copied.
- **Vendor-aware targets.** `21` asks which vendor you mean, `21:temurin` pins it,
  and listings show the vendor and build of every JDK you have.
- **Install many, switch freely, remove cleanly.** Java 8, 17, 21, … from
  different vendors coexist side by side, and `jir uni` deletes the ones you are
  done with.
- **Works offline.** `jir ls`, `jir current` and `jir use <spec>` need no network:
  the version index is cached for an hour, and a stale copy is used when the
  network is unreachable.
- **Keeps itself current.** `jir update` upgrades `jir` to the newest release in
  place.

## Commands

A target is either a feature version (`21`) or a full `version:distro` pair
(`21:temurin`). Give a version only, and `jir` asks which vendor you mean.

| Command | What it does |
| --- | --- |
| `jir`, `jir -h`, `jir --help`, `jir help` | Show the help screen. |
| `jir ls` | Show installed JDKs. |
| `jir ls -i` | Show installable JDKs. |
| `jir ls -i 21` | Show installable JDKs for one feature version. |
| `jir i 21` | Install Java 21 and choose a vendor. |
| `jir i 21:temurin` | Install a specific distro. |
| `jir i 21:temurin 17:corretto` | Install several at once. |
| `jir use` | Pick from every installed JDK and activate it. |
| `jir use 21` | Choose an installed Java 21 distro and activate it. |
| `jir use 21:temurin` | Activate a specific installed distro. |
| `jir current` | Show the active Java runtime. |
| `jir -v` | Print the installed version. |
| `jir uni 21` | Choose an installed Java 21 distro and remove it. |
| `jir uni 21:temurin` | Uninstall a JDK after confirmation (`-y` to skip). |
| `jir update` | Update jir itself to the newest release (`--force` to reinstall). |

Aliases: `ls` = `list`, `i` = `install`, `u` = `use`, `uni` = `uninstall`,
`cur` = `current`, `up` = `update`. The same table is printed by `jir -h`, in colour.

`*` marks the active JDK, `+` marks one that is installed but not active, and a
blank entry is not installed yet. The glyphs differ, so the state is readable
without colour.

## Install

The version index and the installer target Windows x64 only. The runtime home can
be relocated with the `JIR_HOME` environment variable.

The recommended way is the [GUI installer](#gui-installer): it records the
installation, so `jir` shows up in "Apps & features" and can be uninstalled
normally, and it reuses the directory of an existing installation instead of
creating a second copy. The [PowerShell one-liner](#powershell-one-liner) below is
the other channel.

Do not mix the two. Each keeps its own install directory and its own JDK library
under `home/`, so installing both leaves two unrelated copies of `jir` on `PATH`.

Either way, `jir update` upgrades whichever copy you run, so you normally do not
need to download a new installer again.

### GUI Installer

Download `jir-<version>-windows-x64-gui-setup.exe` from the
[releases page](https://github.com/rururunu/Jir/releases/latest) and run it.

The installer lets you choose where `jir` lives. It can also add `jir` to `PATH`
and set `JAVA_HOME` for you. After installing, open a new terminal and check:

```powershell
jir -h
```

### PowerShell One-Liner

This is the portable path. It ships no `uninstall.exe`, so removing it means
deleting its directory and its `PATH` and `JIR_HOME` entries yourself.

```powershell
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.4/install.ps1')
```

It downloads the portable archive, verifies its SHA-256 against the release's
`SHA256SUMS.txt`, extracts it to `%LOCALAPPDATA%\jir\bin`, adds that directory to
your user `PATH`, and sets `JIR_HOME`. It needs no administrator rights.

Point `JAVA_HOME` at `%LOCALAPPDATA%\jir\home\occupy` and you are done — `jir`
keeps that path pointing at whichever JDK is active.

<details>
<summary>Why <code>DownloadString</code> and not <code>irm ... | iex</code>?</summary>

GitHub serves release assets as `application/octet-stream`, so
`Invoke-WebRequest` hands back a byte array and piping that into `iex` fails with
a parse error; `DownloadString` always returns text. Two Windows naming traps: in
PowerShell `curl` is an alias for `Invoke-WebRequest`, so write `curl.exe` when
you mean the real thing, and `wget` is not part of Windows at all (Git for
Windows provides it).

</details>

<details>
<summary>Download and unpack by hand</summary>

```powershell
curl.exe -fL -o jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.4/jir-0.2.4-windows-x64.zip
curl.exe -fL -o SHA256SUMS.txt https://github.com/rururunu/Jir/releases/download/v0.2.4/SHA256SUMS.txt
Get-FileHash .\jir.zip -Algorithm SHA256   # compare with SHA256SUMS.txt
Expand-Archive .\jir.zip -DestinationPath "$env:LOCALAPPDATA\jir\bin"
```

With `wget` from Git for Windows:

```bash
wget -O jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.4/jir-0.2.4-windows-x64.zip
```

</details>

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

`home/occupy` points to the active JDK. The last two entries are managed for you:
`.jir-current` is what lets `jir current` answer without a network call, and
`.meta/` is what lets the listings show a vendor name and build number. Set
`JIR_HOME` to move the whole `home/` tree somewhere else.

## If Java Still Shows the Wrong Version

Open a new terminal first. Environment variables do not update inside terminals that are already open.

Then check:

```powershell
where.exe java
java -version
```

If Windows still finds another JDK first, the installer can request administrator permission and clean up conflicting system-level Java settings.

## Build

See [BUILD.md](BUILD.md).

## Technical Documentation

How `jir` works internally — the on-disk state, the version index format and its
cache, and the execution path of every command — is documented in
[TECHNICAL.md](TECHNICAL.md).
