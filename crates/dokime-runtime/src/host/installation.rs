/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reconcile Installation Plugin Bundles and the Organization Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    env,
    fmt::Display,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::Weak,
};

// Third Party Imports
use harmos_operations::plugins::{List, Replace};
use harmos_protocol::host::status::Event;
use harmos_protocol::host::{PluginEntry, PluginManaged, PluginPlacement, PluginsListRequest, PluginsReplaceRequest, plugin_placement};
use harmos_protocol::model::{FileMetadata, FileReference};
use harmos_runtime::bundle::Bundle;
use prost::Message;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// Local Imports
use super::{Host, lock};
use crate::{Error, Result};
use dokime_operations::host::files::delete;
use dokime_packaging::{bundle, installation, organization};
use dokime_protocol::host::{AppearanceLogo, AppearanceSnapshot, FilesDeleteRequest, LogoVariant, OrganizationIdentity};

/* === Constants === */

const INSTALLATION_ORIGIN: &str = "installation";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    /// The Installed Organization Pack's Appearance; Without a Loadable Pack the Host Keeps the Default.
    pub(crate) fn installed_appearance(&self, message_bytes: usize) -> AppearanceSnapshot {
        let default = || AppearanceSnapshot {
            app_name: "Dokime".into(),
            ..Default::default()
        };
        let pack = match self.installation_directory() {
            Ok(directory) => organization::installed(&directory),
            Err(error) => {
                tracing::error!(%error, "Load Organization Pack: Run the Host From Its Installation Folder");
                return default();
            }
        };
        let Some(pack) = pack else { return default() };
        let loaded = organization::read(&pack).and_then(|resolved| resolved.appearance()).map(snapshot).and_then(|snapshot| {
            if snapshot.encoded_len() > message_bytes {
                return Err(format!("its appearance exceeds the {message_bytes}-byte message budget"));
            }
            Ok(snapshot)
        });
        loaded.unwrap_or_else(|error| {
            let message = format!("Load Organization Pack {}: {error}; Correct the Pack and Restart the Host", pack.display());
            tracing::error!(pack = %pack.display(), %message, "Organization Pack Not Applied");
            default()
        })
    }

    pub(super) async fn reconcile_installation_plugins(&self) {
        let directory = match self.installation_directory() {
            Ok(directory) => directory.join(installation::PLUGINS),
            Err(error) => {
                self.report_plugin_admission_failure(Path::new(installation::PLUGINS), error);
                return;
            }
        };
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
        let mut names = BTreeSet::new();
        for path in files {
            let bundle = match bundle::open(&path) {
                Ok(bundle) => bundle,
                Err(error) => {
                    // An Unverified File Cannot Establish Which Previously Admitted Identity It Replaces.
                    complete = false;
                    self.report_plugin_admission_failure(&path, error);
                    continue;
                }
            };
            if !names.insert(bundle.manifest().identity.name.clone()) {
                self.report_plugin_admission_failure(&path, "Give Each Shipped Bundle a Distinct Plugin Identity");
                continue;
            }
            if let Err(error) = self.admit_installation_bundle(&bundle).await {
                self.report_plugin_admission_failure(&path, error);
            }
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
            if entry.origin != INSTALLATION_ORIGIN || !complete || names.contains(&entry.name) {
                continue;
            }
            if let Err(error) = self.remove_installation_plugin(&entry).await {
                self.report_plugin_admission_failure(directory, format!("Remove {}: {error}", entry.name));
            }
        }
        context.host.refresh_plugin_states().await?;
        Ok(())
    }

    async fn admit_installation_bundle(&self, bundle: &Bundle) -> Result<()> {
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
        let previous = entries.into_iter().find(|entry| entry.name == name);
        if let Some(entry) = &previous {
            let owned = entry.origin == INSTALLATION_ORIGIN;
            if !owned {
                return Err(Error::Invalid(format!(
                    "Admit Installation Plugin {name}: Entry Already Exists; Keep the User Entry and Give the Shipped Bundle an Unused Name"
                )));
            }
            let same_digest = managed(entry).is_some_and(|managed| managed.sha256 == digest);
            if same_digest {
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
                    media_type: "application/x-tar".into(),
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
                let run = error.into();
                if let Err(close) = self.release_unretained_bundle(content).await {
                    return Err(Error::Unsettled {
                        run: Box::new(run),
                        close: Box::new(close),
                    });
                }
                return Err(run);
            }
            if let Some(content) = managed(&entry).and_then(|managed| managed.content.as_ref()) {
                self.release_unretained_bundle(content.clone()).await?;
            }
        } else {
            let result = self.create_plugin(name.clone(), placement, Some(INSTALLATION_ORIGIN.into())).await;
            if let Err(error) = result {
                // Create Persists Failed Admissions Too; Keep Their Content and Registry Failure.
                if let Err(close) = self.release_unretained_bundle(content).await {
                    return Err(Error::Unsettled {
                        run: Box::new(error),
                        close: Box::new(close),
                    });
                }
                return Err(error);
            }
        }
        Ok(())
    }

    async fn remove_installation_plugin(&self, entry: &PluginEntry) -> Result<()> {
        if entry.origin != INSTALLATION_ORIGIN {
            return Err(Error::Invalid("Remove Installation Plugin: Keep the User-Installed Entry".into()));
        }
        let content = managed(entry).and_then(|managed| managed.content.clone());
        self.remove_plugin(&entry.name).await?;
        if let Some(content) = content {
            self.release_unretained_bundle(content).await?;
        }
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
            .filter_map(|entry| managed(entry).and_then(|managed| managed.content.as_ref()))
            .any(|content| content.position == file.position);
        if !retained {
            delete::handle(
                &context,
                FilesDeleteRequest {
                    file: Some(file),
                    request_id: Uuid::now_v7().to_string(),
                },
            )
            .await?;
        }
        Ok(())
    }

    // The Configured Installation Folder, Else the Folder Beside the Running Product Executable.
    fn installation_directory(&self) -> io::Result<PathBuf> {
        match &self.settings.installation {
            Some(directory) => Ok(directory.clone()),
            None => env::current_exe().and_then(|executable| installation::directory(&executable).map(Path::to_path_buf)),
        }
    }

    fn report_plugin_admission_failure(&self, path: &Path, reason: impl Display) {
        let message = format!("Admit Installation Plugin {}: {reason}; Repair or Remove This Bundle and Restart the Host", path.display());
        tracing::error!(file = %path.display(), %message, "Installation Plugin Admission Failed");
        if let Ok(mut failures) = lock(&self.plugin_admission_failures) {
            let name = format!("installation/{}", path.display());
            match failures.iter_mut().find(|failure| failure.0 == name) {
                Some(failure) => {
                    failure.1.push_str("; ");
                    failure.1.push_str(&message);
                }
                None => failures.push((name, message.clone())),
            }
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

// The Pack's Identity, Inline Logos and Self-Contained Stylesheet, Absent When the Pack Has No Theme.
fn snapshot(appearance: organization::Appearance) -> AppearanceSnapshot {
    let logos = appearance.logos.into_iter().map(|(variant, data_uri)| {
        let variant = match variant.as_str() {
            "transparent_on_light" => LogoVariant::TransparentOnLight,
            "transparent_on_dark" => LogoVariant::TransparentOnDark,
            "contained_on_light" => LogoVariant::ContainedOnLight,
            "contained_on_dark" => LogoVariant::ContainedOnDark,
            _ => LogoVariant::Unspecified,
        };
        AppearanceLogo { variant: variant as i32, data_uri }
    });
    AppearanceSnapshot {
        organization: appearance.organization.map(|organization| OrganizationIdentity {
            name: organization.name,
            unit: organization.unit,
            website: organization.website,
        }),
        title_template: Some(appearance.title_template),
        logos: logos.collect(),
        stylesheet: (!appearance.css.is_empty()).then_some(appearance.css),
        app_name: appearance.label,
    }
}

fn managed(entry: &PluginEntry) -> Option<&PluginManaged> {
    match entry.placement.as_ref()?.kind.as_ref()? {
        plugin_placement::Kind::Managed(managed) => Some(managed),
        _ => None,
    }
}

/* === Tests === */
