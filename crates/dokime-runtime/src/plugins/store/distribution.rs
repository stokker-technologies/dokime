/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reconcile Bundled Installer Plugins Once Per Distribution

use std::{collections::BTreeSet, path::Path};

use super::{PluginError, PluginMetadata, PluginStore, digest, read_bounded};

impl PluginStore {
    /// Reconcile Installer Plugins Once Per Distribution, Retrying Incomplete Starts.
    pub async fn install_distribution(
        &self, directory: &Path, configured: &[PluginMetadata], host_sources: &BTreeSet<String>,
    ) -> Result<(), PluginError> {
        let Some(parent) = directory.parent() else {
            let error = PluginError::Invalid("bundled plugin directory has no parent".into());
            return Err(error);
        };
        let metadata = read_bounded(&parent.join("installer.json"), self.archive_bytes)?;
        let distribution = digest(&metadata);
        let previous = self.settings.read(|document| document.bundled_distribution.clone()).await?;
        if previous.as_ref() == Some(&distribution) {
            return Ok(());
        }

        let metadata: serde_json::Value = serde_json::from_slice(&metadata).map_err(|error| PluginError::Invalid(error.to_string()))?;
        if metadata.get("schema").and_then(serde_json::Value::as_u64) != Some(1) {
            return Err(PluginError::Invalid("unsupported bundled distribution schema".into()));
        }
        let files = metadata
            .get("files")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| PluginError::Invalid("bundled distribution has no integrity index".into()))?;
        let mut selected = BTreeSet::new();
        for (relative, expected) in files {
            let relative = Path::new(relative);
            let plugin_folder = relative.parent().and_then(Path::file_name);
            if plugin_folder != Some(std::ffi::OsStr::new("plugins")) || relative.extension().is_none_or(|extension| extension != "dokime")
            {
                continue;
            }
            let name = relative.file_name().expect("indexed bundle has an extension");
            if !selected.insert(name) {
                return Err(PluginError::Invalid(format!("duplicate bundled plugin filename: {}", name.to_string_lossy())));
            }
            let path = directory.join(name);
            let installed = async {
                let bytes = read_bounded(&path, self.archive_bytes)?;
                let actual = digest(&bytes);
                if expected.as_str() != Some(actual.as_str()) {
                    return Err(PluginError::Invalid("bundle differs from the installer integrity index".into()));
                }
                self.install_bundled(&bytes, configured, host_sources).await
            }
            .await;
            installed.map_err(|error| {
                let message = format!("bundled plugin {}: {error}", path.display());
                PluginError::Invalid(message)
            })?;
        }

        self.settings
            .edit(move |document| {
                document.bundled_distribution = Some(distribution);
                Ok::<_, PluginError>(())
            })
            .await
    }
}
