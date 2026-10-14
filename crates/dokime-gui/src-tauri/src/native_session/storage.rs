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
    fs::{File, OpenOptions, TryLockError},
    path::{Path, PathBuf},
};
// Third Party Imports

// Local Imports
use super::{DesktopSessionError as Error, StorageResetReason as Reason};
use dokime::paths::ApplicationPaths;

/* === Constants === */
// The same override the backend reads; the owned backend inherits it.
const DATA_ROOT_ENV: &str = "DOKIME_OUTPUT_ROOT";
// The lease every backend holds on its store while it runs.
const LEASE: &str = "local.lock";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// The data directory an owned local backend opens, resolved the way the
/// backend resolves it, beside the application root it must never enclose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageRoot {
    data: PathBuf,
    application: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl StorageRoot {
    pub fn new(data: PathBuf, application: PathBuf) -> Self {
        Self { data, application }
    }

    /// Resolves the data root the owned backend will open: the environment
    /// override, then the configured `data_root`, then the default location.
    pub fn resolve(application: PathBuf) -> Result<Self, String> {
        let configured = match std::env::var_os(DATA_ROOT_ENV).filter(|value| !value.is_empty()) {
            Some(root) => Some(PathBuf::from(root)),
            None => configured_data_root(&ApplicationPaths::new(&application).settings())?,
        };
        // Keep the root in the form it was configured, as the backend does;
        // canonicalizing would give Windows a verbatim prefix it never uses.
        let data = configured.unwrap_or_else(|| ApplicationPaths::new(&application).default_data());
        let data = std::path::absolute(data).map_err(|error| error.to_string())?;
        Ok(Self::new(data, application))
    }

    pub fn path(&self) -> &Path {
        &self.data
    }

    /// Refuses any directory other than this data root, and a data root that
    /// is not a dedicated directory of its own.
    pub fn verify(&self, confirmed: &Path) -> Result<(), Error> {
        if confirmed != self.data {
            return Err(refusal(
                Reason::OutsideRoot,
                format!(
                    "{} is not the data directory of this desktop's backend ({}).",
                    confirmed.display(),
                    self.data.display()
                ),
            ));
        }
        if !self.data.is_absolute()
            || self.data.parent().is_none()
            || self.application.starts_with(&self.data)
        {
            return Err(refusal(
                Reason::OutsideRoot,
                format!(
                    "{} encloses more than this desktop's data; reset it by hand.",
                    self.data.display()
                ),
            ));
        }
        if std::fs::symlink_metadata(&self.data).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(refusal(
                Reason::OutsideRoot,
                format!(
                    "{} is a symbolic link; reset its target by hand.",
                    self.data.display()
                ),
            ));
        }
        Ok(())
    }

    /// Deletes everything inside the data root, keeping the root itself.
    /// Refuses while any backend still holds the store's lease.
    pub fn clear(&self) -> Result<(), Error> {
        // Prove No Backend Holds The Store
        let lease = match OpenOptions::new().write(true).open(self.data.join(LEASE)) {
            Ok(lease) => Some(lease),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(failure(&self.data, &error)),
        };
        if let Some(lease) = &lease {
            hold(lease, &self.data)?;
        }

        // Delete The Contents
        let entries = match std::fs::read_dir(&self.data) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(failure(&self.data, &error)),
        };
        for entry in entries {
            let entry = entry.map_err(|error| failure(&self.data, &error))?;
            if entry.file_name() == LEASE {
                continue;
            }
            // The entry's own type: a link is removed, never followed.
            let kind = entry
                .file_type()
                .map_err(|error| failure(&entry.path(), &error))?;
            let removed = if kind.is_dir() {
                std::fs::remove_dir_all(entry.path())
            } else {
                std::fs::remove_file(entry.path())
            };
            removed.map_err(|error| failure(&entry.path(), &error))?;
        }

        // Release And Remove The Lease Last
        // An open file cannot be deleted on Windows.
        if lease.is_some() {
            drop(lease);
            let path = self.data.join(LEASE);
            std::fs::remove_file(&path).map_err(|error| failure(&path, &error))?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn configured_data_root(settings: &Path) -> Result<Option<PathBuf>, String> {
    let bytes = match std::fs::read(settings) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{} could not be read: {error}", settings.display())),
    };
    let document: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} could not be parsed: {error}", settings.display()))?;
    Ok(document["data_root"].as_str().map(PathBuf::from))
}

fn hold(lease: &File, root: &Path) -> Result<(), Error> {
    match lease.try_lock() {
        Ok(()) => Ok(()),
        Err(TryLockError::WouldBlock) => Err(refusal(
            Reason::BackendRunning,
            format!(
                "A Dokime backend still holds {}; stop it before resetting.",
                root.display()
            ),
        )),
        Err(TryLockError::Error(error)) => Err(failure(root, &error)),
    }
}

fn refusal(reason: Reason, message: String) -> Error {
    Error::StorageReset { reason, message }
}

fn failure(path: &Path, error: &std::io::Error) -> Error {
    refusal(
        Reason::Failed,
        format!("{} could not be deleted: {error}.", path.display()),
    )
}

/* === Tests === */
