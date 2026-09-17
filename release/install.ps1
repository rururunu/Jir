<#
.SYNOPSIS
    Installs jir for the current user on Windows x64.

.DESCRIPTION
    Downloads the portable archive from a GitHub release, verifies its SHA-256
    against SHA256SUMS.txt, extracts it to <InstallDir>\bin and puts that
    directory on the user PATH. No administrator rights are required.

    Installed JDKs live under JIR_HOME (default <InstallDir>\home), outside the
    install directory, so reinstalling or upgrading jir never deletes them.

.PARAMETER Version
    Release version to install, without the leading `v`.

.PARAMETER InstallDir
    Where jir itself is installed. Defaults to %LOCALAPPDATA%\jir.

.PARAMETER BaseUrl
    Override the download base. Defaults to the GitHub release for -Version.

.EXAMPLE
    irm https://github.com/rururunu/Jir/releases/download/v0.2.3/install.ps1 | iex

.EXAMPLE
    .\install.ps1 -Version 0.2.3 -InstallDir D:\tools\jir

.NOTES
    The JDK store stays put and JAVA_HOME is left alone on purpose: point it at
    <InstallDir>\home\occupy once you have installed a JDK, and this tool will
    keep that path stable for you.
#>
[CmdletBinding()]
param(
    [string]$Version    = "0.2.3",
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA "jir"),
    [string]$BaseUrl    = ""
)

$ErrorActionPreference = "Stop"

# Windows PowerShell 5.1 still defaults to TLS 1.0 on older builds; GitHub needs 1.2.
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch {
    # Older .NET without Tls12 in the enum: leave the defaults alone.
}

function Set-UserEnvironmentVariable {
    param([Parameter(Mandatory = $true)][string]$Name, [Parameter(Mandatory = $true)][string]$Value)

    # Written through the registry rather than [Environment]::SetEnvironmentVariable
    # so an existing REG_EXPAND_SZ path keeps expanding %USERPROFILE% and friends.
    $Key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Environment")
    try { $Key.SetValue($Name, $Value, [Microsoft.Win32.RegistryValueKind]::ExpandString) }
    finally { $Key.Close() }
}

function Add-UserPath {
    param([Parameter(Mandatory = $true)][string]$Directory)

    $Key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Environment", $true)
    if (-not $Key) { throw "Cannot open HKCU\Environment for writing." }
    try {
        $Current = [string]$Key.GetValue("Path", "", [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $Kind    = [Microsoft.Win32.RegistryValueKind]::ExpandString
        if ($Key.GetValueNames() -contains "Path") { $Kind = $Key.GetValueKind("Path") }

        $Parts = @($Current -split ";" | Where-Object { $_ })
        foreach ($Part in $Parts) {
            if ($Part.TrimEnd("\") -ieq $Directory.TrimEnd("\")) { return }
        }
        $Key.SetValue("Path", (($Parts + $Directory) -join ";"), $Kind)
    } finally {
        $Key.Close()
    }
}

# Without this, Windows Explorer keeps handing out its stale environment block,
# so a terminal opened from the Start menu would not find jir yet.
function Send-EnvironmentChange {
    if (-not ("Jir.Native" -as [type])) {
        Add-Type -Namespace Jir -Name Native -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Auto)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr hWnd, uint Msg, System.UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out System.UIntPtr lpdwResult);
"@
    }
    $Result = [UIntPtr]::Zero
    [void][Jir.Native]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, "Environment", 2, 5000, [ref]$Result)
}

if (-not $BaseUrl) { $BaseUrl = "https://github.com/rururunu/Jir/releases/download/v$Version" }

$Archive = "jir-$Version-windows-x64.zip"
$Bin     = Join-Path $InstallDir "bin"
$JirHome = Join-Path $InstallDir "home"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("jir-install-" + [guid]::NewGuid().ToString("N"))

New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
try {
    $ZipPath  = Join-Path $TempDir $Archive
    $SumsPath = Join-Path $TempDir "SHA256SUMS.txt"

    Write-Host "Downloading $BaseUrl/$Archive" -ForegroundColor Cyan
    Invoke-WebRequest -Uri "$BaseUrl/$Archive"       -OutFile $ZipPath  -UseBasicParsing
    Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS.txt" -OutFile $SumsPath -UseBasicParsing

    Write-Host "Verifying SHA-256" -ForegroundColor Cyan
    # Split rather than regex-match: the file is written with CRLF and may use
    # either "<hash>  <name>" or GNU's "<hash> *<name>" form.
    $Expected = $null
    foreach ($LineText in ((Get-Content -Path $SumsPath -Raw) -split "\r?\n")) {
        $Fields = @($LineText.Trim() -split "\s+" | Where-Object { $_ })
        if ($Fields.Count -ge 2 -and $Fields[0] -match '^[0-9a-fA-F]{64}$') {
            if ($Fields[1].TrimStart("*") -eq $Archive) { $Expected = $Fields[0].ToLower(); break }
        }
    }
    if (-not $Expected) { throw "SHA256SUMS.txt has no entry for $Archive; refusing to install." }
    $Actual   = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
    if ($Actual -ne $Expected) {
        throw "Checksum mismatch for $Archive.`n  expected $Expected`n  actual   $Actual`nDownload interrupted or the release was tampered with; nothing was installed."
    }

    if (Test-Path $Bin) { Remove-Item $Bin -Recurse -Force }
    New-Item -ItemType Directory -Path $Bin -Force | Out-Null
    Expand-Archive -Path $ZipPath -DestinationPath $Bin -Force

    $Exe = Join-Path $Bin "jir.exe"
    if (!(Test-Path $Exe)) { throw "Archive did not contain jir.exe." }

    # JDKs must survive reinstalls, so JIR_HOME is kept outside <InstallDir>\bin.
    New-Item -ItemType Directory -Path $JirHome -Force | Out-Null
    Set-UserEnvironmentVariable -Name "JIR_HOME" -Value $JirHome
    Add-UserPath -Directory $Bin
    Send-EnvironmentChange

    $env:JIR_HOME = $JirHome
    $Reported = (& $Exe --version) 2>&1 | Select-Object -First 1

    Write-Host ""
    Write-Host "jir installed to $Bin" -ForegroundColor Green
    Write-Host "  version  : $Reported"
    Write-Host "  JIR_HOME : $JirHome"
    Write-Host ""
    Write-Host "Open a new terminal and run: jir -h"
    Write-Host "Once you have installed a JDK with 'jir i 21', point JAVA_HOME at:"
    Write-Host "  $JirHome\occupy"
} finally {
    Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
