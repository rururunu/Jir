$ErrorActionPreference = 'Stop'

# Substituted by release\build-chocolatey.ps1 from dist\SHA256SUMS.txt.
$version  = '__VERSION__'
$checksum = '__CHECKSUM__'

$archive  = "jir-$version-windows-x64.zip"
$url      = "https://github.com/rururunu/Jir/releases/download/v$version/$archive"
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$jirHome  = Join-Path $env:LOCALAPPDATA 'jir\home'
$occupy   = Join-Path $jirHome 'occupy'

function Add-UserPathEntry {
    param([Parameter(Mandatory = $true)][string]$Entry)

    # Stored as %JAVA_HOME%\bin rather than as an absolute path so a later
    # `jir use` needs no PATH edit. Windows only expands it when the value is an
    # expandable string, which is why the registry is written directly instead of
    # going through Install-ChocolateyPath.
    $Key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Environment", $true)
    if (-not $Key) { throw "Cannot open HKCU\Environment for writing." }
    try {
        $Current = [string]$Key.GetValue("Path", "", [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $Kind    = [Microsoft.Win32.RegistryValueKind]::ExpandString
        if ($Key.GetValueNames() -contains "Path") { $Kind = $Key.GetValueKind("Path") }

        $Parts  = @($Current -split ";" | Where-Object { $_ })
        $Wanted = [Environment]::ExpandEnvironmentVariables($Entry)
        foreach ($Part in $Parts) {
            if ($Part.TrimEnd("\") -ieq $Entry.TrimEnd("\")) { return }
            # an earlier installer may have added the same directory as an
            # absolute path - do not put it on PATH a second time
            if ([Environment]::ExpandEnvironmentVariables($Part).TrimEnd("\") -ieq $Wanted.TrimEnd("\")) { return }
        }
        $Key.SetValue("Path", (($Parts + $Entry) -join ";"), $Kind)
    } finally { $Key.Close() }
}

# Explorer caches the environment block, so without this broadcast a terminal
# opened from the Start menu still would not see JAVA_HOME.
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
    -VariableValue $jirHome `
    -VariableType  'User'

# JAVA_HOME points at the stable `occupy` junction rather than at a JDK: that is
# the whole point of jir, and `jir use 21` re-points it without another environment
# edit. occupy only appears once a JDK is activated, which is why PATH gets the
# %JAVA_HOME%\bin form instead of a directory that exists today.
$Existing = [Environment]::GetEnvironmentVariable('JAVA_HOME', 'User')
if ($Existing -and $Existing.TrimEnd('\') -ine $occupy.TrimEnd('\')) {
    Write-Host "Replacing the existing JAVA_HOME ($Existing) with $occupy." -ForegroundColor Yellow
}
Install-ChocolateyEnvironmentVariable `
    -VariableName  'JAVA_HOME' `
    -VariableValue $occupy `
    -VariableType  'User'
# so the duplicate check below can expand %JAVA_HOME% in an existing PATH entry
$env:JAVA_HOME = $occupy
Add-UserPathEntry -Entry '%JAVA_HOME%\bin'
Send-EnvironmentChange

Write-Host "jir $version installed. Run 'jir -h' in a new terminal." -ForegroundColor Green
Write-Host "JAVA_HOME points at $occupy - run 'jir i 21' and 'jir use 21' to put a JDK there."
