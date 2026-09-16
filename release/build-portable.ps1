# Builds the portable Windows x64 archive that the curl/wget and Chocolatey
# channels install, together with the checksum file they verify against.
#
#   powershell -ExecutionPolicy Bypass -File .\release\build-portable.ps1 -Version 0.2.0

param(
    [string]$Version = "0.2.0"
)

$ErrorActionPreference = "Stop"

$Root  = Resolve-Path (Join-Path $PSScriptRoot "..")
$Dist  = Join-Path $Root "dist"
$Stage = Join-Path $Dist "portable"
$Name  = "jir-$Version-windows-x64.zip"
$Zip   = Join-Path $Dist $Name
$Sums  = Join-Path $Dist "SHA256SUMS.txt"

Write-Host "Building jir $Version..." -ForegroundColor Cyan
Push-Location $Root
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed with exit code $LASTEXITCODE" }
    $Metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
} finally {
    Pop-Location
}

$BuiltExe = Join-Path $Metadata.target_directory "release\jir-cli.exe"
if (!(Test-Path $BuiltExe)) { throw "Build output not found: $BuiltExe" }

New-Item -ItemType Directory -Path $Dist -Force | Out-Null
if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
New-Item -ItemType Directory -Path $Stage -Force | Out-Null

# The GUI installer renames jir-cli.exe to jir.exe; every channel must agree on
# the binary name that ends up on PATH. `home/` is deliberately not shipped:
# runtime state is located through JIR_HOME instead.
Copy-Item $BuiltExe (Join-Path $Stage "jir.exe")
foreach ($File in @("LICENSE", "README.md")) {
    Copy-Item (Join-Path $Root $File) (Join-Path $Stage $File)
}

if (Test-Path $Zip) { Remove-Item $Zip -Force }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -CompressionLevel Optimal
Remove-Item $Stage -Recurse -Force

$Hash = (Get-FileHash $Zip -Algorithm SHA256).Hash.ToLower()
"$Hash  $Name" | Set-Content -Path $Sums -Encoding ascii

Write-Host ""
Write-Host "Portable archive: $Zip" -ForegroundColor Green
Write-Host "Checksums:        $Sums" -ForegroundColor Green
