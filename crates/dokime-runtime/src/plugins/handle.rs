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
use std::collections::BTreeSet;

// Third Party Imports
use uuid::Uuid;

// Local Imports
use super::store::read_bounded;
use super::{PluginEntry, PluginError, PluginProvenance, PluginRestart, PluginState, PluginStore};
use crate::RuntimeHandle;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub fn plugin_process_state(&self, entry: &mut PluginEntry) {
        let attempted = self.plugin_boot.read().expect("plugin boot lock poisoned").get(&entry.installation.install_id).copied();
        process_state(entry, attempted);
        // Only An Installation That Runs Or Should Run Waits For The Backend To Apply It.
        let concerned = entry.active || entry.installation.state == PluginState::Installed;
        entry.restart = match package(entry) {
            Some(package) if concerned => self.plugin_bundle_restart(package),
            _ => PluginRestart::Idle,
        };
    }

    pub fn plugin_store(&self) -> PluginStore {
        PluginStore::with_archive_limit(
            self.settings().clone(),
            *self.schema_settings(),
            usize::try_from(dokime_domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES)
                .expect("validated server plugin bundle limit fits native address space"),
        )
    }

    pub async fn install_plugin(
        &self, bytes: Vec<u8>, validate: impl FnOnce(&PluginEntry) -> Result<(), PluginError> + Send + 'static,
    ) -> Result<PluginEntry, PluginError> {
        let store = self.plugin_store();
        let configured = self.config().plugins.clone();
        let host_sources: BTreeSet<_> = self
            .state_read()
            .plugin_owners
            .iter()
            .filter_map(|(source, owner)| matches!(owner, crate::state::PluginOwner::Host).then_some(source.clone()))
            .collect();
        let inspector = store.clone();
        let configured = tokio::task::spawn_blocking(move || {
            let configured = configured
                .iter()
                .filter(|bundle| matches!(bundle.provenance, PluginProvenance::Configured))
                .map(|bundle| {
                    let bytes = read_bounded(&bundle.path, inspector.archive_bytes())?;
                    super::catalog::inspect_with_archive_limit(&bytes, &inspector.schema, inspector.archive_bytes())
                        .map(|(_, metadata)| metadata)
                })
                .collect::<Result<Vec<_>, PluginError>>()?;
            Ok::<_, PluginError>(configured)
        })
        .await
        .map_err(|error| PluginError::Io(std::io::Error::other(error.to_string())))??;
        let entry = store.install_checked(&bytes, &configured, &host_sources, validate).await?;
        self.apply_plugin_change(&entry);
        Ok(entry)
    }

    // Enable Or Disable An Installation; The Running Backend Starts Or Stops It Once Nothing Uses It.
    pub async fn set_plugin_enabled(&self, install: Uuid, enabled: bool) -> Result<PluginEntry, PluginError> {
        let store = self.plugin_store();
        store.set_enabled(install, enabled).await?;
        let entry = store.get(install).await?;
        self.apply_plugin_change(&entry);
        Ok(entry)
    }

    // Queue An Installation For Removal; The Running Backend Stops It, And The Next Start Deletes Its File.
    pub async fn remove_plugin(&self, install: Uuid) -> Result<Option<PluginEntry>, PluginError> {
        let store = self.plugin_store();
        store.remove(install).await?;
        let entry = match store.get(install).await {
            Ok(entry) => entry,
            Err(PluginError::Unknown(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        self.apply_plugin_change(&entry);
        Ok(Some(entry))
    }

    // Start An Installation Again Through The Same Path: Its Bundle Starts If It Does Not Run,
    // And Programs That Run Restart With Their Saved Settings.
    pub async fn retry_plugin(&self, install: Uuid) -> Result<PluginEntry, PluginError> {
        let entry = self.plugin_store().get(install).await?;
        if entry.installation.state != PluginState::Installed {
            return Err(PluginError::NotInstalled(install));
        }
        self.apply_plugin_change(&entry);
        let running = package(&entry).and_then(|package| self.plugin_runs(package, install));
        for program in running.into_iter().flatten() {
            self.request_plugin_restart(&program);
        }
        Ok(entry)
    }

    // A Store Change Reaches The Running Backend: The Restarter Applies It Once Nothing Uses The Package.
    fn apply_plugin_change(&self, entry: &PluginEntry) {
        let install = entry.installation.install_id;
        let Some(package) = package(entry).map(str::to_owned).or_else(|| self.running_package(install)) else {
            return;
        };
        // Saved Program Values Move With The Installation That Runs Next.
        if entry.installation.state == PluginState::Installed {
            let owners = entry.metadata.iter().flat_map(|metadata| &metadata.owners);
            for owner in owners.filter_map(|owner| Identifier::new(owner).ok()) {
                self.plugin_preferences().bind_install(&owner, install);
            }
        }
        self.request_plugin_apply(&package, install);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn package(entry: &PluginEntry) -> Option<&str> {
    entry.metadata.as_ref().map(|metadata| metadata.identity.name.as_str())
}

fn process_state(entry: &mut PluginEntry, attempted: Option<bool>) {
    entry.active = attempted == Some(true);
    // Only an attempt this process itself made and refused ever surfaces a
    // reason. No caller ever observes a `RuntimeHandle` until `install_plugins`
    // has returned `Ok` -- every early `Err` out of it instead shuts the
    // runtime back down (see `Runtime::start` and
    // `with_execution_fixtures_inner`) -- and every plugin still `false` in
    // `plugin_boot` when it returns `Ok` was already given its own
    // `record_boot` call along the way. A live change that marks a plugin
    // `false` records its outcome the same way. So whenever
    // `attempted == Some(false)` is observable at all, `last_boot` holds
    // *this* process's latest attempt, never a stale one from an earlier one.
    entry.boot_failed = attempted == Some(false);
}

/* === Tests === */
