/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};
use prost::Message;

use dokime_protocol::host::ApplicationAppearance;
use dokime_protocol::host::ApplicationSettings;
use dokime_protocol::host::ClientPreferences;
use dokime_protocol::host::Density;
use dokime_protocol::host::PluginPreferences;
use dokime_protocol::host::ReportDefaults;
use dokime_protocol::host::SettingsGetResponse;
use dokime_protocol::host::TelemetryPreferences;
use dokime_protocol::host::ThemeMode;
use dokime_protocol::model::Downsample;
use harmos_protocol::model::ReadCut;
use std::time::Duration;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Product Defaults Are Copied from the Old Application Preferences; Delete the Old Owner at Cutover.
pub(crate) fn initial() -> SettingsGetResponse {
    SettingsGetResponse {
        revision: Vec::new(),
        value: Some(ApplicationSettings {
            appearance: Some(ApplicationAppearance {
                theme_mode: ThemeMode::System as i32,
                density: Density::Comfortable as i32,
                ..Default::default()
            }),
            report: Some(ReportDefaults {
                source_bytes: 64 * 1024 * 1024,
                value_depth: 128,
                table_rows: 32,
                plot_points: 2000,
                plot_downsample: Downsample::Auto as i32,
            }),
            client: Some(ClientPreferences {
                connect_timeout_ms: 10_000,
                request_timeout_ms: 30_000,
            }),
            telemetry: Some(TelemetryPreferences { interval_ms: 1000 }),
            listener: None,
            plugins: Some(PluginPreferences {
                startup_timeout_s: 60,
                stop_timeout_s: 10,
            }),
        }),
        resolved_at: None,
    }
}

pub(crate) async fn read(context: &Context, at: Option<ReadCut>) -> Result<SettingsGetResponse> {
    let initial = initial().value.unwrap().encode_to_vec();
    let snapshot = crate::host::storage::administration::read(context, "preferences", &initial, at).await?;
    Ok(SettingsGetResponse {
        value: Some(ApplicationSettings::decode(snapshot.payload.as_slice())?),
        revision: snapshot.revision,
        resolved_at: snapshot.resolved_at,
    })
}

pub(crate) fn validate(settings: &ApplicationSettings) -> Result<()> {
    let appearance = settings.appearance.as_ref().ok_or_else(|| invalid("Supply Appearance Preferences"))?;
    if appearance.theme_family.as_ref().is_some_and(|name| name.is_empty() || name.len() > 256) {
        return Err(invalid("Choose a Theme Family of 1 to 256 Bytes"));
    }
    if ThemeMode::try_from(appearance.theme_mode).is_err()
        || appearance.theme_mode == 0
        || Density::try_from(appearance.density).is_err()
        || appearance.density == 0
    {
        return Err(invalid("Choose a Valid Theme Mode and Density"));
    }
    let report = settings.report.as_ref().ok_or_else(|| invalid("Supply Report Defaults"))?;
    if report.source_bytes == 0
        || report.value_depth == 0
        || report.table_rows == 0
        || !(1..=100_000).contains(&report.plot_points)
        || report.plot_downsample == 0
        || Downsample::try_from(report.plot_downsample).is_err()
    {
        return Err(invalid("Supply Positive Report Bounds, at Most 100000 Plot Points and a Downsample Strategy"));
    }
    let client = settings.client.as_ref().ok_or_else(|| invalid("Supply Client Preferences"))?;
    duration(client.connect_timeout_ms)?;
    duration(client.request_timeout_ms)?;
    let telemetry = settings.telemetry.as_ref().ok_or_else(|| invalid("Supply Telemetry Preferences"))?;
    duration(telemetry.interval_ms)?;
    let plugins = settings.plugins.as_ref().ok_or_else(|| invalid("Supply Plugin Preferences"))?;
    if !(1..=3600).contains(&plugins.startup_timeout_s) || !(1..=3600).contains(&plugins.stop_timeout_s) {
        return Err(invalid("Choose Plugin Timeouts from 1 to 3600 Seconds"));
    }
    if settings.listener.is_some() {
        return Err(invalid("Configure the Listener Through Access.Listener.Set"));
    }
    Ok(())
}

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Save Settings: {message}"))
}

fn duration(value: u64) -> Result<()> {
    if value == 0 || value > (1_u64 << 53) - 1 || Duration::from_millis(value).as_nanos() > i64::MAX as u128 {
        return Err(invalid("Supply a Positive Exact Duration Within the Clock Range"));
    }
    Ok(())
}

/* === Tests === */
