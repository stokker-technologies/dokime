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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Product Defaults Are Copied from the Old Application Preferences; Delete the Old Owner at Cutover.
pub(super) fn initial() -> v1::SettingsGetResponse {
    v1::SettingsGetResponse {
        revision: 0_u64.to_be_bytes().to_vec(),
        value: Some(v1::ApplicationSettings {
            appearance: Some(v1::ApplicationAppearance {
                theme_mode: v1::ThemeMode::System as i32,
                density: v1::Density::Comfortable as i32,
                ..Default::default()
            }),
            report: Some(v1::ReportDefaults {
                source_bytes: 64 * 1024 * 1024,
                value_depth: 128,
                table_rows: 32,
                plot_points: 2000,
                plot_downsample: v1::Downsample::Auto as i32,
            }),
            client: Some(v1::ClientPreferences {
                connect_timeout_ms: 10_000,
                request_timeout_ms: 30_000,
            }),
            telemetry: Some(v1::TelemetryPreferences { interval_ms: 1000 }),
            listener: None,
            plugins: Some(v1::PluginPreferences {
                startup_timeout_s: 60,
                stop_timeout_s: 10,
            }),
        }),
        resolved_at:None,
    }
}

pub(super) fn read(context: &Context) -> Result<v1::SettingsGetResponse> {
    let (payload,revision)=context.host.store().administration_value("preferences",&prost::Message::encode_to_vec(&initial().value.unwrap()))?;
    Ok(v1::SettingsGetResponse {value:Some(<v1::ApplicationSettings as prost::Message>::decode(payload.as_slice())?),revision,resolved_at:None})
}

pub(super) fn validate(settings: &v1::ApplicationSettings) -> Result<()> {
    let appearance = settings.appearance.as_ref().ok_or_else(|| invalid("Supply Appearance Preferences"))?;
    if appearance.theme_family.as_ref().is_some_and(|name| name.is_empty() || name.len() > 256) {
        return Err(invalid("Choose a Theme Family of 1 to 256 Bytes"));
    }
    if v1::ThemeMode::try_from(appearance.theme_mode).is_err()
        || appearance.theme_mode == 0
        || v1::Density::try_from(appearance.density).is_err()
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
        || v1::Downsample::try_from(report.plot_downsample).is_err()
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

pub(super) fn plugin(context: &Context, identifier: &str) -> Result<v1::plugins::SettingsGetResponse> {
    let plugin = context
        .host
        .plugins()?
        .into_iter()
        .find(|plugin| plugin.entry == identifier || plugin.description.as_ref().is_some_and(|description| description.id == identifier))
        .ok_or_else(|| invalid("Select an Installed or Remote Plugin"))?;
    for_plugin(&context.host.store(), plugin)
}

pub(crate) fn for_plugin(store: &dokime_storage::Store, plugin: v1::Plugin) -> Result<v1::plugins::SettingsGetResponse> {
    let settings_owner = plugin.description.as_ref().map(|description| description.id.clone()).unwrap_or_else(|| plugin.entry.clone());
    let mut settings = v1::SettingsCatalog::default();
    if let Some(catalog) = plugin.description.and_then(|description| description.catalog) {
        for section in catalog.sections {
            if section.r#type == "dokime.v1.SettingsCatalog" {
                settings.settings.extend(<v1::SettingsCatalog as prost::Message>::decode(section.payload.as_slice())?.settings);
            }
        }
    }
    let mut values = std::collections::HashMap::new();
    for setting in &settings.settings {
        let key = setting
            .metadata
            .as_ref()
            .map(|metadata| metadata.identifier.clone())
            .ok_or_else(|| invalid("Reconnect the Plugin with Named Settings"))?;
        let value = match &setting.kind {
            Some(v1::setting::Kind::Boolean(value)) => v1::value::Kind::Boolean(value.default_value),
            Some(v1::setting::Kind::Text(value)) => v1::value::Kind::Text(value.default_value.clone()),
            Some(v1::setting::Kind::Path(value)) => v1::value::Kind::Text(value.default_value.clone()),
            Some(v1::setting::Kind::Number(value)) => v1::value::Kind::Number(value.default_value),
            Some(v1::setting::Kind::Choice(value)) => v1::value::Kind::Text(value.default_value.clone()),
            Some(v1::setting::Kind::Secret(_)) => continue,
            None => return Err(invalid("Reconnect the Plugin with Typed Settings")),
        };
        values.insert(key, v1::Value { kind: Some(value) });
    }
    let saved: v1::MapValue = store.administration(&format!("plugin-settings:{}", settings_owner))?.unwrap_or_default();
    values.extend(saved.values);
    let mut secrets = Vec::new();
    for setting in &settings.settings {
        if !matches!(setting.kind, Some(v1::setting::Kind::Secret(_))) {
            continue;
        }
        let identifier = setting.metadata.as_ref().unwrap().identifier.clone();
        secrets.push(v1::SecretAvailability {
            available: super::secrets_store::availability(store, &identifier)?,
            identifier,
        });
    }
    Ok(v1::plugins::SettingsGetResponse {
        schema: Some(settings),
        value: Some(v1::MapValue { values }),
        secrets,
        ..Default::default()
    })
}

pub(super) fn validate_plugin(settings: &v1::SettingsCatalog, values: &v1::MapValue) -> Result<()> {
    for (key, value) in &values.values {
        let setting = settings
            .settings
            .iter()
            .find(|setting| setting.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == key))
            .ok_or_else(|| invalid("Refresh Plugin Settings Before Saving an Unknown Identifier"))?;
        let valid = match (&setting.kind, &value.kind) {
            (Some(v1::setting::Kind::Boolean(_)), Some(v1::value::Kind::Boolean(_))) => true,
            (Some(v1::setting::Kind::Text(setting)), Some(v1::value::Kind::Text(value))) => !setting.required || !value.is_empty(),
            (Some(v1::setting::Kind::Path(setting)), Some(v1::value::Kind::Text(value))) => !setting.required || !value.is_empty(),
            (Some(v1::setting::Kind::Number(setting)), Some(v1::value::Kind::Number(value))) => {
                value.is_finite() && setting.min.is_none_or(|min| *value >= min) && setting.max.is_none_or(|max| *value <= max)
            }
            (Some(v1::setting::Kind::Choice(setting)), Some(v1::value::Kind::Text(value))) => {
                setting.choices.iter().any(|choice| &choice.value == value) || setting.provider.is_some()
            }
            (Some(v1::setting::Kind::Secret(_)), Some(v1::value::Kind::Text(value))) => !value.trim().is_empty(),
            _ => false,
        };
        if !valid {
            return Err(invalid(&format!("Supply a Valid Value for Plugin Setting {key}")));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Save Settings: {message}"))
}

fn duration(value: u64) -> Result<()> {
    if value == 0 || value > (1_u64 << 53) - 1 || std::time::Duration::from_millis(value).as_nanos() > i64::MAX as u128 {
        return Err(invalid("Supply a Positive Exact Duration Within the Clock Range"));
    }
    Ok(())
}

/* === Tests === */
