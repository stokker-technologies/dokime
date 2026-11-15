/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Reads Only Declared Secrets

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use dokime::client::EvidenceRecordsListOptions;

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::EvidenceRecordScope;
use dokime::Metadata;
use dokime::SecretWritable;
use dokime::Value;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginState;
use dokime::client::PluginValue;
use dokime::client::PluginsSecretBindingValue;
use dokime::client::PluginsSecretsListOptions;
use dokime::client::SecretsDeleteOptions;
use dokime::client::SecretsSetOptions;
use dokime::record_scope::Kind as RecordScopeKind;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use prost::Message;
use std::collections::HashMap;
use std::error::Error as StdError;
use tokio::time;
use tonic::transport::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(PartialEq, Message)]
struct ReadResult {
    #[prost(string, tag = "1")]
    value: String,

    #[prost(string, tag = "2")]
    refusal: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn read(client: &Client, plugin: &str, identifier: &str) -> Result<ReadResult, Box<dyn StdError>> {
    // Materialize the Requested Identifier and Execute the Plugin's Read
    let parameters = HashMap::from([(
        "identifier".into(),
        vec![Value {
            kind: Some(ValueKind::Text(identifier.into())),
        }],
    )]);
    let cases = super::support::materialize(client, plugin, format!("{plugin}.read"), parameters).await?;
    let cycle = super::support::admit(client, cases).await?;
    let mut progress = client.executions.progress.watch(cycle[0].clone()).await?;
    while let Some(frame) = progress.next().await? {
        if frame.finished {
            break;
        }
    }

    // Read the Result Returned Through the Real Invocation Carrier
    let records = client
        .evidence
        .records
        .list_with(
            EvidenceRecordScope {
                kind: Some(RecordScopeKind::Execution(cycle[0].clone())),
            },
            String::new(),
            EvidenceRecordsListOptions {
                versions: Some(Vec::new()),
                between: Some(Default::default()),
                step: Some(String::new()),
                selection: Some(Default::default()),
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(records.records.len(), 1);
    Ok(ReadResult::decode(records.records[0].payload.as_slice())?)
}

/* === Tests === */

#[test]
fn plugin_secret_bindings_reconcile_on_restart_and_entry_deletion() -> Result<(), Box<dyn StdError>> {
    super::support::run(Box::pin(async {
        use super::{client_runs_thermal_suite_over_grpc::HostProcess, support};
        time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Create One Declared Binding Through the Library Entry
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let mut plugin = PluginProcess::start(&support::secrets_fixture()?, true, &support::settings().placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let placement = PluginPlacement {
                kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
            };
            client.plugins.create("secret_reader", placement.clone(), PluginValue { enabled: true }).await?;
            let options = PluginsSecretsListOptions {
                name: Some("secret_reader".into()),
                ..Default::default()
            };
            let before = client
                .plugins
                .secrets
                .list_with(PluginsSecretsListOptions {
                    name: options.name.clone(),
                    ..Default::default()
                })
                .await?;
            let binding = PluginsSecretBindingValue {
                identifier: "secrets.owned".into(),
                secret: "unavailable.secret".into(),
            };
            let saved = client.plugins.secrets.set("secret_reader", vec![binding], before.revision).await?;
            assert_eq!(saved.bindings[0].secret, "unavailable.secret");
            assert!(!saved.bindings[0].available);
            process.close().await?;

            // Startup Retains the Current Declared Map With Its Revision
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let reopened = client
                .plugins
                .secrets
                .list_with(PluginsSecretsListOptions {
                    name: options.name.clone(),
                    ..Default::default()
                })
                .await?;
            assert_eq!(reopened.bindings, saved.bindings);
            assert_eq!(reopened.revision, saved.revision);

            // Entry Deletion Removes Its Bindings Before the Same Identity Is Reused
            let mut changes = client.backend.overview.watch().await?;
            changes.next().await?.ok_or("Observe Plugin Secrets: Missing Opening")?;
            let entry = client.plugins.get("secret_reader").await?.entry.unwrap();
            client.plugins.delete(entry.name, entry.revision).await?;
            assert!(
                client
                    .plugins
                    .secrets
                    .list_with(PluginsSecretsListOptions {
                        name: options.name.clone(),
                        ..Default::default()
                    })
                    .await
                    .is_err()
            );
            client.plugins.create("secret_reader", placement, PluginValue { enabled: true }).await?;
            loop {
                let current = client
                    .plugins
                    .secrets
                    .list_with(PluginsSecretsListOptions {
                        name: options.name.clone(),
                        ..Default::default()
                    })
                    .await?;
                if current.bindings.iter().all(|binding| binding.secret.is_empty()) {
                    assert!(current.bindings.iter().all(|binding| !binding.available));
                    break;
                }
                changes.next().await?.ok_or("Observe Plugin Secrets: Closed Before Binding Cleanup")?;
            }
            drop(changes);
            process.close().await?;
            plugin.close().await?;
            Ok::<(), Box<dyn StdError>>(())
        })
        .await?
    }))
}

#[test]
fn plugin_reads_only_declared_secrets() -> Result<(), Box<dyn StdError>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Connect Two Plugins with Different Secret Declarations
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let executable = super::support::secrets_fixture()?;
            let mut reader = PluginProcess::start(&executable, true, &settings.placement).await?;
            let mut owner = PluginProcess::start_with_environment(
                &executable,
                true,
                &settings.placement,
                &[("DOKIME_SECRET_FIXTURE_OWNER".into(), "1".into())],
            )
            .await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let reader_entry = client
                .plugins
                .create(
                    "secret_reader",
                    PluginPlacement {
                        kind: Some(PluginPlacementKind::Remote(PluginRemote { url: reader.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let reader_entry =
                super::support::plugin_state(&client.plugins, reader_entry, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(reader_entry.state, PluginState::Ready as i32, "{reader_entry:?}");
            let owner_entry = client
                .plugins
                .create(
                    "secret_owner",
                    PluginPlacement {
                        kind: Some(PluginPlacementKind::Remote(PluginRemote { url: owner.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let owner_entry = super::support::plugin_state(&client.plugins, owner_entry, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(owner_entry.state, PluginState::Ready as i32, "{owner_entry:?}");
            let secrets = Client::from_channel(channel, super::support::client_settings())?.secrets.clone();

            // Administrative Writes and Reads Need No Plugin Declaration
            for identifier in ["secrets.owned", "secrets.other", "secrets.text"] {
                let initial = secrets.list().await?;
                secrets
                    .set_with(
                        identifier,
                        SecretWritable {
                            metadata: Some(Metadata {
                                identifier: identifier.into(),
                                label: "Fixture Secret".into(),
                                ..Default::default()
                            }),
                            value: Some("fixture-secret-value".into()),
                        },
                        initial.generation.to_be_bytes().to_vec(),
                        SecretsSetOptions {
                            request_id: Some(format!("save-{identifier}")),
                            ..Default::default()
                        },
                    )
                    .await?;
                let actual = secrets.get(identifier).await?;
                assert_eq!(actual.value.unwrap().value.as_deref(), Some("fixture-secret-value"));
            }

            // Bind Each Declared Name Without Exposing Credential Values
            for (name, identifier) in [("secret_reader", "secrets.owned"), ("secret_owner", "secrets.other")] {
                let before = client
                    .plugins
                    .secrets
                    .list_with(PluginsSecretsListOptions {
                        name: Some(name.into()),
                        ..Default::default()
                    })
                    .await?;
                assert_eq!(before.bindings.len(), 1);
                assert_eq!(before.bindings[0].declaration.as_ref().unwrap().metadata.as_ref().unwrap().identifier, identifier);
                let bindings = vec![PluginsSecretBindingValue {
                    identifier: identifier.into(),
                    secret: identifier.into(),
                }];
                let saved = client.plugins.secrets.set(name, bindings, before.revision).await?;
                assert!(saved.bindings[0].available);
            }

            // Each Plugin Can Read Its Own Declared Secret
            let own = read(&client, "secret_reader", "secrets.owned").await?;
            assert_eq!(own.value, "fixture-secret-value");
            assert!(own.refusal.is_empty());
            let other = read(&client, "secret_owner", "secrets.other").await?;
            assert_eq!(other, own);

            // Another Plugin's Declaration Does Not Grant Access
            let existing = read(&client, "secret_reader", "secrets.other").await?;
            assert!(existing.value.is_empty());
            assert!(!existing.refusal.is_empty());

            // The Same Identifier Has the Same Refusal After Administrative Deletion
            let initial = secrets.list().await?;
            let deleted = secrets
                .delete_with(
                    vec!["secrets.other".into()],
                    initial.generation.to_be_bytes().to_vec(),
                    SecretsDeleteOptions {
                        request_id: Some("remove-other".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!deleted.cleanup_pending);
            let absent = read(&client, "secret_reader", "secrets.other").await?;
            assert_eq!(absent, existing);

            // A Declared but Missing Secret Retains Its Store Failure
            let missing = read(&client, "secret_owner", "secrets.other").await?;
            assert!(missing.value.is_empty());
            assert!(!missing.refusal.is_empty());

            // A Text Setting Is Not a Secret Declaration
            let text = read(&client, "secret_reader", "secrets.text").await?;
            assert!(text.value.is_empty());
            assert!(!text.refusal.is_empty());

            // Remove Every Credential Created by This Story
            let initial = secrets.list().await?;
            let deleted = secrets
                .delete_with(
                    vec!["secrets.owned".into(), "secrets.text".into()],
                    initial.generation.to_be_bytes().to_vec(),
                    SecretsDeleteOptions {
                        request_id: Some("cleanup".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!deleted.cleanup_pending);
            assert!(secrets.list().await?.entries.is_empty());

            process.close().await?;
            reader.close().await?;
            owner.close().await?;
            Ok::<(), Box<dyn StdError>>(())
        })
        .await?
    }))
}
