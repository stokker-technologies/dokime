#Requires -Version 5.1
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Tool Versions
$POWERSHELL_VERSION = '7.6.7'
$POWERSHELL_HASHES = @{
    x64 = 'd2f952ebdbf24386cb8923553aa58e41a846db35a6155ea04c58142a4b16dbb7'
    arm64 = '119bd9221254ef3637523cb1990797618f5fd68848b57de40f12b561d743149e'
}

# Official Installation Guidance
# https://learn.microsoft.com/powershell/scripting/install/install-powershell-on-windows
# https://mise.jdx.dev/installing-mise.html#windows
# https://mise.jdx.dev/shell-setup.html

function Add-UserPath([string]$Directory) {
    # Preserve Unexpanded Variables And Existing User Entries
    $EnvironmentKey = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
    try {
        $RawPath = [string]$EnvironmentKey.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $Entries = @($RawPath -split ';' | Where-Object { $_ })
        $Present = @($Entries | ForEach-Object {
            [Environment]::ExpandEnvironmentVariables($_).Trim('"').TrimEnd('\')
        })
        if ($Present -notcontains $Directory.TrimEnd('\')) {
            $NewPath = (@($Directory) + $Entries) -join ';'
            $EnvironmentKey.SetValue('Path', $NewPath, [Microsoft.Win32.RegistryValueKind]::ExpandString)
        }
    } finally {
        $EnvironmentKey.Dispose()
    }
    $ProcessEntries = @($env:Path -split ';' | Where-Object {
        $_ -and $_.Trim('"').TrimEnd('\') -ne $Directory.TrimEnd('\')
    })
    $env:Path = (@($Directory) + $ProcessEntries) -join ';'
}

function Invoke-Mise([string[]]$Arguments) {
    & $script:Mise @Arguments
    if ($LASTEXITCODE -ne 0) { throw "mise Failed With Exit Code $LASTEXITCODE" }
}

# Check Native Windows Prerequisites Before Changing Anything
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Run This Bootstrap On Windows; Use The Official Tool Installers On Other Platforms'
}
if ([Environment]::OSVersion.Version -lt [version]'10.0.14393') {
    throw 'Windows 10 Version 1607 Or Newer Is Required; Install OS Updates First'
}
if (-not $env:LOCALAPPDATA -or -not $env:USERPROFILE) {
    throw 'LOCALAPPDATA And USERPROFILE Must Identify Your Windows User Profile'
}
$Architecture = $env:PROCESSOR_ARCHITECTURE
if ($env:PROCESSOR_ARCHITEW6432) { $Architecture = $env:PROCESSOR_ARCHITEW6432 }
switch ($Architecture) {
    'AMD64' { $Architecture = 'x64' }
    'ARM64' { $Architecture = 'arm64' }
    default { throw 'The Windows Bootstrap Requires An x64 Or ARM64 Operating System' }
}

# Use TLS 1.2 On Older Windows PowerShell Images Without Changing Certificate Trust
$PreviousProtocol = [Net.ServicePointManager]::SecurityProtocol
$PreviousProgress = $ProgressPreference
$PreviousInstallerEnvironment = @{}
foreach ($Name in @('MISE_INSTALL_PATH', 'MISE_INSTALL_MODIFY_PATH', 'MISE_INSTALL_SHIMS_PATH')) {
    $PreviousInstallerEnvironment[$Name] = [Environment]::GetEnvironmentVariable($Name, 'Process')
}
$Work = Join-Path ([IO.Path]::GetTempPath()) ('dokime-bootstrap-' + [guid]::NewGuid().ToString('N'))

try {
    [Net.ServicePointManager]::SecurityProtocol = $PreviousProtocol -bor [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = 'SilentlyContinue'
    New-Item -ItemType Directory -Path $Work | Out-Null

    # Install The Verified Self-Contained PowerShell ZIP For This User
    $PowerShellDirectory = Join-Path $env:LOCALAPPDATA "Programs\PowerShell\$POWERSHELL_VERSION-$Architecture"
    $Pwsh = Join-Path $PowerShellDirectory 'pwsh.exe'
    if (-not (Test-Path -LiteralPath $PowerShellDirectory)) {
        $Archive = Join-Path $Work 'powershell.zip'
        $Asset = "PowerShell-$POWERSHELL_VERSION-win-$Architecture.zip"
        $Url = "https://github.com/PowerShell/PowerShell/releases/download/v$POWERSHELL_VERSION/$Asset"
        Write-Host "Installing PowerShell $POWERSHELL_VERSION For The Current User"
        Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Archive
        $Digest = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash
        if ($Digest -ne $POWERSHELL_HASHES[$Architecture]) { throw 'PowerShell ZIP Checksum Mismatch; Installation Refused' }
        $Extracted = Join-Path $Work 'powershell'
        Expand-Archive -LiteralPath $Archive -DestinationPath $Extracted
        Get-ChildItem -LiteralPath $Extracted -File -Recurse | Unblock-File
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $PowerShellDirectory) | Out-Null
        Move-Item -LiteralPath $Extracted -Destination $PowerShellDirectory
    }
    $InstalledVersion = & $Pwsh -NoLogo -NoProfile -NonInteractive -Command '$PSVersionTable.PSVersion.ToString()'
    if ($LASTEXITCODE -ne 0 -or $InstalledVersion -ne $POWERSHELL_VERSION) {
        throw "PowerShell Did Not Start As $POWERSHELL_VERSION; Check OS Prerequisites And $PowerShellDirectory"
    }
    Add-UserPath $PowerShellDirectory

    # The Official Windows Installer Verifies Mise And Installs Its Companion Shim
    # It Supports PowerShell 5.1, Preserves User PATH And Broadcasts Environment Changes.
    $Mise = Join-Path $env:USERPROFILE '.local\bin\mise.exe'
    $env:MISE_INSTALL_PATH = $Mise
    $env:MISE_INSTALL_MODIFY_PATH = '1'
    $env:MISE_INSTALL_SHIMS_PATH = '1'
    $Installer = (Invoke-WebRequest -UseBasicParsing -Uri 'https://mise.run/powershell').Content
    if ($Installer -is [byte[]]) { $Installer = [Text.Encoding]::UTF8.GetString($Installer) }
    & ([scriptblock]::Create([string]$Installer))
    if (-not (Test-Path -LiteralPath $Mise -PathType Leaf)) { throw 'The Mise Installer Did Not Produce mise.exe' }

    # Install uv Globally Without Trusting Or Installing Any Local Project
    Push-Location -LiteralPath $env:USERPROFILE
    try {
        Invoke-Mise -Arguments @('use', '-g', 'uv')
        Invoke-Mise -Arguments @('reshim')
        Invoke-Mise -Arguments @('exec', 'uv', '--', 'uv', '--version')
    } finally {
        Pop-Location
    }

    # Activate Project Environments In PowerShell 7; Shims Also Serve cmd.exe And Editors
    $ProfilePath = & $Pwsh -NoLogo -NoProfile -NonInteractive -Command '$PROFILE.CurrentUserAllHosts'
    if ($LASTEXITCODE -ne 0 -or -not $ProfilePath) { throw 'Cannot Locate The PowerShell 7 User Profile' }
    $Activation = "(& '" + $Mise.Replace("'", "''") + "' activate pwsh) | Out-String | Invoke-Expression"
    $ProfileText = ''
    if (Test-Path -LiteralPath $ProfilePath) { $ProfileText = [IO.File]::ReadAllText($ProfilePath) }
    if (($ProfileText -split "`r?`n") -notcontains $Activation) {
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $ProfilePath) | Out-Null
        $ProfileText = $ProfileText.TrimEnd("`r", "`n") + "`r`n# Activate Mise`r`n$Activation`r`n"
        [IO.File]::WriteAllText($ProfilePath, $ProfileText, [Text.UTF8Encoding]::new($true))
    }

    # Notify Desktop Applications Even When Mise Was Already Installed
    if (-not ('DokimeBootstrap.EnvironmentChange' -as [type])) {
        Add-Type -Namespace DokimeBootstrap -Name EnvironmentChange -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("user32.dll", CharSet = System.Runtime.InteropServices.CharSet.Auto)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr window, uint message,
    System.UIntPtr parameter, string setting, uint flags, uint timeout, out System.UIntPtr result);
'@
    }
    $NotificationResult = [UIntPtr]::Zero
    [void][DokimeBootstrap.EnvironmentChange]::SendMessageTimeout(
        [IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 2000, [ref]$NotificationResult)

    Write-Host 'Bootstrap Complete. Close And Reopen Your Terminal Or Editor, Then Run:'
    Write-Host '  pwsh --version'
    Write-Host '  mise doctor'
    Write-Host '  uv --version'
    Write-Host "PowerShell: $Pwsh"
    Write-Host "Mise: $Mise"
    Write-Host "PowerShell Profile: $ProfilePath"
} catch {
    Write-Error -ErrorAction Continue "Bootstrap Failed: $_. Check HTTPS Access To GitHub And mise.run, OS Updates And Your Organization's Script Policy."
    exit 1
} finally {
    [Net.ServicePointManager]::SecurityProtocol = $PreviousProtocol
    $ProgressPreference = $PreviousProgress
    foreach ($Name in $PreviousInstallerEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($Name, $PreviousInstallerEnvironment[$Name], 'Process')
    }
    if (Test-Path -LiteralPath $Work) { Remove-Item -LiteralPath $Work -Recurse -Force }
}
