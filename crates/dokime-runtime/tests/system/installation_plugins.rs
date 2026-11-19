/* ***
 *
 * dokime-rust-workspace
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
    fs,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_protocol::host::{PluginEntry, PluginManaged, PluginPlacement, PluginState, PluginValue, plugin_placement};
use harmos_runtime::bundle::Identity;
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use super::support;
use dokime::client::{ExecutionState, ResultStatus};
use dokime::{Client, MapValue};
use dokime_packaging::{
    bundle,
    installer::{self, PackageOptions},
};
use dokime_protocol::host::execution_outcome;
use dokime_runtime::{HostSettings, services};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

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

async fn install_user(client: &Client, bytes: Vec<u8>, name: &str) -> Result<PluginEntry, Box<dyn Error>> {
    let sha256 = Sha256::digest(&bytes).to_vec();
    let file = support::bundles::upload(client, bytes).await?;
    let placement = PluginPlacement {
        kind: Some(plugin_placement::Kind::Managed(PluginManaged { content: Some(file), sha256 })),
    };
    let entry = client.plugins.create(name, placement, PluginValue { enabled: true }).await?.entry.unwrap();
    support::plugin_state(&client.plugins, entry, PluginState::Ready).await
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
            fs::write(plugins.join("second.dokime"), shipped_bundle(&executable, "example-pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let first = inventory(&client).await?;
            assert_eq!(first.len(), 2);
            for entry in first.values() {
                assert!(content(entry).content.as_ref().unwrap().metadata.as_ref().unwrap().media_type.contains("source=installation"));
            }

            // Keep a User Install Beside the Shipped Set.
            let user = install_user(&client, shipped_bundle(&executable, "example-pack-user", "1.0.0")?, "example-pack-user").await?;
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
            assert_eq!(unchanged["example-pack-user"].placement, user.placement);
            assert_eq!(copies(root.path())?, installed_copies);
            server.close().await?;

            // A Renamed Bundle Keeps Its Management Identity and Enablement Through Replace.
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
            assert_eq!(updated["example-pack-user"].placement, user.placement);
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
            assert_eq!(remaining["example-pack-user"].placement, user.placement);
            assert_eq!(copies(root.path())?.len(), 1);
            server.close().await?;
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
            fs::write(plugins.join("z-valid.dokime"), shipped_bundle(&support::thermal_fixture()?, "example-pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
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
            let user = install_user(&client, shipped_bundle(&executable, "example-pack", "1.0.0")?, "example-pack").await?;
            server.close().await?;

            // An Installer Cannot Replace a User Entry of the Same Identity.
            let plugins = installation.path().join("plugins");
            fs::create_dir_all(&plugins)?;
            fs::write(plugins.join("example-pack.dokime"), shipped_bundle(&executable, "example-pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let current = inventory(&client).await?;
            assert_eq!(current.len(), 1);
            assert_eq!(current["example-pack"].placement, user.placement);
            assert_eq!(current["example-pack"].revision, user.revision);
            assert_eq!(client.evidence.health_events.list(Default::default()).await?.events.len(), 1);
            server.close().await?;

            // Removing the Shipped Bundle Also Leaves the User Entry Untouched.
            fs::remove_file(plugins.join("example-pack.dokime"))?;
            let (server, client) = open(root.path(), installation.path()).await?;
            assert_eq!(inventory(&client).await?["example-pack"].placement, user.placement);
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
            let path = plugins.join("example-pack.dokime");
            fs::write(&path, shipped_bundle(&support::thermal_fixture()?, "example-pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let original = inventory(&client).await?.remove("example-pack").unwrap();
            server.close().await?;

            // Verification Failure Retains the Existing Entry and Reports the File.
            fs::write(&path, b"corrupt replacement")?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let retained = inventory(&client).await?.remove("example-pack").unwrap();
            assert_eq!(retained.placement, original.placement);
            assert_eq!(retained.revision, original.revision);
            assert_eq!(copies(root.path())?.len(), 1);
            assert_eq!(client.evidence.health_events.list(Default::default()).await?.events.len(), 1);
            server.close().await?;

            // A Verified but Unlaunchable Replacement Also Rolls Back and Releases Its Candidate.
            let invalid = installation.path().join(format!("example-pack{EXE_SUFFIX}"));
            fs::write(&invalid, b"invalid native executable")?;
            fs::write(&path, shipped_bundle(&invalid, "example-pack", "2.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let retained = inventory(&client).await?.remove("example-pack").unwrap();
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
                serde_json::json!({"schema":1,"version":env!("CARGO_PKG_VERSION"),"target":bundle::native_target(),"files":files});
            fs::write(source.path().join("distribution.json"), serde_json::to_vec(&metadata)?)?;
            let plugin = source.path().join("example-pack.dokime");
            fs::write(&plugin, shipped_bundle(&executable, "example-pack", "1.0.0")?)?;
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
            let prepared = installer::prepare(&manifest, &options).await?;
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
            assert!(directory.join("plugins/example-pack.dokime").is_file());

            // List the Shipped Entry and Run Its Template Through the Host Client.
            let product = directory.join(format!("dokime{EXE_SUFFIX}"));
            assert!(product.is_file());
            let directory = dokime_runtime::installation_directory(&product)?;
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
            let invalid = installation.path().join(format!("example-pack{EXE_SUFFIX}"));
            fs::write(&invalid, b"invalid native executable")?;
            let failed = plugins.join("a-failed.dokime");
            fs::write(&failed, shipped_bundle(&invalid, "com.example.testhub", "1.0.0")?)?;
            fs::write(plugins.join("z-valid.dokime"), shipped_bundle(&support::thermal_fixture()?, "example-pack", "1.0.0")?)?;
            let (server, client) = open(root.path(), installation.path()).await?;
            let entries = client.plugins.list().await?.entries;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries.iter().find(|entry| entry.name == "example-pack").unwrap().state, PluginState::Ready as i32);
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
