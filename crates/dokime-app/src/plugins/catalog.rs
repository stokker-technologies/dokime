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
use std::collections::BTreeSet;
// Third Party Imports
use harmos::bundle::{Bundle, Identity, Limits};
use harmos::plugin::inventory::{CarrierSection, Declaration};
use serde::{Deserialize, Serialize};
// Local Imports
use super::PluginError;
use crate::capabilities::installed_plugins::contributions::decode_registrations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub identity: PluginIdentity,
    pub owners: Vec<String>,
    pub size_bytes: u64,
    pub environment_references: Vec<PluginEnvironmentReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginIdentity {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PluginEnvironmentReference {
    pub owner: String,
    pub setting: String,
    pub variable: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl From<&Identity> for PluginIdentity {
    fn from(identity: &Identity) -> Self {
        Self {
            name: identity.name.clone(),
            version: identity.version.to_string(),
            description: identity.description.clone(),
        }
    }
}

/* === Public Functions === */
pub fn inspect(bytes: &[u8]) -> Result<(Bundle, PluginMetadata), PluginError> {
    let bundle = Bundle::inspect(bytes, Limits::default())
        .map_err(|error| PluginError::Invalid(error.to_string()))?;
    let mut owners = BTreeSet::new();
    let mut environment_references = BTreeSet::new();
    // Validate each platform's catalog independently; selected platforms are
    // validated together against the complete host state during admission.
    for file in &bundle.manifest().files {
        let Some(executable) = &file.executable else {
            continue;
        };
        let (id, version, description, operations, contracts) = match bundle.declaration(&file.path)
        {
            Some(declaration) if native(declaration) => (
                &declaration.id,
                &declaration.version,
                &declaration.description,
                &declaration.operations,
                &declaration.contracts.provides,
            ),
            Some(_) => return Err(guest_refused(&file.path)),
            None => {
                return Err(PluginError::Invalid(
                    "executable declaration missing".into(),
                ));
            }
        };
        let installation = decode_registrations(
            id,
            &version.to_string(),
            description,
            &executable.registrations,
        )
        .map_err(|error| PluginError::Invalid(error.to_string()))?;
        super::dispatch::validate_analyses(&installation, contracts)
            .map_err(PluginError::Invalid)?;
        super::dispatch::Bindings::inspect(&installation, operations)
            .map_err(PluginError::Invalid)?;
        for setting in installation.settings.secrets() {
            if let Some(dokime::settings::SecretReference::Environment { variable }) =
                &setting.default
            {
                environment_references.insert(PluginEnvironmentReference {
                    owner: id.clone(),
                    setting: setting.id.clone(),
                    variable: variable.clone(),
                });
            }
        }
        crate::RuntimeState::default()
            .install_plugin(installation)
            .map_err(PluginError::Invalid)?;
        owners.insert(id.clone());
    }
    if owners.is_empty() {
        return Err(PluginError::Invalid(
            "plugin has no executable owners".into(),
        ));
    }
    selected_files(&bundle)?;
    let metadata = PluginMetadata {
        identity: (&bundle.manifest().identity).into(),
        owners: owners.into_iter().collect(),
        size_bytes: bytes.len() as u64,
        environment_references: environment_references.into_iter().collect(),
    };
    Ok((bundle, metadata))
}

pub(crate) fn selected_files(bundle: &Bundle) -> Result<Vec<&harmos::bundle::File>, PluginError> {
    let mut owners = std::collections::BTreeMap::<&str, Vec<&harmos::bundle::File>>::new();
    for file in &bundle.manifest().files {
        let Some(executable) = &file.executable else {
            continue;
        };
        let owner = match bundle.declaration(&file.path) {
            Some(value) if native(value) => value.id.as_str(),
            Some(_) => return Err(guest_refused(&file.path)),
            None => {
                return Err(PluginError::Invalid(
                    "missing executable declaration".into(),
                ));
            }
        };
        let native = owners.entry(owner).or_default();
        if executable.facility == harmos::bundle::Facility::Sidecar
            && executable.target == env!("DOKIME_NATIVE_TARGET")
        {
            native.push(file);
        }
    }
    owners
        .into_iter()
        .map(|(owner, native)| match native.as_slice() {
            [file] => Ok(*file),
            [] => Err(PluginError::Invalid(format!(
                "owner `{owner}` has no executable for this host"
            ))),
            _ => Err(PluginError::Invalid(format!(
                "owner `{owner}` has ambiguous executable variants for this host"
            ))),
        })
        .collect()
}

pub(crate) fn guest_refused(path: &str) -> PluginError {
    PluginError::Invalid(format!(
        "Dokime admits native sidecar plugins only; `{path}` declares a WebAssembly guest"
    ))
}

/* === Private Functions === */
pub(crate) fn native(declaration: &Declaration) -> bool {
    matches!(declaration.carrier, CarrierSection::Sidecar { .. })
}

/* === Tests === */
