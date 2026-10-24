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

struct DocumentBuffer(Zeroizing<Vec<u8>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SecretStore {
    pub(crate) fn export(&self, names: &[String], generation: u64) -> Result<Archive, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = self.selected(names, generation)?;
        let mut serialized = DocumentBuffer(Zeroizing::new(Vec::new()));
        serde_json::to_writer(&mut serialized, &document)
            .map_err(|_| SecretStoreError::Invalid("The selected secrets exceed the transfer limit"))?;
        let data = base64::engine::general_purpose::STANDARD.encode(&*serialized.0).into_bytes();
        Ok(Archive { data })
    }

    pub(crate) fn preview(&self, data: &[u8]) -> Result<Preview, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decode(data)?;
        let current = self.catalog();
        let entries = document
            .entries
            .iter()
            .map(|entry| transfer::Entry { metadata: entry.metadata.clone(), exists: current.entries.contains_key(&entry.metadata.name) })
            .collect();
        Ok(Preview { host_id: self.host.clone(), host_label: self.label.clone(), generation: current.generation, entries })
    }

    pub(crate) fn import(&self, data: &[u8], choices: &[Choice], generation: u64) -> Result<bool, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decode(data)?;
        if choices.len() != document.entries.len() {
            return Err(SecretStoreError::Invalid("Choose replace or skip for every imported name"));
        }
        let mut actions = BTreeMap::new();
        for choice in choices {
            if actions.insert(choice.name.as_str(), choice.action).is_some() {
                return Err(SecretStoreError::Invalid("Duplicate import choice"));
            }
        }
        let mut changes = Vec::new();
        for mut entry in document.entries {
            match actions.remove(entry.metadata.name.as_str()) {
                Some(Action::Skip) => (),
                Some(Action::Replace) => changes
                    .push(Change { metadata: entry.metadata.clone(), value: Some(SecretValue::new(std::mem::take(&mut entry.value))) }),
                None => return Err(SecretStoreError::Invalid("Import choices do not match the archive")),
            }
        }
        self.apply(changes, &[], generation)
    }

    fn selected(&self, names: &[String], generation: u64) -> Result<Document, SecretStoreError> {
        if names.is_empty() || names.len() > catalog::MAX_SECRETS {
            return Err(SecretStoreError::Invalid("Select between 1 and 1024 secrets"));
        }
        let _guard = self.writer.lock().map_err(|_| SecretStoreError::Unavailable)?;
        let current = self.catalog();
        if current.generation != generation {
            return Err(SecretStoreError::Conflict);
        }
        let mut unique = BTreeSet::new();
        let mut entries = Vec::new();
        let mut bytes = 0;
        for name in names {
            if !unique.insert(name) {
                return Err(SecretStoreError::Invalid("Duplicate export selection"));
            }
            let slot = current.entries.get(name).ok_or(SecretStoreError::Invalid("A selected secret is missing or was deleted"))?;
            let value = self.backend.get(&self.account(&slot.slot))?.ok_or(SecretStoreError::Unavailable)?;
            bytes += value.expose().len();
            if bytes > transfer::MAX_DOCUMENT_BYTES {
                return Err(SecretStoreError::Invalid("The selected secrets exceed the transfer limit"));
            }
            entries.push(Entry { metadata: slot.metadata.clone(), value: value.expose().to_owned() });
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
        if bytes.len() > transfer::MAX_DOCUMENT_BYTES.saturating_sub(self.0.len()) {
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
    if data.is_empty() || data.len() > transfer::MAX_ARCHIVE_BYTES {
        return Err(SecretStoreError::Invalid("Secret archive exceeds its limit or is empty"));
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| SecretStoreError::Invalid("Secret archive is not valid Base64"))?;
    let document = Zeroizing::new(decoded);
    if document.len() > transfer::MAX_DOCUMENT_BYTES {
        return Err(SecretStoreError::Invalid("Decoded secret archive exceeds its limit"));
    }

    // Validate The Decoded Document Before Publishing Any Value.
    let document: Document =
        serde_json::from_slice(&document).map_err(|_| SecretStoreError::Invalid("Secret archive has an invalid document"))?;
    if document.format_version != 1 || document.entries.is_empty() || document.entries.len() > catalog::MAX_SECRETS {
        return Err(SecretStoreError::Invalid("Unsupported secret archive version or entry count"));
    }
    let mut names = BTreeSet::new();
    for entry in &document.entries {
        entry.metadata.validate().map_err(SecretStoreError::Invalid)?;
        catalog::validate_value(&entry.value).map_err(SecretStoreError::Invalid)?;
        if !names.insert(&entry.metadata.name) {
            return Err(SecretStoreError::Invalid("Duplicate secret name in archive"));
        }
    }
    Ok(document)
}

/* === Tests === */
