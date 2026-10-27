/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Remote Listener Settings And One-Time Secrets

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

#[harmos_domain::message(skip_default)]
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

impl Default for RemoteListenerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            address: "0.0.0.0:8765".into(),
            tls_enabled: true,
            authentication_enabled: true,
            certificate_names: Vec::new(),
        }
    }
}

impl RemoteListenerSettings {
    pub fn validate(&self) -> Result<(), String> {
        self.address.parse::<std::net::SocketAddr>().map_err(|_| "listener address must be an IP address and port")?;
        if self.certificate_names.len() > 32 {
            return Err("at most 32 certificate identities are allowed".into());
        }
        for (index, name) in self.certificate_names.iter().enumerate() {
            if name.is_empty() || name.len() > 253 || self.certificate_names[..index].contains(name) {
                return Err("certificate identities must be distinct bounded DNS names or IP addresses".into());
            }
            if name.parse::<std::net::IpAddr>().is_ok() {
                continue;
            }
            if !name.split('.').all(valid_dns_label) {
                return Err("certificate identities must be exact DNS names or IP addresses without wildcards or ports".into());
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for GeneratedAccessKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("GeneratedAccessKey").field("access_key", &"<redacted>").field("listener", &self.listener).finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn valid_dns_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
        && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/* === Tests === */
