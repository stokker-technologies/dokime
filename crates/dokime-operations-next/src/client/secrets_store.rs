/* ***
 *
 * Dokime
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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use base64::Engine;
use prost::Message;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

const SERVICE: &str = "dokime.shared-secrets";
const CATALOG_ITEMS: usize = 1024;
const SECRET_BYTES: usize = 65536;
const DOCUMENT_BYTES: usize = 524288;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Catalog {
    #[prost(uint64, tag = "1")]
    generation: u64,

    #[prost(string, tag = "3")]
    host_id: String,

    #[prost(message, repeated, tag = "2")]
    entries: Vec<Slot>,
}

#[derive(Clone, PartialEq, Message)]
struct Slot {
    #[prost(message, optional, tag = "1")]
    metadata: Option<v1::Metadata>,

    #[prost(string, tag = "2")]
    slot: String,
}

#[derive(Clone, PartialEq, Message)]
struct KeySlots {
    #[prost(string, tag = "1")]
    host_id: String,
    #[prost(string, repeated, tag = "2")]
    slots: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format_version: u8,
    entries: Vec<Transfer>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transfer {
    metadata: Metadata,
    value: Zeroizing<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    identifier: String,
    label: String,
    description: Option<String>,
}

/* === Enums === */

pub(super) enum Action {
    List,
    Lookup(String),
    Put(v1::SecretsSetRequest),
    Remove(v1::SecretsDeleteRequest),
    Export(v1::SecretsExportRequest),
    Preview(Vec<u8>),
    Import(v1::SecretsImportRequest),
}
pub(super) enum Reply {
    Snapshot(v1::SecretSnapshot),
    Lookup(v1::SecretWritable),
    Archive(v1::SecretArchive),
    Preview(v1::SecretImportPreview),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn remove_root_secrets(store: &dokime_storage::Store) -> Result<()> {
    let catalog: Catalog = store.administration("secrets")?.unwrap_or_default();
    let journal: KeySlots = store.administration("secret-key-slots")?.unwrap_or_default();
    let accounts: BTreeSet<_> = catalog
        .entries
        .iter()
        .map(|slot| format!("{}:{}", catalog.host_id, slot.slot))
        .chain(journal.slots.iter().map(|slot| format!("{}:{slot}", journal.host_id)))
        .collect();
    for account in accounts {
        let entry = keyring::Entry::new(SERVICE, &account).map_err(|_| unavailable())?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err(unavailable()),
        }
    }
    store.delete_administration("secret-key-slots")?;
    store.delete_administration("secrets")?;
    Ok(())
}

pub fn register_redactions(context: &Context) -> Result<()> {
    let catalog: Catalog = context.host.store().administration("secrets")?.unwrap_or_default();
    for slot in &catalog.entries {
        if let Some(metadata) = &slot.metadata {
            lookup(context, &catalog, &metadata.identifier)?;
        }
    }
    Ok(())
}

pub(super) async fn perform(context: &Context, action: Action) -> Result<Reply> {
    let context = context.clone();
    // Own Secure Publication Through Cancellation and Serialize Readers with Writers.
    tokio::spawn(async move {
        let guard = context.administration.clone().lock_owned().await;
        tokio::task::spawn_blocking(move || {
            let _guard = guard;
            run(&context, action)
        })
        .await
        .map_err(|_| unavailable())?
    })
    .await
    .map_err(|_| unavailable())?
}

pub(crate) fn availability(store: &dokime_storage::Store, identifier: &str) -> Result<bool> {
    let catalog: Catalog = store.administration("secrets")?.unwrap_or_default();
    Ok(catalog.entries.iter().any(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier)))
}
pub(crate) fn read_secret(store: &dokime_storage::Store, id: &str) -> Result<Vec<u8>> {
    identifier(id)?;
    let catalog: Catalog = store.administration("secrets")?.unwrap_or_default();
    let slot = catalog
        .entries
        .iter()
        .find(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == id))
        .ok_or_else(|| invalid("Set the Named Secret Before Reading It"))?;
    let entry = keyring::Entry::new(SERVICE, &format!("{}:{}", catalog.host_id, slot.slot)).map_err(|_| unavailable())?;
    let secret = Zeroizing::new(entry.get_password().map_err(credential_read_error)?);
    value(&secret)?;
    Ok(secret.as_bytes().to_vec())
}

/* === Private Functions === */

