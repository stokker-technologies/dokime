/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Remote Listener Settings And One-Time Credentials

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteListenerSettings {
    pub enabled: bool,
    pub address: String,
    pub tls_enabled: bool,
    pub authentication_enabled: bool,
    pub certificate_names: Vec<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteListenerStatus {
    pub settings: Option<RemoteListenerSettings>,
    pub listening_address: Option<String>,
    pub certificate_pem: Option<String>,
    pub key_configured: bool,
    pub error: Option<String>,
}

#[harmos_domain::message(skip_debug)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedAccessKey {
    pub access_key: String,
    pub listener: Option<RemoteListenerStatus>,
}

/// One remote client's calls from one peer address, admitted or rejected.
///
/// Times are Unix milliseconds. Transport drops are not observable, so a
/// session closes after a quiet interval and `disconnected_at` is its last call.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSession {
    pub id: String,
    pub peer_address: String,
    /// `access-key` for a verified key, otherwise `anonymous`.
    pub principal: String,
    pub connected_at: u64,
    pub last_seen_at: u64,
    pub disconnected_at: Option<u64>,
    pub calls: u64,
    pub outcome: RemoteSessionOutcome,
    /// Why the host refused the calls; set only for a rejected session.
    pub rejection: Option<String>,
}

/// Newest first; `next_cursor` continues with older sessions.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSessionPage {
    pub sessions: Vec<RemoteSession>,
    pub next_cursor: Option<String>,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteSessionOutcome {
    Unspecified,
    Admitted,
    Rejected,
}

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for GeneratedAccessKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("GeneratedAccessKey").field("access_key", &"<redacted>").field("listener", &self.listener).finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
