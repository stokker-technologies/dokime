/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_gui::testing::FirewallStatus;
use std::{io::Write as _, process::Stdio};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn firewall_effective_policy_checks_program_port_profile_and_blocks() {
    for (scenario, expected) in [
        ("allow", "allowed"),
        ("missing", "blocked"),
        ("block", "blocked"),
        ("other-program", "blocked"),
        ("other-port", "blocked"),
        ("public", "blocked"),
        ("domain", "allowed"),
        ("managed", "blocked"),
        ("disabled", "allowed"),
        ("package-empty", "allowed"),
        ("local-subnet", "allowed"),
        ("allow-action", "allowed"),
        ("allow-public", "allowed"),
        ("allow-multiple", "allowed"),
        ("specific-private", "allowed"),
        ("no-network", "blocked"),
        ("cancel", "error"),
        ("remove", "cleanup"),
    ] {
        let fixture = format!(
            r#"
            $Action='Check'; $Program="C:\Fixture's\app.exe"; $BundleId='fixture'; $Label="Fixture's Lab"; $Port=8765; $BindAddress='0.0.0.0'
            $Scenario='{scenario}'
            $global:Applied=$false
            if ($Scenario -in @('allow-action', 'allow-public', 'allow-multiple', 'specific-private', 'cancel')) {{ $Action='Allow' }}
            if ($Scenario -eq 'specific-private') {{ $BindAddress='192.168.7.4' }}
            if ($Scenario -eq 'remove') {{ $Action='Remove' }}
            function Start-Process {{
                param($FilePath, $ArgumentList, $Verb, $WindowStyle, [switch]$Wait, [switch]$PassThru)
                if ($Action -ne 'Allow' -or $Scenario -eq 'cancel') {{ throw 'unexpected or cancelled elevation' }}
                $helper = [Text.Encoding]::Unicode.GetString([Convert]::FromBase64String($ArgumentList[-1]))
                & ([scriptblock]::Create($helper.Replace('exit 0', 'return').Replace('exit 1', 'throw')))
                if (-not $global:Applied) {{ throw 'no rule was created' }}
                [pscustomobject]@{{ ExitCode=0 }}
            }}
            function Remove-NetFirewallRule {{ param([Parameter(ValueFromPipeline=$true)]$InputObject) }}
            function New-NetFirewallRule {{
                param($Name, $DisplayName, $Direction, $Action, $Program, $Protocol, $LocalPort, $LocalAddress, $RemoteAddress, [Alias('Profile')]$RuleProfiles, $Enabled)
                if ($Program -ne "C:\Fixture's\app.exe" -or $DisplayName -ne "Fixture's Lab Remote Access") {{ throw 'native strings were changed' }}
                if ($Direction -ne 'Inbound' -or $Action -ne 'Allow' -or $Protocol -ne 'TCP' -or $LocalPort -ne 8765 -or $Enabled -ne 'True') {{ throw 'wrong listener rule' }}
                if ($RemoteAddress -ne 'LocalSubnet') {{ throw 'rule must restrict peers to local subnet' }}
                $expectedProfiles = if ($Scenario -eq 'allow-public') {{ @('Public') }} elseif ($Scenario -eq 'allow-multiple') {{ @('Private','Public') }} else {{ @('Private') }}
                if (@(Compare-Object @($RuleProfiles) $expectedProfiles).Count) {{ throw 'wrong served profiles' }}
                $expectedAddress = if ($Scenario -eq 'specific-private') {{ '192.168.7.4' }} else {{ 'Any' }}
                if ($LocalAddress -ne $expectedAddress) {{ throw 'wrong bound address' }}
                $global:Applied=$true
            }}
            function Get-NetIPAddress {{
                if ($Scenario -eq 'no-network') {{ return }}
                [pscustomobject]@{{ IPAddress='192.168.7.4'; InterfaceIndex=7; AddressState='Preferred' }}
                [pscustomobject]@{{ IPAddress='169.254.23.41'; InterfaceIndex=8; AddressState='Preferred' }}
                [pscustomobject]@{{ IPAddress='fe80::1'; InterfaceIndex=9; AddressState='Preferred' }}
            }}
            function Get-NetConnectionProfile {{
                $category = if ($Scenario -in @('public', 'allow-public')) {{ 'Public' }} elseif ($Scenario -eq 'domain') {{ 'DomainAuthenticated' }} else {{ 'Private' }}
                [pscustomobject]@{{ InterfaceIndex=7; InterfaceAlias='Ethernet'; Name='Unidentified network'; NetworkCategory=$category }}
                if ($Scenario -in @('allow-multiple', 'specific-private')) {{
                    [pscustomobject]@{{ InterfaceIndex=8; InterfaceAlias='Cable'; Name='Unidentified network'; NetworkCategory='Public' }}
                }}
                [pscustomobject]@{{ InterfaceIndex=9; InterfaceAlias='IPv6 Only'; Name='Other network'; NetworkCategory='Public' }}
            }}
            function Get-NetFirewallProfile {{
                [pscustomobject]@{{ Enabled = $(if ($Scenario -eq 'disabled') {{ 'False' }} else {{ 'True' }}); DefaultInboundAction='Block'; AllowInboundRules='True'; AllowLocalFirewallRules=$(if ($Scenario -eq 'managed') {{ 'False' }} else {{ 'True' }}) }}
            }}
            function Get-NetFirewallRule {{
                param($PolicyStore, [switch]$TracePolicyStore, $Enabled, $Direction)
                if ($PolicyStore -eq 'PersistentStore') {{ return }}
                if ($Scenario -eq 'missing') {{ return }}
                $ruleProfile = if ($global:Applied) {{ $profiles -join ', ' }} elseif ($Scenario -eq 'domain') {{ 'Domain' }} else {{ 'Private' }}
                [pscustomobject]@{{ Name='unrelated-rule'; Profile=$ruleProfile; PolicyStoreSourceType='Local'; Action='Allow' }}
                if ($Scenario -eq 'block') {{ [pscustomobject]@{{ Profile='Private'; PolicyStoreSourceType='Local'; Action='Block' }} }}
            }}
            function Get-NetFirewallApplicationFilter {{ [pscustomobject]@{{ Program=$(if ($Scenario -eq 'other-program') {{ 'C:\Other\app.exe' }} else {{ $Program }}); Package=$(if ($Scenario -ne 'package-empty') {{ 'Any' }}) }} }}
            function Get-NetFirewallPortFilter {{ [pscustomobject]@{{ Protocol='TCP'; LocalPort=$(if ($Scenario -eq 'other-port') {{ '99' }} else {{ '8765' }}); RemotePort='Any' }} }}
            function Get-NetFirewallAddressFilter {{ [pscustomobject]@{{ LocalAddress=$(if ($global:Applied) {{ $localAddress }} else {{ 'Any' }}); RemoteAddress=$(if ($global:Applied -or $Scenario -eq 'local-subnet') {{ 'LocalSubnet' }} else {{ 'Any' }}) }} }}
            function Get-NetFirewallServiceFilter {{ [pscustomobject]@{{ Service='Any' }} }}
        "#
        );
        let script = format!("{fixture}\n{}", include_str!("../../../src/app/firewall.ps1"));
        let executable =
            std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut child = dokime::domain::process::command(executable)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::InputEncoding=[Text.UTF8Encoding]::new($false); & ([scriptblock]::Create([Console]::In.ReadToEnd()))",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(script.as_bytes()).unwrap();
        let result = child.wait_with_output().unwrap();
        if expected == "error" {
            assert!(!result.status.success(), "cancelled elevation must fail without applying a rule");
            continue;
        }
        assert!(result.status.success(), "{scenario}: {}", String::from_utf8_lossy(&result.stderr));
        let status: FirewallStatus = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(status.inspect().0, expected == "blocked", "{scenario}");
        if expected == "blocked" && scenario != "no-network" {
            let message = status.inspect().1.unwrap();
            assert!(message.contains("Ethernet"), "{scenario}: {message}");
            if scenario == "public" {
                assert!(message.contains("Public"));
            }
        }
    }
}
