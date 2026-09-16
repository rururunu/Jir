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
powershell -ExecutionPolicy Bypass -File .\packaging\windows\build-installer.ps1 -Version 0.2.0
```

输出：

```text
dist/jir-0.2.0-windows-x64-gui-setup.exe
```

如果输出文件被占用，脚本会生成带时间戳的安装器：

```text
dist/jir-0.2.0-windows-x64-gui-setup-YYYYMMDD-HHMMSS.exe
```

安装器会内嵌：

- `jir.exe`
- `uninstall.exe`

## 发布流水线

`jir` 有三个安装渠道：Chocolatey、PowerShell 一行安装（curl/wget），以及图形化安装器。前两者共用同一个产物——便携包，而图形化安装器不产出它。

```powershell
# 1. 生成便携包与校验和文件，后面几步都依赖它
powershell -ExecutionPolicy Bypass -File .\release\build-portable.ps1 -Version 0.2.0

# 2. 打包 Chocolatey 包（读取第 1 步产出的校验和）
powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.0

# 3. 推送到社区源（需要 API key）
$env:CHOCO_API_KEY = '<your key>'
powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.0 -Push

# 4. 打 tag，由 CI 发布一行安装所依赖的资产
git tag v0.2.0
git push origin v0.2.0
```

输出：

```text
dist/jir-0.2.0-windows-x64.zip      便携包（jir.exe、LICENSE、README.md）
dist/SHA256SUMS.txt                 便携包的 SHA-256
dist/jir.0.2.0.nupkg                Chocolatey 包
```

版本号仍然只以 `Cargo.toml` 为唯一来源。`.github/workflows/release.yml` 会在任何 `v*` tag 上发布，并在 tag 与 `Cargo.toml` 不一致时中止，避免发布出一个自报版本不同的二进制。

`chocolatey/jir.nuspec` 与 `chocolatey/tools/*.ps1` 里保留 `__VERSION__` 和 `__CHECKSUM__` 占位符，由 `release/build-chocolatey.ps1` 在打包时替换。这样打出的版本不可能指向一个并非由它构建的归档，也不需要任何人手工拷贝哈希。

关于 Chocolatey 社区源：`choco push` 会进入人工审核队列，推送后不会立即可安装；每个 release 都需要新的版本号，推送前建议先本地 `choco pack` 验证。

图形化安装器被有意排除在这条流水线之外：`.gitignore` 忽略了 `/packaging`，干净检出里没有 `build-installer.ps1` 可运行。要发布它，请在含有该目录的工作区里手动构建，或取消忽略该目录。

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

