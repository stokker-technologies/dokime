/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Optional Windows Host Firewall Access

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

// Local Imports
use crate::{
    connection::ConnectionSession,
    native_session::{BackendOwnership, DesktopSessionControl},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Deserialize, Serialize)]
pub struct FirewallStatus {
    required: bool,
    message: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn desktop_firewall(
    app: AppHandle, connection: State<'_, ConnectionSession>, owner: State<'_, DesktopSessionControl>, generation: String, allow: bool,
) -> Result<FirewallStatus, String> {
    let lease = connection.lease(&generation).map_err(|error| format!("{error:?}"))?;
    if !matches!(owner.status().ownership, BackendOwnership::OwnedLocal) {
        return Ok(FirewallStatus { required: false, message: None });
    }
    let listener = lease.client().call(dokime::operations::remote::ReadListener {}).await.map_err(|error| error.to_string())?;
    let address = listener.listening_address.as_deref().and_then(|address| address.parse::<std::net::SocketAddr>().ok());
    let exposed = address.is_some_and(|address| !address.ip().to_canonical().is_loopback());
    let preferences =
        lease.client().call(dokime::operations::settings::ReadApplicationSettings {}).await.map_err(|error| error.to_string())?;
    let enabled = preferences.settings.listener.enabled;
    if allow && (!exposed || !enabled) {
        return Err("Remote access is not listening on a network address.".into());
    }
    let action = if !enabled {
        "Remove"
    } else if allow {
        "Allow"
    } else if exposed {
        "Check"
    } else {
        return Ok(FirewallStatus { required: false, message: None });
    };
    let port = address.map(|address| address.port()).unwrap_or_default();
    connection.lease(&generation).map_err(|error| format!("{error:?}"))?;
    run(app, action, port, address.map(|address| address.ip().to_string()).unwrap_or_default()).await
}

pub async fn remove(app: AppHandle) -> Result<FirewallStatus, String> {
    run(app, "Remove", 0, String::new()).await
}

/* === Private Functions === */

async fn run(app: AppHandle, action: &'static str, port: u16, bind_address: String) -> Result<FirewallStatus, String> {
    #[cfg(not(windows))]
    {
        let _ = (app, action, port, bind_address);
        Ok(FirewallStatus { required: false, message: None })
    }
    #[cfg(windows)]
    {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        use tauri::Manager;
        let program = std::env::current_exe().map_err(|error| error.to_string())?;
        let identifier =
            dokime::paths::distribution_identity(&program, &app.config().identifier).map_err(|error| error.to_string())?.identifier;
        let label = app.state::<super::organization::OrganizationState>().label().to_owned();
        let literal = |value: &str| format!("'{}'", value.replace('\'', "''"));
        let script = format!(
            "$Action={}; $Program={}; $BundleId={}; $Label={}; $BindAddress={}; $Port={port}\n{}",
            literal(action),
            literal(&program.to_string_lossy()),
            literal(&identifier),
            literal(&label),
            literal(&bind_address),
            include_str!("firewall.ps1")
        );
        let encoded = STANDARD.encode(script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>());
        let output = tauri::async_runtime::spawn_blocking(move || {
            let executable = std::env::var_os("SystemRoot")
                .map(std::path::PathBuf::from)
                .ok_or("Windows system directory is unavailable")?
                .join("System32/WindowsPowerShell/v1.0/powershell.exe");
            dokime::domain::process::command(executable)
                .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
                .output()
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| error.to_string())??;
        if !output.status.success() {
            if action == "Check" {
                return Ok(FirewallStatus {
                    required: true,
                    message: Some(
                        "Windows Firewall access could not be confirmed. Ask an administrator to check the listener's network policy."
                            .into(),
                    ),
                });
            }
            if action == "Remove" {
                return Ok(FirewallStatus {
                    required: false,
                    message: Some(
                        "Remote access is disabled. Firewall cleanup could not complete; ask an administrator to remove the app's rule."
                            .into(),
                    ),
                });
            }
            return Err("Windows Firewall could not be updated or inspected. Administrator approval may have been cancelled or policy may prevent changes.".into());
        }
        let result = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(result.trim()).map_err(|_| "Windows Firewall returned an unexpected result.".into())
    }
}

/* === Tests === */

#[cfg(all(test, windows))]
mod tests {
    use std::{io::Write as _, process::Stdio};

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
            let script = format!("{fixture}\n{}", include_str!("firewall.ps1"));
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
            let status: super::FirewallStatus = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(status.required, expected == "blocked", "{scenario}");
            if expected == "blocked" && scenario != "no-network" {
                let message = status.message.unwrap();
                assert!(message.contains("Ethernet"), "{scenario}: {message}");
                if scenario == "public" {
                    assert!(message.contains("Public"));
                }
            }
        }
    }
}
