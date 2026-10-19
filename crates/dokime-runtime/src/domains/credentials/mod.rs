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
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// Third Party Imports
use keyring::{Entry, Error as KeyringError};
use thiserror::Error;

// Local Imports
use crate::config::RuntimeCredentialConfig;
use dokime_domain::model::credentials::CredentialBackendKind;
use dokime_domain::model::credentials::RedactedCredentialBinding;

/* === Constants === */

const KEYRING_SERVICE: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct CredentialStore {
    backend: Arc<dyn CredentialBackend>,
    backend_kind: CredentialBackendKind,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CredentialSecret(String);

#[derive(Default)]
struct MemoryCredentialBackend {
    secrets: Mutex<BTreeMap<String, String>>,
}

struct LocalFileCredentialBackend {
    path: PathBuf,
    lock: Mutex<()>,
}

#[derive(Default)]
struct OsKeyringCredentialBackend;

/* === Enums === */

#[derive(Debug, Error)]
pub(crate) enum CredentialStoreError {
    #[error("credential id must not be empty")]
    EmptyId,

    #[error("credential id {0:?} is invalid")]
    InvalidId(String),

    #[error("credential backend failed: {0}")]
    Backend(String),
}

/* === Traits === */

trait CredentialBackend: Send + Sync {
    fn set(&self, credential_id: &str, secret: &str) -> Result<(), CredentialStoreError>;
    fn get(&self, credential_id: &str) -> Result<Option<CredentialSecret>, CredentialStoreError>;
}

/* === Implementations === */

impl CredentialStore {
    pub(crate) fn from_config(config: &RuntimeCredentialConfig) -> Result<Self, CredentialStoreError> {
        match config.backend {
            CredentialBackendKind::OsKeyring => {
                Ok(Self { backend: Arc::new(OsKeyringCredentialBackend), backend_kind: CredentialBackendKind::OsKeyring })
            }
            CredentialBackendKind::Memory => Ok(Self::memory()),
            CredentialBackendKind::LocalFile => {
                let path = config
                    .local_file
                    .clone()
                    .ok_or_else(|| CredentialStoreError::Backend("local-file credential backend requires a file path".to_owned()))?;
                Ok(Self {
                    backend: Arc::new(LocalFileCredentialBackend { path, lock: Mutex::new(()) }),
                    backend_kind: CredentialBackendKind::LocalFile,
                })
            }
        }
    }

    pub(crate) fn memory() -> Self {
        Self { backend: Arc::new(MemoryCredentialBackend::default()), backend_kind: CredentialBackendKind::Memory }
    }

    pub(crate) fn set_secret(&self, credential_id: &str, secret: &str) -> Result<(), CredentialStoreError> {
        validate_credential_id(credential_id)?;
        self.backend.set(credential_id, secret)
    }

    pub(crate) fn get_secret(&self, credential_id: &str) -> Result<Option<CredentialSecret>, CredentialStoreError> {
        validate_credential_id(credential_id)?;
        self.backend.get(credential_id)
    }

    pub(crate) fn contains(&self, credential_id: &str) -> Result<bool, CredentialStoreError> {
        Ok(self.get_secret(credential_id)?.is_some())
    }

    pub(crate) fn redacted_binding(&self, credential_id: impl Into<String>) -> Result<RedactedCredentialBinding, CredentialStoreError> {
        let credential_id = credential_id.into();
        Ok(RedactedCredentialBinding { configured: self.contains(&credential_id)?, backend: self.backend_kind, credential_id })
    }
}

impl fmt::Debug for CredentialSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialSecret(<redacted>)")
    }
}

impl CredentialSecret {
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl MemoryCredentialBackend {
    fn secrets(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, String>> {
        self.secrets.lock().expect("memory credential backend lock is poisoned")
    }
}

impl CredentialBackend for MemoryCredentialBackend {
    fn set(&self, credential_id: &str, secret: &str) -> Result<(), CredentialStoreError> {
        self.secrets().insert(credential_id.to_owned(), secret.to_owned());
        Ok(())
    }

    fn get(&self, credential_id: &str) -> Result<Option<CredentialSecret>, CredentialStoreError> {
        Ok(self.secrets().get(credential_id).cloned().map(CredentialSecret))
    }
}

impl CredentialBackend for LocalFileCredentialBackend {
    fn set(&self, credential_id: &str, secret: &str) -> Result<(), CredentialStoreError> {
        let _guard = self.lock.lock().expect("local credential backend lock is poisoned");
        let mut secrets = read_local_file(&self.path)?;
        secrets.insert(credential_id.to_owned(), secret.to_owned());
        write_local_file(&self.path, &secrets)
    }

    fn get(&self, credential_id: &str) -> Result<Option<CredentialSecret>, CredentialStoreError> {
        let _guard = self.lock.lock().expect("local credential backend lock is poisoned");
        Ok(read_local_file(&self.path)?.get(credential_id).cloned().map(CredentialSecret))
    }
}

impl CredentialBackend for OsKeyringCredentialBackend {
    fn set(&self, credential_id: &str, secret: &str) -> Result<(), CredentialStoreError> {
        keyring_entry(credential_id)?.set_password(secret).map_err(keyring_error)
    }

    fn get(&self, credential_id: &str) -> Result<Option<CredentialSecret>, CredentialStoreError> {
        match keyring_entry(credential_id)?.get_password() {
            Ok(secret) => Ok(Some(CredentialSecret(secret))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error(error)),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_credential_id(credential_id: &str) -> Result<(), CredentialStoreError> {
    if credential_id.trim().is_empty() {
        return Err(CredentialStoreError::EmptyId);
    }
    if credential_id.contains(['/', '\\']) {
        return Err(CredentialStoreError::InvalidId(credential_id.to_owned()));
    }
    Ok(())
}

fn keyring_entry(credential_id: &str) -> Result<Entry, CredentialStoreError> {
    Entry::new(KEYRING_SERVICE, credential_id).map_err(keyring_error)
}

fn keyring_error(error: KeyringError) -> CredentialStoreError {
    CredentialStoreError::Backend(error.to_string())
}

fn read_local_file(path: &Path) -> Result<BTreeMap<String, String>, CredentialStoreError> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let content = fs::read_to_string(path).map_err(|error| CredentialStoreError::Backend(error.to_string()))?;
    serde_json::from_str(&content).map_err(|error| CredentialStoreError::Backend(error.to_string()))
}

fn write_local_file(path: &Path, secrets: &BTreeMap<String, String>) -> Result<(), CredentialStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| CredentialStoreError::Backend(error.to_string()))?;
    }
    let content = serde_json::to_string_pretty(secrets).map_err(|error| CredentialStoreError::Backend(error.to_string()))?;
    fs::write(path, content).map_err(|error| CredentialStoreError::Backend(error.to_string()))
}

/* === Tests === */
