# jir

**Language:** English | [中文](README.zh-CN.md)

`jir` helps you manage Java versions without fighting `JAVA_HOME`.

Install a JDK, switch to it, and keep your active Java runtime behind one stable path: `home/occupy`.

## Why

If you often switch between Java 8, 17, 21, or different vendors like Temurin, Corretto, Zulu, Oracle, and Microsoft OpenJDK, `jir` keeps that workflow simple.

You can set `JAVA_HOME` to `home/occupy` once. After that, `jir use 21:temurin` switches Java without editing environment variables again.

On Windows, switching uses a directory junction, so it is fast and does not copy the whole JDK.

## Install

Download or build the Windows GUI installer:

```text
dist/jir-0.1.0-windows-x64-gui-setup.exe
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

- `jir`, `jir -h`, `jir -help`, `jir --help`: show help.
- `jir ls`: show installed JDKs.
- `jir ls -i`: show installable JDKs.
- `jir i 21`: install Java 21 and choose a vendor.
- `jir i 21:temurin`: install a specific distro.
- `jir use 21`: choose an installed Java 21 distro and activate it.
- `jir use 21:temurin`: activate a specific installed distro.
- `jir current`: show the active Java runtime.
- `jir uni 21:temurin`: uninstall a JDK after confirmation.

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
    └── occupy/
```

`home/occupy` points to the active JDK. The installer can set `JAVA_HOME` to this path for you.

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

