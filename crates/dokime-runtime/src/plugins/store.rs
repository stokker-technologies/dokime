/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod query;

/* === Re-Exports === */
pub use query::PluginPage;

/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
// Third Party Imports
use sha2::{Digest, Sha256};
use uuid::Uuid;
// Local Imports
use super::model::{Installation, PluginSettings};
use super::{PluginEntry, PluginLastBoot, PluginMetadata, PluginState, inspect};
use crate::config::Settings;
use crate::core::Timestamp;

/* === Constants === */
const BUNDLE_EXTENSION: &str = "dokime";
const STAGE_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct PluginStore {
    settings: Settings,
    root: PathBuf,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin query budget exceeded")]
    Limit,
    #[error("plugin listing cursor changed or is invalid")]
    Cursor,
    #[error("invalid plugin: {0}")]
    Invalid(String),
    #[error("plugin install `{0}` was not found")]
    Unknown(Uuid),
    #[error("plugin install `{0}` is not staged")]
    NotStaged(Uuid),
    #[error("plugin install `{0}` is not installed")]
    NotInstalled(Uuid),
    #[error("plugin install `{0}` no longer matches its authorized digest")]
    Changed(Uuid),
    #[error("plugin owner `{0}` is already reserved")]
    Collision(String),
    #[error("plugin storage failed: {0}")]
    Io(#[from] std::io::Error),
}

/* === Traits === */

/* === Implementations === */
impl PluginStore {
    pub fn new(settings: Settings) -> Self {
        let root = settings.paths().plugins();
        Self { settings, root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, id: Uuid) -> PathBuf {
        self.root.join(format!("{id}.{BUNDLE_EXTENSION}"))
    }

    pub fn list(&self) -> Result<Vec<PluginEntry>, PluginError> {
        Ok(self
            .settings
            .read(|document| document.plugins.clone())
            .into_iter()
            .map(|(id, plugin)| self.entry(id, &plugin))
            .collect())
    }

    pub fn installed(&self) -> Result<Vec<super::PluginBundle>, PluginError> {
        Ok(self
            .settings
            .read(|document| document.plugins.clone())
            .into_iter()
            .filter(|(_, plugin)| plugin.state == PluginState::Installed)
            .map(|(install_id, plugin)| super::PluginBundle {
                path: self.path(install_id),
                programs: Default::default(),
                provenance: super::PluginProvenance::Installed {
                    install_id,
                    sha256: plugin.digest,
                },
            })
            .collect())
    }

    pub fn stage(&self, bytes: &[u8]) -> Result<PluginEntry, PluginError> {
        let (_, metadata) = inspect(bytes)?;
        let install_id = Uuid::now_v7();
        let sha256 = digest(bytes);

        // Publish The Verified Bundle File
        dokime_filesystem::publication::create_dir_all(&self.root).map_err(std::io::Error::from)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        let written = read_bounded(temporary.path(), bytes.len())?;
        if digest(&written) != sha256 {
            return Err(PluginError::Invalid(
                "written bundle differs from the inspected bytes".into(),
            ));
        }
        executable(temporary.as_file())?;
        dokime_filesystem::publication::install(temporary.path(), &self.path(install_id))
            .map_err(std::io::Error::from)?;

        // Authorize It
        // An unreferenced file grants nothing; the boot sweep reclaims it
        // when this edit fails after the rename.
        let staged_at = crate::model::time::now_utc();
        let plugin = PluginSettings {
            state: PluginState::Staged,
            digest: sha256,
            staged_at,
            expires_at: Some(Timestamp::from_system_time(
                SystemTime::now() + STAGE_RETENTION,
            )),
            consented_at: None,
            last_boot: None,
            programs: Default::default(),
        };
        self.settings.edit(|document| {
            document.plugins.insert(install_id, plugin.clone());
            Ok::<_, PluginError>(())
        })?;
        Ok(entry(plugin.installation(install_id), Some(metadata), None))
    }

    pub fn consent(
        &self,
        id: Uuid,
        configured: &[PluginMetadata],
        host_sources: &BTreeSet<String>,
    ) -> Result<PluginEntry, PluginError> {
        self.settings.edit(|document| {
            let target = document.plugins.get(&id).ok_or(PluginError::Unknown(id))?;
            if target.state == PluginState::RemovalPending {
                return Err(PluginError::NotStaged(id));
            }
            let (_, metadata) = self.verified(id, target)?;
            let mut occupied = host_sources.clone();
            for configured in configured {
                if configured.identity.name == metadata.identity.name {
                    return Err(PluginError::Collision(metadata.identity.name.clone()));
                }
                occupied.extend(configured.owners.iter().cloned());
            }
            let mut replaced = Vec::new();
            for (other_id, other) in &document.plugins {
                if *other_id == id
                    || !matches!(other.state, PluginState::Installed | PluginState::Disabled)
                {
                    continue;
                }
                // Unavailable installed bytes must be repaired or removed before
                // consenting a potentially colliding package. Never trust a cache.
                let (_, other_metadata) = self.verified(*other_id, other)?;
                if other_metadata.identity.name == metadata.identity.name {
                    replaced.push(*other_id);
                } else {
                    occupied.extend(other_metadata.owners);
                }
            }
            if let Some(owner) = metadata
                .owners
                .iter()
                .find(|owner| occupied.contains(*owner))
            {
                return Err(PluginError::Collision(owner.clone()));
            }

            let mut inherited = Default::default();
            for other_id in replaced {
                let other = document.plugins.get_mut(&other_id).expect("listed above");
                other.state = PluginState::RemovalPending;
                inherited = std::mem::take(&mut other.programs);
            }
            let plugin = document.plugins.get_mut(&id).expect("checked above");
            plugin.state = PluginState::Installed;
            plugin.expires_at = None;
            plugin.consented_at.get_or_insert_with(crate::model::time::now_utc);
            if plugin.programs.is_empty() {
                plugin.programs = inherited;
            }
            Ok(entry(plugin.installation(id), Some(metadata), None))
        })
    }

    pub fn remove(&self, id: Uuid) -> Result<(), PluginError> {
        // Revoke first, so failed deletion cannot admit bytes.
        let staged = self.settings.edit(|document| {
            let plugin = document
                .plugins
                .get_mut(&id)
                .ok_or(PluginError::Unknown(id))?;
            plugin.state = PluginState::RemovalPending;
            Ok::<_, PluginError>(plugin.consented_at.is_none())
        })?;
        if staged {
            self.discard(|candidate, _| candidate == id)?;
        }
        Ok(())
    }

    pub fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<(), PluginError> {
        self.settings.edit(|document| {
            let plugin = document
                .plugins
                .get_mut(&id)
                .ok_or(PluginError::Unknown(id))?;
            if !matches!(plugin.state, PluginState::Installed | PluginState::Disabled) {
                return Err(PluginError::NotInstalled(id));
            }
            if enabled {
                self.verified(id, plugin)?;
            }
            plugin.state = if enabled {
                PluginState::Installed
            } else {
                PluginState::Disabled
            };
            Ok(())
        })
    }

    pub fn sweep(&self) -> Result<(), PluginError> {
        let now = crate::model::time::now_utc();
        self.settings.edit(|document| {
            let expired = expired_stages(document.plugins.iter(), &now);
            for id in expired {
                if let Some(plugin) = document.plugins.get_mut(&id) {
                    plugin.state = PluginState::RemovalPending;
                }
            }
            Ok::<_, PluginError>(())
        })?;
        self.discard(|_, plugin| plugin.state == PluginState::RemovalPending)?;

        let authorized: BTreeSet<_> = self.settings.read(|document| {
            document
                .plugins
                .keys()
                .map(|id| format!("{id}.{BUNDLE_EXTENSION}"))
                .collect()
        });
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            if !entry.file_type()?.is_file()
                || name.to_str().is_some_and(|name| authorized.contains(name))
            {
                continue;
            }
            if let Err(error) = fs::remove_file(entry.path()) {
                tracing::warn!(%error, file = ?name, "could not delete an unauthorized plugin file");
            }
        }
        Ok(())
    }

    pub fn record_boot(&self, id: Uuid, error: Option<String>) -> Result<(), PluginError> {
        self.settings.edit(|document| {
            let plugin = document
                .plugins
                .get_mut(&id)
                .ok_or(PluginError::Unknown(id))?;
            let at = crate::model::time::now_utc();
            plugin.last_boot = Some(match error {
                Some(error) => PluginLastBoot::Failed { error, at },
                None => PluginLastBoot::Loaded { at },
            });
            Ok(())
        })
    }

    pub(crate) fn verified(
        &self,
        id: Uuid,
        plugin: &PluginSettings,
    ) -> Result<(harmos::bundle::Bundle, PluginMetadata), PluginError> {
        let bytes = read_bounded(
            &self.path(id),
            harmos::bundle::Limits::default().archive_bytes,
        )?;
        if digest(&bytes) != plugin.digest {
            return Err(PluginError::Changed(id));
        }
        inspect(&bytes)
    }

    fn entry(&self, id: Uuid, plugin: &PluginSettings) -> PluginEntry {
        Self::entry_from_verification(id, plugin, self.verified(id, plugin))
    }

    fn entry_from_verification(
        id: Uuid,
        plugin: &PluginSettings,
        verified: Result<(harmos::bundle::Bundle, PluginMetadata), PluginError>,
    ) -> PluginEntry {
        match verified {
            Ok((_, metadata)) => entry(plugin.installation(id), Some(metadata), None),
            Err(error) => entry(plugin.installation(id), None, Some(error.to_string())),
        }
    }

    fn discard(&self, selected: impl Fn(Uuid, &PluginSettings) -> bool) -> Result<(), PluginError> {
        let candidates: Vec<Uuid> = self.settings.read(|document| {
            document
                .plugins
                .iter()
                .filter(|(id, plugin)| selected(**id, plugin))
                .map(|(id, _)| *id)
                .collect()
        });
        if candidates.is_empty() {
            return Ok(());
        }
        let mut failures = Vec::new();
        for id in &candidates {
            match fs::remove_file(self.path(*id)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => failures.push((*id, error.to_string())),
            }
        }
        sync_directory(&self.root)?;
        self.settings.edit(|document| {
            for id in &candidates {
                if !failures.iter().any(|(failed, _)| failed == id) {
                    document.plugins.remove(id);
                }
            }
            for (id, error) in failures {
                if let Some(plugin) = document.plugins.get_mut(&id) {
                    plugin.last_boot = Some(PluginLastBoot::Failed {
                        error,
                        at: crate::model::time::now_utc(),
                    });
                }
            }
            Ok(())
        })
    }
}

/* === Public Functions === */
pub(crate) fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, PluginError> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(PluginError::Invalid("bundle must be a regular file".into()));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(PluginError::Invalid("bundle exceeds byte limit".into()));
    }
    Ok(bytes)
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

/* === Private Functions === */
fn entry(
    installation: Installation,
    metadata: Option<PluginMetadata>,
    error: Option<String>,
) -> PluginEntry {
    PluginEntry {
        restart_required: installation.state == PluginState::Installed,
        installation,
        metadata,
        error,
        active: false,
        boot_error: None,
    }
}

fn expired_stages<'a>(
    plugins: impl IntoIterator<Item = (&'a Uuid, &'a PluginSettings)>,
    now: &Timestamp,
) -> BTreeSet<Uuid> {
    let now = now.unix_millis().unwrap_or(0);
    plugins
        .into_iter()
        .filter(|(_, plugin)| {
            plugin.consented_at.is_none()
                && matches!(
                    plugin.state,
                    PluginState::Staged | PluginState::RemovalPending
                )
                && plugin
                    .expires_at
                    .as_ref()
                    .and_then(Timestamp::unix_millis)
                    .is_some_and(|at| at < now)
        })
        .map(|(id, _)| *id)
        .collect()
}

fn executable(file: &File) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = file;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    match File::open(path) {
        Ok(directory) => directory.sync_all()?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/* === Tests === */
