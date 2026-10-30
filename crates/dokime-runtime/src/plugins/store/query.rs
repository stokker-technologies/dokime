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

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PluginPage {
    pub entries: Vec<PluginEntry>,
    pub manifest_sha256: String,
    pub total: usize,
    pub next: Option<Uuid>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginStore {
    pub(crate) async fn program_settings(&self, owner: &str) -> Result<Option<(Uuid, crate::settings::Settings)>, PluginError> {
        let plugins = self.settings.read(|document| document.plugins.clone()).await?;
        for (id, plugin) in plugins {
            if plugin.state == PluginState::RemovalPending {
                continue;
            }
            let Ok((bundle, metadata)) = self.verified(id, &plugin) else {
                continue;
            };
            if !metadata.owners.iter().any(|candidate| candidate == owner) {
                continue;
            }
            for file in super::super::catalog::selected_files(&bundle)? {
                let declaration = bundle.declaration(&file.path).expect("verified executable declaration");
                if declaration.id != owner {
                    continue;
                }
                let executable = file.executable.as_ref().expect("selected executable");
                let program = crate::domains::installed_plugins::contributions::decode_registrations(
                    owner,
                    &declaration.version.to_string(),
                    &declaration.description,
                    &executable.registrations,
                )
                .map_err(|error| PluginError::Invalid(error.to_string()))?;
                return Ok(Some((id, program.settings)));
            }
        }
        Ok(None)
    }

    pub async fn get(&self, id: Uuid) -> Result<PluginEntry, PluginError> {
        let plugin = self.settings.read(|document| document.plugins.get(&id).cloned()).await?.ok_or(PluginError::Unknown(id))?;
        Ok(self.entry(id, &plugin))
    }

    pub async fn page(&self, after: Option<(Uuid, &str)>, limit: usize) -> Result<PluginPage, PluginError> {
        let limit = u32::try_from(limit).map_err(|_| PluginError::Limit)?;
        let limit = dokime_domain::model::execution::wire_cursor::page_size(limit, "").map_err(|_| PluginError::Limit)?;
        let plugins = self.settings.read(|document| document.plugins.clone()).await?;
        let identity = digest(&serde_json::to_vec(&plugins).map_err(std::io::Error::other)?);
        if let Some((id, expected)) = after
            && (expected != identity || !plugins.contains_key(&id))
        {
            return Err(PluginError::Cursor);
        }
        let total = plugins.len();
        let mut selected = plugins.into_iter().filter(|(id, _)| after.is_none_or(|(after, _)| *id > after));
        let mut entries = Vec::new();
        // Native verification handles one archive at a time; bound total page IO
        // independently of the table size and of the per-archive native ceiling.
        let mut remaining = 512 * 1024 * 1024usize;
        for (id, plugin) in selected.by_ref().take(limit) {
            let verified = (|| {
                let path = self.path(id);
                if !fs::symlink_metadata(&path)?.file_type().is_file() {
                    return Err(PluginError::Invalid("bundle must be a regular file".into()));
                }
                let allowed = remaining.min(self.archive_bytes);
                let file = File::open(path)?;
                let mut bytes = Vec::new();
                let read = file.take(allowed as u64 + 1).read_to_end(&mut bytes);
                // Charge actual IO, including partial reads and a possible
                // concurrent external rewrite, before inspection/metadata copies.
                remaining = remaining.checked_sub(bytes.len()).ok_or(PluginError::Limit)?;
                read?;
                if bytes.len() > self.archive_bytes {
                    return Err(PluginError::Invalid("bundle exceeds byte limit".into()));
                }
                if digest(&bytes) != plugin.digest {
                    return Err(PluginError::Changed(id));
                }
                super::super::inspect_with_archive_limit(&bytes, &self.schema, self.archive_bytes)
            })();
            if matches!(verified, Err(PluginError::Limit)) {
                return Err(PluginError::Limit);
            }
            entries.push(Self::entry_from_verification(id, &plugin, verified));
        }
        let next = selected.next().and_then(|_| entries.last().map(|entry| entry.installation.install_id));
        Ok(PluginPage {
            entries,
            manifest_sha256: identity,
            total,
            next,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
