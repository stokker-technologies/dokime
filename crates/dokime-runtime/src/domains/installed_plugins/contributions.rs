/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::de::DeserializeOwned;

use crate::InstalledPlugin;
use crate::definition::SuiteSet;
use crate::domains::{OptionProviderContribution, Permission};
use crate::plugin::registration::PROMPTS_NAMESPACE;
use crate::plugin::registration::{
    APPEARANCE_NAMESPACE, AppearanceRegistration, LISTENERS_NAMESPACE, OPTIONS_NAMESPACE, PERMISSIONS_NAMESPACE, RECORDS_NAMESPACE,
    SETTINGS_NAMESPACE, SUITES_NAMESPACE, SettingsRegistration, TELEMETRY_NAMESPACE,
};
use crate::prompts::PromptDefinition;
use crate::records::RecordContract;
use crate::telemetry::TelemetryContract;
use crate::{runtime::Error, runtime::Result};

pub(crate) fn decode_registrations(
    source: &str, version: &str, description: &str, registrations: &[harmos::sidecar::Registration],
) -> Result<InstalledPlugin> {
    if registrations.iter().any(|registration| matches!(registration.namespace.as_str(), "dokime.config" | "dokime.secrets")) {
        return Err(Error::Operation(format!(
            "plugin `{source}` uses retired configuration or secret catalogs; rebuild with the unified settings catalog"
        )));
    }
    let settings = decode_once::<SettingsRegistration>(registrations, source, SETTINGS_NAMESPACE)?.unwrap_or_default();
    settings.catalog.validate().map_err(|error| Error::Operation(error.to_string()))?;
    let appearance = decode_once::<AppearanceRegistration>(registrations, source, APPEARANCE_NAMESPACE)?.unwrap_or_default();
    for customization in &appearance.customizations {
        for (index, logo) in customization.logo_assets.iter().enumerate() {
            if customization.logo_assets[..index].iter().any(|other| other.variant == logo.variant || other.asset_id == logo.asset_id) {
                return Err(Error::Operation(format!("plugin `{source}` registered duplicate logo categories or asset identifiers")));
            }
        }
    }
    let installation = InstalledPlugin {
        source: source.to_owned(),
        version: version.to_owned(),
        description: description.to_owned(),
        enabled: true,
        suites: decode_once::<SuiteSet>(registrations, source, SUITES_NAMESPACE)?.unwrap_or_default(),
        settings: settings.catalog,
        permissions: decode_once::<Vec<Permission>>(registrations, source, PERMISSIONS_NAMESPACE)?.unwrap_or_default(),
        record_contracts: decode_once::<Vec<RecordContract>>(registrations, source, RECORDS_NAMESPACE)?.unwrap_or_default(),
        listeners: decode_once::<Vec<crate::domains::PluginListener>>(registrations, source, LISTENERS_NAMESPACE)?.unwrap_or_default(),
        prompts: decode_once::<Vec<PromptDefinition>>(registrations, source, PROMPTS_NAMESPACE)?.unwrap_or_default(),
        telemetry_contracts: decode_once::<Vec<TelemetryContract>>(registrations, source, TELEMETRY_NAMESPACE)?.unwrap_or_default(),
        analyses: decode_once::<Vec<crate::analysis::Definition>>(registrations, source, crate::plugin::registration::ANALYSES_NAMESPACE)?
            .unwrap_or_default(),
        settings_status: settings.status_entrypoint,
        settings_discoveries: settings.discoveries,
        app_customizations: appearance.customizations,
        app_themes: appearance.themes,
        option_providers: decode_once::<Vec<OptionProviderContribution>>(registrations, source, OPTIONS_NAMESPACE)?.unwrap_or_default(),
    };
    crate::prompts::admission::validate_installation(&installation).map_err(Error::Operation)?;
    Ok(installation)
}

fn decode_once<T: DeserializeOwned>(registrations: &[harmos::sidecar::Registration], source: &str, namespace: &str) -> Result<Option<T>> {
    let matching = registrations.iter().filter(|entry| entry.namespace == namespace).collect::<Vec<_>>();
    let Some(registration) = matching.first() else {
        return Ok(None);
    };
    if matching.len() != 1 {
        return Err(Error::Operation(format!("plugin `{source}` registered `{namespace}` more than once")));
    }
    if registration.schema != crate::plugin::registration::domain_schema(namespace) {
        return Err(Error::Operation(format!("plugin `{source}` registered unsupported `{namespace}` schema {}", registration.schema)));
    }
    serde_json::from_slice(&registration.payload)
        .map(Some)
        .map_err(|error| Error::Operation(format!("plugin `{source}` registered invalid `{namespace}` data: {error}")))
}
