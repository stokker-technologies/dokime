/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Secret Requirements

/* === Modules === */

pub mod list;

/* === Re-Exports === */

/* === Imports === */

use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::plugins::SecretRequirement;
use dokime_protocol::model::{SecretDeclaration, SecretsDeclarations};
use harmos_protocol::host::PluginEntry;
use prost::Message;
use std::collections::BTreeMap;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Refuses an identifier the calling plugin did not declare; a declared one names its host-wide secret directly.
pub async fn require_declared(context: &Context, plugin: &str, identifier: &str, action: &str) -> Result<()> {
    let refused = || Error::Permission(format!("{action}: Declare the Secret {identifier:?} in the Calling Plugin"));
    let plugins = context.host.plugins().await?;
    let plugin = plugins.iter().find(|entry| entry.name == plugin).ok_or_else(refused)?;
    let declared = declarations(plugin)?
        .iter()
        .any(|declaration| declaration.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier));
    if !declared {
        return Err(refused());
    }
    Ok(())
}
pub(crate) fn declarations(plugin: &PluginEntry) -> Result<Vec<SecretDeclaration>> {
    let mut declarations = BTreeMap::new();
    let sections = plugin
        .description
        .as_ref()
        .and_then(|description| description.declarations.as_ref())
        .map(|catalog| catalog.sections.as_slice())
        .unwrap_or_default();
    for section in sections {
        if section.r#type != "dokime.model.v1.SecretsDeclarations" {
            continue;
        }
        for declaration in SecretsDeclarations::decode(section.payload.as_slice())?.secrets {
            let identifier = declaration
                .metadata
                .as_ref()
                .map(|metadata| metadata.identifier.as_str())
                .filter(|identifier| !identifier.is_empty())
                .ok_or_else(|| Error::Invalid("Read Secret Declarations: Supply a Declared Name".into()))?;
            if declarations.insert(identifier.to_owned(), declaration).is_some() {
                return Err(Error::Invalid("Read Secret Declarations: Declare Each Name Once".into()));
            }
        }
    }
    Ok(declarations.into_values().collect())
}
/// Each declared identifier and whether Secret Storage holds a secret under exactly that identifier.
pub(crate) async fn requirements(context: &Context, name: &str) -> Result<Vec<SecretRequirement>> {
    let plugins = context.host.plugins().await?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.name == name)
        .ok_or_else(|| Error::NotFound("List Plugin Secrets: Select a Plugin Entry".into()))?;
    let mut requirements = Vec::new();
    for declaration in declarations(plugin)? {
        let identifier = &declaration.metadata.as_ref().unwrap().identifier;
        let available = crate::host::secrets::store::availability(&context.host.store(), identifier)?;
        requirements.push(SecretRequirement {
            declaration: Some(declaration),
            available,
        });
    }
    Ok(requirements)
}
pub(crate) fn selected<'a>(name: Option<&'a str>, action: &str) -> Result<&'a str> {
    name.filter(|name| !name.is_empty()).ok_or_else(|| Error::Invalid(format!("{action}: Select a Plugin Entry")))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::declarations;
    use dokime_protocol::model::{Metadata, SecretDeclaration, SecretsDeclarations};
    use harmos_protocol::host::{PluginDescription, PluginEntry};
    use harmos_protocol::model::{Declarations, Section};
    use prost::Message;

    fn plugin(names: &[&str]) -> PluginEntry {
        let declarations = SecretsDeclarations {
            secrets: names
                .iter()
                .map(|name| SecretDeclaration {
                    metadata: Some(Metadata {
                        identifier: (*name).into(),
                        ..Default::default()
                    }),
                    required: true,
                })
                .collect(),
        };
        PluginEntry {
            name: "provider".into(),
            description: Some(PluginDescription {
                declarations: Some(Declarations {
                    sections: vec![Section {
                        r#type: "dokime.model.v1.SecretsDeclarations".into(),
                        payload: declarations.encode_to_vec(),
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    #[test]
    fn catalog_admission_rejects_duplicate_secret_declarations() {
        assert!(declarations(&plugin(&["same", "same"])).is_err());
        assert_eq!(declarations(&plugin(&["second", "first"])).unwrap().len(), 2);
    }
}
