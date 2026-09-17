<div align="center">

<img src="assets/jir.png" alt="jir" width="200">

**Manage Java runtimes fast.**

![version](https://img.shields.io/github/v/release/rururunu/Jir?style=flat-square&label=version&color=CE2029)
![platform](https://img.shields.io/badge/platform-Windows%20x64-586173?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-3A3030?style=flat-square)

**文档语言：** 中文 | [English](README.md) · **官网：** [rururunu.github.io/Jir](https://rururunu.github.io/Jir/site/index.html)

</div>

`jir` 用一条命令安装和切换 JDK——Java 8、17、21，以及 Temurin、Corretto、Zulu、
Oracle、Microsoft OpenJDK 等，全都放在同一个固定路径后面，不用再手动改 `JAVA_HOME`。

## 快速开始

```powershell
# 安装（Windows x64，无需管理员权限）
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.4/install.ps1')

jir ls -i        # 看看能装哪些
jir i 21         # 安装 Java 21（会问你用哪个发行商）
jir use 21       # 切过去——一条命令，别的都不用动
jir current      # 确认现在用的是谁
```

把 `JAVA_HOME` 指向安装目录里的 `home\occupy` 一次（图形化安装器可以帮你设置），
之后每次切换都只需要一条 `jir use`。更想用常规的 Windows 安装程序？直接从
[发布页](https://github.com/rururunu/Jir/releases/latest)下载图形化安装器——两种渠道
都写在[安装](#安装)一节里。

## 能做什么

- **一个固定路径。** 所有 JDK 都放在 `home/` 下，`jir use` 用目录联接（junction）
  把 `home/occupy` 重新指过去——瞬间完成，不复制文件。
- **目标带发行商。** `21` 会问你要哪个发行商，`21:temurin` 直接指定；列表里会显示
  每个 JDK 的发行商与构建号。
- **装多个、随便切、随时删。** Java 8、17、21……不同发行商可以并存，`jir uni`
  用来删掉不再需要的那些。
- **可离线使用。** `jir ls`、`jir current` 和 `jir use <spec>` 不需要联网：版本索引
  缓存一小时，网络不可达时使用过期缓存。
- **能自我更新。** `jir update` 就地升级到最新版本。

## 命令

目标写法有两种：只写特性版本（`21`），或写完整的 `version:distro`（`21:temurin`）。只写版本时，`jir` 会询问你要用哪个发行商。

| 命令 | 作用 |
| --- | --- |
| `jir`、`jir -h`、`jir --help`、`jir help` | 显示帮助。 |
| `jir ls` | 查看已安装的 JDK。 |
| `jir ls -i` | 查看可安装的 JDK。 |
| `jir ls -i 21` | 只查看某个特性版本的可安装 JDK。 |
| `jir i 21` | 安装 Java 21，并选择发行商。 |
| `jir i 21:temurin` | 安装指定发行版。 |
| `jir i 21:temurin 17:corretto` | 一次安装多个。 |
| `jir use` | 从所有已安装的 JDK 中挑一个并激活。 |
| `jir use 21` | 从已安装的 Java 21 中选择一个并激活。 |
| `jir use 21:temurin` | 直接激活指定发行版。 |
| `jir current` | 查看当前激活的 Java。 |
| `jir -v` | 查看当前 jir 版本。 |
| `jir uni 21` | 从已安装的 Java 21 中选择一个并卸载。 |
| `jir uni 21:temurin` | 确认后卸载某个 JDK（`-y` 可跳过确认）。 |
| `jir update` | 把 jir 自身更新到最新版（`--force` 可强制重装）。 |

别名：`ls` = `list`，`i` = `install`，`u` = `use`，`uni` = `uninstall`，`cur` = `current`，`up` = `update`。同一张表由 `jir -h` 彩色打印。

`*` 表示当前激活的 JDK，`+` 表示已安装但未激活，空白表示尚未安装。三个字形不同，所以不依赖颜色也能读出状态。

## 安装

版本索引与安装器目前只面向 Windows x64。运行时目录可以通过 `JIR_HOME` 环境变量重定向。

推荐用[图形化安装器](#图形化安装器)：它会记录安装信息，因此 `jir` 会出现在“应用和功能”里、可以正常卸载，并且会复用已有安装的目录，而不是再装出第二份。另一条渠道是下面的 [PowerShell 一行安装](#powershell-一行安装)。

不要混用两种渠道。它们各自维护自己的安装目录和 `home/` 下的 JDK 库，两个都装会让 `PATH` 上出现两份互不相干的 `jir`。

无论用哪种方式，`jir update` 都会就地更新你运行的那一份，所以通常不必再下载新的安装包。

### 图形化安装器

到[发布页](https://github.com/rururunu/Jir/releases/latest)下载 `jir-<版本>-windows-x64-gui-setup.exe` 并运行。

安装器可以选择安装目录，也可以帮你把 `jir` 加到 `PATH`，以及设置 `JAVA_HOME`。安装完成后，打开一个新的终端确认：

```powershell
jir -h
```

### PowerShell 一行安装

这是便携路径。它不带 `uninstall.exe`，卸载需要自己删除目录并清理 `PATH` 与 `JIR_HOME`。

```powershell
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.4/install.ps1')
```

这条命令会下载便携包、用 Release 里的 `SHA256SUMS.txt` 校验 SHA-256、解压到 `%LOCALAPPDATA%\jir\bin`，把该目录加入用户 `PATH`，并设置 `JIR_HOME`。不需要管理员权限。

把 `JAVA_HOME` 指向 `%LOCALAPPDATA%\jir\home\occupy` 即可——`jir` 会让这个路径始终指向当前激活的 JDK。

<details>
<summary>为什么用 <code>DownloadString</code>，而不是 <code>irm ... | iex</code>？</summary>

GitHub 的 Release 资产以 `application/octet-stream` 返回，`Invoke-WebRequest` 会给出字节数组，管道给 `iex` 会报解析错误；而 `DownloadString` 始终返回文本。Windows 上还有两个命名陷阱：PowerShell 里 `curl` 是 `Invoke-WebRequest` 的别名，真正要用时请写 `curl.exe`；`wget` 也不属于 Windows 自带（由 Git for Windows 提供）。

</details>

<details>
<summary>手动下载并解压</summary>

```powershell
curl.exe -fL -o jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.4/jir-0.2.4-windows-x64.zip
curl.exe -fL -o SHA256SUMS.txt https://github.com/rururunu/Jir/releases/download/v0.2.4/SHA256SUMS.txt
Get-FileHash .\jir.zip -Algorithm SHA256   # 与 SHA256SUMS.txt 对比
Expand-Archive .\jir.zip -DestinationPath "$env:LOCALAPPDATA\jir\bin"
```

用 Git for Windows 自带的 `wget`：

```bash
wget -O jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.4/jir-0.2.4-windows-x64.zip
```

</details>

## 目录结构

所有东西都放在安装目录里：

```text
<install>/
├── jir.exe
├── uninstall.exe
└── home/
    ├── 21/
    │   └── temurin/
    ├── 17/
    │   └── zulu/
    ├── occupy/            指向当前激活的 JDK
    ├── .jir-current       记录当前激活的是哪个 JDK
    └── .meta/             缓存每个 JDK 的厂商与构建信息
```

`home/occupy` 指向当前激活的 JDK。最后两项由 `jir` 自行维护：`.jir-current` 让 `jir current` 无需联网就能给出答案，`.meta/` 让列表能显示厂商名与构建号。把整个 `home/` 目录挪到别处可以设置 `JIR_HOME`。

## 如果 Java 版本还是不对

先打开一个新的终端。已经打开的终端不会自动刷新环境变量。

然后检查：

```powershell
where.exe java
java -version
```

如果 Windows 还是优先找到别的 JDK，安装器可以请求管理员权限，帮你清理系统级 Java 环境冲突。

## 构建

见 [BUILD.zh-CN.md](BUILD.zh-CN.md)。

## 技术文档

`jir` 的内部实现——磁盘上的状态、版本索引的格式与缓存、以及每个命令的执行路径——见 [TECHNICAL.zh-CN.md](TECHNICAL.zh-CN.md)。
