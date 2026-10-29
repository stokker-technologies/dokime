/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Decode Complete Bounded Files Before Preview Or Atomic Publication.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
};

// Third Party Imports
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

// Local Imports
use super::{Change, SecretStore, SecretStoreError, SecretValue};
use dokime_domain::model::secrets::{
    catalog,
    transfer::{self, Action, Archive, Choice, Preview},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format_version: u8,
    entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    metadata: catalog::Metadata,
    value: String,
}

struct DocumentBuffer(Zeroizing<Vec<u8>>, usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SecretStore {
    pub(crate) async fn export(&self, identifiers: &[String], generation: u64) -> Result<Archive, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = self.selected(identifiers, generation).await?;
        let mut serialized = DocumentBuffer(Zeroizing::new(Vec::new()), catalog::DOCUMENT_BYTES as usize);
        serde_json::to_writer(&mut serialized, &document)
            .map_err(|_| SecretStoreError::Invalid("The selected secrets exceed the transfer limit"))?;
        let data = base64::engine::general_purpose::STANDARD.encode(&*serialized.0).into_bytes();
        Ok(Archive { data })
    }

    pub(crate) async fn preview(&self, data: &[u8]) -> Result<Preview, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decode(data)?;
        let current = self.catalog().await?;
        let entries = document
            .entries
            .iter()
            .map(|entry| transfer::Entry {
                metadata: entry.metadata.clone(),
                exists: current.entries.contains_key(&entry.metadata.identifier),
            })
            .collect();
        Ok(Preview {
            host_id: self.host.clone(),
            host_label: self.label.clone(),
            generation: current.generation,
            entries,
        })
    }

    pub(crate) async fn import(&self, data: &[u8], choices: &[Choice], generation: u64) -> Result<bool, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decode(data)?;
        if choices.len() != document.entries.len() {
            return Err(SecretStoreError::Invalid("Choose replace or skip for every imported identifier"));
        }
        let mut actions = BTreeMap::new();
        for choice in choices {
            if actions.insert(choice.identifier.as_str(), choice.action).is_some() {
                return Err(SecretStoreError::Invalid("Duplicate import choice"));
            }
        }
        let mut changes = Vec::new();
        for mut entry in document.entries {
            match actions.remove(entry.metadata.identifier.as_str()) {
                Some(Action::Skip) => (),
                Some(Action::Replace) => changes.push(Change {
                    metadata: entry.metadata.clone(),
                    value: Some(SecretValue::new(std::mem::take(&mut entry.value))),
                }),
                None => return Err(SecretStoreError::Invalid("Import choices do not match the archive")),
            }
        }
        self.apply(changes, &[], generation).await
    }

    async fn selected(&self, identifiers: &[String], generation: u64) -> Result<Document, SecretStoreError> {
        if identifiers.is_empty() || identifiers.len() as u64 > catalog::CATALOG_ITEMS {
            return Err(SecretStoreError::Invalid("Select a nonempty set within the configured secret count"));
        }
        let guard = self.writer.clone().lock_owned().await;
        let current = self.catalog().await?;
        let store = self.clone();
        let identifiers = identifiers.to_vec();
        tokio::task::spawn_blocking(move || {
            let _guard = guard;
            store.selected_values(&identifiers, generation, current)
        })
        .await
        .map_err(|_| SecretStoreError::Unavailable)?
    }

    fn selected_values(&self, identifiers: &[String], generation: u64, current: catalog::Catalog) -> Result<Document, SecretStoreError> {
        if current.generation != generation {
            return Err(SecretStoreError::Conflict);
        }
        let mut unique = BTreeSet::new();
        let mut entries = Vec::new();
        let mut bytes = 0usize;
        for identifier in identifiers {
            if !unique.insert(identifier) {
                return Err(SecretStoreError::Invalid("Duplicate export selection"));
            }
            let slot = current.entries.get(identifier).ok_or(SecretStoreError::Invalid("A selected secret is missing or was deleted"))?;
            let value = self.backend.get(&self.account(&slot.slot))?.ok_or(SecretStoreError::Unavailable)?;
            catalog::validate_value(value.expose()).map_err(SecretStoreError::Invalid)?;
            catalog::check_value_limit(value.expose()).map_err(SecretStoreError::Invalid)?;
            bytes = bytes.checked_add(value.expose().len()).ok_or(SecretStoreError::Invalid("Secret transfer size overflow"))?;
            if bytes as u64 > catalog::DOCUMENT_BYTES {
                return Err(SecretStoreError::Invalid("The selected secrets exceed the transfer limit"));
            }
            entries.push(Entry {
                metadata: slot.metadata.clone(),
                value: value.expose().to_owned(),
            });
        }
        Ok(Document { format_version: 1, entries })
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

impl Write for DocumentBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.1.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other("secret transfer limit exceeded"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn decode(data: &[u8]) -> Result<Document, SecretStoreError> {
    if data.is_empty() || data.len() as u64 > catalog::DOCUMENT_BYTES.div_ceil(3) * 4 {
        return Err(SecretStoreError::Invalid("Secret archive exceeds its limit or is empty"));
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| SecretStoreError::Invalid("Secret archive is not valid Base64"))?;
    let document = Zeroizing::new(decoded);
    if document.len() as u64 > catalog::DOCUMENT_BYTES {
        return Err(SecretStoreError::Invalid("Decoded secret archive exceeds its limit"));
    }

    // Validate The Decoded Document Before Publishing Any Value.
    let document: Document =
        serde_json::from_slice(&document).map_err(|_| SecretStoreError::Invalid("Secret archive has an invalid document"))?;
    if document.format_version != 1 || document.entries.is_empty() || document.entries.len() as u64 > catalog::CATALOG_ITEMS {
        return Err(SecretStoreError::Invalid("Unsupported secret archive version or entry count"));
    }
    let mut identifiers = BTreeSet::new();
    for entry in &document.entries {
        entry.metadata.validate().map_err(SecretStoreError::Invalid)?;
        entry.metadata.check_limits().map_err(SecretStoreError::Invalid)?;
        catalog::validate_value(&entry.value).map_err(SecretStoreError::Invalid)?;
        catalog::check_value_limit(&entry.value).map_err(SecretStoreError::Invalid)?;
        if !identifiers.insert(&entry.metadata.identifier) {
            return Err(SecretStoreError::Invalid("Duplicate secret identifier in archive"));
        }
    }
    Ok(document)
}

/* === Tests === */
