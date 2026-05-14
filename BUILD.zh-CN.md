# 构建指南

**文档语言：** 中文 | [English](BUILD.md)

本文档说明如何构建、运行、测试和打包 `jir`。

## 环境要求

- Windows 10/11
- Rust 和 Cargo
- PowerShell
- .NET Framework C# 编译器（`csc.exe`），用于构建图形化安装器

检查 Rust 工具链：

```powershell
rustc --version
cargo --version
```

## 开发构建

构建调试版本：

```powershell
cargo build
```

开发时运行命令：

```powershell
cargo run -- -h
cargo run -- ls -i
cargo run -- i 21
cargo run -- use 21
cargo run -- current
```

## 发布构建

构建优化版本：

```powershell
cargo build --release
```

查看 Cargo 实际输出目录：

```powershell
cargo metadata --no-deps --format-version 1
```

打包前的 release 二进制名为 `jir-cli.exe`。安装器会把它写入为 `jir.exe`。

## 版本索引

Java 版本索引文件：

```text
bat/version.json
```

该文件会在编译时内嵌到二进制中。如果可执行文件旁存在外部 `bat/version.json`，则优先使用外部文件。

## 图形化安装器

构建独立 Windows 图形化安装器：

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1
```

默认输出：

```text
dist/jir-0.1.0-windows-x64-gui-setup.exe
```

如果输出文件被占用，脚本会生成带时间戳的安装器：

```text
dist/jir-0.1.0-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

安装器会内嵌：

- `jir.exe`
- `uninstall.exe`

## 安装器行为

安装器支持：

- 选择安装目录
- 将 `jir` 添加到用户 `PATH`
- 将 `JAVA_HOME` 设置为 `<install>\home\occupy`
- 将 `%JAVA_HOME%\bin` 添加到 `PATH`
- 当需要修复系统级 Java 变量时请求管理员权限

安装后的目录结构：

```text
<install>/
├── jir.exe
├── uninstall.exe
└── home/
    └── occupy/
```

## 卸载器

生成的 `uninstall.exe` 会删除：

- 安装目录
- `home/` 下所有已安装的 JDK
- `jir` 添加到用户 `PATH` 的条目
- 当用户 `JAVA_HOME` 指向 `home\occupy` 时清除它

如果系统环境变量也指向 `jir`，卸载器可以以管理员权限重启并清理它们。

## 项目结构

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

## 清理

清理 Rust 构建产物：

```powershell
cargo clean
```

删除生成的安装器输出：

```powershell
Remove-Item .\dist -Recurse -Force
```

只有在没有安装器窗口打开时，才执行删除 `dist/` 的命令。

