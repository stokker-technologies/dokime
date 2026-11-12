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
use super::{
    bundles::{self, Bundle},
    state::State,
};
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
    pub(super) fn saved_installations(&self) -> Result<v1::InstallationPage> {
        Ok(self.host.inner.store.administration("plugin-installations")?.unwrap_or_default())
    }
    pub(super) fn managed_executables(&self) -> Result<v1::MapValue> {
        Ok(self.host.inner.store.administration("managed-executables")?.unwrap_or_default())
    }
    pub(super) fn managed_executable(&self, identifier: &str) -> Result<std::path::PathBuf> {
        let values = self.managed_executables()?;
        let value = values.values.get(identifier).and_then(|value| value.kind.as_ref());
        let Some(v1::value::Kind::Text(path)) = value else {
            return Err(Error::Invalid("Reload Plugin: Restore Its Managed Bundle and Reinstall It".into()));
        };
        let relative = std::path::Path::new(path);
        if relative.is_absolute() || relative.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
            return Err(Error::Invalid("Reload Plugin: Back Up and Reset the Incompatible Managed Paths".into()));
        }
        let installation = self
            .saved_installations()?
            .installations
            .into_iter()
            .find(|installation| installation.install_id == identifier)
            .ok_or_else(|| Error::Invalid("Reload Plugin: Reinstall Its Missing Installation".into()))?;
        let archive =
            std::fs::read(self.root.join("plugins").join(format!("{identifier}.dokime"))).map_err(crate::Error::from).map_err(convert)?;
        if bundles::digest(&archive) != installation.sha256 {
            return Err(Error::Invalid("Reload Plugin: Restore Its Verified Bundle or Reinstall It".into()));
        }
        let bundle = Bundle::read(&archive).map_err(convert)?;
        for (name, expected) in &bundle.files {
            let file = self.root.join("plugins").join(identifier).join(name);
            if std::fs::read(file).map(|bytes| bytes != *expected).unwrap_or(true) {
                return Err(Error::Invalid("Reload Plugin: Restore Its Verified Files by Reinstalling the Bundle".into()));
            }
        }
        Ok(self.root.join(relative))
    }
    pub(super) fn list_installations(&self) -> Result<Vec<v1::PluginInstallation>> {
        let entries = self.list_entries()?;
        let mut installations = self.saved_installations()?.installations;
        for installation in &mut installations {
            installation.plugin = entries.iter().find(|entry| entry.entry == installation.install_id).cloned();
            if let Some(metadata) = &mut installation.metadata {
                metadata.owners = installation
                    .plugin
                    .as_ref()
                    .and_then(|plugin| plugin.description.as_ref())
                    .map(|description| vec![description.id.clone()])
                    .unwrap_or_default();
            }
            let path = self.root.join("plugins").join(format!("{}.dokime", installation.install_id));
            if std::fs::read(path).map(|bytes| bundles::digest(&bytes) != installation.sha256).unwrap_or(true) {
                installation.error = Some("Restore the Verified Bundle or Remove and Reinstall This Plugin".into());
                if let Some(plugin) = &mut installation.plugin {
                    plugin.state = v1::PluginState::Failed as i32;
                    plugin.reason = installation.error.clone().unwrap();
                }
            }
        }
        installations.sort_by(|left, right| left.install_id.cmp(&right.install_id));
        Ok(installations)
    }
    pub(super) async fn restore_managed(&self) -> Result<()> {
        for installation in self.saved_installations()?.installations {
            if installation.state != v1::PluginInstallationState::Installed as i32 {
                continue;
            }
            let entry = self.saved_entries()?.plugins.into_iter().find(|entry| entry.entry == installation.install_id);
            if let Some(entry) = entry {
                let _ = self.connect_entry(entry).await?;
            }
        }
        Ok(())
    }
    pub(super) async fn install_bundle(&self, bytes: Vec<u8>) -> Result<v1::PluginInstallation> {
        let bundle = Bundle::read(&bytes).map_err(convert)?;
        let package = bundle.metadata.identity.as_ref().unwrap().name.clone();
        let mut installations = self.saved_installations()?;
        let previous = installations
            .installations
            .iter()
            .find(|installation| {
                installation.state != v1::PluginInstallationState::RemovalPending as i32
                    && installation
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.identity.as_ref())
                        .is_some_and(|identity| identity.name == package)
            })
            .cloned();
        let digest = bundles::digest(&bytes);
        if let Some(previous) = &previous
            && previous.sha256 == digest
        {
            return Ok(self.list_installations()?.into_iter().find(|installation| installation.install_id == previous.install_id).unwrap());
        }
        let identifier = uuid::Uuid::now_v7().to_string();
        let folder = self.root.join("plugins");
        std::fs::create_dir_all(&folder).map_err(crate::Error::from).map_err(convert)?;
        let temporary = tempfile::tempdir_in(&folder).map_err(crate::Error::from).map_err(convert)?;
        bundle.extract(temporary.path()).map_err(convert)?;
        let installed = folder.join(&identifier);
        std::fs::rename(temporary.path(), &installed).map_err(crate::Error::from).map_err(convert)?;
        let archive = folder.join(format!("{identifier}.dokime"));
        std::fs::write(&archive, &bytes).map_err(crate::Error::from).map_err(convert)?;
        let mut executables = self.managed_executables()?;
        let relative = std::path::Path::new("plugins").join(&identifier).join(&bundle.executable).to_string_lossy().into_owned();
        executables.values.insert(
            identifier.clone(),
            v1::Value {
                kind: Some(v1::value::Kind::Text(relative)),
            },
        );
        let disabled = previous.as_ref().is_some_and(|previous| previous.state == v1::PluginInstallationState::Disabled as i32);
        let entry = v1::Plugin {
            entry: identifier.clone(),
            placement: Some(v1::plugin::Placement::Managed(v1::ManagedPlugin { bundle: identifier.clone() })),
            state: if disabled { v1::PluginState::Unavailable } else { v1::PluginState::Starting } as i32,
            reason: if disabled { "Enable This Plugin to Start It".into() } else { String::new() },
            ..Default::default()
        };
        let mut entries = self.saved_entries()?;
        entries.plugins.push(entry.clone());
        let installation = v1::PluginInstallation {
            install_id: identifier.clone(),
            sha256: digest,
            installed_at: Some(prost_types::Timestamp {
                seconds: self.host.now_ns().div_euclid(1_000_000_000),
                nanos: self.host.now_ns().rem_euclid(1_000_000_000) as i32,
            }),
            state: if disabled { v1::PluginInstallationState::Disabled } else { v1::PluginInstallationState::Installed } as i32,
            metadata: Some(bundle.metadata),
            plugin: Some(entry.clone()),
            ..Default::default()
        };
        installations.installations.push(installation);
        self.host.inner.store.save_plugin_administration(&entries, &installations, &executables)?;
        if let Some(previous) = &previous {
            let previous_entry = self.saved_entries()?.plugins.into_iter().find(|entry| entry.entry == previous.install_id);
            if let Some(description) = previous_entry.as_ref().and_then(|entry| entry.description.as_ref()) {
                self.host.detach_plugin(&description.id).await.map_err(convert)?;
            }
        }
        let connected = if disabled { entry } else { self.connect_entry(entry).await? };
        if connected.state != v1::PluginState::Ready as i32 && !disabled {
            if let Some(previous) = &previous {
                let previous_entry = self.saved_entries()?.plugins.into_iter().find(|entry| entry.entry == previous.install_id).unwrap();
                let _ = self.connect_entry(previous_entry).await?;
                if let Some(current) = installations.installations.iter_mut().find(|installation| installation.install_id == identifier) {
                    current.state = v1::PluginInstallationState::RemovalPending as i32;
                    current.error = Some(connected.reason.clone());
                }
                self.host.inner.store.save_administration("plugin-installations", &installations)?;
            }
        } else if let Some(previous) = previous {
            // Replace the Entry Without Removing Frozen Execution Evidence or Saved Settings.
            self.remove_managed(&previous.install_id).await?;
        }
        self.list_installations()?
            .into_iter()
            .find(|installation| installation.install_id == identifier)
            .ok_or_else(|| Error::Invalid("Install Plugin: Refresh the Installation List".into()))
    }
    pub(super) async fn enable_managed(&self, identifier: &str, enabled: bool) -> Result<v1::PluginInstallation> {
        let mut installations = self.saved_installations()?;
        let installation = installations
            .installations
            .iter_mut()
            .find(|installation| installation.install_id == identifier)
            .ok_or_else(|| Error::Invalid("Enable Plugin: Select an Existing Installation".into()))?;
        if installation.state == v1::PluginInstallationState::RemovalPending as i32 {
            return Err(Error::Invalid("Remove This Installation Before Enabling It".into()));
        }
        let mut entries = self.saved_entries()?;
        let entry = entries
            .plugins
            .iter_mut()
            .find(|entry| entry.entry == identifier)
            .ok_or_else(|| Error::Invalid("Enable Plugin: Restore Its Entry by Reinstalling the Bundle".into()))?;
        if !enabled {
            if let Some(description) = &entry.description {
                self.host.detach_plugin(&description.id).await.map_err(convert)?;
            }
            entry.state = v1::PluginState::Unavailable as i32;
            entry.reason = "Enable This Plugin to Start It".into();
        }
        installation.state = if enabled { v1::PluginInstallationState::Installed } else { v1::PluginInstallationState::Disabled } as i32;
        let entry = entry.clone();
        self.host.inner.store.save_plugin_administration(&entries, &installations, &self.managed_executables()?)?;
        if enabled {
            let _ = self.connect_entry(entry).await?;
        }
        self.list_installations()?
            .into_iter()
            .find(|installation| installation.install_id == identifier)
            .ok_or_else(|| Error::Invalid("Enable Plugin: Refresh the Installation List".into()))
    }
    pub(super) async fn remove_managed(&self, identifier: &str) -> Result<()> {
        uuid::Uuid::parse_str(identifier).map_err(|_| Error::Invalid("Remove Plugin: Select a Valid Installation Identifier".into()))?;
        let mut installations = self.saved_installations()?;
        let installation = installations
            .installations
            .iter_mut()
            .find(|installation| installation.install_id == identifier)
            .ok_or_else(|| Error::Invalid("Remove Plugin: Select an Existing Installation".into()))?;
        let mut entries = self.saved_entries()?;
        if let Some(description) =
            entries.plugins.iter().find(|entry| entry.entry == identifier).and_then(|entry| entry.description.as_ref())
            && self.host.description(&description.id).is_ok_and(|current| current.started_at == description.started_at)
        {
            self.host.detach_plugin(&description.id).await.map_err(convert)?;
        }
        installation.state = v1::PluginInstallationState::RemovalPending as i32;
        self.host.inner.store.save_administration("plugin-installations", &installations)?;
        let folder = self.root.join("plugins").canonicalize().map_err(crate::Error::from).map_err(convert)?;
        let target = folder.join(identifier);
        if target.exists() {
            let resolved = target.canonicalize().map_err(crate::Error::from).map_err(convert)?;
            if resolved.parent() != Some(folder.as_path()) {
                return Err(Error::Invalid("Remove Plugin: Restore Its Owned Installation Folder Before Retrying".into()));
            }
            std::fs::remove_dir_all(&resolved).map_err(crate::Error::from).map_err(convert)?;
        }
        let archive = folder.join(format!("{identifier}.dokime"));
        if archive.exists() {
            std::fs::remove_file(archive).map_err(crate::Error::from).map_err(convert)?;
        }
        if let Some(description) =
            entries.plugins.iter().find(|entry| entry.entry == identifier).and_then(|entry| entry.description.as_ref())
        {
            // A Replacement May Still Own the Same Live Plugin Identity.
            if !entries
                .plugins
                .iter()
                .any(|entry| entry.entry != identifier && entry.description.as_ref().is_some_and(|other| other.id == description.id))
            {
                self.host.inner.store.remove_plugin(&description.id)?;
            }
        }
        entries.plugins.retain(|entry| entry.entry != identifier);
        installations.installations.retain(|installation| installation.install_id != identifier);
        let mut executables = self.managed_executables()?;
        executables.values.remove(identifier);
        self.host.inner.store.save_plugin_administration(&entries, &installations, &executables)?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
