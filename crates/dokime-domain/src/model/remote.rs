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

/* === Enums === */

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
