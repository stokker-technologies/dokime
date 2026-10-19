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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    pub target: ConnectionTarget,
    pub credentials: ConnectionCredentials,
    pub expected_backend_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSnapshot {
    pub generation: String,
    pub phase: ConnectionPhase,
    pub endpoint: Option<String>,
    pub remote: bool,
    pub backend: Option<BackendSnapshot>,
    pub error: Option<ConnectionError>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendSnapshot {
    pub backend_id: String,
    pub epoch: String,
    pub display_name: String,
    pub product_version: String,
    pub api_version: String,
    pub descriptor_sha256: String,
    pub methods: Vec<String>,
    pub limits: BackendLimits,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendLimits {
    pub max_request_bytes: String,
    pub max_response_bytes: String,
    pub max_content_bytes: Option<String>,
    pub max_evidence_bytes: Option<String>,
    pub max_chunk_bytes: Option<u32>,
    pub max_page_items: Option<u32>,
    pub max_stream_buffer_items: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AdmissionIssue {
    pub path: String,
    pub message: String,
    pub expected: Option<String>,
}

/* === Enums === */
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ConnectionTarget {
    Local { endpoint: String },
    Remote { endpoint: String, ca_certificate_pem: Option<String> },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionCredentials {
    None,
    Saved,
    Bearer { token: String, remember: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionPhase {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionError {
    InvalidTarget { message: String },
    CredentialsUnavailable,
    AuthenticationRequired,
    Unauthenticated,
    PermissionDenied,
    Transport,
    IncompatibleApi,
    IdentityChanged,
    BackendUnavailable,
    ExecutionRejected { code: String, message: String, issues: Vec<AdmissionIssue> },
    WaitTimedOut,
    Superseded,
    NotConnected,
    GenerationExhausted,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
