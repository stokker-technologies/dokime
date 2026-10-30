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
use crate::connection::{ConnectRequest, ConnectionSecrets, ConnectionTarget};

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
        let (name, value) = argument.split_once('=').map_or((argument.as_str(), None), |(name, value)| (name, Some(value)));
        let slot = match name {
            "--endpoint" => &mut endpoint,
            "--expected-backend-id" => &mut backend_id,
            _ => return Err("Unknown desktop argument".into()),
        };
        if slot.is_some() {
            return Err(format!("Duplicate desktop argument: {name}"));
        }
        *slot = Some(
            value
                .map(str::to_owned)
                .or_else(|| arguments.next())
                .filter(|value| !value.starts_with("--") && !value.is_empty())
                .ok_or_else(|| format!("Missing value for {name}"))?,
        );
    }
    match endpoint {
        Some(endpoint) => {
            let remote = !endpoint.starts_with("http://");
            let target = if remote {
                ConnectionTarget::Remote {
                    endpoint,
                    ca_certificate_pem: None,
                    endpoints: Vec::new(),
                }
            } else {
                ConnectionTarget::Local { endpoint }
            };
            crate::connection::ConnectionSession::validate_target(target.clone())
                .map_err(|_| "Invalid native backend endpoint".to_owned())?;
            Ok(Some(ConnectRequest {
                target,
                secrets: if remote { ConnectionSecrets::Saved } else { ConnectionSecrets::None },
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
        // The local server is linked into this desktop.
        local_backend_available: true,
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

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::connection::ConnectionTarget;

    #[test]
    fn launch_value_forms() {
        for arguments in [
            vec!["--endpoint", "http://127.0.0.1:3210", "--expected-backend-id=backend"],
            vec!["--endpoint=http://127.0.0.1:3210", "--expected-backend-id", "backend"],
        ] {
            let request = parse(arguments.into_iter().map(str::to_owned)).unwrap().unwrap();
            assert!(matches!(request.target, ConnectionTarget::Local { endpoint } if endpoint == "http://127.0.0.1:3210"));
            assert_eq!(request.expected_backend_id.as_deref(), Some("backend"));
        }
    }

    #[test]
    fn launch_rejects_invalid_values() {
        for arguments in [vec!["--endpoint"], vec!["--endpoint="], vec!["--endpoint", "--expected-backend-id=backend"]] {
            assert_eq!(parse(arguments.into_iter().map(str::to_owned)).err().as_deref(), Some("Missing value for --endpoint"));
        }
        for arguments in [
            vec!["--endpoint=http://127.0.0.1:3210", "--endpoint", "http://127.0.0.1:3211"],
            vec!["--endpoint", "http://127.0.0.1:3210", "--endpoint=http://127.0.0.1:3211"],
        ] {
            assert_eq!(parse(arguments.into_iter().map(str::to_owned)).err().as_deref(), Some("Duplicate desktop argument: --endpoint"));
        }
        assert_eq!(parse(["--unknown=value".to_owned()]).err().as_deref(), Some("Unknown desktop argument"));
    }
}
