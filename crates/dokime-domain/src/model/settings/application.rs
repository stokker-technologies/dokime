/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Application Preferences And Revision-Checked Updates

use crate::model::{document::DownsampleTechnique, remote::RemoteListenerSettings};
use serde::{Deserialize, Serialize};

#[harmos_domain::message]
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSettings {
    pub appearance: Appearance,
    pub report: ReportDefaults,
    pub client: ClientPreferences,
    pub telemetry: TelemetryPreferences,
    pub listener: RemoteListenerSettings,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub theme_mode: ThemeMode,
    pub theme_family: Option<String>,
    pub density: Density,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDefaults {
    pub plot_points: u32,
    pub plot_downsample: DownsampleTechnique,
}

#[harmos_domain::message(skip_default)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientPreferences {
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
}

#[harmos_domain::message(skip_default)]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPreferences {
    pub interval_ms: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSnapshot {
    pub revision: u64,
    pub settings: ApplicationSettings,
}

#[harmos_domain::message]
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

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsScope {
    Local,
    Backend,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Comfortable,
    Compact,
    Touch,
}

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
        if self.report.plot_points == 0 || self.report.plot_points > 100_000 {
            return Err("plot points must be between 1 and 100000".into());
        }
        if self.report.plot_downsample == DownsampleTechnique::Unspecified {
            return Err("plot downsampling strategy is required".into());
        }
        self.client.validate()?;
        self.telemetry.validate()?;
        self.listener.validate()?;
        Ok(())
    }

    pub fn validate_scope(&self, previous: &Self, scope: SettingsScope) -> Result<(), String> {
        self.validate()?;
        let allowed = match scope {
            SettingsScope::Local => {
                self.report == previous.report && self.telemetry == previous.telemetry && self.listener == previous.listener
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

fn validate_duration(value: u64) -> Result<(), String> {
    let duration = std::time::Duration::from_millis(value);
    if value == 0 || value > (1_u64 << 53) - 1 || duration.as_nanos() > i64::MAX as u128 {
        return Err("duration must be a positive exact integer within the monotonic clock range".into());
    }
    Ok(())
}
