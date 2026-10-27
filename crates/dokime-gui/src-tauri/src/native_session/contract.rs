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
    pub status_revision: u64,
    pub ownership: BackendOwnership,
    pub phase: Option<String>,
    pub archived_bytes: Option<u64>,
    pub connection: ConnectionSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDiagnostics {
    pub cause: String,
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
    Startup { message: String, diagnostics: Option<Box<BackendDiagnostics>> },
    Shutdown { message: String },
    Connection { error: ConnectionError },
    StorageReset { reason: StorageResetReason, message: String },
    Archived { archive: String, error: Box<DesktopSessionError> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageResetReason {
    DevelopmentOnly,
    NotOwned,
    BackendRunning,
    OutsideRoot,
    Failed,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
