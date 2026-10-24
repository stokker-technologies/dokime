/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The points and strategy a plot draws with when neither its author nor its
// operator chose, live and in every report.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPreference {
    pub plot_points: u32,
    pub plot_downsample: crate::model::document::DownsampleTechnique,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearancePreference {
    pub theme_mode: ThemeMode,
    pub theme_family: Option<String>,
    pub density: Density,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub appearance: Option<AppearancePreference>,
    pub report: Option<ReportPreference>,
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

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoragePolicy {
    pub compaction_interval_ms: u64,
    pub local_retention_ms: Option<u64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoragePolicyView {
    pub revision: u64,
    pub configured: Option<StoragePolicy>,
    pub effective: Option<StoragePolicy>,
    pub restart_required: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageSource {
    pub identifier: String,
    pub adapter: String,
    pub priority: i64,
    pub enabled: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageSources {
    pub sources: Vec<StorageSource>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsFrame {
    pub sections: Vec<SettingsSection>,
    pub cards: Vec<SettingsCard>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsSection {
    pub section_id: String,
    pub label: String,
    pub description: Option<String>,
    pub owner: Option<SettingsSectionOwner>,
    pub order: i32,
}

#[harmos_domain::message]
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

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardBody {
    pub blocks: Vec<HostCardBlock>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardBlock {
    pub kind: Option<HostCardBlockKind>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardStatus {
    pub label: String,
    pub tone: HostCardTone,
    pub detail: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardKeyValueList {
    pub items: Vec<HostCardKeyValue>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardKeyValue {
    pub label: String,
    pub value: String,
    pub detail: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardMetrics {
    pub items: Vec<HostCardMetric>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HostCardMetric {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
    pub trend: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsCardRefreshPolicy {
    pub policy: Option<SettingsCardRefreshPolicyPolicy>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualRefresh {}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedRefresh {
    pub max_age_ms: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OnDemandRefresh {}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeSettings {
    pub settings: Option<SettingsCatalog>,
    pub values: Option<Value>,
    pub secret_bindings: Option<PluginRuntimeSecretBindings>,

    // Values a plugin's discovery routes computed, by setting id, for the fields
    // nobody saved. A value here is shown on the card with a `Discovered` marker
    // and is used by no run: saving it is what turns it into an edited value.
    #[row(
        description = "Values a plugin's discovery routes computed, by setting id, for the fields nobody saved. A value here is shown on the card with a `Discovered` marker and is used by no run: saving it is what turns it into an edited value."
    )]
    pub discovered: Option<Value>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsCatalog {
    pub contributions: Vec<SettingContribution>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingContribution {
    pub kind: Option<SettingContributionKind>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TextAreaSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub required: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PathSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub required: bool,
}

#[harmos_domain::message]
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

#[harmos_domain::message]
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

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingChoice {
    pub value: String,
    pub title: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: Option<SecretReference>,
    pub required: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretReference {
    pub kind: Option<SecretReferenceKind>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeSecretBindingsBindingsEntry {
    pub key: String,
    pub value: Option<RedactedSecretBinding>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeSecretBindings {
    pub bindings: Vec<PluginRuntimeSecretBindingsBindingsEntry>,
    pub missing_required: Vec<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RedactedSecretBinding {
    pub secret_id: String,
    pub backend: SecretBackendKind,
    pub configured: bool,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Unspecified,
    System,
    Light,
    Dark,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Density {
    Unspecified,
    Comfortable,
    Compact,
    Touch,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostCardTone {
    Unspecified,
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

#[harmos_domain::message]
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

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretBackendKind {
    Unspecified,
    OsKeyring,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsSectionOwner {
    App(bool),
    InstalledPluginId(String),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum HostCardBlockKind {
    Text(String),
    Status(HostCardStatus),
    KeyValueList(HostCardKeyValueList),
    Metrics(HostCardMetrics),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsCardRefreshPolicyPolicy {
    Manual(ManualRefresh),
    Cached(CachedRefresh),
    OnDemand(OnDemandRefresh),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingContributionKind {
    Boolean(BooleanSetting),
    TextArea(TextAreaSetting),
    Path(PathSetting),
    Number(NumberSetting),
    MultipleChoice(MultipleChoiceSetting),
    SecretReference(SecretSetting),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SecretReferenceKind {
    Environment(String),
    StoredSecret(String),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
