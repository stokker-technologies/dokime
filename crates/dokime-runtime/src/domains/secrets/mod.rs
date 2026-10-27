/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod transfer;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

// Third Party Imports
use keyring::{Entry, Error as KeyringError};
use thiserror::Error;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

// Local Imports
use crate::config::{RuntimeConfig, Settings};
use dokime_domain::model::secrets::catalog::{self, Catalog, Metadata, Slot};
use dokime_domain::model::secrets::{RedactedSecretBinding, SecretBackendKind};

/* === Constants === */

const SHARED_SERVICE: &str = "com.stokker.dokime.shared-secrets";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct SecretStore {
    settings: Settings,
    backend: Arc<dyn Backend>,
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

/* === Enums === */

#[derive(Debug, Error)]
pub(crate) enum SecretStoreError {
    #[error("Secret catalog storage failed: {0}")]
    Storage(#[from] crate::storage::Error),
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
    pub(crate) async fn from_config(config: &RuntimeConfig, settings: Settings) -> Result<Self, SecretStoreError> {
        settings.read(|document| document.secrets.validate()).await?.map_err(SecretStoreError::Invalid)?;
        settings.read(|document| document.secrets.check_limits()).await?.map_err(SecretStoreError::Invalid)?;
        let host = settings.read(|document| document.identity.node_id.to_string()).await?;
        let backend: Arc<dyn Backend> = match &config.credential_store {
            Some(store) => Arc::new(CredentialStore(Arc::clone(store))),
            None => Arc::new(OsKeyring),
        };
        Ok(Self {
            settings,
            backend,
            writer: Arc::new(Mutex::new(())),
            transfer: Arc::new(Mutex::new(())),
            host,
            label: config.hub_metadata.label.clone(),
        })
    }

    pub(crate) async fn catalog(&self) -> Result<Catalog, SecretStoreError> {
        Ok(self.settings.read(|document| document.secrets.clone()).await?)
    }

    pub(crate) async fn snapshot(&self, cleanup_pending: bool) -> Result<catalog::Snapshot, SecretStoreError> {
        let host = self.host.clone();
        let label = self.label.clone();
        Ok(self
            .settings
            .read(move |document| {
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
                    .map(|(identifier, entry)| catalog::Summary {
                        metadata: entry.metadata.clone(),
                        known_uses: uses.remove(identifier).unwrap_or_default(),
                    })
                    .collect();
                catalog::Snapshot { host_id: host, host_label: label, generation: document.secrets.generation, entries, cleanup_pending }
            })
            .await?)
    }

    pub(crate) async fn set_secret(&self, identifier: &str, value: &str) -> Result<(), SecretStoreError> {
        let current = self.catalog().await?;
        let metadata = current.entries.get(identifier).map(|entry| entry.metadata.clone()).unwrap_or_else(|| Metadata {
            identifier: identifier.into(),
            label: identifier.into(),
            description: None,
        });
        let change = Change { metadata, value: Some(SecretValue::new(value.to_owned())) };
        self.apply(vec![change], &[], current.generation).await?;
        Ok(())
    }

    pub(crate) async fn get_secret(&self, identifier: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        catalog::validate_identifier(identifier).map_err(SecretStoreError::Invalid)?;
        catalog::check_identifier_limit(identifier).map_err(SecretStoreError::Invalid)?;
        let guard = Arc::clone(&self.writer).lock_owned().await;
        let current = self.catalog().await?;
        let Some(entry) = current.entries.get(identifier) else { return Ok(None) };
        let account = self.account(&entry.slot);
        let backend = Arc::clone(&self.backend);
        let value = tokio::task::spawn_blocking(move || {
            let _guard = guard;
            backend.get(&account)
        })
        .await
        .map_err(|_| SecretStoreError::Unavailable)??
        .ok_or(SecretStoreError::Unavailable)?;
        catalog::validate_value(value.expose()).map_err(SecretStoreError::Invalid)?;
        catalog::check_value_limit(value.expose()).map_err(SecretStoreError::Invalid)?;
        Ok(Some(value))
    }

    pub(crate) async fn contains(&self, identifier: &str) -> Result<bool, SecretStoreError> {
        catalog::validate_identifier(identifier).map_err(SecretStoreError::Invalid)?;
        catalog::check_identifier_limit(identifier).map_err(SecretStoreError::Invalid)?;
        Ok(self.catalog().await?.entries.contains_key(identifier))
    }

    pub(crate) async fn redacted_binding(&self, identifier: impl Into<String>) -> Result<RedactedSecretBinding, SecretStoreError> {
        let secret_id = identifier.into();
        Ok(RedactedSecretBinding { configured: self.contains(&secret_id).await?, backend: SecretBackendKind::OsKeyring, secret_id })
    }

    pub(crate) async fn apply(&self, changes: Vec<Change>, removed: &[String], expected: u64) -> Result<bool, SecretStoreError> {
        let store = self.clone();
        let removed = removed.to_vec();
        // Publication And Secure Cleanup Remain Owned After Caller Cancellation.
        tokio::spawn(async move { store.apply_owned(changes, &removed, expected).await })
            .await
            .map_err(|_| SecretStoreError::Persistence)?
    }

    async fn apply_owned(&self, changes: Vec<Change>, removed: &[String], expected: u64) -> Result<bool, SecretStoreError> {
        let _guard = self.writer.lock().await;
        let previous = self.catalog().await?;
        if previous.generation != expected {
            return Err(SecretStoreError::Conflict);
        }
        let mut next = previous.clone();
        let mut identifiers = BTreeSet::new();
        for change in &changes {
            change.metadata.validate().map_err(SecretStoreError::Invalid)?;
            change.metadata.check_limits().map_err(SecretStoreError::Invalid)?;
            if !identifiers.insert(change.metadata.identifier.clone()) {
                return Err(SecretStoreError::Invalid("Duplicate secret selection"));
            }
            if let Some(value) = &change.value {
                catalog::validate_value(value.expose()).map_err(SecretStoreError::Invalid)?;
                catalog::check_value_limit(value.expose()).map_err(SecretStoreError::Invalid)?;
            }
            let slot = match &change.value {
                Some(_) => uuid::Uuid::now_v7().to_string(),
                None => previous
                    .entries
                    .get(&change.metadata.identifier)
                    .ok_or(SecretStoreError::Invalid("A new secret requires a value"))?
                    .slot
                    .clone(),
            };
            next.entries.insert(change.metadata.identifier.clone(), Slot { metadata: change.metadata.clone(), slot });
        }
        for identifier in removed {
            catalog::validate_identifier(identifier).map_err(SecretStoreError::Invalid)?;
            catalog::check_identifier_limit(identifier).map_err(SecretStoreError::Invalid)?;
            if !identifiers.insert(identifier.clone()) {
                return Err(SecretStoreError::Invalid("Duplicate secret selection"));
            }
            next.entries.remove(identifier);
        }
        next.validate().map_err(SecretStoreError::Invalid)?;
        next.check_limits().map_err(SecretStoreError::Invalid)?;
        if next == previous {
            return Ok(false);
        }
        next.generation = expected.checked_add(1).ok_or(SecretStoreError::Invalid("Secret catalog generation exhausted"))?;

        // Secure Slots Exist Before Their Names Become Visible.
        let mut staged = Vec::new();
        for change in changes {
            let Some(value) = change.value else { continue };
            let slot = self.account(&next.entries[&change.metadata.identifier].slot);
            staged.push(slot.clone());
            let backend = Arc::clone(&self.backend);
            let publication =
                tokio::task::spawn_blocking(move || backend.set(&slot, value.expose())).await.map_err(|_| SecretStoreError::Unavailable)?;
            if let Err(error) = publication {
                for slot in staged {
                    let _ = self.remove_secure(slot).await;
                }
                return Err(error);
            }
        }
        let published = next.clone();
        let publication = self.settings.edit(move |document| {
            if document.secrets.generation != expected {
                return Err(SecretStoreError::Conflict);
            }
            document.secrets = published;
            Ok(())
        });
        // Retain Secure Slots If The Database Commit Cannot Be Confirmed.
        publication.await?;

        // Readers Hold The Same Guard Through Their Snapshot's Secure Reads.
        let mut cleanup_pending = false;
        for (identifier, entry) in previous.entries {
            if next.entries.get(&identifier).is_none_or(|next| next.slot != entry.slot) {
                cleanup_pending |= self.remove_secure(self.account(&entry.slot)).await.is_err();
            }
        }
        Ok(cleanup_pending)
    }

    async fn remove_secure(&self, account: String) -> Result<(), SecretStoreError> {
        let backend = Arc::clone(&self.backend);
        tokio::task::spawn_blocking(move || backend.remove(&account)).await.map_err(|_| SecretStoreError::Unavailable)?
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