fn credential_read_error(error: keyring::Error) -> Error {
    match error {
        keyring::Error::NoEntry => {
            Error::Unavailable("Secret Storage: The Protected Slot Is Missing; Restore the Credential Store or Set the Secret Again".into())
        }
        keyring::Error::BadEncoding(_) => {
            Error::Unavailable("Secret Storage: The Protected Slot Cannot Be Decoded; Set the Secret Again".into())
        }
        _ => unavailable(),
    }
}
fn unavailable() -> Error {
    Error::Unavailable("Secret Storage Is Unavailable; Unlock the Operating System Credential Store and Retry".into())
}
fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Secret Storage: {message}"))
}
fn identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || matches!(value, "." | "..")
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(invalid("Use an Identifier of at Most 128 ASCII Letters, Digits, Dots, Underscores or Hyphens"));
    }
    Ok(())
}
fn metadata(value: &v1::Metadata) -> Result<()> {
    identifier(&value.identifier)?;
    if value.label.trim().is_empty()
        || value.label.len() > 256
        || value.label.contains('\0')
        || value.description.len() > 2048
        || value.description.contains('\0')
    {
        return Err(invalid("Supply a Nonempty Label up to 256 Bytes and Description up to 2048 Bytes Without NUL"));
    }
    Ok(())
}
fn value(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > SECRET_BYTES || value.contains('\0') {
        return Err(invalid("Supply a Nonempty Secret up to 65536 Bytes Without NUL"));
    }
    Ok(())
}
fn entry(context: &Context, slot: &str) -> Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, &format!("{}:{slot}", context.host.identity().backend_id)).map_err(|_| unavailable())
}
fn lookup(context: &Context, catalog: &Catalog, id: &str) -> Result<Zeroizing<String>> {
    identifier(id)?;
    let slot = catalog
        .entries
        .iter()
        .find(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == id))
        .ok_or_else(|| invalid("Set the Named Secret Before Looking It Up"))?;
    let result = Zeroizing::new(entry(context, &slot.slot)?.get_password().map_err(credential_read_error)?);
    value(&result)?;
    context.host.register_sensitive(&result)?;
    Ok(result)
}
fn snapshot(context: &Context, catalog: &Catalog, cleanup_pending: bool) -> Result<v1::SecretSnapshot> {
    let journal: KeySlots = context.host.store().administration("secret-key-slots")?.unwrap_or_default();
    let cleanup_pending = cleanup_pending || journal.slots.iter().any(|slot| !catalog.entries.iter().any(|entry| &entry.slot == slot));
    let plugins = context.host.plugins()?;
    let mut uses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for plugin in plugins {
        let Some(description) = plugin.description else {
            continue;
        };
        let Some(definition) = description.catalog else {
            continue;
        };
        for section in definition.sections {
            if section.r#type != "dokime.v1.SettingsCatalog" {
                continue;
            }
            for setting in v1::SettingsCatalog::decode(section.payload.as_slice())?.settings {
                if matches!(setting.kind, Some(v1::setting::Kind::Secret(_)))
                    && let Some(metadata) = setting.metadata
                {
                    uses.entry(metadata.identifier).or_default().insert(plugin.entry.clone());
                }
            }
        }
    }
    let identity = context.host.identity();
    Ok(v1::SecretSnapshot {
        host_id: identity.backend_id,
        host_label: identity.display_name,
        generation: catalog.generation,
        entries: catalog
            .entries
            .iter()
            .map(|slot| v1::SecretSummary {
                metadata: slot.metadata.clone(),
                known_uses: slot
                    .metadata
                    .as_ref()
                    .and_then(|metadata| uses.remove(&metadata.identifier))
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            })
            .collect(),
        cleanup_pending,
    })
}
fn decode(data: &[u8]) -> Result<Document> {
    if data.len() > DOCUMENT_BYTES.div_ceil(3) * 4 {
        return Err(invalid("Choose a Secret File Within the Transfer Limit"));
    }
    let bytes =
        Zeroizing::new(base64::engine::general_purpose::STANDARD.decode(data).map_err(|_| invalid("Choose a Valid Base64 Secret File"))?);
    if bytes.len() > DOCUMENT_BYTES {
        return Err(invalid("Choose a Secret File Within the Transfer Limit"));
    }
    let document: Document = serde_json::from_slice(&bytes).map_err(|_| invalid("Choose a Valid v1 Secret File"))?;
    if document.format_version != 1 || document.entries.is_empty() || document.entries.len() > CATALOG_ITEMS {
        return Err(invalid("Choose a Nonempty v1 Secret File Within the Entry Limit"));
    }
    let mut identifiers = BTreeSet::new();
    for item in &document.entries {
        let converted = v1::Metadata {
            identifier: item.metadata.identifier.clone(),
            label: item.metadata.label.clone(),
            description: item.metadata.description.clone().unwrap_or_default(),
            ..Default::default()
        };
        metadata(&converted)?;
        value(&item.value)?;
        if !identifiers.insert(&item.metadata.identifier) {
            return Err(invalid("Remove Duplicate Identifiers from the Secret File"));
        }
    }
    Ok(document)
}
fn run(context: &Context, action: Action) -> Result<Reply> {
    let store = context.host.store();
    let saved: Option<Catalog> = store.administration("secrets")?;
    let previous = saved.clone().unwrap_or_default();
    let (changes, removed, generation) = match action {
        Action::List => return Ok(Reply::Snapshot(snapshot(context, &previous, false)?)),
        Action::Lookup(id) => {
            return Ok(Reply::Lookup(v1::SecretWritable {
                metadata: previous
                    .entries
                    .iter()
                    .find(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == id))
                    .and_then(|slot| slot.metadata.clone()),
                value: Some(lookup(context, &previous, &id)?.to_string()),
            }));
        }
        Action::Put(request) => {
            let generation = u64::from_be_bytes(
                request.expected_revision.as_slice().try_into().map_err(|_| invalid("Use the Revision from Get Secret"))?,
            );
            let value = request.value.ok_or_else(|| invalid("Supply the Whole Secret Value"))?;
            let metadata = value.metadata.ok_or_else(|| invalid("Supply Secret Metadata"))?;
            if metadata.identifier != request.name {
                return Err(invalid("Keep the Secret Name and Metadata Identifier Consistent"));
            }
            (vec![(metadata, value.value.map(Zeroizing::new))], Vec::new(), generation)
        }
        Action::Remove(request) => {
            let generation = u64::from_be_bytes(
                request.expected_revision.as_slice().try_into().map_err(|_| invalid("Use the Revision from Get Secret"))?,
            );
            (Vec::new(), request.names, generation)
        }
        Action::Export(request) => {
            if request.generation != previous.generation {
                return Err(invalid("Refresh the Secret Catalog Before Exporting"));
            }
            if request.names.is_empty() || request.names.len() > CATALOG_ITEMS {
                return Err(invalid("Select a Nonempty Set Within the Entry Limit"));
            }
            let mut unique = BTreeSet::new();
            let mut entries = Vec::new();
            for id in request.names {
                if !unique.insert(id.clone()) {
                    return Err(invalid("Select Each Identifier Only Once"));
                }
                let secret = lookup(context, &previous, &id)?;
                let slot =
                    previous.entries.iter().find(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == id)).unwrap();
                let metadata = slot.metadata.as_ref().unwrap();
                entries.push(Transfer {
                    metadata: Metadata {
                        identifier: id,
                        label: metadata.label.clone(),
                        description: (!metadata.description.is_empty()).then(|| metadata.description.clone()),
                    },
                    value: secret,
                });
            }
            let document = Document { format_version: 1, entries };
            let bytes =
                Zeroizing::new(serde_json::to_vec(&document).map_err(|_| invalid("Reduce the Secret Selection to the Transfer Limit"))?);
            if bytes.len() > DOCUMENT_BYTES {
                return Err(invalid("Reduce the Secret Selection to the Transfer Limit"));
            }
            return Ok(Reply::Archive(v1::SecretArchive {
                data: base64::engine::general_purpose::STANDARD.encode(&*bytes).into_bytes(),
            }));
        }
        Action::Preview(data) => {
            let document = decode(&data)?;
            let identity = context.host.identity();
            return Ok(Reply::Preview(v1::SecretImportPreview {
                host_id: identity.backend_id,
                host_label: identity.display_name,
                generation: previous.generation,
                entries: document
                    .entries
                    .into_iter()
                    .map(|item| v1::SecretImportEntry {
                        exists: previous
                            .entries
                            .iter()
                            .any(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == item.metadata.identifier)),
                        metadata: Some(v1::Metadata {
                            identifier: item.metadata.identifier,
                            label: item.metadata.label,
                            description: item.metadata.description.unwrap_or_default(),
                            ..Default::default()
                        }),
                    })
                    .collect(),
            }));
        }
        Action::Import(request) => {
            let document = decode(&request.data)?;
            if request.choices.len() != document.entries.len() {
                return Err(invalid("Choose Replace or Skip for Every Imported Identifier"));
            }
            let mut choices = BTreeMap::new();
            for choice in request.choices {
                if !matches!(
                    v1::SecretImportAction::try_from(choice.action),
                    Ok(v1::SecretImportAction::Replace | v1::SecretImportAction::Skip)
                ) || choices.insert(choice.identifier, choice.action).is_some()
                {
                    return Err(invalid("Choose Replace or Skip Exactly Once per Identifier"));
                }
            }
            let mut changes = Vec::new();
            for item in document.entries {
                let action = choices.remove(&item.metadata.identifier).ok_or_else(|| invalid("Match Every Choice to the Imported File"))?;
                if action == v1::SecretImportAction::Skip as i32 {
                    continue;
                }
                changes.push((
                    v1::Metadata {
                        identifier: item.metadata.identifier,
                        label: item.metadata.label,
                        description: item.metadata.description.unwrap_or_default(),
                        ..Default::default()
                    },
                    Some(item.value),
                ));
            }
            (changes, Vec::new(), request.generation)
        }
    };
    if generation != previous.generation {
        return Err(Error::Conflict("Secret Storage: Refresh the Secret Catalog Before Saving This Change".into()));
    }
    let mut next = previous.clone();
    next.host_id = context.host.identity().backend_id;
    let mut touched = BTreeSet::new();
    let mut staged = Vec::new();
    for (metadata_value, secret) in &changes {
        metadata(metadata_value)?;
        if !touched.insert(metadata_value.identifier.clone()) {
            return Err(invalid("Select Each Identifier Only Once"));
        }
        if let Some(secret) = secret {
            value(secret)?;
            context.host.register_sensitive(secret)?;
        }
        let old = previous
            .entries
            .iter()
            .find(|slot| slot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == metadata_value.identifier));
        let slot = match secret {
            Some(_) => uuid::Uuid::now_v7().to_string(),
            None => old.ok_or_else(|| invalid("Supply a Value for a New Secret"))?.slot.clone(),
        };
        next.entries.retain(|slot| slot.metadata.as_ref().is_none_or(|metadata| metadata.identifier != metadata_value.identifier));
        next.entries.push(Slot {
            metadata: Some(metadata_value.clone()),
            slot: slot.clone(),
        });
        if let Some(secret) = secret {
            staged.push((slot, secret));
        }
    }
    for id in removed {
        identifier(&id)?;
        if !touched.insert(id.clone()) {
            return Err(invalid("Select Each Identifier Only Once"));
        }
        next.entries.retain(|slot| slot.metadata.as_ref().is_none_or(|metadata| metadata.identifier != id));
    }
    if next.entries.len() > CATALOG_ITEMS {
        return Err(invalid("Remove Secrets Before Exceeding the Entry Limit"));
    }
    next.entries.sort_by(|left, right| left.metadata.as_ref().unwrap().identifier.cmp(&right.metadata.as_ref().unwrap().identifier));
    if next == previous {
        return Ok(Reply::Snapshot(snapshot(context, &previous, false)?));
    }
    next.generation =
        previous.generation.checked_add(1).ok_or_else(|| invalid("Back Up Secret Storage Before Resetting an Exhausted Generation"))?;
    // Own Every Native Slot Durably Before Writing Any Protected Value.
    let mut journal: KeySlots = store.administration("secret-key-slots")?.unwrap_or_default();
    if (!journal.host_id.is_empty() && journal.host_id != next.host_id)
        || (!previous.host_id.is_empty() && previous.host_id != next.host_id)
    {
        return Err(invalid("Restore the Original Backend Identity Before Changing Secrets"));
    }
    journal.host_id = next.host_id.clone();
    journal.slots.extend(previous.entries.iter().map(|slot| slot.slot.clone()));
    journal.slots.extend(staged.iter().map(|(slot, _)| slot.clone()));
    journal.slots.sort();
    journal.slots.dedup();
    store.save_administration("secret-key-slots", &journal)?;
    for (slot, secret) in &staged {
        let written = entry(context, slot)?.set_password(secret);
        let confirmed = written.is_ok()
            && entry(context, slot)?.get_password().map(Zeroizing::new).is_ok_and(|value| value.as_str() == secret.as_str());
        if !confirmed {
            for (slot, _) in &staged {
                let _ = entry(context, slot)?.delete_credential();
            }
            return Err(unavailable());
        }
    }
    // Catalog Publication Follows Secure Writes; Uncertain Commits Retain the Secure Values.
    if !store.compare_administration("secrets", saved.as_ref(), &next)? {
        for (slot, _) in &staged {
            let _ = entry(context, slot)?.delete_credential();
        }
        return Err(Error::Conflict("Secret Storage: Refresh the Secret Catalog Before Saving This Change".into()));
    }
    let mut cleanup_pending = false;
    journal.slots.retain(|slot| {
        if next.entries.iter().any(|next| &next.slot == slot) {
            return true;
        }
        let removed = keyring::Entry::new(SERVICE, &format!("{}:{slot}", journal.host_id)).and_then(|entry| entry.delete_credential());
        if matches!(removed, Ok(()) | Err(keyring::Error::NoEntry)) {
            return false;
        }
        cleanup_pending = true;
        true
    });
    if let Err(error) = store.save_administration("secret-key-slots", &journal) {
        cleanup_pending = true;
        tracing::warn!(%error, "Record Secret Credential Cleanup: Retained Its Earlier Write Intents");
    }
    Ok(Reply::Snapshot(snapshot(context, &next, cleanup_pending)?))
}

/* === Tests === */
