/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Secret Bindings

/* === Modules === */

pub mod delete;
pub mod list;
pub mod set;

/* === Re-Exports === */

/* === Imports === */

use crate::host::{Context, storage::administration};
use crate::{Error, Result};
use dokime_protocol::host::plugins::{SecretBinding, SecretBindingValue};
use dokime_protocol::model::{SecretDeclaration, SecretsDeclarations};
use harmos_protocol::host::PluginEntry;
use harmos_protocol::model::Receipt;
use prost::Message;
use std::collections::{BTreeMap, BTreeSet};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
pub(crate) struct Bindings {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<Entry>,
}
#[derive(Clone, PartialEq, Message)]
struct Entry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(message, repeated, tag = "2")]
    bindings: Vec<SecretBindingValue>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn reconcile(context: &Context) -> Result<()> {
    let plugins = context.host.plugins().await?;
    let (entries, _) = administration::scan(context, "plugin-secrets/", None).await?;
    for entry in entries {
        let mut catalog = Bindings::decode(entry.payload.as_slice())?;
        let previous = catalog.clone();
        reconcile_entries(&mut catalog, &plugins)?;
        if catalog != previous {
            let snapshot = administration::read(context, &entry.key, &[], None).await?;
            let request = format!("reconcile/{}", uuid::Uuid::now_v7());
            administration::replace(context, &entry.key, snapshot.revision, &request, catalog.encode_to_vec()).await?;
            context.host.signal_change();
        }
    }
    Ok(())
}
pub async fn resolve(context: &Context, plugin: &str, identifier: &str) -> Result<(String, Vec<u8>)> {
    let refused = || Error::Permission("Get Secret: The Calling Plugin Cannot Read This Name".into());
    let plugins = context.host.plugins().await?;
    let plugin = plugins.iter().find(|entry| entry.name == plugin).ok_or_else(refused)?;
    let declarations = declarations(plugin).map_err(|_| refused())?;
    if !declarations.iter().any(|declaration| declaration.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier)) {
        return Err(refused());
    }
    let key = key(&plugin.name);
    let (catalog, revision) = administration::message::<Bindings>(context, &key).await?;
    let secret = catalog
        .entries
        .iter()
        .find(|entry| entry.name == plugin.name)
        .and_then(|entry| entry.bindings.iter().find(|binding| binding.identifier == identifier))
        .map(|binding| binding.secret.clone())
        .filter(|secret| !secret.is_empty())
        .ok_or_else(refused)?;
    Ok((secret, revision))
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
pub(crate) async fn values(context: &Context, name: &str, catalog: &Bindings) -> Result<Vec<SecretBinding>> {
    let plugins = context.host.plugins().await?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.name == name)
        .ok_or_else(|| Error::NotFound("List Plugin Secrets: Select a Plugin Entry".into()))?;
    let bindings = catalog.entries.iter().find(|entry| entry.name == name).map(|entry| entry.bindings.as_slice()).unwrap_or_default();
    let mut values = Vec::new();
    for declaration in declarations(plugin)? {
        let identifier = &declaration.metadata.as_ref().unwrap().identifier;
        let secret =
            bindings.iter().find(|binding| &binding.identifier == identifier).map(|binding| binding.secret.clone()).unwrap_or_default();
        let available = !secret.is_empty() && crate::host::secrets::store::availability(&context.host.store(), &secret)?;
        values.push(SecretBinding {
            declaration: Some(declaration),
            secret,
            available,
        });
    }
    Ok(values)
}
pub(crate) async fn update(
    context: &Context, name: &str, mut bindings: Vec<SecretBindingValue>, expected: Vec<u8>, request_id: &str,
) -> Result<(Vec<SecretBinding>, Vec<u8>, Receipt)> {
    let plugins = context.host.plugins().await?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.name == name)
        .ok_or_else(|| Error::NotFound("Set Plugin Secrets: Select a Plugin Entry".into()))?;
    validate(&declarations(plugin)?, &bindings)?;
    bindings.sort_by(|left, right| left.identifier.cmp(&right.identifier));
    let catalog = Bindings {
        entries: vec![Entry { name: name.into(), bindings }],
    };
    let key = key(name);
    let (snapshot, receipt) = administration::replace(context, &key, expected, request_id, catalog.encode_to_vec()).await?;
    let catalog = Bindings::decode(snapshot.payload.as_slice())?;
    let values = values(context, name, &catalog).await?;
    context.host.signal_change();
    Ok((values, snapshot.revision, receipt))
}
pub(crate) fn key(name: &str) -> String {
    format!("plugin-secrets/{name}")
}
pub(crate) fn selected<'a>(name: Option<&'a str>, action: &str) -> Result<&'a str> {
    name.filter(|name| !name.is_empty()).ok_or_else(|| Error::Invalid(format!("{action}: Select a Plugin Entry")))
}

