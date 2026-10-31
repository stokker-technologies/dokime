# Dokime — SPDX-License-Identifier: Apache-2.0
# Inputs Are Native-Owned Literals Prepended By The Desktop.
$ErrorActionPreference = 'Stop'
try {
    $programPath = [IO.Path]::GetFullPath($Program)
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        $digest = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($programPath.ToLowerInvariant()))).Replace('-', '')
    } finally { $sha.Dispose() }
    $ruleName = "$BundleId.remote.$digest"
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    $admin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if ($Action -eq 'Remove') {
        $existing = Get-NetFirewallRule -PolicyStore PersistentStore -ErrorAction Stop | Where-Object Name -EQ $ruleName
        if (-not $existing) { 'inactive'; exit 0 }
        if (-not $admin) { 'skipped'; exit 0 }
        $existing | Remove-NetFirewallRule
        'inactive'; exit 0
    }

    if ($Action -eq 'Allow') {
        # Elevate Only This Explicit Action; Pass Fixed Code And Escaped Native Values.
        $quote = { param($value) "'" + $value.Replace("'", "''") + "'" }
        $helper = '$ErrorActionPreference="Stop"; try {{ Get-NetFirewallRule -PolicyStore PersistentStore | Where-Object Name -EQ {0} | Remove-NetFirewallRule; New-NetFirewallRule -Name {0} -DisplayName {1} -Direction Inbound -Action Allow -Program {2} -Profile Private -Enabled True | Out-Null; exit 0 }} catch {{ exit 1 }}' -f `
            (& $quote $ruleName), (& $quote "$Label Remote Access"), (& $quote $programPath)
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($helper))
        $child = Start-Process -FilePath "$PSHOME\powershell.exe" -ArgumentList @('-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded) `
            -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        if ($child.ExitCode -ne 0) { exit 1 }
    }

    # Inspect Effective Policy, Including Explicit Blocks And Active Network Profiles.
    $profiles = @(Get-NetConnectionProfile | ForEach-Object {
        if ($_.NetworkCategory -eq 'DomainAuthenticated') { 'Domain' } else { [string]$_.NetworkCategory }
    } | Select-Object -Unique)
    if (-not $profiles.Count) { $profiles = @('Private') }
    $rules = @(Get-NetFirewallRule -PolicyStore ActiveStore -TracePolicyStore -Enabled True -Direction Inbound)
    foreach ($profile in $profiles) {
        $policy = Get-NetFirewallProfile -PolicyStore ActiveStore -Name $profile
        if ($policy.Enabled -eq 'False') { continue }
        if ($policy.AllowInboundRules -eq 'False') { 'blocked'; exit 0 }
        $allowed = $policy.DefaultInboundAction -eq 'Allow'
        foreach ($rule in $rules) {
            if ($policy.AllowLocalFirewallRules -eq 'False' -and $rule.PolicyStoreSourceType -eq 'Local') { continue }
            if ($rule.Profile -ne 'Any' -and ([string]$rule.Profile -split ',\s*') -notcontains $profile) { continue }
            $application = $rule | Get-NetFirewallApplicationFilter
            $path = [Environment]::ExpandEnvironmentVariables($application.Program)
            if ($path -ne 'Any' -and $path -ne $programPath) { continue }
            $ports = $rule | Get-NetFirewallPortFilter
            if ($ports.Protocol -notin @('Any', 'TCP', '6')) { continue }
            $matchesPort = @($ports.LocalPort) -contains 'Any'
            foreach ($range in $ports.LocalPort) {
                if ($range -eq [string]$Port) { $matchesPort = $true }
                if ($range -match '^(\d+)-(\d+)$' -and $Port -ge [int]$Matches[1] -and $Port -le [int]$Matches[2]) { $matchesPort = $true }
            }
            if (-not $matchesPort) { continue }
            $service = $rule | Get-NetFirewallServiceFilter
            if ($service.Service -ne 'Any' -or $application.Package -notin @($null, '', 'Any')) { continue }
            if ($rule.Action -eq 'Block') { 'blocked'; exit 0 }
            $addresses = $rule | Get-NetFirewallAddressFilter
            # Restricted Allows Do Not Establish Reachability For Other PCs.
            if ($rule.Action -eq 'Allow' -and @($addresses.LocalAddress) -contains 'Any' `
                -and (@($addresses.RemoteAddress) -contains 'Any' -or @($addresses.RemoteAddress) -contains 'LocalSubnet') `
                -and @($ports.RemotePort) -contains 'Any') { $allowed = $true }
        }
        if (-not $allowed) { 'blocked'; exit 0 }
    }
    'allowed'
    exit 0
} catch {
    exit 1
}
