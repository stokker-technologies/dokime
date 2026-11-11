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

#[cfg(feature = "testing")]
impl FirewallStatus {
    pub fn inspect(&self) -> (bool, Option<&str>) {
        (self.required, self.message.as_deref())
    }
}

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
