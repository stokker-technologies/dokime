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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::connection::{ConnectRequest, ConnectionError, ConnectionSnapshot};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSessionSnapshot {
    pub ownership: BackendOwnership,
    pub phase: Option<String>,
    pub connection: ConnectionSnapshot,
}

/// What an owned backend left behind when it failed to start: the resolved
/// executable, how it exited, the redacted stderr tail and its full log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDiagnostics {
    pub executable: String,
    pub exit_code: Option<i32>,
    pub exit_status: Option<String>,
    pub tail: String,
    pub log: Option<String>,
}

/* === Enums === */
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackendSelection {
    OwnedLocal,
    Profile { name: String },
    Attach { request: ConnectRequest },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendOwnership {
    None,
    OwnedLocal,
    Profile { name: String },
    Attached,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DesktopSessionError {
    LocalBackendUnavailable,
    ShuttingDown,
    Superseded,
    Startup {
        message: String,
        diagnostics: Option<BackendDiagnostics>,
    },
    Shutdown {
        message: String,
    },
    Connection {
        error: ConnectionError,
    },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
