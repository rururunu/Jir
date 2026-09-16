$ErrorActionPreference = 'Stop'

# Substituted by release\build-chocolatey.ps1 from dist\SHA256SUMS.txt.
$version  = '__VERSION__'
$checksum = '__CHECKSUM__'

$archive  = "jir-$version-windows-x64.zip"
$url      = "https://github.com/rururunu/Jir/releases/download/v$version/$archive"
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

Install-ChocolateyZipPackage `
    -PackageName   'jir' `
    -Url           $url `
    -Checksum      $checksum `
    -ChecksumType  'sha256' `
    -UnzipLocation $toolsDir

Install-BinFile -Name 'jir' -Path (Join-Path $toolsDir 'jir.exe')

# `choco upgrade` deletes and recreates tools\, so the JDK store has to live
# outside the package directory or every upgrade would take the JDKs with it.
Install-ChocolateyEnvironmentVariable `
    -VariableName  'JIR_HOME' `
    -VariableValue (Join-Path $env:LOCALAPPDATA 'jir\home') `
    -VariableType  'User'

Write-Host "jir $version installed. Run 'jir -h' in a new terminal." -ForegroundColor Green
Write-Host "Set JAVA_HOME to $(Join-Path $env:LOCALAPPDATA 'jir\home\occupy') once a JDK is installed."
