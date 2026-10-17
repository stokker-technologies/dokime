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
use harmos::bundle::Bundle;
use harmos::plugin::inventory::CarrierSection;
use serde::Deserialize;
// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) struct EnvironmentReference {
    pub(crate) owner: String,
    pub(crate) setting: String,
    pub(crate) variable: String,
}

// Read only the disclosure slice of the frozen product catalog. The host
// validates the complete domain; the CLI does not depend on its runtime.
#[derive(Deserialize)]
struct SettingsRegistration {
    catalog: SettingsCatalog,
}

#[derive(Deserialize)]
struct SettingsCatalog {
    contributions: Vec<SettingContribution>,
}

/* === Enums === */
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SettingContribution {
    SecretReference {
        id: String,
        default: Option<SecretDefault>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SecretDefault {
    Environment {
        variable: String,
    },
    #[serde(other)]
    Other,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// The environment variables a bundle's settings would read secrets from, by
/// name only, for the operator to review before consenting.
pub(crate) fn environment_references(bundle: &Bundle) -> Result<BTreeSet<EnvironmentReference>> {
    let mut references = BTreeSet::new();
    for file in &bundle.manifest().files {
        let Some(executable) = &file.executable else {
            continue;
        };
        let owner = match bundle.declaration(&file.path) {
            Some(declaration) if matches!(declaration.carrier, CarrierSection::Sidecar { .. }) => &declaration.id,
            Some(_) => {
                return Err(Error::Input(format!(
                    "Dokime admits native sidecar plugins only; `{}` declares a WebAssembly guest",
                    file.path
                )));
            }
            None => return Err(Error::Input("executable declaration missing".into())),
        };
        for registration in &executable.registrations {
            if registration.namespace != "dokime.settings" {
                continue;
            }
            if registration.schema != 3 {
                return Err(Error::Input(format!(
                    "cannot review settings schema {} for {owner}; rebuild the plugin with the current SDK",
                    registration.schema
                )));
            }
            let settings: SettingsRegistration = serde_json::from_slice(&registration.payload)?;
            for contribution in settings.catalog.contributions {
                if let SettingContribution::SecretReference { id, default: Some(SecretDefault::Environment { variable }) } = contribution {
                    let reference = EnvironmentReference { owner: owner.clone(), setting: id, variable };
                    references.insert(reference);
                }
            }
        }
    }
    Ok(references)
}

/* === Private Functions === */

/* === Tests === */
