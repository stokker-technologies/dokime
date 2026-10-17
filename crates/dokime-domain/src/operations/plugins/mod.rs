/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod consent;
mod get_installation;
mod list_admitted_plugins;
mod list_installations;
mod remove;
mod set_enabled;
mod stage;

/* === Re-Exports === */
pub use consent::Consent;
pub use get_installation::GetInstallation;
pub use list_admitted_plugins::ListAdmittedPlugins;
pub use list_installations::ListInstallations;
pub use remove::Remove;
pub use set_enabled::SetEnabled;
pub use stage::Stage;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::common::*;
use super::{Access, Endpoint, INSTALLATION, ORDINARY};

/* === Constants === */

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
pub struct ListInstallationsRequest {
    pub page_size: u32,
    pub after: Option<InstallationCursor>,
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
pub struct GetInstallationRequest {
    pub install_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsentPluginRequest {
    pub install_id: String,
    pub expected_sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetPluginEnabledRequest {
    pub install_id: String,
    pub enabled: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RemovePluginRequest {
    pub install_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RemovePluginReply {
    // Absent means the staged installation was removed. Consented installations
    // remain RemovalPending until next boot; the active process is untouched.
    #[row(
        description = "Absent means the staged installation was removed. Consented installations remain RemovalPending until next boot; the active process is untouched."
    )]
    pub installation: Option<PluginInstallation>,
}

// Exactly one header first, followed by contiguous nonempty chunks <=256KiB.
// EOF must match the declared size and SHA-256. Native archive limit is 272 MiB.
// No resumable upload session or automatic retry: after uncertain acknowledgement,
// reconcile the manifest by digest before explicitly starting another stage.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StagePluginFrame {
    pub frame: Option<StagePluginFrameFrame>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StagePluginHeader {
    pub size_bytes: u64,
    pub sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StagePluginChunk {
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
pub struct PluginEnvironmentReference {
    pub owner: String,
    pub setting: String,
    pub variable: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub identity: Option<PluginIdentity>,
    pub owners: Vec<String>,
    pub size_bytes: u64,
    pub environment_references: Vec<PluginEnvironmentReference>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginLastBoot {
    pub at: Option<Timestamp>,
    pub result: Option<PluginLastBootResult>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginInstallation {
    pub install_id: String,
    pub sha256: String,
    pub staged_at: Option<Timestamp>,
    pub consented_at: Option<Timestamp>,
    pub state: PluginInstallationState,
    pub last_boot: Option<PluginLastBoot>,
    // Absent if bytes cannot be verified or full metadata exceeds the response
    // budget; error explains why. Permission disclosure is never truncated.
    #[row(
        description = "Absent if bytes cannot be verified or full metadata exceeds the response budget; error explains why. Permission disclosure is never truncated."
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
pub struct ListAdmittedPluginsRequest {
    pub page_size: u32,
    pub after: Option<AdmittedPluginCursor>,
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
    Staged,
    Installed,
    Disabled,
    RemovalPending,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StagePluginFrameFrame {
    Header(StagePluginHeader),
    Chunk(StagePluginChunk),
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

#[harmos::catalog]
pub enum Operations {
    ListInstallations(ListInstallations),
    GetInstallation(GetInstallation),
    Stage(Stage),
    Consent(Consent),
    SetEnabled(SetEnabled),
    Remove(Remove),
    ListAdmittedPlugins(ListAdmittedPlugins),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListInstallations {
    const PATH: &'static str = "/dokime.v1.Plugins/ListInstallations";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for GetInstallation {
    const PATH: &'static str = "/dokime.v1.Plugins/GetInstallation";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Stage {
    const PATH: &'static str = "/dokime.v1.Plugins/Stage";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Consent {
    const PATH: &'static str = "/dokime.v1.Plugins/Consent";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for SetEnabled {
    const PATH: &'static str = "/dokime.v1.Plugins/SetEnabled";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Remove {
    const PATH: &'static str = "/dokime.v1.Plugins/Remove";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for ListAdmittedPlugins {
    const PATH: &'static str = "/dokime.v1.Plugins/ListAdmittedPlugins";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
