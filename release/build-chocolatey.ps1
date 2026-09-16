# Packs the Chocolatey package for a published GitHub release.
#
#   powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.1
#   powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.1 -Push
#
# The checksum describes the bytes users actually download, so it is computed
# from the asset at the release URL - never from a local build.
# Compress-Archive stamps timestamps, so rebuilding the same sources produces
# different bytes, and a local hash silently breaks every install.

param(
    [Parameter(Mandatory = $true)][string]$Version,
    [switch]$Push,
    [string]$ApiKey = $env:CHOCO_API_KEY
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Dist = Join-Path $Root "dist"

$Archive = "jir-$Version-windows-x64.zip"
$Url     = "https://github.com/rururunu/Jir/releases/download/v$Version/$Archive"

$Temp = Join-Path ([System.IO.Path]::GetTempPath()) "jir-choco-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $Temp -Force | Out-Null
try {
    $Downloaded = Join-Path $Temp $Archive
    Write-Host "Downloading $Url" -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Downloaded -UseBasicParsing
    } catch {
        throw "Cannot fetch v$Version from GitHub - publish the release (git push origin v$Version) before packing Chocolatey. $($_.Exception.Message)"
    }
    $Checksum = (Get-FileHash $Downloaded -Algorithm SHA256).Hash.ToLower()
} finally {
    Remove-Item $Temp -Recurse -Force -ErrorAction SilentlyContinue
}
Write-Host "Release checksum: $Checksum"

$Stage = Join-Path $Dist "chocolatey"
if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
New-Item -ItemType Directory -Path (Join-Path $Stage "tools") -Force | Out-Null

foreach ($Rel in @("jir.nuspec", "tools\chocolateyinstall.ps1", "tools\chocolateyuninstall.ps1")) {
    $Text = (Get-Content -Path (Join-Path $Root "chocolatey\$Rel") -Raw).
        Replace("__VERSION__", $Version).
        Replace("__CHECKSUM__", $Checksum)
    Set-Content -Path (Join-Path $Stage $Rel) -Value $Text -Encoding UTF8
}

Push-Location $Stage
try { choco pack "jir.nuspec" --outputdirectory $Dist }
finally { Pop-Location }
if ($LASTEXITCODE -ne 0) { throw "choco pack failed with exit code $LASTEXITCODE" }

$Nupkg = Join-Path $Dist "jir.$Version.nupkg"
if (!(Test-Path $Nupkg)) { throw "Package was not created: $Nupkg" }

# Inspect what was packed: a leftover placeholder or a hash that does not match
# the release is exactly how the first 0.2.0 push shipped broken.
Add-Type -AssemblyName System.IO.Compression.FileSystem
$Zip = [IO.Compression.ZipFile]::OpenRead($Nupkg)
try {
    $Entry = $Zip.Entries | Where-Object { $_.FullName -eq "tools/chocolateyinstall.ps1" } | Select-Object -First 1
    if (-not $Entry) { throw "tools/chocolateyinstall.ps1 is missing from $Nupkg" }
    $Reader = New-Object IO.StreamReader($Entry.Open())
    $Packed = $Reader.ReadToEnd()
    $Reader.Close()
} finally {
    $Zip.Dispose()
}
if ($Packed -match "__CHECKSUM__|__VERSION__") { throw "Placeholders were not substituted in the packed install script." }
if ($Packed -notmatch [regex]::Escape($Checksum)) { throw "The packed install script does not carry the release checksum $Checksum." }
Write-Host "Verified: packed script carries the release checksum." -ForegroundColor Green

Write-Host ""
Write-Host "Chocolatey package: $Nupkg" -ForegroundColor Green

if ($Push) {
    if (-not $ApiKey) { throw "-Push needs an API key via -ApiKey or the CHOCO_API_KEY environment variable." }
    choco apikey --key $ApiKey --source https://push.chocolatey.org/
    if ($LASTEXITCODE -ne 0) { throw "choco apikey failed with exit code $LASTEXITCODE" }
    choco push $Nupkg --source https://push.chocolatey.org/
    if ($LASTEXITCODE -ne 0) { throw "choco push failed with exit code $LASTEXITCODE" }
    Write-Host "Pushed $Nupkg to the community feed. New packages go through moderation." -ForegroundColor Green
}
