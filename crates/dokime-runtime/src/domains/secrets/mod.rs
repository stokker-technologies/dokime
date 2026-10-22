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
use crate::config::RuntimeSecretConfig;
use dokime_domain::model::secrets::RedactedSecretBinding;
use dokime_domain::model::secrets::SecretBackendKind;

/* === Constants === */

const KEYRING_SERVICE: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct SecretStore {
    backend: Arc<dyn SecretBackend>,
    backend_kind: SecretBackendKind,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretValue(String);

#[derive(Default)]
struct MemorySecretBackend {
    secrets: Mutex<BTreeMap<String, String>>,
}

struct LocalFileSecretBackend {
    path: PathBuf,
    lock: Mutex<()>,
}

#[derive(Default)]
struct OsKeyringSecretBackend;

/* === Enums === */

#[derive(Debug, Error)]
pub(crate) enum SecretStoreError {
    #[error("secret id must not be empty")]
    EmptyId,

    #[error("secret id {0:?} is invalid")]
    InvalidId(String),

    #[error("secret backend failed: {0}")]
    Backend(String),
}

/* === Traits === */

trait SecretBackend: Send + Sync {
    fn set(&self, secret_id: &str, secret: &str) -> Result<(), SecretStoreError>;
    fn get(&self, secret_id: &str) -> Result<Option<SecretValue>, SecretStoreError>;
}

/* === Implementations === */

impl SecretStore {
    pub(crate) fn from_config(config: &RuntimeSecretConfig) -> Result<Self, SecretStoreError> {
        match config.backend {
            SecretBackendKind::OsKeyring => {
                Ok(Self { backend: Arc::new(OsKeyringSecretBackend), backend_kind: SecretBackendKind::OsKeyring })
            }
            SecretBackendKind::Memory => Ok(Self::memory()),
            SecretBackendKind::LocalFile => {
                let path = config
                    .local_file
                    .clone()
                    .ok_or_else(|| SecretStoreError::Backend("local-file secret backend requires a file path".to_owned()))?;
                Ok(Self {
                    backend: Arc::new(LocalFileSecretBackend { path, lock: Mutex::new(()) }),
                    backend_kind: SecretBackendKind::LocalFile,
                })
            }
        }
    }

    pub(crate) fn memory() -> Self {
        Self { backend: Arc::new(MemorySecretBackend::default()), backend_kind: SecretBackendKind::Memory }
    }

    pub(crate) fn set_secret(&self, secret_id: &str, secret: &str) -> Result<(), SecretStoreError> {
        validate_secret_id(secret_id)?;
        self.backend.set(secret_id, secret)
    }

    pub(crate) fn get_secret(&self, secret_id: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        validate_secret_id(secret_id)?;
        self.backend.get(secret_id)
    }

    pub(crate) fn contains(&self, secret_id: &str) -> Result<bool, SecretStoreError> {
        Ok(self.get_secret(secret_id)?.is_some())
    }

    pub(crate) fn redacted_binding(&self, secret_id: impl Into<String>) -> Result<RedactedSecretBinding, SecretStoreError> {
        let secret_id = secret_id.into();
        Ok(RedactedSecretBinding { configured: self.contains(&secret_id)?, backend: self.backend_kind, secret_id })
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue(<redacted>)")
    }
}

impl SecretValue {
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl MemorySecretBackend {
    fn secrets(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, String>> {
        self.secrets.lock().expect("memory secret backend lock is poisoned")
    }
}

impl SecretBackend for MemorySecretBackend {
    fn set(&self, secret_id: &str, secret: &str) -> Result<(), SecretStoreError> {
        self.secrets().insert(secret_id.to_owned(), secret.to_owned());
        Ok(())
    }

    fn get(&self, secret_id: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        Ok(self.secrets().get(secret_id).cloned().map(SecretValue))
    }
}

impl SecretBackend for LocalFileSecretBackend {
    fn set(&self, secret_id: &str, secret: &str) -> Result<(), SecretStoreError> {
        let _guard = self.lock.lock().expect("local secret backend lock is poisoned");
        let mut secrets = read_local_file(&self.path)?;
        secrets.insert(secret_id.to_owned(), secret.to_owned());
        write_local_file(&self.path, &secrets)
    }

    fn get(&self, secret_id: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        let _guard = self.lock.lock().expect("local secret backend lock is poisoned");
        Ok(read_local_file(&self.path)?.get(secret_id).cloned().map(SecretValue))
    }
}

impl SecretBackend for OsKeyringSecretBackend {
    fn set(&self, secret_id: &str, secret: &str) -> Result<(), SecretStoreError> {
        keyring_entry(secret_id)?.set_password(secret).map_err(keyring_error)
    }

    fn get(&self, secret_id: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        match keyring_entry(secret_id)?.get_password() {
            Ok(secret) => Ok(Some(SecretValue(secret))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error(error)),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_secret_id(secret_id: &str) -> Result<(), SecretStoreError> {
    if secret_id.trim().is_empty() {
        return Err(SecretStoreError::EmptyId);
    }
    if secret_id.contains(['/', '\\']) {
        return Err(SecretStoreError::InvalidId(secret_id.to_owned()));
    }
    Ok(())
}

fn keyring_entry(secret_id: &str) -> Result<Entry, SecretStoreError> {
    Entry::new(KEYRING_SERVICE, secret_id).map_err(keyring_error)
}

fn keyring_error(error: KeyringError) -> SecretStoreError {
    SecretStoreError::Backend(error.to_string())
}

fn read_local_file(path: &Path) -> Result<BTreeMap<String, String>, SecretStoreError> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let content = fs::read_to_string(path).map_err(|error| SecretStoreError::Backend(error.to_string()))?;
    serde_json::from_str(&content).map_err(|error| SecretStoreError::Backend(error.to_string()))
}

fn write_local_file(path: &Path, secrets: &BTreeMap<String, String>) -> Result<(), SecretStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| SecretStoreError::Backend(error.to_string()))?;
    }
    let content = serde_json::to_string_pretty(secrets).map_err(|error| SecretStoreError::Backend(error.to_string()))?;
    fs::write(path, content).map_err(|error| SecretStoreError::Backend(error.to_string()))
}

/* === Tests === */
