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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Constants === */

pub const INSTALL_ARCHIVE_MAX_BYTES: u64 = 272 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallationCursor {
    pub manifest_sha256: String,
    pub after_install_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallationPage {
    pub installations: Vec<PluginInstallation>,
    pub next: Option<InstallationCursor>,
    pub total_count: u64,
    pub manifest_sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RemovePluginReply {
    #[row(description = "RemovalPending persists until the next boot; the active process is untouched.")]
    pub installation: Option<PluginInstallation>,
}

// Exactly one header first, followed by contiguous nonempty chunks of at most the
// backend's advertised chunk size, within the request limit. EOF must match the
// declared size and SHA-256.
// Native archive limit is `INSTALL_ARCHIVE_MAX_BYTES`.
// No resumable upload session or automatic retry: after uncertain acknowledgement,
// reconcile the manifest by digest before explicitly starting another installation.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPluginFrame {
    pub frame: Option<InstallPluginFrameFrame>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPluginHeader {
    pub size_bytes: u64,
    pub sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPluginChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginIdentity {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub identity: Option<PluginIdentity>,
    pub owners: Vec<String>,
    pub size_bytes: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginLastBoot {
    pub at: Option<Timestamp>,
    pub result: Option<PluginLastBootResult>,
    pub failure: Option<PluginStartFailure>,
}

#[harmos::message]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginStartFailure {
    pub plugin_id: String,
    pub plugin_name: String,
    pub message: String,
    pub missing_secret: Option<String>,
    pub carrier: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginInstallation {
    pub install_id: String,
    pub sha256: String,
    pub installed_at: Option<Timestamp>,
    pub state: PluginInstallationState,
    pub last_boot: Option<PluginLastBoot>,

    #[row(
        description = "Absent if bytes cannot be verified or full metadata exceeds the response budget; error explains why. Metadata is never truncated."
    )]
    pub metadata: Option<PluginMetadata>,

    pub error: Option<String>,
    pub active: bool,
    pub restart_required: bool,
    pub boot_error: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedPluginCursor {
    pub epoch: String,
    pub revision: u64,
    pub after_plugin_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedPluginPage {
    pub plugins: Vec<AdmittedPlugin>,
    pub next: Option<AdmittedPluginCursor>,
    pub total_count: u64,
    pub epoch: String,
    pub revision: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedBundleOwner {
    pub package: String,
    // Host paths are available only through a local listener.
    pub path: Option<String>,
    pub installation_id: Option<String>,
    pub content_digest: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedEmbeddedOwner {
    pub generation: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmittedPlugin {
    pub plugin_id: String,
    pub label: String,
    pub version: String,
    pub enabled: bool,
    pub owner: Option<AdmittedPluginOwner>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginInstallationState {
    Unspecified,
    Installed,
    Disabled,
    RemovalPending,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum InstallPluginFrameFrame {
    Header(InstallPluginHeader),
    Chunk(InstallPluginChunk),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginLastBootResult {
    Loaded(bool),
    Error(String),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AdmittedPluginOwner {
    Host(bool),
    Bundle(AdmittedBundleOwner),
    Embedded(AdmittedEmbeddedOwner),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
