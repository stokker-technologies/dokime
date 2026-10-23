/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authenticate Complete Bounded Archives Before Preview Or Atomic Publication.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
};

// Third Party Imports
use age::secrecy::SecretString;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

// Local Imports
use super::{Change, SecretStore, SecretStoreError, SecretValue};
use dokime_domain::model::secrets::{
    catalog,
    transfer::{self, Action, Archive, Choice, Preview},
};

/* === Constants === */

const WORK_FACTOR: u8 = 18;

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

struct Plaintext(Zeroizing<Vec<u8>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SecretStore {
    pub(crate) fn export(&self, names: &[String], password: Zeroizing<String>, generation: u64) -> Result<Archive, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        validate_password(&password)?;
        let document = self.selected(names, generation)?;
        let mut plaintext = Plaintext(Zeroizing::new(Vec::new()));
        serde_json::to_writer(&mut plaintext, &document)
            .map_err(|_| SecretStoreError::Invalid("The selected secrets exceed the transfer limit"))?;
        let ciphertext = encrypt(&plaintext.0, &password)?;
        Ok(Archive { ciphertext })
    }

    pub(crate) fn preview(&self, ciphertext: &[u8], password: Zeroizing<String>) -> Result<Preview, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decrypt(ciphertext, &password)?;
        let current = self.catalog();
        let entries = document
            .entries
            .iter()
            .map(|entry| transfer::Entry { metadata: entry.metadata.clone(), exists: current.entries.contains_key(&entry.metadata.name) })
            .collect();
        Ok(Preview { host_id: self.host.clone(), host_label: self.label.clone(), generation: current.generation, entries })
    }

    pub(crate) fn import(
        &self, ciphertext: &[u8], password: Zeroizing<String>, choices: &[Choice], generation: u64,
    ) -> Result<bool, SecretStoreError> {
        let _transfer = self.transfer.try_lock().map_err(|_| SecretStoreError::Invalid("Another secret transfer is running"))?;
        let document = decrypt(ciphertext, &password)?;
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
                None => return Err(SecretStoreError::Invalid("Import choices do not match the authenticated archive")),
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
            if bytes > transfer::MAX_PLAINTEXT_BYTES {
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

impl Write for Plaintext {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > transfer::MAX_PLAINTEXT_BYTES.saturating_sub(self.0.len()) {
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

fn validate_password(password: &str) -> Result<(), SecretStoreError> {
    if password.is_empty() || password.len() > transfer::MAX_PASSWORD_BYTES {
        return Err(SecretStoreError::Invalid("A transfer password must contain 1–1024 bytes"));
    }
    Ok(())
}

pub(super) fn encrypt(plaintext: &[u8], password: &str) -> Result<Vec<u8>, SecretStoreError> {
    let mut recipient = age::scrypt::Recipient::new(SecretString::from(password.to_owned()));
    recipient.set_work_factor(WORK_FACTOR);
    let recipients = std::iter::once(&recipient as &dyn age::Recipient);
    let encryptor = age::Encryptor::with_recipients(recipients).map_err(|_| SecretStoreError::Unavailable)?;
    let mut output = Vec::new();
    let mut writer = encryptor.wrap_output(&mut output).map_err(|_| SecretStoreError::Unavailable)?;
    writer.write_all(plaintext).map_err(|_| SecretStoreError::Unavailable)?;
    writer.finish().map_err(|_| SecretStoreError::Unavailable)?;
    if output.len() > transfer::MAX_CIPHERTEXT_BYTES {
        return Err(SecretStoreError::Invalid("Encrypted secret archive exceeds its limit"));
    }
    Ok(output)
}

fn decrypt(ciphertext: &[u8], password: &str) -> Result<Document, SecretStoreError> {
    validate_password(password)?;
    if ciphertext.is_empty() || ciphertext.len() > transfer::MAX_CIPHERTEXT_BYTES {
        return Err(SecretStoreError::Invalid("Encrypted secret archive exceeds its limit or is empty"));
    }
    let invalid = || SecretStoreError::Invalid("The secret archive could not be authenticated; check its password and contents");
    let decryptor = age::Decryptor::new(ciphertext).map_err(|_| invalid())?;
    if !decryptor.is_scrypt() {
        return Err(invalid());
    }
    let mut identity = age::scrypt::Identity::new(SecretString::from(password.to_owned()));
    identity.set_max_work_factor(WORK_FACTOR);
    let identities = std::iter::once(&identity as &dyn age::Identity);
    let reader = decryptor.decrypt(identities).map_err(|_| invalid())?;
    let mut plaintext = Zeroizing::new(Vec::new());
    reader.take((transfer::MAX_PLAINTEXT_BYTES + 1) as u64).read_to_end(&mut plaintext).map_err(|_| invalid())?;
    if plaintext.len() > transfer::MAX_PLAINTEXT_BYTES {
        return Err(SecretStoreError::Invalid("Decrypted secret archive exceeds its limit"));
    }

    // read_to_end Has Verified The Final AEAD Chunk And Rejected Trailing Ciphertext.
    let document: Document = serde_json::from_slice(&plaintext)
        .map_err(|_| SecretStoreError::Invalid("Authenticated secret archive has an invalid document"))?;
    if document.format_version != 1 || document.entries.is_empty() || document.entries.len() > catalog::MAX_SECRETS {
        return Err(SecretStoreError::Invalid("Unsupported secret archive version or entry count"));
    }
    let mut names = BTreeSet::new();
    for entry in &document.entries {
        entry.metadata.validate().map_err(SecretStoreError::Invalid)?;
        catalog::validate_value(&entry.value).map_err(SecretStoreError::Invalid)?;
        if !names.insert(&entry.metadata.name) {
            return Err(SecretStoreError::Invalid("Duplicate secret name in authenticated archive"));
        }
    }
    Ok(document)
}

/* === Tests === */
