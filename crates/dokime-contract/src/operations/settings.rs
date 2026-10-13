/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod get_plugin_settings;
mod get_settings;
mod list_storage_sources;
mod read_application_settings;
mod read_storage_policy;
mod reorder_storage_sources;
mod set_storage_source_enabled;
mod update_appearance;
mod update_data_root;
mod update_plugin_credential;
mod update_plugin_settings;
mod update_storage_policy;

/* === Re-Exports === */
pub use get_plugin_settings::GetPluginSettings;
pub use get_settings::GetSettings;
pub use list_storage_sources::ListStorageSources;
pub use read_application_settings::ReadApplicationSettings;
pub use read_storage_policy::ReadStoragePolicy;
pub use reorder_storage_sources::ReorderStorageSources;
pub use set_storage_source_enabled::SetStorageSourceEnabled;
pub use update_appearance::UpdateAppearance;
pub use update_data_root::UpdateDataRoot;
pub use update_plugin_credential::UpdatePluginCredential;
pub use update_plugin_settings::UpdatePluginSettings;
pub use update_storage_policy::UpdateStoragePolicy;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadApplicationSettingsRequest {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateAppearanceRequest {
    pub appearance: Option<AppearancePreference>,
}

// An absent data root restores the default beneath the application root.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDataRootRequest {
    pub data_root: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearancePreference {
    pub theme_mode: ThemeMode,
    pub theme_family: Option<String>,
    pub density: Density,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub appearance: Option<AppearancePreference>,
    pub application_root: String,
    // Absent when settings.json names none and the default applies.
    #[row(description = "Absent when settings.json names none and the default applies.")]
    pub configured_data_root: Option<String>,
    pub default_data_root: String,
    // The data root this backend process is using.
    #[row(description = "The data root this backend process is using.")]
    pub effective_data_root: String,
    pub restart_required: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadStoragePolicyRequest {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateStoragePolicyRequest {
    pub expected_revision: u64,
    pub policy: Option<StoragePolicy>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoragePolicy {
    pub save_interval_ms: u64,
    pub compaction_interval_ms: u64,
    pub local_retention_ms: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoragePolicyView {
    pub revision: u64,
    pub configured: Option<StoragePolicy>,
    pub effective: Option<StoragePolicy>,
    pub restart_required: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSettingsRequest {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListStorageSourcesRequest {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorderStorageSourcesRequest {
    pub identifiers: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetStorageSourceEnabledRequest {
    pub identifier: String,
    pub enabled: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageSource {
    pub identifier: String,
    pub adapter: String,
    pub priority: i64,
    pub enabled: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageSources {
    pub sources: Vec<StorageSource>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsFrame {
    pub sections: Vec<SettingsSection>,
    pub cards: Vec<SettingsCard>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsSection {
    pub section_id: String,
    pub label: String,
    pub description: Option<String>,
    pub owner: Option<SettingsSectionOwner>,
    pub order: i32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsCard {
    pub plugin_id: String,
    pub card_id: String,
    pub section_id: String,
    pub label: String,
    pub description: Option<String>,
    pub body: Option<HostCardBody>,
    pub refresh: Option<SettingsCardRefreshPolicy>,
    pub state: SettingsCardState,
    pub order: i32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardBody {
    pub blocks: Vec<HostCardBlock>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardBlock {
    pub kind: Option<HostCardBlockKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardStatus {
    pub label: String,
    pub tone: HostCardTone,
    pub detail: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardKeyValueList {
    pub items: Vec<HostCardKeyValue>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardKeyValue {
    pub label: String,
    pub value: String,
    pub detail: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardMetrics {
    pub items: Vec<HostCardMetric>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardMetric {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
    pub trend: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsCardRefreshPolicy {
    pub policy: Option<SettingsCardRefreshPolicyPolicy>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualRefresh {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedRefresh {
    pub max_age_ms: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OnDemandRefresh {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPluginSettingsRequest {
    pub plugin_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePluginSettingsRequest {
    pub plugin_id: String,
    pub values: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePluginCredentialRequest {
    pub plugin_id: String,
    pub credential_id: String,
    pub secret: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeSettings {
    pub settings: Option<SettingsCatalog>,
    pub values: Option<Value>,
    pub credential_bindings: Option<PluginRuntimeCredentialBindings>,
    // Values a plugin's discovery routes computed, by setting id, for the fields
    // nobody saved. A value here is shown on the card with a `Discovered` marker
    // and is used by no run: saving it is what turns it into an edited value.
    #[row(
        description = "Values a plugin's discovery routes computed, by setting id, for the fields nobody saved. A value here is shown on the card with a `Discovered` marker and is used by no run: saving it is what turns it into an edited value."
    )]
    pub discovered: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsCatalog {
    pub contributions: Vec<SettingContribution>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingContribution {
    pub kind: Option<SettingContributionKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TextAreaSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub required: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PathSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub required: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MultipleChoiceSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub choices: Vec<SettingChoice>,
    // The id of the plugin route that fills this dropdown at run time.
    #[row(description = "The id of the plugin route that fills this dropdown at run time.")]
    pub provider: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingChoice {
    pub value: String,
    pub title: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: Option<SecretReference>,
    pub required: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretReference {
    pub kind: Option<SecretReferenceKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeCredentialBindingsBindingsEntry {
    pub key: String,
    pub value: Option<RedactedCredentialBinding>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeCredentialBindings {
    pub bindings: Vec<PluginRuntimeCredentialBindingsBindingsEntry>,
    pub missing_required: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RedactedCredentialBinding {
    pub credential_id: String,
    pub backend: CredentialBackendKind,
    pub configured: bool,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Unspecified,
    System,
    Light,
    Dark,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    Unspecified,
    Comfortable,
    Compact,
    Touch,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostCardTone {
    Unspecified,
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsCardState {
    Unspecified,
    Ready,
    Disabled,
    Disconnected,
    Stale,
    Error,
    PermissionDenied,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialBackendKind {
    Unspecified,
    OsKeyring,
    Memory,
    LocalFile,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsSectionOwner {
    App(bool),
    InstalledPluginId(String),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum HostCardBlockKind {
    Text(String),
    Status(HostCardStatus),
    KeyValueList(HostCardKeyValueList),
    Metrics(HostCardMetrics),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsCardRefreshPolicyPolicy {
    Manual(ManualRefresh),
    Cached(CachedRefresh),
    OnDemand(OnDemandRefresh),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingContributionKind {
    Boolean(BooleanSetting),
    TextArea(TextAreaSetting),
    Path(PathSetting),
    Number(NumberSetting),
    MultipleChoice(MultipleChoiceSetting),
    SecretReference(SecretSetting),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SecretReferenceKind {
    Environment(String),
    StoredCredential(String),
}

#[harmos::catalog]
pub enum Operations {
    ReadStoragePolicy(ReadStoragePolicy),
    UpdateStoragePolicy(UpdateStoragePolicy),
    GetSettings(GetSettings),
    ListStorageSources(ListStorageSources),
    ReorderStorageSources(ReorderStorageSources),
    SetStorageSourceEnabled(SetStorageSourceEnabled),
    GetPluginSettings(GetPluginSettings),
    UpdatePluginSettings(UpdatePluginSettings),
    UpdatePluginCredential(UpdatePluginCredential),
    ReadApplicationSettings(ReadApplicationSettings),
    UpdateAppearance(UpdateAppearance),
    UpdateDataRoot(UpdateDataRoot),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ReadStoragePolicy {
    const PATH: &'static str = "/dokime.v1.Settings/ReadStoragePolicy";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for UpdateStoragePolicy {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateStoragePolicy";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for GetSettings {
    const PATH: &'static str = "/dokime.v1.Settings/GetSettings";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListStorageSources {
    const PATH: &'static str = "/dokime.v1.Settings/ListStorageSources";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ReorderStorageSources {
    const PATH: &'static str = "/dokime.v1.Settings/ReorderStorageSources";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for SetStorageSourceEnabled {
    const PATH: &'static str = "/dokime.v1.Settings/SetStorageSourceEnabled";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for GetPluginSettings {
    const PATH: &'static str = "/dokime.v1.Settings/GetPluginSettings";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for UpdatePluginSettings {
    const PATH: &'static str = "/dokime.v1.Settings/UpdatePluginSettings";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for UpdatePluginCredential {
    const PATH: &'static str = "/dokime.v1.Settings/UpdatePluginCredential";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for ReadApplicationSettings {
    const PATH: &'static str = "/dokime.v1.Settings/ReadApplicationSettings";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for UpdateAppearance {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateAppearance";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for UpdateDataRoot {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateDataRoot";
    const ACCESS: Access = Access::Admin;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
