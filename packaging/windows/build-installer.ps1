param(
    [string]$Configuration = "release",
    [string]$Version = "0.2.3"
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Version)) {
    throw "A version is required: pass -Version <x.y.z>."
}

$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Dist = Join-Path $Root "dist"
$BuildDir = Join-Path $Dist "build"
$Out = Join-Path $Dist "jir-$Version-windows-x64-gui-setup.exe"

Write-Host "Building jir ($Configuration)..." -ForegroundColor Cyan
Push-Location $Root
cargo build --release
$Metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
Pop-Location

$TargetDir = $Metadata.target_directory
$BuiltExe = Join-Path $TargetDir "$Configuration\jir-cli.exe"

if (!(Test-Path $BuiltExe)) {
    throw "Built executable not found: $BuiltExe"
}

New-Item -ItemType Directory -Path $Dist -Force | Out-Null
if (Test-Path $BuildDir) {
    Remove-Item $BuildDir -Recurse -Force
}
New-Item -ItemType Directory -Path $BuildDir -Force | Out-Null

# The .NET Framework csc rejects a string in /define (CS2029: not a valid
# identifier), so the version goes into a generated source file that is compiled
# alongside JirSetup.cs.
$VersionInfoSource = Join-Path $BuildDir "JirBuildInfo.cs"
[System.IO.File]::WriteAllLines($VersionInfoSource, @(
    "namespace JirSetup",
    "{",
    "    internal static class JirBuildInfo",
    "    {",
    "        public const string Version = `"$Version`";",
    "    }",
    "}"
))

$Csc = (Get-Command csc.exe -ErrorAction SilentlyContinue).Source
if (-not $Csc) {
    $Csc = Get-ChildItem "$env:WINDIR\Microsoft.NET\Framework64" -Filter csc.exe -Recurse -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $Csc) {
    throw "csc.exe was not found. Cannot build Windows GUI installer."
}

$Icon = Join-Path $Root "assets\jir.ico"
$Manifest = Join-Path $PSScriptRoot "app.manifest"
$DpiSource = Join-Path $PSScriptRoot "DpiLayout.cs"
foreach ($asset in @($Icon, $Manifest, $DpiSource)) {
    if (!(Test-Path $asset)) {
        throw "Missing $asset. Regenerate the icon with: python packaging\windows\make-icon.py"
    }
}

# Start-Process passes ArgumentList through untouched, so each switch carries its own
# quotes the same way the /out switch and source paths below do.
#
# The manifest is what makes the process DPI-aware. Without it Windows stretches the
# finished window as a single bitmap, which is what made the installer look soft on a
# scaled display. DpiLayout.cs is what then grows the 96-DPI coordinates to match.
$Win32 = '/win32manifest:"' + $Manifest + '" /win32icon:"' + $Icon + '" '

if (Test-Path $Out) {
    try {
        Remove-Item $Out -Force
    } catch {
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $Out = Join-Path $Dist "jir-$Version-windows-x64-gui-setup-$stamp.exe"
        Write-Host "Existing installer is locked. Using new output path:" -ForegroundColor Yellow
        Write-Host "  $Out"
    }
}

Write-Host "Building standalone GUI installer..." -ForegroundColor Cyan
$Uninstaller = Join-Path $BuildDir "uninstall.exe"
if (Test-Path $Uninstaller) {
    Remove-Item $Uninstaller -Force
}

# Start-Process passes ArgumentList through untouched. Bare quotes are what argv
# parsing strips back off, so they wrap whole arguments; " survives as a literal
# quote inside a value, which is what /define needs around a version string.
$UninstallSource = Join-Path $PSScriptRoot "JirUninstall.cs"
$UninstallArgs = '/nologo /target:winexe "/out:' + $Uninstaller + '" ' + $Win32 +
    '/reference:System.Windows.Forms.dll /reference:System.Drawing.dll ' +
    '"' + $UninstallSource + '" "' + $DpiSource + '"'
$UninstallBuild = Start-Process -FilePath $Csc -ArgumentList $UninstallArgs -Wait -PassThru -NoNewWindow
if ($UninstallBuild.ExitCode -ne 0) {
    throw "csc failed to build the uninstaller (exit $($UninstallBuild.ExitCode))."
}

if (!(Test-Path $Uninstaller)) {
    throw "Uninstaller was not created: $Uninstaller"
}

$SetupSource = Join-Path $PSScriptRoot "JirSetup.cs"
$SetupArgs = '/nologo /target:winexe "/out:' + $Out + '" ' + $Win32 +
    ' /reference:System.Windows.Forms.dll /reference:System.Drawing.dll ' +
    '"/resource:' + $BuiltExe + ',jir.exe" "/resource:' + $Uninstaller + ',uninstall.exe" ' +
    '"' + $SetupSource + '" "' + $VersionInfoSource + '" "' + $DpiSource + '"'
$SetupBuild = Start-Process -FilePath $Csc -ArgumentList $SetupArgs -Wait -PassThru -NoNewWindow
if ($SetupBuild.ExitCode -ne 0) {
    throw "csc failed to build the installer (exit $($SetupBuild.ExitCode))."
}

if (!(Test-Path $Out)) {
    throw "Installer was not created: $Out"
}

Remove-Item $BuildDir -Recurse -Force

Write-Host ""
Write-Host "Installer created:" -ForegroundColor Green
Write-Host "  $Out"
Write-Host "  (icon: $Icon, DPI-aware manifest + layout)"

