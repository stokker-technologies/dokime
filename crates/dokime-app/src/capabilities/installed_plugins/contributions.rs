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
use serde::de::DeserializeOwned;
// Local Imports
use crate::InstalledPlugin;
use crate::definition::SuiteSet;
use crate::records::RecordContract;
use crate::{Error, Result};
use dokime::domains::{OptionProviderContribution, Permission};
use dokime::plugin::registration::PROMPTS_NAMESPACE;
use dokime::plugin::registration::{
    APPEARANCE_NAMESPACE, AppearanceRegistration, LISTENERS_NAMESPACE, OPTIONS_NAMESPACE,
    PERMISSIONS_NAMESPACE, RECORDS_NAMESPACE, SETTINGS_NAMESPACE, SUITES_NAMESPACE,
    SettingsRegistration, TELEMETRY_NAMESPACE,
};
use dokime::prompts::PromptDefinition;
use dokime::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn decode_registrations(
    source: &str,
    version: &str,
    description: &str,
    registrations: &[harmos::plugin::sidecar::Registration],
) -> Result<InstalledPlugin> {
    if registrations.iter().any(|registration| {
        matches!(
            registration.namespace.as_str(),
            "dokime.config" | "dokime.credentials"
        )
    }) {
        return Err(Error::Operation(format!(
            "plugin `{source}` uses retired configuration or credential catalogs; rebuild with the unified settings catalog"
        )));
    }
    let settings = decode_once::<SettingsRegistration>(registrations, source, SETTINGS_NAMESPACE)?
        .unwrap_or_default();
    settings
        .catalog
        .validate()
        .map_err(|error| Error::Operation(error.to_string()))?;
    let appearance =
        decode_once::<AppearanceRegistration>(registrations, source, APPEARANCE_NAMESPACE)?
            .unwrap_or_default();
    Ok(InstalledPlugin {
        source: source.to_owned(),
        version: version.to_owned(),
        description: description.to_owned(),
        enabled: true,
        suites: decode_once::<SuiteSet>(registrations, source, SUITES_NAMESPACE)?
            .unwrap_or_default(),
        settings: settings.catalog,
        permissions: decode_once::<Vec<Permission>>(registrations, source, PERMISSIONS_NAMESPACE)?
            .unwrap_or_default(),
        record_contracts: decode_once::<Vec<RecordContract>>(
            registrations,
            source,
            RECORDS_NAMESPACE,
        )?
        .unwrap_or_default(),
        listeners: decode_once::<Vec<dokime::domains::PluginListener>>(
            registrations,
            source,
            LISTENERS_NAMESPACE,
        )?
        .unwrap_or_default(),
        prompts: decode_once::<Vec<PromptDefinition>>(registrations, source, PROMPTS_NAMESPACE)?
            .unwrap_or_default(),
        telemetry_contracts: decode_once::<Vec<TelemetryContract>>(
            registrations,
            source,
            TELEMETRY_NAMESPACE,
        )?
        .unwrap_or_default(),
        analyses: decode_once::<Vec<dokime::analysis::Definition>>(
            registrations,
            source,
            dokime::plugin::registration::ANALYSES_NAMESPACE,
        )?
        .unwrap_or_default(),
        settings_status: settings.status_entrypoint,
        settings_discoveries: settings.discoveries,
        app_customizations: appearance.customizations,
        app_themes: appearance.themes,
        option_providers: decode_once::<Vec<OptionProviderContribution>>(
            registrations,
            source,
            OPTIONS_NAMESPACE,
        )?
        .unwrap_or_default(),
    })
}

/* === Private Functions === */
fn decode_once<T: DeserializeOwned>(
    registrations: &[harmos::plugin::sidecar::Registration],
    source: &str,
    namespace: &str,
) -> Result<Option<T>> {
    let matching = registrations
        .iter()
        .filter(|entry| entry.namespace == namespace)
        .collect::<Vec<_>>();
    let Some(registration) = matching.first() else {
        return Ok(None);
    };
    if matching.len() != 1 {
        return Err(Error::Operation(format!(
            "plugin `{source}` registered `{namespace}` more than once"
        )));
    }
    if registration.schema != dokime::plugin::registration::domain_schema(namespace) {
        return Err(Error::Operation(format!(
            "plugin `{source}` registered unsupported `{namespace}` schema {}",
            registration.schema
        )));
    }
    serde_json::from_slice(&registration.payload)
        .map(Some)
        .map_err(|error| {
            Error::Operation(format!(
                "plugin `{source}` registered invalid `{namespace}` data: {error}"
            ))
        })
}

/* === Tests === */
