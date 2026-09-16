# Packs the Chocolatey package for an already-built release archive.
#
#   powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.0
#   powershell -ExecutionPolicy Bypass -File .\release\build-chocolatey.ps1 -Version 0.2.0 -Push
#
# Run release\build-portable.ps1 first: the checksum is read from its output so
# the package can never point at an archive it was not built from.

param(
    [Parameter(Mandatory = $true)][string]$Version,
    [switch]$Push,
    [string]$ApiKey = $env:CHOCO_API_KEY
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Dist = Join-Path $Root "dist"
$Sums = Join-Path $Dist "SHA256SUMS.txt"
if (!(Test-Path $Sums)) { throw "$Sums is missing - run release\build-portable.ps1 -Version $Version first." }

$Archive = "jir-$Version-windows-x64.zip"
$Line    = Select-String -Path $Sums -SimpleMatch $Archive | Select-Object -First 1
if (-not $Line) { throw "SHA256SUMS.txt has no entry for $Archive." }
$Checksum = ($Line.Line -split "\s+")[0].ToLower()

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
