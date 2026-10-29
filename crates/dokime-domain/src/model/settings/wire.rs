/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use super::application::{
    Appearance, ApplicationSettings, ClientPreferences, Density, ReportDefaults, SettingsScope, SettingsSnapshot, TelemetryPreferences,
    ThemeMode, UpdateSettings,
};

use serde::Deserialize;
use serde::Serialize;

use crate::model::values::wire::Value;

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
    pub bootstrap_paths: Option<BootstrapPaths>,
    pub sections: Vec<SettingsSection>,
    pub cards: Vec<SettingsCard>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BootstrapPaths {
    pub application: String,
    pub storage: String,
    pub plugins: String,
    pub logs: String,
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
pub struct PluginRuntimeSettings {
    pub settings: Option<SettingsCatalog>,
    pub values: Option<Value>,
    pub secret_bindings: Option<PluginRuntimeSecretBindings>,

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
pub struct PluginRuntimeSecretBindingsBindingsEntry {
    pub key: String,
    pub value: Option<RedactedSecretBinding>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeSecretBindings {
    pub bindings: Vec<PluginRuntimeSecretBindingsBindingsEntry>,
    pub missing_required: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RedactedSecretBinding {
    pub secret_id: String,
    pub backend: SecretBackendKind,
    pub configured: bool,
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
pub enum SecretBackendKind {
    Unspecified,
    OsKeyring,
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
    StoredSecret(String),
}
