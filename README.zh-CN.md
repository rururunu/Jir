# jir

**文档语言：** 中文 | [English](README.md)

`jir` 用来帮你管理 Java 版本，尽量少折腾 `JAVA_HOME`。

你可以安装 JDK、切换 JDK，并让当前使用的 Java 始终指向一个固定目录：`home/occupy`。

## 为什么做这个

如果你经常在 Java 8、17、21，或者 Temurin、Corretto、Zulu、Oracle、Microsoft OpenJDK 之间切换，`jir` 可以把这个流程变简单。

你只需要把 `JAVA_HOME` 设置到 `home/occupy` 一次。之后执行 `jir use 21:temurin`，Java 版本就切过去了，不用反复改环境变量。

在 Windows 上，切换使用目录联接（junction），速度很快，不会复制整个 JDK。

> **平台说明：** 版本索引与安装器目前只面向 Windows x64。运行时目录可以通过 `JIR_HOME` 环境变量重定向。

## 安装

### PowerShell 一行安装（curl / wget）

```powershell
iex (New-Object Net.WebClient).DownloadString('https://github.com/rururunu/Jir/releases/download/v0.2.2/install.ps1')
```

这条命令会下载便携包、用 Release 里的 `SHA256SUMS.txt` 校验 SHA-256、解压到 `%LOCALAPPDATA%\jir\bin`，把该目录加入用户 `PATH`，并设置 `JIR_HOME`。不需要管理员权限。

> 请用这种写法，不要用 `irm ... | iex`。GitHub 的 Release 资产以 `application/octet-stream` 返回，`Invoke-WebRequest` 会给出字节数组，管道给 `iex` 会报解析错误；而 `DownloadString` 始终返回文本。Windows 上还有两个命名陷阱：PowerShell 里 `curl` 是 `Invoke-WebRequest` 的别名，真正要用时请写 `curl.exe`；`wget` 也不属于 Windows 自带（由 Git for Windows 提供）。

如果你想手动下载并解压：

```powershell
curl.exe -fL -o jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.2/jir-0.2.2-windows-x64.zip
curl.exe -fL -o SHA256SUMS.txt https://github.com/rururunu/Jir/releases/download/v0.2.2/SHA256SUMS.txt
Get-FileHash .\jir.zip -Algorithm SHA256   # 与 SHA256SUMS.txt 对比
Expand-Archive .\jir.zip -DestinationPath "$env:LOCALAPPDATA\jir\bin"
```

用 Git for Windows 自带的 `wget`：

```bash
wget -O jir.zip https://github.com/rururunu/Jir/releases/download/v0.2.2/jir-0.2.2-windows-x64.zip
```

无论用哪种方式，装好 JDK 后把 `JAVA_HOME` 指向 `%LOCALAPPDATA%\jir\home\occupy` 即可——`jir` 会让这个路径始终指向当前激活的 JDK。

### 图形化安装器

使用或构建 Windows 图形化安装器：

```text
dist/jir-0.2.2-windows-x64-gui-setup.exe
```

安装器可以选择安装目录，也可以帮你把 `jir` 加到 `PATH`，以及设置 `JAVA_HOME`。

安装完成后，打开一个新的终端确认：

```powershell
jir -h
```

## 快速开始

看看能安装哪些 JDK：

```powershell
jir ls -i
```

安装 Java 21。如果有多个发行商，`jir` 会让你选择：

```powershell
jir i 21
```

如果你已经知道要装哪个：

```powershell
jir i 21:temurin
```

切换到它：

```powershell
jir use 21:temurin
```

看看现在用的是谁：

```powershell
jir current
```

不需要了就删掉：

```powershell
jir uni 21:temurin
```

## 命令

目标写法有两种：只写特性版本（`21`），或写完整的 `version:distro`（`21:temurin`）。只写版本时，`jir` 会询问你要用哪个发行商。

- `jir`、`jir -h`、`jir --help`、`jir help <命令>`：显示帮助。
- `jir ls`：查看已安装的 JDK。
- `jir ls -i`：查看可安装的 JDK。
- `jir ls -i 21`：只查看某个特性版本的可安装 JDK。
- `jir i 21`：安装 Java 21，并选择发行商。
- `jir i 21:temurin`：安装指定发行版。
- `jir i 21:temurin 17:corretto`：一次安装多个。
- `jir use`：从所有已安装的 JDK 中挑一个并激活。
- `jir use 21`：从已安装的 Java 21 中选择一个并激活。
- `jir use 21:temurin`：直接激活指定发行版。
- `jir current`：查看当前激活的 Java。
- `jir uni 21`：从已安装的 Java 21 中选择一个并卸载。
- `jir uni 21:temurin`：确认后卸载某个 JDK（`-y` 可跳过确认）。

别名：`ls` = `list`，`i` = `install`，`u` = `use`，`uni` = `uninstall`，`cur` = `current`。查看某个命令自己的帮助用 `jir help use`。

### 怎么看列表

`*` 表示当前激活的 JDK，`+` 表示已安装但未激活，空白表示尚未安装。三个字形不同，所以不依赖颜色也能读出状态。

`jir ls`、`jir current` 和 `jir use <spec>` 都可以离线工作。版本索引会缓存一小时，网络不可达时使用过期缓存。

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

`home/occupy` 指向当前激活的 JDK。安装器可以帮你把 `JAVA_HOME` 设置到这里。

最后两项由 `jir` 自行维护：`.jir-current` 让 `jir current` 无需联网就能给出答案，`.meta/` 让列表能显示厂商名与构建号。把整个 `home/` 目录挪到别处可以设置 `JIR_HOME`。

## 如果 Java 版本还是不对

先打开一个新的终端。已经打开的终端不会自动刷新环境变量。

然后检查：

```powershell
where.exe java
java -version
```

如果 Windows 还是优先找到别的 JDK，安装器可以请求管理员权限，帮你清理系统级 Java 环境冲突。

## 构建

见 `BUILD.zh-CN.md`。

## 技术文档

`jir` 的内部实现——磁盘上的状态、版本索引的格式与缓存、以及每个命令的执行路径——见 `TECHNICAL.zh-CN.md`。

