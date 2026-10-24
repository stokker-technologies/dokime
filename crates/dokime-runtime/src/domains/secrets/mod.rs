/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(test)]
mod tests;
mod transfer;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{Arc, Mutex},
};

// Third Party Imports
use keyring::{Entry, Error as KeyringError};
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

// Local Imports
use crate::config::{RuntimeConfig, RuntimeSecretConfig, Settings};
use dokime_domain::model::secrets::catalog::{self, Catalog, Metadata, Slot};
use dokime_domain::model::secrets::{RedactedSecretBinding, SecretBackendKind};

/* === Constants === */

const SHARED_SERVICE: &str = "com.stokker.dokime.shared-secrets";
const LEGACY_SERVICE: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct SecretStore {
    settings: Settings,
    backend: Arc<dyn Backend>,
    legacy: RuntimeSecretConfig,
    writer: Arc<Mutex<()>>,
    transfer: Arc<Mutex<()>>,
    host: String,
    label: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretValue(Zeroizing<String>);

pub(crate) struct Change {
    pub metadata: Metadata,
    pub value: Option<SecretValue>,
}

struct OsKeyring;

struct CredentialStore(Arc<keyring_core::CredentialStore>);

#[cfg(test)]
#[derive(Clone, Default)]
struct TestStore {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

/* === Enums === */

#[derive(Debug, Error)]
pub(crate) enum SecretStoreError {
    #[error("{0}")]
    Invalid(&'static str),

    #[error("Secure secret storage is unavailable; no plaintext fallback is used")]
    Unavailable,

    #[error("Secret catalog changed; refresh and review the change again")]
    Conflict,

    #[error("Secret catalog publication could not be confirmed; secure values were retained. Refresh the catalog before retrying")]
    Persistence,
}

/* === Traits === */

trait Backend: Send + Sync {
    fn set(&self, slot: &str, value: &str) -> Result<(), SecretStoreError>;
    fn get(&self, slot: &str) -> Result<Option<SecretValue>, SecretStoreError>;
    fn remove(&self, slot: &str) -> Result<(), SecretStoreError>;
}

/* === Implementations === */

impl SecretStore {
    pub(crate) fn from_config(config: &RuntimeConfig, settings: Settings) -> Result<Self, SecretStoreError> {
        settings.read(|document| document.secrets.validate()).map_err(SecretStoreError::Invalid)?;
        let host = settings.read(|document| document.identity.node_id.to_string());
        let backend: Arc<dyn Backend> = match &config.credential_store {
            Some(store) => Arc::new(CredentialStore(Arc::clone(store))),
            None => Arc::new(OsKeyring),
        };
        Ok(Self {
            settings,
            backend,
            legacy: config.secret_backend.clone(),
            writer: Arc::new(Mutex::new(())),
            transfer: Arc::new(Mutex::new(())),
            host,
            label: config.hub_metadata.label.clone(),
        })
    }

    pub(crate) fn catalog(&self) -> Catalog {
        self.settings.read(|document| document.secrets.clone())
    }

    pub(crate) fn snapshot(&self, cleanup_pending: bool) -> catalog::Snapshot {
        self.settings.read(|document| {
            let mut uses: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for plugin in document.plugins.values() {
                for (program, values) in &plugin.programs {
                    for (setting, value) in &values.values {
                        if let Ok(crate::settings::SecretReference::StoredSecret { secret_id }) =
                            crate::settings::SecretReference::try_from(value)
                        {
                            uses.entry(secret_id).or_default().push(format!("{program}.{setting}"));
                        }
                    }
                }
            }
            let entries = document
                .secrets
                .entries
                .iter()
                .map(|(name, entry)| catalog::Summary {
                    metadata: entry.metadata.clone(),
                    known_uses: uses.remove(name).unwrap_or_default(),
                })
                .collect();
            catalog::Snapshot {
                host_id: self.host.clone(),
                host_label: self.label.clone(),
                generation: document.secrets.generation,
                entries,
                cleanup_pending,
            }
        })
    }

    pub(crate) fn set_secret(&self, name: &str, value: &str) -> Result<(), SecretStoreError> {
        let current = self.catalog();
        let metadata = current.entries.get(name).map(|entry| entry.metadata.clone()).unwrap_or_else(|| Metadata {
            name: name.into(),
            label: name.into(),
            description: None,
        });
        let change = Change { metadata, value: Some(SecretValue::new(value.to_owned())) };
        self.apply(vec![change], &[], current.generation)?;
        Ok(())
    }

    pub(crate) fn get_secret(&self, name: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        catalog::validate_name(name).map_err(SecretStoreError::Invalid)?;
        let _guard = self.writer.lock().map_err(|_| SecretStoreError::Unavailable)?;
        let current = self.catalog();
        let Some(entry) = current.entries.get(name) else { return Ok(None) };
        self.backend.get(&self.account(&entry.slot))?.map(Some).ok_or(SecretStoreError::Unavailable)
    }

    pub(crate) fn contains(&self, name: &str) -> Result<bool, SecretStoreError> {
        catalog::validate_name(name).map_err(SecretStoreError::Invalid)?;
        Ok(self.catalog().entries.contains_key(name))
    }

    pub(crate) fn redacted_binding(&self, name: impl Into<String>) -> Result<RedactedSecretBinding, SecretStoreError> {
        let secret_id = name.into();
        Ok(RedactedSecretBinding { configured: self.contains(&secret_id)?, backend: SecretBackendKind::OsKeyring, secret_id })
    }

    pub(crate) fn apply(&self, changes: Vec<Change>, removed: &[String], expected: u64) -> Result<bool, SecretStoreError> {
        let _guard = self.writer.lock().map_err(|_| SecretStoreError::Unavailable)?;
        let previous = self.catalog();
        if previous.generation != expected {
            return Err(SecretStoreError::Conflict);
        }
        let mut next = previous.clone();
        let mut names = BTreeSet::new();
        for change in &changes {
            change.metadata.validate().map_err(SecretStoreError::Invalid)?;
            if !names.insert(change.metadata.name.clone()) {
                return Err(SecretStoreError::Invalid("Duplicate secret selection"));
            }
            if let Some(value) = &change.value {
                catalog::validate_value(value.expose()).map_err(SecretStoreError::Invalid)?;
            }
            let slot = match &change.value {
                Some(_) => uuid::Uuid::now_v7().to_string(),
                None => previous
                    .entries
                    .get(&change.metadata.name)
                    .ok_or(SecretStoreError::Invalid("A new secret requires a value"))?
                    .slot
                    .clone(),
            };
            next.entries.insert(change.metadata.name.clone(), Slot { metadata: change.metadata.clone(), slot });
        }
        for name in removed {
            catalog::validate_name(name).map_err(SecretStoreError::Invalid)?;
            if !names.insert(name.clone()) {
                return Err(SecretStoreError::Invalid("Duplicate secret selection"));
            }
            next.entries.remove(name);
        }
        next.validate().map_err(SecretStoreError::Invalid)?;
        if next == previous {
            return Ok(false);
        }
        next.generation = expected.checked_add(1).ok_or(SecretStoreError::Invalid("Secret catalog generation exhausted"))?;

        // Secure Slots Exist Before Their Names Become Visible.
        let mut staged = Vec::new();
        for change in changes {
            let Some(value) = change.value else { continue };
            let slot = self.account(&next.entries[&change.metadata.name].slot);
            staged.push(slot.clone());
            if let Err(error) = self.backend.set(&slot, value.expose()) {
                for slot in staged {
                    let _ = self.backend.remove(&slot);
                }
                return Err(error);
            }
        }
        let publication = self.settings.edit(|document| {
            if document.secrets.generation != expected {
                return Err(SecretStoreError::Conflict);
            }
            document.secrets = next.clone();
            Ok(())
        });
        // A Directory Flush Can Fail After Rename. Retain Both Generations' Slots
        // So Either Complete Catalog Remains Readable After Recovery.
        publication?;

        // Readers Hold The Same Guard Through Their Snapshot's Secure Reads.
        let mut cleanup_pending = false;
        for (name, entry) in previous.entries {
            if next.entries.get(&name).is_none_or(|next| next.slot != entry.slot) {
                cleanup_pending |= self.backend.remove(&self.account(&entry.slot)).is_err();
            }
        }
        Ok(cleanup_pending)
    }

    pub(crate) fn adopt(&self, metadata: Metadata, legacy_id: &str, expected: u64) -> Result<bool, SecretStoreError> {
        metadata.validate().map_err(SecretStoreError::Invalid)?;
        catalog::validate_name(legacy_id).map_err(SecretStoreError::Invalid)?;
        let value = match self.legacy.backend {
            SecretBackendKind::OsKeyring => read_entry(LEGACY_SERVICE, legacy_id)?,
            SecretBackendKind::Memory => None,
            SecretBackendKind::LocalFile => {
                let path = self.legacy.local_file.as_ref().ok_or(SecretStoreError::Invalid("No legacy secret file is configured"))?;
                let bytes = harmos::filesystem::input::read(path, catalog::MAX_EXPORT_BYTES).map_err(|_| SecretStoreError::Unavailable)?;
                let bytes = Zeroizing::new(bytes);
                let mut values: BTreeMap<String, String> = serde_json::from_slice(&bytes)
                    .map_err(|_| SecretStoreError::Invalid("Legacy secret file is invalid and was retained"))?;
                let value = values.get(legacy_id).cloned().map(SecretValue::new);
                for value in values.values_mut() {
                    value.zeroize();
                }
                value
            }
        }
        .ok_or(SecretStoreError::Invalid("The selected legacy secret was not found"))?;
        self.apply(vec![Change { metadata, value: Some(value) }], &[], expected)
    }

    fn account(&self, slot: &str) -> String {
        format!("{}:{slot}", self.host)
    }
}

impl From<std::io::Error> for SecretStoreError {
    fn from(_: std::io::Error) -> Self {
        Self::Persistence
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue(<redacted>)")
    }
}

impl SecretValue {
    pub(crate) fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl Backend for OsKeyring {
    fn set(&self, slot: &str, value: &str) -> Result<(), SecretStoreError> {
        let entry = Entry::new(SHARED_SERVICE, slot).map_err(|_| SecretStoreError::Unavailable)?;
        entry.set_password(value).map_err(|_| SecretStoreError::Unavailable)
    }
    fn get(&self, slot: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        read_entry(SHARED_SERVICE, slot)
    }
    fn remove(&self, slot: &str) -> Result<(), SecretStoreError> {
        let entry = Entry::new(SHARED_SERVICE, slot).map_err(|_| SecretStoreError::Unavailable)?;
        match entry.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err(SecretStoreError::Unavailable),
        }
    }
}

impl CredentialStore {
    fn entry(&self, slot: &str) -> Result<keyring_core::Entry, SecretStoreError> {
        self.0.build(SHARED_SERVICE, slot, None).map_err(|_| SecretStoreError::Unavailable)
    }
}

impl Backend for CredentialStore {
    fn set(&self, slot: &str, value: &str) -> Result<(), SecretStoreError> {
        self.entry(slot)?.set_password(value).map_err(|_| SecretStoreError::Unavailable)
    }
    fn get(&self, slot: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        match self.entry(slot)?.get_password() {
            Ok(value) => Ok(Some(SecretValue::new(value))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err(SecretStoreError::Unavailable),
        }
    }
    fn remove(&self, slot: &str) -> Result<(), SecretStoreError> {
        match self.entry(slot)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err(SecretStoreError::Unavailable),
        }
    }
}

#[cfg(test)]
impl Backend for TestStore {
    fn set(&self, slot: &str, value: &str) -> Result<(), SecretStoreError> {
        self.values.lock().unwrap().insert(slot.into(), value.into());
        Ok(())
    }
    fn get(&self, slot: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        Ok(self.values.lock().unwrap().get(slot).cloned().map(SecretValue::new))
    }
    fn remove(&self, slot: &str) -> Result<(), SecretStoreError> {
        self.values.lock().unwrap().remove(slot);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn read_entry(service: &str, slot: &str) -> Result<Option<SecretValue>, SecretStoreError> {
    let entry = Entry::new(service, slot).map_err(|_| SecretStoreError::Unavailable)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(SecretValue::new(value))),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(_) => Err(SecretStoreError::Unavailable),
    }
}

/* === Tests === */
