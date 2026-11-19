/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reconcile Installation Plugin Bundles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    env,
    fmt::Display,
    fs,
    io::{ErrorKind, Read},
    path::Path,
    sync::Weak,
};

// Third Party Imports
use harmos_operations::plugins::{List, Replace};
use harmos_protocol::host::status::Event;
use harmos_protocol::host::{PluginEntry, PluginManaged, PluginPlacement, PluginsListRequest, PluginsReplaceRequest, plugin_placement};
use harmos_protocol::model::{FileMetadata, FileReference};
use harmos_runtime::bundle::{Bundle, Limits};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// Local Imports
use super::{Host, installation_directory, lock};
use crate::{Error, Result};
use dokime_operations::host::files::delete;
use dokime_protocol::host::FilesDeleteRequest;

/* === Constants === */

// Provenance Lives in the Managed Content Reference Already Persisted by the Registry.
const INSTALLATION_SOURCE: &str = "application/x-tar; source=installation; file=";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub(super) async fn reconcile_installation_plugins(&self) {
        let directory = match &self.settings.installation {
            Some(directory) => directory.clone(),
            None => match env::current_exe().and_then(|executable| installation_directory(&executable).map(Path::to_path_buf)) {
                Ok(directory) => directory,
                Err(error) => {
                    self.report_plugin_admission_failure(Path::new("plugins"), error);
                    return;
                }
            },
        };
        let directory = directory.join("plugins");
        if let Err(error) = self.reconcile_directory(&directory).await {
            self.report_plugin_admission_failure(&directory, error);
        }
    }

    async fn reconcile_directory(&self, directory: &Path) -> Result<()> {
        let mut files = Vec::new();
        let mut complete = true;
        match fs::read_dir(directory) {
            Ok(entries) => {
                // Finish Enumeration Before Removing Anything; an Incomplete Scan Is Not a Removal.
                for entry in entries {
                    let path = match entry {
                        Ok(entry) => entry.path(),
                        Err(error) => {
                            complete = false;
                            self.report_plugin_admission_failure(directory, error);
                            continue;
                        }
                    };
                    if path.extension().is_some_and(|extension| extension == "dokime") {
                        files.push(path);
                    }
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        files.sort();
        let mut present = BTreeSet::new();
        let mut candidates = Vec::new();
        for path in files {
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                // Do Not Interpret an Unreadable Filename as Removing a Previously Shipped Bundle.
                complete = false;
                self.report_plugin_admission_failure(&path, "Use a UTF-8 Bundle Filename");
                continue;
            };
            let filename = filename.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            let source = format!("{INSTALLATION_SOURCE}{filename}");
            present.insert(source.clone());
            candidates.push((path, source));
        }

        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Reconcile Installation Plugins: Assemble the Host Context".into()))?;
        let entries = context.operations.call_protobuf::<List>(PluginsListRequest { at: None }).await?.entries;
        for entry in entries {
            let Some(content) = installation_content(&entry) else { continue };
            let source = &content.metadata.as_ref().expect("installation metadata").media_type;
            if !complete || present.contains(source) {
                continue;
            }
            let result = self.remove_installation_plugin(&entry).await;
            if let Err(error) = result {
                self.report_plugin_admission_failure(directory, format!("Remove {}: {error}", entry.name));
            }
        }
        for (path, source) in candidates {
            let result = async {
                let bundle = read_bundle(&path)?;
                self.admit_installation_bundle(&bundle, &source).await
            }
            .await;
            if let Err(error) = result {
                self.report_plugin_admission_failure(&path, error);
            }
        }
        context.host.refresh_plugin_states().await?;
        Ok(())
    }

    async fn admit_installation_bundle(&self, bundle: &Bundle, source: &str) -> Result<()> {
        let name = bundle.manifest().identity.name.clone();
        let digest = Sha256::digest(bundle.bytes()).to_vec();
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Admit Installation Plugin: Assemble the Host Context".into()))?;
        let entries = context.operations.call_protobuf::<List>(PluginsListRequest { at: None }).await?.entries;
        for entry in &entries {
            let replaced_identity = entry.name != name
                && installation_content(entry)
                    .is_some_and(|content| content.metadata.as_ref().is_some_and(|metadata| metadata.media_type == source));
            if replaced_identity {
                self.remove_installation_plugin(entry).await?;
            }
        }
        let previous = entries.into_iter().find(|entry| entry.name == name);
        if let Some(entry) = &previous {
            let owned = installation_content(entry)
                .is_some_and(|content| content.metadata.as_ref().is_some_and(|metadata| metadata.media_type == source));
            if !owned {
                return Err(Error::Invalid(format!(
                    "Admit Installation Plugin {name}: Entry Already Exists; Keep the User Entry and Give the Shipped Bundle an Unused Name"
                )));
            }
            if managed(entry).is_some_and(|managed| managed.sha256 == digest) {
                if entry.value.as_ref().is_some_and(|value| value.enabled) {
                    self.await_plugin_ready(&name).await?;
                }
                return Ok(());
            }
        }

        let content = self
            .upload_plugin_bundle(
                bundle.bytes(),
                FileMetadata {
                    media_type: source.into(),
                    ..Default::default()
                },
            )
            .await?;
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Managed(PluginManaged {
                sha256: digest,
                content: Some(content.clone()),
            })),
        };
        if let Some(entry) = previous {
            let result = context
                .operations
                .call_protobuf::<Replace>(PluginsReplaceRequest {
                    name,
                    placement: Some(placement),
                    expected_revision: entry.revision.clone(),
                    request_id: Uuid::now_v7().to_string(),
                })
                .await;
            if let Err(error) = result {
                self.release_unretained_bundle(content).await?;
                return Err(error.into());
            }
            if let Some(content) = installation_content(&entry) {
                self.release_bundle(content.clone()).await?;
            }
        } else {
            let result = self.create_plugin(name.clone(), placement).await;
            if let Err(error) = result {
                // Create Persists Failed Admissions Too; Keep Their Content and Registry Failure.
                self.release_unretained_bundle(content).await?;
                return Err(error);
            }
        }
        Ok(())
    }

    async fn remove_installation_plugin(&self, entry: &PluginEntry) -> Result<()> {
        let content = installation_content(entry)
            .ok_or_else(|| Error::Invalid("Remove Installation Plugin: Keep the User-Installed Entry".into()))?;
        self.remove_plugin(&entry.name).await?;
        self.release_bundle(content.clone()).await
    }

    async fn release_bundle(&self, file: FileReference) -> Result<()> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Release Plugin Bundle: Assemble the Host Context".into()))?;
        delete::handle(
            &context,
            FilesDeleteRequest {
                file: Some(file),
                request_id: Uuid::now_v7().to_string(),
            },
        )
        .await?;
        Ok(())
    }

    async fn release_unretained_bundle(&self, file: FileReference) -> Result<()> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Release Plugin Bundle: Assemble the Host Context".into()))?;
        let retained = context
            .operations
            .call_protobuf::<List>(PluginsListRequest { at: None })
            .await?
            .entries
            .iter()
            .any(|entry| managed(entry).and_then(|managed| managed.content.as_ref()) == Some(&file));
        if !retained {
            self.release_bundle(file).await?;
        }
        Ok(())
    }

    fn report_plugin_admission_failure(&self, path: &Path, reason: impl Display) {
        let message = format!("Admit Installation Plugin {}: {reason}; Repair or Remove This Bundle and Restart the Host", path.display());
        tracing::error!(file = %path.display(), %message, "Installation Plugin Admission Failed");
        if let Ok(mut failures) = lock(&self.plugin_admission_failures) {
            failures.push((format!("installation/{}", path.display()), message.clone()));
        }
        if let Some(assembly) = &self.assembly {
            let event = Event {
                target: "dokime.plugins".into(),
                level: "ERROR".into(),
                fields: [("file".into(), path.display().to_string()), ("message".into(), message)].into(),
                ..Default::default()
            };
            if let Err(error) = assembly.status.record_log(event) {
                tracing::error!(%error, "Record Installation Plugin Failure");
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn read_bundle(path: &Path) -> Result<Bundle> {
    let limits = Limits::default();
    let file = fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error::Invalid("Admit Installation Plugin: Supply a Regular Bundle File".into()));
    }
    let mut bytes = Vec::new();
    file.take(limits.archive_bytes as u64 + 1).read_to_end(&mut bytes)?;
    Bundle::inspect(&bytes, limits).map_err(|error| Error::Invalid(error.to_string()))
}

fn managed(entry: &PluginEntry) -> Option<&PluginManaged> {
    match entry.placement.as_ref()?.kind.as_ref()? {
        plugin_placement::Kind::Managed(managed) => Some(managed),
        _ => None,
    }
}

fn installation_content(entry: &PluginEntry) -> Option<&FileReference> {
    let content = managed(entry)?.content.as_ref()?;
    content.metadata.as_ref()?.media_type.starts_with(INSTALLATION_SOURCE).then_some(content)
}

/* === Tests === */