/* === Private Functions === */

fn validate(declarations: &[SecretDeclaration], bindings: &[SecretBindingValue]) -> Result<()> {
    let names: BTreeSet<_> =
        declarations.iter().filter_map(|declaration| declaration.metadata.as_ref().map(|metadata| metadata.identifier.as_str())).collect();
    let mut seen = BTreeSet::new();
    for binding in bindings {
        if !names.contains(binding.identifier.as_str()) || !seen.insert(&binding.identifier) || binding.secret.is_empty() {
            return Err(Error::Invalid("Set Plugin Secrets: Bind Each Selected Declared Name Once to a Secret".into()));
        }
    }
    Ok(())
}
fn reconcile_entries(catalog: &mut Bindings, plugins: &[PluginEntry]) -> Result<()> {
    catalog.entries.retain(|entry| plugins.iter().any(|plugin| plugin.name == entry.name));
    for entry in &mut catalog.entries {
        let plugin = plugins.iter().find(|plugin| plugin.name == entry.name).unwrap();
        // Keep Cached Bindings Until the Library Has Observed the Plugin's Declarations.
        if plugin.description.is_none() {
            continue;
        }
        let declarations = declarations(plugin)?;
        let names: BTreeSet<_> = declarations
            .iter()
            .filter_map(|declaration| declaration.metadata.as_ref().map(|metadata| metadata.identifier.as_str()))
            .collect();
        entry.bindings.retain(|binding| names.contains(binding.identifier.as_str()));
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Bindings, Entry, declarations, reconcile_entries, validate};
    use dokime_protocol::host::plugins::SecretBindingValue;
    use dokime_protocol::model::{Metadata, SecretDeclaration, SecretsDeclarations};
    use harmos_protocol::host::{PluginDescription, PluginEntry};
    use harmos_protocol::model::{Declarations, Section};
    use prost::Message;

    fn declaration(name: &str) -> SecretDeclaration {
        SecretDeclaration {
            metadata: Some(Metadata {
                identifier: name.into(),
                ..Default::default()
            }),
            required: true,
        }
    }
    fn plugin(name: &str, names: &[&str]) -> PluginEntry {
        let declarations = SecretsDeclarations {
            secrets: names.iter().map(|name| declaration(name)).collect(),
        };
        PluginEntry {
            name: name.into(),
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
    fn binding_maps_reject_undeclared_duplicate_and_empty_targets() {
        let names = vec![declaration("credential")];
        let binding = SecretBindingValue {
            identifier: "credential".into(),
            secret: "protected-slot".into(),
        };
        assert!(validate(&names, &[binding.clone()]).is_ok());
        assert!(validate(&names, &[binding.clone(), binding]).is_err());
        assert!(
            validate(
                &names,
                &[SecretBindingValue {
                    identifier: "unknown".into(),
                    secret: "protected-slot".into()
                }]
            )
            .is_err()
        );
        assert!(
            validate(
                &names,
                &[SecretBindingValue {
                    identifier: "credential".into(),
                    secret: String::new()
                }]
            )
            .is_err()
        );
    }
    #[test]
    fn reconciliation_removes_deleted_plugins_and_retired_declarations() {
        let mut catalog = Bindings {
            entries: vec![
                Entry {
                    name: "present".into(),
                    bindings: vec![
                        SecretBindingValue {
                            identifier: "retained".into(),
                            secret: "first".into(),
                        },
                        SecretBindingValue {
                            identifier: "retired".into(),
                            secret: "second".into(),
                        },
                    ],
                },
                Entry {
                    name: "removed".into(),
                    bindings: vec![SecretBindingValue {
                        identifier: "credential".into(),
                        secret: "third".into(),
                    }],
                },
            ],
        };
        reconcile_entries(&mut catalog, &[plugin("present", &["retained"])]).unwrap();
        assert_eq!(catalog.entries.len(), 1);
        assert_eq!(catalog.entries[0].bindings.len(), 1);
        assert_eq!(catalog.entries[0].bindings[0].identifier, "retained");
    }
    #[test]
    fn catalog_admission_rejects_duplicate_secret_declarations() {
        assert!(declarations(&plugin("provider", &["same", "same"])).is_err());
        let mut catalog = Bindings {
            entries: vec![Entry {
                name: "unavailable".into(),
                bindings: vec![SecretBindingValue {
                    identifier: "cached".into(),
                    secret: "slot".into(),
                }],
            }],
        };
        let before = catalog.clone();
        reconcile_entries(
            &mut catalog,
            &[PluginEntry {
                name: "unavailable".into(),
                ..Default::default()
            }],
        )
        .unwrap();
        assert_eq!(catalog, before);
    }
}
