/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Installation Plugins Survive Product Startup

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::consts::EXE_SUFFIX,
    error::Error,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// Third Party Imports
use dokime_protocol::host::{PluginEntry, PluginManaged, PluginPlacement, PluginState, PluginValue, plugin_placement};
use harmos_runtime::bundle::Identity;
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id},
};
use tracing_subscriber::{
    Layer,
    layer::{Context, SubscriberExt},
};

// Local Imports
use super::support;
use dokime::MapValue;
use dokime_client_internal::{Client, ExecutionState, ResultStatus};
use dokime_packaging::{
    bundle,
    installer::{self, PackageOptions},
};
use dokime_protocol::host::execution_outcome;
use dokime_runtime::{Host, HostSettings, services};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct AdmissionMutations(Arc<Mutex<BTreeMap<String, usize>>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Visit for AdmissionMutations {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.record_str(field, &format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if !matches!(field.name(), "operation" | "action") {
            return;
        }
        if matches!(
            value,
            "harmos.host.v1.Plugins/Create"
                | "harmos.host.v1.Plugins/Replace"
                | "harmos.host.v1.Plugins/Delete"
                | "dokime.host.v1.files.Uploads/Create"
                | "dokime.host.v1.files.Uploads/Append"
                | "dokime.host.v1.files.Uploads/Publish"
                | "dokime.host.v1.Files/Delete"
        ) {
            *self.0.lock().unwrap().entry(value.into()).or_default() += 1;
        }
    }
}

impl<S: Subscriber> Layer<S> for AdmissionMutations {
    fn on_new_span(&self, attributes: &Attributes<'_>, _: &Id, _: Context<'_, S>) {
        if attributes.metadata().name() == "operations.admit_entry" {
            attributes.record(&mut self.clone());
        }
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() == "dokime_operations::host::files::transfer" {
            event.record(&mut self.clone());
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn shipped_bundle(executable: &Path, name: &str, version: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let identity = Identity {
        name: name.into(),
        version: version.parse()?,
        description: String::new(),
        package: Default::default(),
    };
    bundle::create(executable, &[], identity, None).map_err(|error| error as Box<dyn Error>)
}

async fn open(root: &Path, installation: &Path) -> Result<(services::Server, Client), Box<dyn Error>> {
    let host = HostSettings {
        installation: Some(installation.to_path_buf()),
        ..support::settings()
    };
    let server = services::serve("127.0.0.1:0".parse()?, root, host, support::service_settings()).await?;
    let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
    let client = Client::from_channel(channel, support::client_settings())?;
    Ok((server, client))
}

async fn inventory(client: &Client) -> Result<BTreeMap<String, PluginEntry>, Box<dyn Error>> {
    let mut entries = BTreeMap::new();
    for entry in client.plugins.list().await?.entries {
        let state = if entry.value.as_ref().unwrap().enabled { PluginState::Ready } else { PluginState::Stopped };
        let entry = support::plugin_state(&client.plugins, entry, state).await?;
        entries.insert(entry.name.clone(), entry);
    }
    Ok(entries)
}

fn content(entry: &PluginEntry) -> &PluginManaged {
    let Some(plugin_placement::Kind::Managed(managed)) = entry.placement.as_ref().unwrap().kind.as_ref() else {
        panic!("Managed Entry Required")
    };
    managed
}

fn copies(root: &Path) -> Result<BTreeSet<PathBuf>, Box<dyn Error>> {
    fs::read_dir(root.join("plugins"))?.map(|entry| Ok(entry?.path())).collect()
}

async fn install_user(client: &Client, bytes: Vec<u8>, name: &str, enabled: bool) -> Result<PluginEntry, Box<dyn Error>> {
    let sha256 = Sha256::digest(&bytes).to_vec();
    let file = support::bundles::upload(client, bytes).await?;
    let placement = PluginPlacement {
        kind: Some(plugin_placement::Kind::Managed(PluginManaged { content: Some(file), sha256 })),
    };
    let entry = client.plugins.create(name, placement, PluginValue { enabled }).await?.entry.unwrap();
    let state = if enabled { PluginState::Ready } else { PluginState::Stopped };
    support::plugin_state(&client.plugins, entry, state).await
}

/* === Tests === */

#[test]
fn installation_plugins_mirror_bundles_across_restarts() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit Two Fabricated Bundles Through Startup.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            let executable = support::thermal_fixture()?;
            let mut first_path = plugins.join("first.dokime");
            fs::write(&first_path, shipped_bundle(&executable, "com.example.testhub", "1.0.0")?)?;
            fs::write(plugins.join("second.dokime"), shipped_bundle(&support::ping_fixture()?, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let first = inventory(&client).await?;
            assert_eq!(first.len(), 2);
            for entry in first.values() {
                assert_eq!(entry.origin, "installation");
            }

            // Keep a Disabled User Install Beside the Shipped Set.
            let user = install_user(
                &client,
                shipped_bundle(&support::capture_fixture()?, "example_pack_user", "1.0.0")?,
                "example_pack_user",
                false,
            )
            .await?;
            let installed_copies = copies(root.path())?;
            assert_eq!(installed_copies.len(), 3);
            server.close().await?;

            // Restart Without Uploading, Replacing or Re-extracting Unchanged Bundles.
            let (server, client) = open(root.path(), installation.path()).await?;
            let unchanged = inventory(&client).await?;
            for (name, entry) in &first {
                assert_eq!(unchanged[name].revision, entry.revision);
                assert_eq!(unchanged[name].placement, entry.placement);
            }
            assert_eq!(unchanged["example_pack_user"].placement, user.placement);
            assert_eq!(copies(root.path())?, installed_copies);
            server.close().await?;

            // A Renamed Bundle Keeps Its Management Identity and Enablement Without a Write.
            let renamed = plugins.join("renamed.dokime");
            fs::rename(&first_path, &renamed)?;
            first_path = renamed;
            let (server, client) = open(root.path(), installation.path()).await?;
            let renamed = inventory(&client).await?;
            assert_eq!(renamed.len(), 3);
            assert_eq!(renamed["com.example.testhub"].name, first["com.example.testhub"].name);
            assert_eq!(renamed["com.example.testhub"].value, first["com.example.testhub"].value);
            assert_eq!(renamed["com.example.testhub"].description.as_ref().unwrap().version, "1.0.0");
            assert_eq!(copies(root.path())?.len(), 3);
            server.close().await?;

            // Replace Changed Content and Uninstall a Removed Shipped Bundle.
            fs::write(&first_path, shipped_bundle(&executable, "com.example.testhub", "2.0.0")?)?;
            fs::remove_file(plugins.join("second.dokime"))?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let updated = inventory(&client).await?;
            assert_eq!(updated.len(), 2);
            assert_ne!(updated["com.example.testhub"].revision, first["com.example.testhub"].revision);
            assert_ne!(updated["com.example.testhub"].placement, first["com.example.testhub"].placement);
            assert_eq!(updated["com.example.testhub"].description.as_ref().unwrap().version, "2.0.0");
            assert_eq!(updated["example_pack_user"].placement, user.placement);
            assert_eq!(copies(root.path())?.len(), 2);
            for entry in first.values() {
                assert!(client.files.get(content(entry).content.clone().unwrap()).await.is_err());
            }
            server.close().await?;

            // Removing the Whole Folder Retires Only Installation Entries.
            fs::remove_file(&first_path)?;
            fs::remove_dir(&plugins)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let remaining = inventory(&client).await?;
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining["example_pack_user"].placement, user.placement);
            assert_eq!(copies(root.path())?.len(), 1);
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn unchanged_restart_performs_no_admission_mutations() -> Result<(), Box<dyn Error>> {
    let mutations = AdmissionMutations::default();
    let subscriber = tracing_subscriber::registry().with(mutations.clone());
    // Each Story Runs in Its Own Process; Observe Background Threads Too.
    tracing::subscriber::set_global_default(subscriber)?;
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit One Shipped Bundle.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            let executable = support::thermal_fixture()?;
            let path = plugins.join("example_pack.dokime");
            fs::write(&path, shipped_bundle(&executable, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let original = inventory(&client).await?;
            server.close().await?;

            // Confirm the Existing Operation and Transfer Traces Observe Initial Admission.
            let admitted = mutations.0.lock().unwrap().clone();
            assert_eq!(admitted["harmos.host.v1.Plugins/Create"], 1);
            assert_eq!(admitted["dokime.host.v1.files.Uploads/Create"], 1);
            assert!(admitted["dokime.host.v1.files.Uploads/Append"] > 0);
            assert_eq!(admitted["dokime.host.v1.files.Uploads/Publish"], 1);
            mutations.0.lock().unwrap().clear();

            // Restore the Unchanged Folder Without Admission Mutations.
            let (server, client) = open(root.path(), installation.path()).await?;
            let unchanged = inventory(&client).await?;
            assert_eq!(unchanged.keys().collect::<Vec<_>>(), original.keys().collect::<Vec<_>>());
            for (name, entry) in &original {
                assert_eq!(unchanged[name].name, entry.name);
                assert_eq!(unchanged[name].origin, entry.origin);
                assert_eq!(unchanged[name].revision, entry.revision);
                assert_eq!(unchanged[name].placement, entry.placement);
                assert_eq!(unchanged[name].description.as_ref().unwrap().version, entry.description.as_ref().unwrap().version);
            }
            server.close().await?;
            assert!(mutations.0.lock().unwrap().is_empty(), "An Unchanged Restart Must Issue No Admission Mutations");

            // Confirm the Counter Observes an Actual Replacement.
            fs::write(&path, shipped_bundle(&executable, "example_pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let updated = inventory(&client).await?;
            assert_ne!(updated["example_pack"].revision, original["example_pack"].revision);
            server.close().await?;
            let replaced = mutations.0.lock().unwrap().clone();
            assert_eq!(replaced["harmos.host.v1.Plugins/Replace"], 1);
            assert_eq!(replaced["dokime.host.v1.files.Uploads/Create"], 1);
            assert!(replaced["dokime.host.v1.files.Uploads/Append"] > 0);
            assert_eq!(replaced["dokime.host.v1.files.Uploads/Publish"], 1);
            assert_eq!(replaced["dokime.host.v1.Files/Delete"], 1);
            mutations.0.lock().unwrap().clear();

            // Confirm the Counter Observes Entry Deletion and Content Release.
            fs::remove_file(&path)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert!(inventory(&client).await?.is_empty());
            let health = client.evidence.health_events.list(Default::default()).await?;
            assert!(health.events.is_empty(), "{health:?}");
            server.close().await?;
            let removed = mutations.0.lock().unwrap().clone();
            assert_eq!(removed["harmos.host.v1.Plugins/Delete"], 1);
            assert_eq!(removed["dokime.host.v1.Files/Delete"], 1);
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn corrupt_installation_bundle_reports_health_and_keeps_valid_bundles() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Put a Corrupt Bundle Before a Valid Bundle in Enumeration Order.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            let corrupt = plugins.join("a-corrupt.dokime");
            fs::write(&corrupt, b"corrupt archive")?;
            fs::write(plugins.join("z-valid.dokime"), shipped_bundle(&support::thermal_fixture()?, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert!(client.status.get().await?.status.unwrap().available);
            assert_eq!(inventory(&client).await?.len(), 1);
            let health = client.evidence.health_events.list(Default::default()).await?;
            assert_eq!(health.events.len(), 1);
            assert!(health.events[0].message.contains(&corrupt.display().to_string()));
            assert!(!health.events[0].message.is_empty());
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn user_plugin_wins_a_shipped_name_collision() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // A Development Layout Without Plugins Preserves User Installation Behaviour.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let executable = support::thermal_fixture()?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert!(inventory(&client).await?.is_empty());
            assert!(client.evidence.health_events.list(Default::default()).await?.events.is_empty());
            let user = install_user(&client, shipped_bundle(&executable, "example_pack", "1.0.0")?, "example_pack", true).await?;
            server.close().await?;

            // An Installer Cannot Replace a User Entry of the Same Identity.
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            fs::write(plugins.join("example_pack.dokime"), shipped_bundle(&executable, "example_pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let current = inventory(&client).await?;
            assert_eq!(current.len(), 1);
            assert_eq!(current["example_pack"].placement, user.placement);
            assert_eq!(current["example_pack"].revision, user.revision);
            assert_eq!(client.evidence.health_events.list(Default::default()).await?.events.len(), 1);
            server.close().await?;

            // Removing the Shipped Bundle Also Leaves the User Entry Untouched.
            fs::remove_file(plugins.join("example_pack.dokime"))?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert_eq!(inventory(&client).await?["example_pack"].placement, user.placement);
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn corrupt_update_preserves_the_previously_admitted_bundle() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit the Original Bundle.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            let path = plugins.join("example_pack.dokime");
            fs::write(&path, shipped_bundle(&support::thermal_fixture()?, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let original = inventory(&client).await?.remove("example_pack").unwrap();
            server.close().await?;

            // Verification Failure Retains the Existing Entry and Reports the File.
            fs::write(&path, b"corrupt replacement")?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let retained = inventory(&client).await?.remove("example_pack").unwrap();
            assert_eq!(retained.placement, original.placement);
            assert_eq!(retained.revision, original.revision);
            assert_eq!(copies(root.path())?.len(), 1);
            assert_eq!(client.evidence.health_events.list(Default::default()).await?.events.len(), 1);
            server.close().await?;

            // A Verified but Unlaunchable Replacement Also Rolls Back and Releases Its Candidate.
            let invalid = installation.path().join(format!("example_pack{EXE_SUFFIX}"));
            fs::write(&invalid, b"invalid native executable")?;
            fs::write(&path, shipped_bundle(&invalid, "example_pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let retained = inventory(&client).await?.remove("example_pack").unwrap();
            assert_eq!(retained.placement, original.placement);
            assert_eq!(retained.revision, original.revision);
            assert_eq!(copies(root.path())?.len(), 1);
            assert_eq!(client.evidence.health_events.list(Default::default()).await?.events.len(), 1);
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn prepared_installer_layout_lists_and_runs_its_shipped_template() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Fabricate the Distribution Inventory and Its Selected Plugin Through the Bundle API.
            let source = tempfile::tempdir()?;
            let installation = tempfile::tempdir()?;
            let root = support::tempdir()?;
            let executable = support::thermal_fixture()?;
            fs::create_dir_all(source.path().join("bin"))?;
            fs::create_dir_all(source.path().join("organization"))?;
            let mut files = BTreeMap::new();
            let product = fs::read(&executable)?;
            for name in [format!("bin/dokime{EXE_SUFFIX}"), format!("bin/dokime-gui{EXE_SUFFIX}")] {
                fs::write(source.path().join(&name), &product)?;
                files.insert(name, product.clone());
            }
            for name in ["LICENSE", "NOTICE", "THIRD-PARTY-NOTICES"] {
                fs::write(source.path().join(name), b"example terms")?;
                files.insert(name.into(), b"example terms".to_vec());
            }
            let organization = b"[app]\nlabel='Example Test Hub'\n[app.artwork]\napplication='icon.png'\ninstaller='icon.png'\n";
            let icon = include_bytes!("../../../dokime-gui/src-tauri/icons/icon.png");
            fs::write(source.path().join("organization/organization.toml"), organization)?;
            fs::write(source.path().join("organization/icon.png"), icon)?;
            files.insert("organization/organization.toml".into(), organization.to_vec());
            files.insert("organization/icon.png".into(), icon.to_vec());
            let files: BTreeMap<_, _> = files.into_iter().map(|(name, bytes)| (name, harmos_runtime::bundle::digest(&bytes))).collect();
            let metadata =
                serde_json::json!({"schema":1,"version":env!("CARGO_PKG_VERSION"),"target":bundle::native_target(),"source_revision":"f7c50e68","channel":"nightly","plugins":[],"files":files});
            fs::write(source.path().join("distribution.json"), serde_json::to_vec(&metadata)?)?;
            let plugin = source.path().join("example_pack.dokime");
            fs::write(&plugin, shipped_bundle(&executable, "example_pack", "1.0.0")?)?;
            let format = if cfg!(target_os = "windows") { "nsis" } else { "deb" };
            let manifest = source.path().join("installer.toml");
            fs::write(
                &manifest,
                format!(
                    "[installer]\nversion='1.2.3'\nidentifier='com.example.testhub'\nformats=['{format}']\n[dokime]\nversion='{}'\n",
                    env!("CARGO_PKG_VERSION")
                ),
            )?;

            // Prepare the Exact Native Payload Without Running an Installer Tool.
            let options = PackageOptions {
                command: |_| panic!("Native Installer Tools Must Not Run in This Story"),
                artifacts: Some(source.path().to_path_buf()),
                plugins: vec![plugin],
                output: PathBuf::new(),
                sign: false,
                formats: Vec::new(),
                target: None,
            };
            let prepared = installer::prepare(&manifest, &options, &mut installer::Observer::default()).await?;
            let directory = if cfg!(target_os = "windows") {
                prepared.settings().copy_resources(installation.path())?;
                for name in [format!("dokime{EXE_SUFFIX}"), format!("dokime-gui{EXE_SUFFIX}")] {
                    fs::copy(prepared.settings().project_out_directory().join(&name), installation.path().join(name))?;
                }
                installation.path().to_path_buf()
            } else {
                for (destination, source) in &prepared.settings().deb().files {
                    let destination = installation.path().join(destination.strip_prefix("/")?);
                    fs::create_dir_all(destination.parent().unwrap())?;
                    fs::copy(source, destination)?;
                }
                installation.path().join("usr/lib/com-example-testhub")
            };
            assert!(directory.join("plugins/example_pack.dokime").is_file());

            // List the Shipped Entry and Run Its Template Through the Host Client.
            let product = directory.join(format!("dokime{EXE_SUFFIX}"));
            assert!(product.is_file());
            let directory = dokime_packaging::installation::directory(&product)?;
            let (server, client) = open(root.path(), directory).await?;
            assert_eq!(inventory(&client).await?.len(), 1);
            let suite = client.suites.list(String::new()).await?.values.remove(0);
            let suite_id = suite.metadata.as_ref().unwrap().identifier.clone();
            let template = suite.templates[0].metadata.as_ref().unwrap().identifier.clone();
            let cases = support::materialize(&client, suite_id, &template, HashMap::new()).await?;
            let execution = support::admit(&client, cases).await?.remove(0);
            let mut progress = client.executions.progress.watch(execution.clone()).await?;
            let mut sealed = false;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    let pending = client.prompts.list(execution.clone(), true).await?;
                    let prompt = pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
                    client.prompts.answers.create(prompt, "0", MapValue::default(), true).await?;
                }
                sealed |= frame.sealed;
            }
            assert!(sealed);
            let detail = client.executions.get(execution).await?.execution.unwrap();
            assert_eq!(detail.summary.unwrap().state, ExecutionState::Sealed as i32);
            let Some(execution_outcome::Attribution::Evaluated(outcome)) = detail.outcome.and_then(|outcome| outcome.attribution) else {
                panic!("Template Must Finish With an Evaluated Outcome")
            };
            assert_eq!(outcome.status, ResultStatus::Passed as i32);
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn failed_admission_does_not_block_other_installation_plugins() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // A Verified Bundle Can Still Fail Native Process Admission.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            let invalid = installation.path().join(format!("example_pack{EXE_SUFFIX}"));
            fs::write(&invalid, b"invalid native executable")?;
            let failed = plugins.join("a-failed.dokime");
            fs::write(&failed, shipped_bundle(&invalid, "com.example.testhub", "1.0.0")?)?;
            fs::write(plugins.join("z-valid.dokime"), shipped_bundle(&support::thermal_fixture()?, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert!(client.status.get().await?.status.unwrap().available);
            let entries = client.plugins.list().await?.entries;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries.iter().find(|entry| entry.name == "example_pack").unwrap().state, PluginState::Ready as i32);
            let unavailable = entries.iter().find(|entry| entry.name == "com.example.testhub").unwrap();
            assert_eq!(unavailable.state, PluginState::Unavailable as i32);
            assert!(unavailable.failure.is_some());
            let health = client.evidence.health_events.list(Default::default()).await?;
            assert!(health.events.iter().any(|event| event.message.contains(&failed.display().to_string())));
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn user_entry_reusing_installation_content_survives_reconciliation() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit an Installation Bundle.
            let root = support::tempdir()?;
            let installation = tempfile::tempdir()?;
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            fs::write(plugins.join("example_pack.dokime"), shipped_bundle(&support::thermal_fixture()?, "example_pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let shipped = inventory(&client).await?.remove("example_pack").unwrap();
            let file = content(&shipped).content.clone().unwrap();
            assert_eq!(shipped.origin, "installation");

            // A Disabled User Entry Reuses Content Without Registering the Same Operations Twice.
            let user = client
                .plugins
                .create("com.example.testhub", shipped.placement.unwrap(), PluginValue { enabled: false })
                .await?
                .entry
                .unwrap();
            let user = support::plugin_state(&client.plugins, user, PluginState::Stopped).await?;
            assert!(user.origin.is_empty());
            server.close().await?;

            // Startup Must Preserve Both Entries and Their Shared Immutable Content.
            let (server, client) = open(root.path(), installation.path()).await?;
            let entries = inventory(&client).await?;
            let content_available = client.files.get(file.clone()).await.is_ok();
            let health = client.evidence.health_events.list(Default::default()).await?;
            server.close().await?;
            assert!(entries.contains_key("com.example.testhub"));
            assert!(entries.contains_key("example_pack"));
            assert!(content_available);
            assert!(health.events.is_empty());

            // Replacing the Shipped Bundle Must Keep Content Still Referenced by the User Entry.
            fs::write(plugins.join("example_pack.dokime"), shipped_bundle(&support::thermal_fixture()?, "example_pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let entries = inventory(&client).await?;
            assert_eq!(entries["example_pack"].origin, "installation");
            assert_eq!(entries["com.example.testhub"].revision, user.revision);
            assert_eq!(entries["com.example.testhub"].placement, user.placement);
            assert!(client.files.get(file.clone()).await.is_ok());
            let current = entries["example_pack"].clone();
            let current_file = content(&current).content.clone().unwrap();
            let sharing =
                client.plugins.create("example.current", current.placement.unwrap(), PluginValue { enabled: false }).await?.entry.unwrap();
            let sharing = support::plugin_state(&client.plugins, sharing, PluginState::Stopped).await?;
            assert!(sharing.origin.is_empty());
            server.close().await?;

            // Removing the Shipped Bundle Retires Only Its Entry, Preserving the User's Shared Content.
            fs::remove_file(plugins.join("example_pack.dokime"))?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let entries = inventory(&client).await?;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries["example.current"].placement, sharing.placement);
            assert_eq!(entries["com.example.testhub"].revision, user.revision);
            assert_eq!(entries["com.example.testhub"].placement, user.placement);
            assert!(client.files.get(file).await.is_ok());
            assert!(client.files.get(current_file).await.is_ok());
            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn host_startup_future_can_cross_threads() {
    fn require_send(_: impl Send) {}
    fn check(root: &Path) {
        let startup = Host::open(root, support::settings(), support::service_settings());
        require_send(startup);
    }
    check(Path::new("."));
}
