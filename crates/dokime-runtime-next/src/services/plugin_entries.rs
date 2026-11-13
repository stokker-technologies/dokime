/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Third Party Imports

// Local Imports
use super::state::State;
use dokime_operations_next::{Error, Result};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(super) fn saved_entries(&self) -> Result<v1::PluginEntries> {
        Ok(self.host.store.administration("plugin-entries")?.unwrap_or_default())
    }
    pub(super) fn save_entries(&self, entries: &v1::PluginEntries) -> Result<()> {
        self.host.store.save_administration("plugin-entries", entries)?;
        self.host.signal_change();
        Ok(())
    }
    pub(super) async fn connect_entry(&self, mut entry: v1::Plugin) -> Result<v1::Plugin> {
        let managed = matches!(entry.placement, Some(v1::plugin::Placement::Managed(_)));
        let mut placement = self.host.settings.placement.clone();
        let preferences: Option<v1::SettingsSnapshot> = self.host.store.administration("preferences")?;
        let preferences =
            preferences.unwrap_or_else(dokime_operations_next::client::settings_read_application_settings::initial_preferences);
        if let Some(plugins) = preferences.settings.and_then(|settings| settings.plugins) {
            placement.connect_timeout = std::time::Duration::from_secs(plugins.startup_timeout_s.into());
            placement.stop_grace = std::time::Duration::from_secs(plugins.stop_timeout_s.into());
        }
        let result = match &entry.placement {
            Some(v1::plugin::Placement::Remote(remote)) => self.host.add_remote_configured(&remote.url, &placement).await,
            Some(v1::plugin::Placement::Managed(_)) => {
                self.host.add_managed_configured(&self.managed_executable(&entry.entry)?, &placement).await
            }
            None => return Err(Error::Invalid("Reconnect a Remote URL or Reinstall Its Managed Bundle".into())),
        };
        match result {
            Ok(identifier) => {
                entry.description = Some(self.host.description(&identifier).map_err(convert)?);
                entry.state = v1::PluginState::Ready as i32;
                entry.reason.clear();
            }
            Err(error) => {
                entry.state = if managed { v1::PluginState::Failed } else { v1::PluginState::Unavailable } as i32;
                entry.reason = format!("Check Plugin Settings and Secrets, Then Reload Its Entry: {error}");
            }
        }
        let mut entries = self.saved_entries()?;
        entries.plugins.retain(|saved| saved.entry != entry.entry);
        entries.plugins.push(entry.clone());
        entries.plugins.sort_by(|left, right| left.entry.cmp(&right.entry));
        self.save_entries(&entries)?;
        Ok(entry)
    }
    pub(super) async fn add_entry(&self, url: &str) -> Result<v1::Plugin> {
        if !(url.starts_with("http://") || url.starts_with("https://")) || url.contains('@') || url.contains('#') {
            return Err(Error::Invalid("Add Plugin: Supply an HTTP or HTTPS URL Without Embedded Credentials or a Fragment".into()));
        }
        tonic::transport::Endpoint::from_shared(url.to_owned())
            .map_err(|_| Error::Invalid("Add Plugin: Supply a Valid HTTP or HTTPS URL".into()))?;
        let identifier = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, url.as_bytes()).to_string();
        let mut entries = self.saved_entries()?;
        if let Some(entry) = entries.plugins.iter().find(|entry| entry.entry == identifier) {
            let entry = entry.clone();
            if entry
                .description
                .as_ref()
                .is_some_and(|description| self.host.entry_state(&description.id).is_ok_and(|state| state == crate::EntryState::Ready))
            {
                return Ok(entry);
            }
            return self.connect_entry(entry).await;
        }
        let entry = v1::Plugin {
            entry: identifier,
            placement: Some(v1::plugin::Placement::Remote(v1::RemotePlugin { url: url.into() })),
            state: v1::PluginState::Connecting as i32,
            ..Default::default()
        };
        entries.plugins.push(entry.clone());
        self.save_entries(&entries)?;
        self.connect_entry(entry).await
    }
    pub(super) async fn refresh_entry_states(&self) -> Result<()> {
        let _guard = self.administration.lock().await;
        let mut entries = self.saved_entries()?;
        let mut changed = false;
        for entry in &mut entries.plugins {
            if entry.state != v1::PluginState::Ready as i32 {
                continue;
            }
            let Some(description) = entry.description.as_ref() else {
                continue;
            };
            let native = match self.host.entry(&description.id) {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let endpoint = match &entry.placement {
                Some(v1::plugin::Placement::Remote(remote)) => remote.url.clone(),
                Some(v1::plugin::Placement::Managed(_)) => {
                    let process = native.process.lock().await;
                    let Some(process) = process.as_ref() else {
                        continue;
                    };
                    process.endpoint().to_owned()
                }
                None => continue,
            };
            let settings = &self.host.settings.placement;
            let probe = async {
                let endpoint = tonic::transport::Endpoint::from_shared(endpoint)
                    .map_err(|_| crate::Error::Invalid("Reload the Plugin with a Valid Endpoint".into()))?
                    .connect_timeout(settings.connect_timeout);
                let channel = endpoint.connect().await?;
                let reply = harmos_protocol::v1::plugin_client::PluginClient::new(channel)
                    .max_decoding_message_size(settings.message_bytes.get())
                    .max_encoding_message_size(settings.message_bytes.get())
                    .describe(harmos_protocol::v1::DescribeRequest {
                        revision: 1,
                        host: "dokime".into(),
                        host_version: env!("CARGO_PKG_VERSION").into(),
                    })
                    .await?
                    .into_inner();
                Ok::<bool, crate::Error>(
                    reply.id == description.id
                        && reply.started_at == description.started_at
                        && reply.catalog.as_ref().map(|catalog| &catalog.digest)
                            == description.catalog.as_ref().map(|catalog| &catalog.digest),
                )
            };
            if matches!(tokio::time::timeout(settings.connect_timeout, probe).await, Ok(Ok(true))) {
                continue;
            }
            let managed = matches!(entry.placement, Some(v1::plugin::Placement::Managed(_)));
            entry.state = if managed { v1::PluginState::Failed } else { v1::PluginState::Unavailable } as i32;
            entry.reason = "The Plugin Stopped or Changed; Check Its Settings and Reload the Entry".into();
            // Keep Active Execution Resources Until Their Cleanup Has Settled.
            let _ = self.host.detach_plugin(&description.id).await;
            changed = true;
        }
        if changed {
            self.save_entries(&entries)?;
        }
        Ok(())
    }
    pub(super) fn list_entries(&self) -> Result<Vec<v1::Plugin>> {
        let mut entries = self.saved_entries()?.plugins;
        let live = crate::host::lock(&self.host.plugins).map_err(convert)?.clone();
        let installations = self.saved_installations()?;
        for entry in &mut entries {
            if installations.installations.iter().any(|installation| {
                installation.install_id == entry.entry && installation.state == v1::PluginInstallationState::Disabled as i32
            }) {
                entry.state = v1::PluginState::Unavailable as i32;
                entry.reason = "Enable This Plugin to Start It".into();
                continue;
            }
            let Some(description) = &entry.description else {
                continue;
            };
            if entry.state == v1::PluginState::Ready as i32
                && live.get(&description.id).is_some_and(|native| {
                    native.state == crate::EntryState::Ready && native.description.started_at == description.started_at
                })
            {
                entry.state = v1::PluginState::Ready as i32;
                entry.reason.clear();
            } else {
                let managed = matches!(entry.placement, Some(v1::plugin::Placement::Managed(_)));
                entry.state = if managed { v1::PluginState::Failed } else { v1::PluginState::Unavailable } as i32;
                entry.reason = if managed {
                    "Check Plugin Settings and Secrets, Then Reload Its Entry"
                } else {
                    "Start the Remote Plugin and Reload Its Entry"
                }
                .into();
            }
        }
        for (identifier, state) in self.host.plugin_entries().map_err(convert)? {
            if entries.iter().any(|entry| entry.description.as_ref().is_some_and(|description| description.id == identifier)) {
                continue;
            }
            entries.push(v1::Plugin {
                entry: identifier.clone(),
                description: Some(self.host.description(&identifier).map_err(convert)?),
                state: if state == crate::EntryState::Ready { v1::PluginState::Ready as i32 } else { v1::PluginState::Unavailable as i32 },
                ..Default::default()
            });
        }
        entries.sort_by(|left, right| left.entry.cmp(&right.entry));
        Ok(entries)
    }
    pub(super) async fn remove_entry(&self, identifier: &str) -> Result<()> {
        let mut entries = self.saved_entries()?;
        let entry = entries
            .plugins
            .iter()
            .find(|entry| entry.entry == identifier)
            .cloned()
            .ok_or_else(|| Error::Invalid("Remove Plugin: Refresh the Entry List and Select an Existing Entry".into()))?;
        if matches!(entry.placement, Some(v1::plugin::Placement::Managed(_))) {
            return self.remove_managed(identifier).await;
        }
        if let Some(description) = &entry.description {
            self.host.detach_plugin(&description.id).await.map_err(convert)?;
            self.host.store.remove_plugin(&description.id)?;
        }
        entries.plugins.retain(|entry| entry.entry != identifier);
        self.save_entries(&entries)?;
        Ok(())
    }
    pub(super) async fn reload_entry(&self, identifier: &str) -> Result<()> {
        if self
            .saved_installations()?
            .installations
            .iter()
            .any(|installation| installation.install_id == identifier && installation.state == v1::PluginInstallationState::Disabled as i32)
        {
            return Err(Error::Invalid("Reload Plugin: Enable Its Disabled Installation First".into()));
        }
        let entry = self
            .saved_entries()?
            .plugins
            .into_iter()
            .find(|entry| entry.entry == identifier || entry.description.as_ref().is_some_and(|description| description.id == identifier))
            .ok_or_else(|| Error::Invalid("Reload Plugin: Select an Existing Entry".into()))?;
        if let Some(description) = &entry.description {
            self.host.detach_plugin(&description.id).await.map_err(convert)?;
        }
        let result = self.connect_entry(entry).await?;
        if result.state != v1::PluginState::Ready as i32 {
            return Err(Error::Invalid(result.reason));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
