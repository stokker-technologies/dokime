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
    pub(super) fn saved_entries(&self) -> Result<v1::PluginEntries> { Ok(self.host.inner.store.administration("plugin-entries")?.unwrap_or_default()) }
    pub(super) fn save_entries(&self, entries: &v1::PluginEntries) -> Result<()> { self.host.inner.store.save_administration("plugin-entries", entries)?; Ok(()) }
    pub(super) async fn connect_entry(&self, mut entry: v1::Plugin) -> Result<v1::Plugin> {
        let endpoint = match &entry.placement { Some(v1::plugin::Placement::Remote(remote)) => remote.url.clone(), _ => return Err(Error::Invalid("Reconnect a Remote URL or Enable Its Managed Installation".into())) };
        let result = self.host.add_remote(&endpoint).await;
        match result {
            Ok(identifier) => {
                entry.description = Some(self.host.description(&identifier).map_err(convert)?);
                entry.state = v1::PluginState::Ready as i32;
                entry.reason.clear();
            }
            Err(error) => {
                entry.state = v1::PluginState::Unavailable as i32;
                entry.reason = format!("Start the Remote Plugin and Reload Its Entry: {error}");
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
        if !(url.starts_with("http://") || url.starts_with("https://")) || url.contains('@') || url.contains('#') { return Err(Error::Invalid("Add Plugin: Supply an HTTP or HTTPS URL Without Embedded Credentials or a Fragment".into())); }
        tonic::transport::Endpoint::from_shared(url.to_owned()).map_err(|_| Error::Invalid("Add Plugin: Supply a Valid HTTP or HTTPS URL".into()))?;
        let identifier = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, url.as_bytes()).to_string();
        let mut entries = self.saved_entries()?;
        if let Some(entry) = entries.plugins.iter().find(|entry| entry.entry == identifier) {
            let entry = entry.clone();
            if entry.description.as_ref().is_some_and(|description| self.host.entry_state(&description.id).is_ok_and(|state| state == crate::EntryState::Ready)) { return Ok(entry); }
            return self.connect_entry(entry).await;
        }
        let entry = v1::Plugin { entry: identifier, placement: Some(v1::plugin::Placement::Remote(v1::RemotePlugin { url: url.into() })), state: v1::PluginState::Connecting as i32, ..Default::default() };
        entries.plugins.push(entry.clone());
        self.save_entries(&entries)?;
        self.connect_entry(entry).await
    }
    pub(super) fn list_entries(&self) -> Result<Vec<v1::Plugin>> {
        let mut entries = self.saved_entries()?.plugins;
        for entry in &mut entries {
            let Some(description) = &entry.description else { continue; };
            if self.host.entry_state(&description.id).map_err(convert)? == crate::EntryState::Ready {
                entry.state = v1::PluginState::Ready as i32;
                entry.reason.clear();
            } else {
                entry.state = v1::PluginState::Unavailable as i32;
                entry.reason = "Start the Remote Plugin and Reload Its Entry".into();
            }
        }
        for (identifier, state) in self.host.plugin_entries().map_err(convert)? {
            if entries.iter().any(|entry| entry.description.as_ref().is_some_and(|description| description.id == identifier)) { continue; }
            entries.push(v1::Plugin { entry: identifier.clone(), description: Some(self.host.description(&identifier).map_err(convert)?), state: if state == crate::EntryState::Ready { v1::PluginState::Ready as i32 } else { v1::PluginState::Unavailable as i32 }, ..Default::default() });
        }
        entries.sort_by(|left, right| left.entry.cmp(&right.entry));
        Ok(entries)
    }
    pub(super) async fn remove_entry(&self, identifier: &str) -> Result<()> {
        let mut entries = self.saved_entries()?;
        let entry = entries.plugins.iter().find(|entry| entry.entry == identifier).cloned().ok_or_else(|| Error::Invalid("Remove Plugin: Refresh the Entry List and Select an Existing Entry".into()))?;
        if let Some(description) = &entry.description {
            self.host.detach_plugin(&description.id).await.map_err(convert)?;
            self.host.inner.store.remove_plugin(&description.id)?;
        }
        entries.plugins.retain(|entry| entry.entry != identifier);
        self.save_entries(&entries)?;
        Ok(())
    }
    pub(super) async fn reload_entry(&self, identifier: &str) -> Result<()> {
        let entry = self.saved_entries()?.plugins.into_iter().find(|entry| entry.entry == identifier || entry.description.as_ref().is_some_and(|description| description.id == identifier)).ok_or_else(|| Error::Invalid("Reload Plugin: Select an Existing Entry".into()))?;
        if let Some(description) = &entry.description { self.host.detach_plugin(&description.id).await.map_err(convert)?; }
        let result = self.connect_entry(entry).await?;
        if result.state != v1::PluginState::Ready as i32 { return Err(Error::Invalid(result.reason)); }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error { Error::Invalid(error.to_string()) }

/* === Tests === */
