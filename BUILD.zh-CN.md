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

`jir` 会从下面的地址加载 Java 版本索引：

```text
https://rururunu.github.io/Jir/bat/version.json
```

仓库中的 `bat/version.json` 仍然保留，作为发布到该地址的源文件。

## 图形化安装器

构建独立 Windows 图形化安装器：

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1 -Version 0.2.3
```

输出：

```text
dist/jir-0.2.3-windows-x64-gui-setup.exe
```

如果输出文件被占用，脚本会生成带时间戳的安装器：

```text
dist/jir-0.2.3-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

安装器会内嵌：

- `jir.exe`
- `uninstall.exe`

`-Version` 会被编译进安装器，因此它记录到系统里的版本号始终与它实际安装的二进制一致。`packaging\windows\` 已纳入 git，干净检出也能构建安装器。

## 发布流水线

`jir` 有两个安装渠道：PowerShell 一行安装（curl/wget）和图形化安装器。只有前者使用便携包，图形化安装器不产出它。

```powershell
# 1. 生成便携包与校验和文件，一行安装从这里取
powershell -ExecutionPolicy Bypass -File .\release\build-portable.ps1 -Version 0.2.3

# 2. 打 tag，由 CI 发布一行安装所依赖的资产
git tag v0.2.3
git push origin v0.2.3
```

输出：

```text
dist/jir-0.2.3-windows-x64.zip      便携包（jir.exe、LICENSE、README.md）
dist/SHA256SUMS.txt                 便携包的 SHA-256
```

版本号仍然只以 `Cargo.toml` 为唯一来源。`.github/workflows/release.yml` 会在任何 `v*` tag 上发布，并在 tag 与 `Cargo.toml` 不一致时中止，避免发布出一个自报版本不同的二进制。

图形化安装器不在这条流水线里：CI 只发布便携包。请在本地用 `packaging\windows\build-installer.ps1` 构建，并把产出的 exe 手动附到 release 上。

## 安装器行为

安装器支持：

- 选择安装目录，并预填已有安装的位置
- 将 `jir` 添加到用户 `PATH`
- 将 `JAVA_HOME` 设置为 `<install>\home\occupy`
- 将 `%JAVA_HOME%\bin` 添加到 `PATH`
- 当需要修复系统级 Java 变量时请求管理员权限

安装器会在 `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\jir` 下登记自己，因此 `jir` 会带着版本号出现在“应用和功能”中，可以直接从那里卸载。

安装到不同的目录时，旧目录会从 `PATH` 中移除，但旧文件仍留在磁盘上；完成页会给出旧安装的位置及其卸载器路径。如果 `jir` 正在运行，安装会以明确提示失败，请你先关闭使用它的终端，而不是抛出原始的共享冲突异常。

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
- `HKCU` 中的卸载登记项，但仅当它仍指向本目录时

只有当 `JAVA_HOME` 仍指向本安装时，`%JAVA_HOME%\bin` 才会从 `PATH` 中移除。如果你已经把 `JAVA_HOME` 改指向自己的 JDK，该条目会保留。

确认对话框会先告知 `<install>\home` 路径以及其中 JDK 的占用大小，再执行删除。

如果系统环境变量也指向 `jir`，卸载器可以以管理员权限重启并清理它们。

## 项目结构

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

