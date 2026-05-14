# jir

**文档语言：** 中文 | [English](README.md)

`jir` 用来帮你管理 Java 版本，尽量少折腾 `JAVA_HOME`。

你可以安装 JDK、切换 JDK，并让当前使用的 Java 始终指向一个固定目录：`home/occupy`。

## 为什么做这个

如果你经常在 Java 8、17、21，或者 Temurin、Corretto、Zulu、Oracle、Microsoft OpenJDK 之间切换，`jir` 可以把这个流程变简单。

你只需要把 `JAVA_HOME` 设置到 `home/occupy` 一次。之后执行 `jir use 21:temurin`，Java 版本就切过去了，不用反复改环境变量。

在 Windows 上，切换使用目录联接（junction），速度很快，不会复制整个 JDK。

## 安装

使用或构建 Windows 图形化安装器：

```text
dist/jir-0.1.0-windows-x64-gui-setup.exe
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

- `jir`、`jir -h`、`jir -help`、`jir --help`：显示帮助。
- `jir ls`：查看已安装的 JDK。
- `jir ls -i`：查看可安装的 JDK。
- `jir i 21`：安装 Java 21，并选择发行商。
- `jir i 21:temurin`：安装指定发行版。
- `jir use 21`：从已安装的 Java 21 中选择一个并激活。
- `jir use 21:temurin`：直接激活指定发行版。
- `jir current`：查看当前激活的 Java。
- `jir uni 21:temurin`：确认后卸载某个 JDK。

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
    └── occupy/
```

`home/occupy` 指向当前激活的 JDK。安装器可以帮你把 `JAVA_HOME` 设置到这里。

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

