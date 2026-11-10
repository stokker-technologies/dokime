/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Application Preferences and Revision-Checked Updates

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::{document::DownsampleTechnique, remote::RemoteListenerSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationSettings {
    pub appearance: Appearance,
    pub report: ReportDefaults,
    pub client: ClientPreferences,
    pub telemetry: TelemetryPreferences,
    pub listener: RemoteListenerSettings,
    pub plugins: PluginPreferences,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Appearance {
    pub theme_mode: ThemeMode,
    pub theme_family: Option<String>,
    pub density: Density,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReportDefaults {
    pub source_bytes: u32,
    pub value_depth: u32,
    pub table_rows: u32,
    pub plot_points: u32,
    pub plot_downsample: DownsampleTechnique,
}

#[harmos::message(skip_default)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClientPreferences {
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
}

#[harmos::message(skip_default)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TelemetryPreferences {
    pub interval_ms: u64,
}

// Host Budgets For Plugin Programs; A Change Applies To The Next Start, Restart Or Stop.
#[harmos::message(skip_default)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PluginPreferences {
    pub startup_timeout_s: u32,
    pub stop_timeout_s: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSnapshot {
    pub revision: u64,
    pub settings: ApplicationSettings,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSettings {
    pub expected_revision: u64,
    pub settings: ApplicationSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsConflict {
    pub expected: u64,
    pub actual: u64,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsScope {
    Local,
    Backend,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Comfortable,
    Compact,
    Touch,
}

/* === Traits === */

/* === Implementations === */

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            theme_family: None,
            density: Density::Comfortable,
        }
    }
}

impl Default for ReportDefaults {
    fn default() -> Self {
        Self {
            source_bytes: 64 * 1024 * 1024,
            value_depth: 128,
            table_rows: 32,
            plot_points: 2_000,
            plot_downsample: DownsampleTechnique::Auto,
        }
    }
}

impl Default for ClientPreferences {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 10_000,
            request_timeout_ms: 30_000,
        }
    }
}

impl Default for PluginPreferences {
    fn default() -> Self {
        // Covers a sidecar that loads a large vendor library and opens its client
        // before it announces, with headroom for on-access scanning on first launch.
        Self {
            startup_timeout_s: 60,
            stop_timeout_s: 10,
        }
    }
}

impl Default for TelemetryPreferences {
    fn default() -> Self {
        Self { interval_ms: 1_000 }
    }
}

impl ApplicationSettings {
    pub fn validate(&self) -> Result<(), String> {
        let invalid_family = self.appearance.theme_family.as_ref().is_some_and(|name| name.is_empty() || name.len() > 256);
        if invalid_family {
            return Err("theme family must be nonempty and at most 256 bytes".into());
        }
        if self.report.source_bytes == 0 || self.report.value_depth == 0 {
            return Err("report source bytes and value depth must be positive".into());
        }
        if self.report.table_rows == 0 {
            return Err("report table rows must be positive".into());
        }
        if self.report.plot_points == 0 || self.report.plot_points > 100_000 {
            return Err("plot points must be between 1 and 100000".into());
        }
        if self.report.plot_downsample == DownsampleTechnique::Unspecified {
            return Err("plot downsampling strategy is required".into());
        }
        self.client.validate()?;
        self.telemetry.validate()?;
        self.listener.validate()?;
        self.plugins.validate()?;
        Ok(())
    }

    pub fn validate_scope(&self, previous: &Self, scope: SettingsScope) -> Result<(), String> {
        self.validate()?;
        let allowed = match scope {
            SettingsScope::Local => {
                self.report == previous.report
                    && self.telemetry == previous.telemetry
                    && self.listener == previous.listener
                    && self.plugins == previous.plugins
            }
            SettingsScope::Backend => self.appearance == previous.appearance && self.client == previous.client,
        };
        if !allowed {
            return Err("settings update changes fields outside its scope".into());
        }
        Ok(())
    }
}

