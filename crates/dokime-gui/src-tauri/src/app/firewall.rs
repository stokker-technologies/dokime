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
use serde::Serialize;
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

#[derive(Serialize)]
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
    run(app, action, port).await
}

pub async fn remove(app: AppHandle) -> Result<FirewallStatus, String> {
    run(app, "Remove", 0).await
}

/* === Private Functions === */

async fn run(app: AppHandle, action: &'static str, port: u16) -> Result<FirewallStatus, String> {
    #[cfg(not(windows))]
    {
        let _ = (app, action, port);
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
            "$Action={}; $Program={}; $BundleId={}; $Label={}; $Port={port}\n{}",
            literal(action),
            literal(&program.to_string_lossy()),
            literal(&identifier),
            literal(&label),
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
                        "Windows Firewall access could not be confirmed. Ask an administrator to check private-network policy.".into(),
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
        let result = result.trim();
        match result {
            "blocked" => Ok(FirewallStatus { required: true, message: None }),
            "skipped" => Ok(FirewallStatus {
                required: false,
                message: Some(
                    "Remote access is disabled. The firewall rule was left in place because removing it needs an administrator.".into(),
                ),
            }),
            "allowed" | "inactive" => Ok(FirewallStatus { required: false, message: None }),
            _ => Err("Windows Firewall returned an unexpected result.".into()),
        }
    }
}

/* === Tests === */

#[cfg(all(test, windows))]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

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
            ("cancel", "error"),
            ("remove", "cleanup"),
        ] {
            let fixture = format!(
                r#"
                $Action='Check'; $Program="C:\Fixture's\app.exe"; $BundleId='fixture'; $Label="Fixture's Lab"; $Port=8765
                $Scenario='{scenario}'
                if ($Scenario -in @('allow-action', 'cancel')) {{ $Action='Allow' }}
                if ($Scenario -eq 'remove') {{ $Action='Remove' }}
                function Start-Process {{
                    param($FilePath, $ArgumentList, $Verb, $WindowStyle, [switch]$Wait, [switch]$PassThru)
                    if ($Scenario -ne 'allow-action') {{ throw 'unexpected or cancelled elevation' }}
                    $helper = [Text.Encoding]::Unicode.GetString([Convert]::FromBase64String($ArgumentList[-1]))
                    $null = [scriptblock]::Create($helper)
                    if (-not $helper.Contains("'C:\Fixture''s\app.exe'")) {{ throw 'program argument was not preserved' }}
                    if (-not $helper.Contains("'Fixture''s Lab Remote Access'")) {{ throw 'label argument was not preserved' }}
                    [pscustomobject]@{{ ExitCode=0 }}
                }}
                function Remove-NetFirewallRule {{ throw 'unexpected rule removal' }}
                function Get-NetConnectionProfile {{ [pscustomobject]@{{ NetworkCategory = $(if ($Scenario -eq 'public') {{ 'Public' }} elseif ($Scenario -eq 'domain') {{ 'DomainAuthenticated' }} else {{ 'Private' }}) }} }}
                function Get-NetFirewallProfile {{
                    [pscustomobject]@{{ Enabled = $(if ($Scenario -eq 'disabled') {{ 'False' }} else {{ 'True' }}); DefaultInboundAction='Block'; AllowInboundRules='True'; AllowLocalFirewallRules=$(if ($Scenario -eq 'managed') {{ 'False' }} else {{ 'True' }}) }}
                }}
                function Get-NetFirewallRule {{
                    if ($Scenario -eq 'missing') {{ return }}
                    [pscustomobject]@{{ Name='unrelated-rule'; Profile=$(if ($Scenario -eq 'domain') {{ 'Domain' }} else {{ 'Private' }}); PolicyStoreSourceType='Local'; Action='Allow' }}
                    if ($Scenario -eq 'block') {{ [pscustomobject]@{{ Profile='Private'; PolicyStoreSourceType='Local'; Action='Block' }} }}
                }}
                function Get-NetFirewallApplicationFilter {{ [pscustomobject]@{{ Program=$(if ($Scenario -eq 'other-program') {{ 'C:\Other\app.exe' }} else {{ $Program }}); Package=$(if ($Scenario -ne 'package-empty') {{ 'Any' }}) }} }}
                function Get-NetFirewallPortFilter {{ [pscustomobject]@{{ Protocol='TCP'; LocalPort=$(if ($Scenario -eq 'other-port') {{ '99' }} else {{ '8765' }}); RemotePort='Any' }} }}
                function Get-NetFirewallAddressFilter {{ [pscustomobject]@{{ LocalAddress='Any'; RemoteAddress=$(if ($Scenario -eq 'local-subnet') {{ 'LocalSubnet' }} else {{ 'Any' }}) }} }}
                function Get-NetFirewallServiceFilter {{ [pscustomobject]@{{ Service='Any' }} }}
            "#
            );
            let script = format!("{fixture}\n{}", include_str!("firewall.ps1"));
            let encoded = STANDARD.encode(script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>());
            let executable =
                std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/WindowsPowerShell/v1.0/powershell.exe");
            let result = dokime::domain::process::command(executable)
                .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
                .output()
                .unwrap();
            if expected == "error" {
                assert!(!result.status.success(), "cancelled elevation must fail without applying a rule");
                continue;
            }
            assert!(result.status.success(), "{scenario}: {}", String::from_utf8_lossy(&result.stderr));
            if expected == "cleanup" {
                assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "inactive");
                continue;
            }
            assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), expected, "{scenario}");
        }
    }
}
