$ErrorActionPreference = 'Stop'

Uninstall-BinFile -Name 'jir'

if ([Environment]::GetEnvironmentVariable('JIR_HOME', 'User')) {
    Uninstall-ChocolateyEnvironmentVariable -VariableName 'JIR_HOME' -VariableType 'User'
}

# Uninstalling the tool must not delete several hundred MB of downloaded JDKs.
$jirHome = Join-Path $env:LOCALAPPDATA 'jir\home'
if (Test-Path $jirHome) {
    Write-Host "Kept $jirHome (installed JDKs). Delete that folder to reclaim disk space." -ForegroundColor Yellow
}