impl ClientPreferences {
    pub fn validate(&self) -> Result<(), String> {
        validate_duration(self.connect_timeout_ms)?;
        validate_duration(self.request_timeout_ms)
    }
}

impl TelemetryPreferences {
    pub fn validate(&self) -> Result<(), String> {
        validate_duration(self.interval_ms)
    }
}

impl PluginPreferences {
    // The Longest Budget Either Field Accepts: One Hour.
    pub const MAXIMUM_SECONDS: u32 = 3_600;

    pub fn validate(&self) -> Result<(), String> {
        let bounded = |seconds: u32| (1..=Self::MAXIMUM_SECONDS).contains(&seconds);
        if !bounded(self.startup_timeout_s) || !bounded(self.stop_timeout_s) {
            return Err(format!("plugin startup and stop timeouts must be between 1 and {} seconds", Self::MAXIMUM_SECONDS));
        }
        Ok(())
    }

    pub fn startup_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.startup_timeout_s.into())
    }

    pub fn stop_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.stop_timeout_s.into())
    }
}

impl ReportDefaults {
    pub fn apply(&self, document: &mut crate::model::document::DocumentDefinition) {
        for card in &mut document.cards {
            self.apply_card(&mut card.body);
        }
        for section in &mut document.sections {
            for card in &mut section.cards {
                self.apply_card(&mut card.body);
            }
        }
    }

    fn apply_card(&self, body: &mut crate::model::document::DocumentCardBody) {
        let crate::model::document::DocumentCardBody::Plot { plot } = body else {
            return;
        };
        plot.display_points.get_or_insert(self.plot_points);
        if matches!(plot.downsample, DownsampleTechnique::Auto | DownsampleTechnique::Unspecified) {
            plot.downsample = self.plot_downsample;
        }
    }
}

impl std::fmt::Display for SettingsConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "settings revision conflict: expected {}, actual {}", self.expected, self.actual)
    }
}

impl std::error::Error for SettingsConflict {}

/* === Public Functions === */

/* === Private Functions === */

fn validate_duration(value: u64) -> Result<(), String> {
    let duration = std::time::Duration::from_millis(value);
    if value == 0 || value > (1_u64 << 53) - 1 || duration.as_nanos() > i64::MAX as u128 {
        return Err("duration must be a positive exact integer within the monotonic clock range".into());
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_preference_sections_and_fields_use_defaults() {
        let defaults = ApplicationSettings::default();
        let mut saved = defaults.clone();
        saved.report.plot_points = 321;
        saved.client.request_timeout_ms = 4567;
        saved.plugins.startup_timeout_s = 90;
        saved.plugins.stop_timeout_s = 20;
        let body = serde_json::to_value(&saved).unwrap();
        let default_body = serde_json::to_value(&defaults).unwrap();

        for (section, fields) in body.as_object().unwrap() {
            let mut missing_section = body.clone();
            missing_section.as_object_mut().unwrap().remove(section);
            let mut expected = body.clone();
            expected[section] = default_body[section].clone();
            let loaded: ApplicationSettings = serde_json::from_value(missing_section).unwrap();
            loaded.validate().unwrap();
            assert_eq!(serde_json::to_value(loaded).unwrap(), expected);

            for field in fields.as_object().unwrap().keys() {
                let mut missing_field = body.clone();
                missing_field[section].as_object_mut().unwrap().remove(field);
                let mut expected = body.clone();
                expected[section][field] = default_body[section][field].clone();
                let loaded: ApplicationSettings = serde_json::from_value(missing_field).unwrap();
                loaded.validate().unwrap();
                assert_eq!(serde_json::to_value(loaded).unwrap(), expected);
            }
        }
    }

    #[test]
    fn explicit_zero_plugin_timeouts_are_not_defaulted() {
        for field in ["startup_timeout_s", "stop_timeout_s"] {
            let body = serde_json::json!({ "plugins": { (field): 0 } });
            let loaded: ApplicationSettings = serde_json::from_value(body).unwrap();
            assert!(loaded.validate().is_err());
        }
    }
}
