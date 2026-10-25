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

// Local Imports
use super::store::read_bounded;
use super::{PluginEntry, PluginError, PluginProvenance, PluginStore, inspect};
use crate::RuntimeHandle;

/* === Constants === */

const GENERIC_BOOT_ERROR: &str = "Plugin was not admitted at this application startup.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub fn plugin_process_state(&self, entry: &mut PluginEntry) {
        let boot = self.plugin_boot.read().expect("plugin boot lock poisoned");
        let attempted = boot.get(&entry.installation.install_id).copied();
        process_state(entry, attempted);
    }

    pub fn plugin_store(&self) -> PluginStore {
        PluginStore::new(self.settings().clone())
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
        tokio::task::spawn_blocking(move || {
            let configured = configured
                .iter()
                .filter(|bundle| matches!(bundle.provenance, PluginProvenance::Configured))
                .map(|bundle| {
                    let bytes = read_bounded(&bundle.path, harmos_packaging::bundle::Limits::default().archive_bytes)?;
                    inspect(&bytes).map(|(_, metadata)| metadata)
                })
                .collect::<Result<Vec<_>, PluginError>>()?;
            store.install_checked(&bytes, &configured, &host_sources, validate)
        })
        .await
        .map_err(|error| PluginError::Io(std::io::Error::other(error.to_string())))?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn process_state(entry: &mut PluginEntry, attempted: Option<bool>) {
    entry.active = attempted == Some(true);
    entry.restart_required = match entry.installation.state {
        super::PluginState::Installed => attempted.is_none(),
        super::PluginState::Disabled => entry.active,
        super::PluginState::RemovalPending => true,
    };
    // Only an attempt this process itself made and refused ever surfaces a
    // reason. No caller ever observes a `RuntimeHandle` until `install_plugins`
    // has returned `Ok` -- every early `Err` out of it instead shuts the
    // runtime back down (see `Runtime::start` and
    // `with_execution_fixtures_inner`) -- and every plugin still `false` in
    // `plugin_boot` when it returns `Ok` was already given its own
    // `record_boot` call along the way. So whenever `attempted == Some(false)`
    // is observable at all, `last_boot` necessarily holds *this* attempt's
    // outcome, never a stale one from an earlier process.
    entry.boot_error = (attempted == Some(false)).then(|| match &entry.installation.last_boot {
        Some(super::PluginLastBoot::Failed { error, .. }) => error.clone(),
        _ => GENERIC_BOOT_ERROR.to_owned(),
    });
}

/* === Tests === */
