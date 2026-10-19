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
use serde::Serialize;

// Local Imports
use crate::connection::{ConnectRequest, ConnectionCredentials, ConnectionTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopLaunch {
    pub target: Option<ConnectionTarget>,
    pub expected_backend_id: Option<String>,
    pub local_backend_available: bool,
    pub error: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// A command line names an attach target or nothing; the owned local backend
// is the default and shared profiles are selected from the desktop session.
pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Option<ConnectRequest>, String> {
    let mut arguments = arguments.into_iter();
    let mut endpoint = None;
    let mut backend_id = None;
    let mut shutdown_stdin = false;
    while let Some(argument) = arguments.next() {
        if argument == "--shutdown-stdin" {
            if shutdown_stdin {
                return Err("Duplicate desktop argument: --shutdown-stdin".into());
            }
            shutdown_stdin = true;
            continue;
        }
        let slot = match argument.as_str() {
            "--endpoint" => &mut endpoint,
            "--expected-backend-id" => &mut backend_id,
            _ => return Err("Unknown desktop argument".into()),
        };
        if slot.is_some() {
            return Err(format!("Duplicate desktop argument: {argument}"));
        }
        *slot = Some(
            arguments
                .next()
                .filter(|value| !value.starts_with("--") && !value.is_empty())
                .ok_or_else(|| format!("Missing value for {argument}"))?,
        );
    }
    match endpoint {
        Some(endpoint) => {
            let remote = !endpoint.starts_with("http://");
            let target =
                if remote { ConnectionTarget::Remote { endpoint, ca_certificate_pem: None } } else { ConnectionTarget::Local { endpoint } };
            crate::connection::ConnectionSession::validate_target(target.clone())
                .map_err(|_| "Invalid native backend endpoint".to_owned())?;
            Ok(Some(ConnectRequest {
                target,
                credentials: if remote { ConnectionCredentials::Saved } else { ConnectionCredentials::None },
                expected_backend_id: backend_id,
            }))
        }
        None if shutdown_stdin => Err("--shutdown-stdin requires --endpoint".into()),
        None if backend_id.is_some() => Err("--expected-backend-id requires --endpoint".into()),
        None => Ok(None),
    }
}

pub fn configuration(arguments: impl IntoIterator<Item = String>) -> DesktopLaunch {
    let mut launch = DesktopLaunch {
        target: None,
        expected_backend_id: None,
        // The owned session runs the backend the installer shipped beside
        // this application, so availability is what discovery finds.
        local_backend_available: crate::native_session::backend::locate().is_ok(),
        error: None,
    };
    match parse(arguments) {
        Ok(Some(request)) => {
            launch.target = Some(request.target);
            launch.expected_backend_id = request.expected_backend_id;
        }
        Ok(None) => {}
        Err(error) => launch.error = Some(error),
    }
    launch
}

pub fn desktop_session_launch(state: tauri::State<'_, DesktopLaunch>) -> Result<DesktopLaunch, String> {
    Ok(state.inner().clone())
}

/* === Private Functions === */

/* === Tests === */
