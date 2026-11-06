/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Preserve Plugin Installations Across An Explicit Storage Reset.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
};

// Third Party Imports
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// Local Imports
use super::{PluginError, PluginSettings, PluginState, ProgramValues};
use crate::{config::ApplicationMetadata, paths::ApplicationPaths, storage::sqlite::State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Temporary Copy Of Installed Bundles And Independently Readable Metadata.
pub struct PluginRecovery {
    bundles: tempfile::TempDir,
    metadata: ApplicationMetadata,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginRecovery {
    /// Read Without Opening Or Upgrading The Execution Store.
    /// Unreadable Metadata Does Not Prevent Recovering The Bundle Files.
    pub fn capture(root: &Path, application: &ApplicationPaths) -> Result<Self, PluginError> {
        let bundles = tempfile::tempdir()?;
        let mut metadata = ApplicationMetadata::new();
        let mut removed = BTreeSet::new();
        let mut disabled = BTreeSet::new();
        let mut inventory = None;
        if let Some(saved) = read_metadata(root) {
            if let Some(identity) = saved.get("identity").and_then(|value| serde_json::from_value(value.clone()).ok()) {
                metadata.identity = identity;
            }
            if let Some(secrets) = saved.get("secrets").and_then(|value| serde_json::from_value(value.clone()).ok()) {
                metadata.secrets = secrets;
            }
            if let Some(plugins) = saved.get("plugins").and_then(serde_json::Value::as_object) {
                inventory = Some(plugins.keys().filter_map(|id| id.parse::<Uuid>().ok()).collect::<BTreeSet<_>>());
                for (id, value) in plugins {
                    let Ok(id) = id.parse::<Uuid>() else {
                        continue;
                    };
                    let state = value.get("state").and_then(|state| serde_json::from_value::<PluginState>(state.clone()).ok());
                    if state == Some(PluginState::Disabled) {
                        disabled.insert(id);
                    }
                    if state == Some(PluginState::RemovalPending) {
                        removed.insert(id);
                        continue;
                    }
                    // Keep Readable Programs Without Letting One Bad Settings Record
                    // Discard The Installation's State Or Authorized Bundle Digest.
                    let Some(mut saved) = value.as_object().cloned() else { continue };
                    let programs = value
                        .get("programs")
                        .and_then(serde_json::Value::as_object)
                        .into_iter()
                        .flatten()
                        .filter_map(|(owner, values)| {
                            serde_json::from_value::<ProgramValues>(values.clone()).ok().map(|values| (owner.clone(), values))
                        })
                        .collect::<BTreeMap<_, _>>();
                    let programs = serde_json::to_value(programs).map_err(|error| PluginError::Invalid(error.to_string()))?;
                    saved.insert("programs".into(), programs);
                    saved.insert("last_boot".into(), serde_json::Value::Null);
                    let Ok(plugin) = serde_json::from_value::<PluginSettings>(serde_json::Value::Object(saved)) else {
                        continue;
                    };
                    if plugin.validate().is_ok() {
                        metadata.plugins.insert(id, plugin);
                    }
                }
            }
        }

        let entries = match fs::read_dir(application.plugins()) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Self { bundles, metadata }),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_none_or(|extension| extension != "dokime") || !entry.file_type()?.is_file() {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|name| name.to_str()).and_then(|name| name.parse::<Uuid>().ok()) else {
                continue;
            };
            // A Readable Inventory Never Resurrects Removed Or Orphan Bundles.
            if removed.contains(&id) || inventory.as_ref().is_some_and(|installed| !installed.contains(&id)) {
                continue;
            }
            let destination = bundles.path().join(entry.file_name());
            fs::copy(&path, &destination)?;
            fs::OpenOptions::new().write(true).open(&destination)?.sync_all()?;
            if let std::collections::btree_map::Entry::Vacant(entry) = metadata.plugins.entry(id) {
                let digest = file_digest(&destination)?;
                entry.insert(PluginSettings {
                    state: if disabled.contains(&id) { PluginState::Disabled } else { PluginState::Installed },
                    digest,
                    installed_at: crate::model::time::now_utc(),
                    last_boot: None,
                    programs: BTreeMap::new(),
                });
            }
        }
        metadata.validate().map_err(PluginError::Invalid)?;
        Ok(Self { bundles, metadata })
    }

    /// Recreate Installation Storage; Ordinary Startup Verifies And Admits Each Bundle.
    pub async fn restore(mut self, root: &Path, application: &ApplicationPaths) -> Result<(), PluginError> {
        // Only The New Database Receives Recovered Metadata, Never Execution State.
        let database = State::open_with(root, crate::config::local_storage_defaults()).await?;
        let result = async {
            let initialized = database
                .call(|connection| {
                    connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='application_metadata')", [], |row| {
                        row.get::<_, bool>(0)
                    })
                })
                .await?;
            if initialized {
                return Err(PluginError::Invalid("plugin recovery requires a fresh storage root".into()));
            }
            fs::create_dir_all(application.plugins())?;
            for entry in fs::read_dir(self.bundles.path())? {
                let entry = entry?;
                fs::copy(entry.path(), application.plugins().join(entry.file_name()))?;
            }
            for (id, plugin) in &mut self.metadata.plugins {
                plugin.last_boot = None;
                retain_valid_settings(&application.plugins().join(format!("{id}.dokime")), plugin);
            }
            database.initialize_settings().await?;
            let body = serde_json::to_vec(&self.metadata).map_err(|error| PluginError::Invalid(error.to_string()))?;
            database
                .call(move |connection| {
                    connection.execute("UPDATE application_metadata SET body=?1 WHERE id=1", [&body])?;
                    Ok(())
                })
                .await?;
            Ok::<_, PluginError>(())
        }
        .await;
        let closed = database.close().await;
        result?;
        closed?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn read_metadata(root: &Path) -> Option<serde_json::Value> {
    let connection = Connection::open_with_flags(root.join("local.sqlite"), OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    let body: Option<Vec<u8>> =
        connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)).optional().ok()?;
    serde_json::from_slice(&body?).ok()
}

fn file_digest(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

fn retain_valid_settings(path: &Path, plugin: &mut PluginSettings) {
    let verified = super::store::read_bounded(path, harmos_packaging::bundle::Limits::default().archive_bytes)
        .and_then(|bytes| super::inspect(&bytes, &crate::config::schema_defaults()));
    let Ok((bundle, _)) = verified else { return };
    let Ok(files) = super::catalog::selected_files(&bundle) else { return };
    let mut declarations = BTreeMap::new();
    for file in files {
        let Some(declaration) = bundle.declaration(&file.path) else { continue };
        let Some(executable) = &file.executable else { continue };
        if let Ok(program) = crate::domains::installed_plugins::contributions::decode_registrations(
            &declaration.id,
            &declaration.version.to_string(),
            &declaration.description,
            &executable.registrations,
        ) {
            declarations.insert(declaration.id.clone(), program.settings);
        }
    }
    plugin.programs.retain(|owner, values| {
        let Some(schema) = declarations.get(owner) else { return false };
        values.values.retain(|id, value| {
            schema.get(id).is_some_and(|definition| {
                !matches!(definition, crate::settings::SettingsEntry::Secret(_)) && definition.validate_value(value).is_ok()
            })
        });
        values.saved.retain(|id| values.values.contains_key(id));
        true
    });
}

/* === Tests === */
