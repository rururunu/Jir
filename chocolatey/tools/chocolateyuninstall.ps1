$ErrorActionPreference = 'Stop'

function Remove-UserPathEntry {
    param([Parameter(Mandatory = $true)][string]$Entry)

    # The user PATH is read raw so untouched entries keep their %VAR% form, and it
    # is written back with the kind it already had.
    $Key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Environment", $true)
    if (-not $Key) { return }
    try {
        $Current = [string]$Key.GetValue("Path", "", [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $Kind    = [Microsoft.Win32.RegistryValueKind]::ExpandString
        if ($Key.GetValueNames() -contains "Path") { $Kind = $Key.GetValueKind("Path") }

        $Parts = @($Current -split ";" | Where-Object { $_ })
        $Kept  = @($Parts | Where-Object { $_.TrimEnd("\") -ine $Entry.TrimEnd("\") })
        if ($Kept.Count -ne $Parts.Count) { $Key.SetValue("Path", ($Kept -join ";"), $Kind) }
    } finally { $Key.Close() }
}

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

Uninstall-BinFile -Name 'jir'

$jirHome = Join-Path $env:LOCALAPPDATA 'jir\home'
$occupy  = Join-Path $jirHome 'occupy'

# JAVA_HOME and %JAVA_HOME%\bin are only ours to remove while they are still what
# the install script left behind: a user who pointed JAVA_HOME at a JDK of their
# own keeps it, and their %JAVA_HOME%\bin entry keeps meaning their JDK.
if ([Environment]::GetEnvironmentVariable('JAVA_HOME', 'User') -ieq $occupy) {
    Uninstall-ChocolateyEnvironmentVariable -VariableName 'JAVA_HOME' -VariableType 'User'
    Remove-UserPathEntry -Entry '%JAVA_HOME%\bin'
}
# absolute form, for a PATH entry added by the GUI installer or by hand
Remove-UserPathEntry -Entry (Join-Path $occupy 'bin')
Send-EnvironmentChange

if ([Environment]::GetEnvironmentVariable('JIR_HOME', 'User')) {
    Uninstall-ChocolateyEnvironmentVariable -VariableName 'JIR_HOME' -VariableType 'User'
}

# Uninstalling the tool must not delete several hundred MB of downloaded JDKs.
if (Test-Path $jirHome) {
    Write-Host "Kept $jirHome (installed JDKs). Delete that folder to reclaim disk space." -ForegroundColor Yellow
}
