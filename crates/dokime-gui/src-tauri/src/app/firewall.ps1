# Dokime — SPDX-License-Identifier: Apache-2.0
# Inputs Are Native-Owned Literals Prepended By The Desktop.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
function Write-Status($required, $message) {
    @{ required = [bool]$required; message = $message } | ConvertTo-Json -Compress
}
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
        if (-not $existing) { Write-Status $false $null; exit 0 }
        if (-not $admin) {
            Write-Status $false 'Remote access is disabled. The firewall rule was left in place because removing it needs an administrator.'
            exit 0
        }
        $existing | Remove-NetFirewallRule
        Write-Status $false $null; exit 0
    }

    # Match Active Networks To The Listener's Address And Address Family.
    $listen = [Net.IPAddress]::Parse($BindAddress)
    $wildcard = $listen.Equals([Net.IPAddress]::Any) -or $listen.Equals([Net.IPAddress]::IPv6Any)
    $localAddress = if ($wildcard) { 'Any' } else { $BindAddress }
    $listenerAddresses = @(Get-NetIPAddress | Where-Object {
        $ip = [Net.IPAddress]::Parse($_.IPAddress)
        $ip.AddressFamily -eq $listen.AddressFamily -and -not [Net.IPAddress]::IsLoopback($ip) `
            -and $_.AddressState -notin @('Invalid', 'Tentative', 'Duplicate') -and ($wildcard -or $ip.Equals($listen))
    })
    $networks = @(Get-NetConnectionProfile | Where-Object { $_.InterfaceIndex -in $listenerAddresses.InterfaceIndex })
    if (-not $networks.Count) {
        if ($Action -eq 'Allow') { throw 'No active listener network could be identified.' }
        Write-Status $true 'The listener has no identified active network. Connect the cable and check its address before allowing firewall access.'
        exit 0
    }
    $profiles = @($networks | ForEach-Object {
        if ($_.NetworkCategory -eq 'DomainAuthenticated') { 'Domain' } else { [string]$_.NetworkCategory }
    } | Select-Object -Unique)
    $description = ($networks | ForEach-Object {
        $category = if ($_.NetworkCategory -eq 'DomainAuthenticated') { 'Domain' } else { [string]$_.NetworkCategory }
        "$($_.InterfaceAlias) ($($_.Name), $category network)"
    }) -join '; '

    if ($Action -eq 'Allow') {
        # Elevate Only This Explicit Action; Pass Fixed Code And Escaped Native Values.
        $quote = { param($value) "'" + $value.Replace("'", "''") + "'" }
        $profileArguments = ($profiles | ForEach-Object { & $quote $_ }) -join ','
        $helper = '$ErrorActionPreference="Stop"; try {{ Get-NetFirewallRule -PolicyStore PersistentStore | Where-Object Name -EQ {0} | Remove-NetFirewallRule; New-NetFirewallRule -Name {0} -DisplayName {1} -Direction Inbound -Action Allow -Program {2} -Protocol TCP -LocalPort {3} -LocalAddress {4} -RemoteAddress LocalSubnet -Profile {5} -Enabled True | Out-Null; exit 0 }} catch {{ exit 1 }}' -f `
            (& $quote $ruleName), (& $quote "$Label Remote Access"), (& $quote $programPath), $Port, (& $quote $localAddress), $profileArguments
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($helper))
        $child = Start-Process -FilePath "$PSHOME\powershell.exe" -ArgumentList @('-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded) `
            -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        if ($child.ExitCode -ne 0) { exit 1 }
    }

    # Inspect Effective Policy, Including Explicit Blocks And Active Network Profiles.
    $rules = @(Get-NetFirewallRule -PolicyStore ActiveStore -TracePolicyStore -Enabled True -Direction Inbound)
    foreach ($network in $networks) {
        $networkProfile = if ($network.NetworkCategory -eq 'DomainAuthenticated') { 'Domain' } else { [string]$network.NetworkCategory }
        $affected = "$($network.InterfaceAlias) ($($network.Name), $networkProfile network)"
        $blocked = "Windows Firewall blocks remote access on $affected. Allow this app and listener port for PCs on the same subnet; administrator approval is required."
        $policy = Get-NetFirewallProfile -PolicyStore ActiveStore -Name $networkProfile
        if ($policy.Enabled -eq 'False') { continue }
        if ($policy.AllowInboundRules -eq 'False') { Write-Status $true $blocked; exit 0 }
        $allowed = $policy.DefaultInboundAction -eq 'Allow'
        foreach ($rule in $rules) {
            if ($policy.AllowLocalFirewallRules -eq 'False' -and $rule.PolicyStoreSourceType -eq 'Local') { continue }
            if ($rule.Profile -ne 'Any' -and ([string]$rule.Profile -split ',\s*') -notcontains $networkProfile) { continue }
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
            $addresses = $rule | Get-NetFirewallAddressFilter
            if (-not (@($addresses.LocalAddress) -contains 'Any' -or (-not $wildcard -and @($addresses.LocalAddress) -contains $BindAddress))) { continue }
            if ($rule.Action -eq 'Block') { Write-Status $true $blocked; exit 0 }
            # Restricted Allows Do Not Establish Reachability For Other PCs.
            if ($rule.Action -eq 'Allow' `
                -and (@($addresses.RemoteAddress) -contains 'Any' -or @($addresses.RemoteAddress) -contains 'LocalSubnet') `
                -and @($ports.RemotePort) -contains 'Any') { $allowed = $true }
        }
        if (-not $allowed) { Write-Status $true $blocked; exit 0 }
    }
    if ($Action -eq 'Allow') {
        Write-Status $false "Windows Firewall allows this app's listener for PCs on the same subnet on $description."
    } else { Write-Status $false $null }
    exit 0
} catch {
    exit 1
}
