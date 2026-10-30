# Dokime — SPDX-License-Identifier: Apache-2.0
param(
    [Parameter(Mandatory)][ValidateSet('Install', 'Remove')][string]$Action,
    [Parameter(Mandatory)][string]$Program,
    [Parameter(Mandatory)][string]$BundleId
)

$ErrorActionPreference = 'Stop'
try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        $arguments = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ('"{0}"' -f $PSCommandPath),
            '-Action', $Action, '-Program', ('"{0}"' -f $Program), '-BundleId', ('"{0}"' -f $BundleId))
        $child = Start-Process -FilePath "$PSHOME\powershell.exe" -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        exit $child.ExitCode
    }

    $programPath = [IO.Path]::GetFullPath($Program).ToLowerInvariant()
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        $digest = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($programPath))).Replace('-', '')
    } finally {
        $sha.Dispose()
    }
    $ruleName = "$BundleId.remote.$digest"
    $existing = Get-NetFirewallRule -Name $ruleName -ErrorAction SilentlyContinue
    if ($existing) { $existing | Remove-NetFirewallRule }
    if ($Action -eq 'Install') {
        New-NetFirewallRule -Name $ruleName -DisplayName "$BundleId Remote Access" -Direction Inbound -Action Allow `
            -Program $programPath -Profile Private -Enabled True | Out-Null
    }
    exit 0
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
