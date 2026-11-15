/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Collects Backend Diagnostics

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::Metadata;
use dokime::SecretWritable;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::SecretsDeleteOptions;
use dokime::client::SecretsSetOptions;
use dokime::client::StatusLogsWatchOptions;
use dokime::client::StatusTraceEventsListOptions;
use dokime::client::StatusTraceEventsWatchOptions;
use dokime::client::StatusWatchOptions;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_runtime::PluginProcess;
use dokime_storage::Error as DokimeStorageError;
use dokime_storage::Store;
use harmos_protocol::host::ComponentState;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use harmos_protocol::model::PageRequest;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use tokio::process::Command;
use tonic::transport::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn client_collects_backend_diagnostics() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Read Host Identity and Subscribe Before a Component Appears
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let status = client.status.get().await?;
            let initial = status.status.unwrap();
            assert!(initial.available);
            assert_eq!(initial.identity.as_ref().unwrap().application_version, env!("CARGO_PKG_VERSION"));
            let mut status_watch = client
                .status
                .watch_with(StatusWatchOptions {
                    at: status.resolved_at,
                    ..Default::default()
                })
                .await?;
            assert_eq!(status_watch.next().await?.unwrap().status, Some(initial.clone()));

            // Secret Values Are Registered Before Host Events Can Retain Them
            let secret = "thermal";
            let secrets = Client::from_channel(channel.clone(), super::support::client_settings())?.secrets.clone();
            let generation = secrets.list().await?.generation;
            let saved = secrets
                .set_with(
                    "diagnostics.access",
                    SecretWritable {
                        metadata: Some(Metadata {
                            identifier: "diagnostics.access".into(),
                            label: "Diagnostics Access".into(),
                            ..Default::default()
                        }),
                        value: Some(secret.into()),
                    },
                    generation.to_be_bytes().to_vec(),
                    SecretsSetOptions {
                        request_id: Some("diagnostic-secret".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let mut logs = client.status.logs.watch_with(4096, 16, StatusLogsWatchOptions { at: None, ..Default::default() }).await?;
            assert!(logs.next().await?.unwrap().events.is_empty());
            let mut trace =
                client.status.trace_events.watch_with(4096, 16, StatusTraceEventsWatchOptions { at: None, ..Default::default() }).await?;
            assert!(trace.next().await?.unwrap().events.is_empty());

            // Real Plugin Admission Drives Both Event Histories and Product Health
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement).await?;
            let plugins = Client::from_channel(channel, super::support::client_settings())?.plugins.clone();
            let entry = plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let entry = super::support::plugin_state(&plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            assert!(entry.description.as_ref().unwrap().id.contains(secret));
            loop {
                let frame = logs.next().await?.ok_or("Watch Logs: Closed Before Plugin Admission")?;
                for event in &frame.events {
                    assert!(!event.message.contains(secret));
                    assert!(event.fields.values().all(|value| !value.contains(secret)));
                }
                if frame.events.iter().any(|event| event.fields.contains_key("plugin")) {
                    break;
                }
            }
            let traced = trace.next().await?.ok_or("Watch Trace Events: Closed Before Plugin Admission")?;
            assert!(!traced.events.is_empty());
            assert!(traced.events.iter().flat_map(|event| event.fields.values()).all(|value| !value.contains(secret)));
            loop {
                let changed = status_watch.next().await?.ok_or("Watch Status: Closed Before Plugin Health")?.status.unwrap();
                let admitted = changed.components.len() > initial.components.len();
                let healthy = changed.components.iter().all(|component| component.state == ComponentState::Healthy as i32);
                if admitted && healthy {
                    break;
                }
            }
            drop((logs, trace, status_watch));

            // Continue a Trace Page at Its Issued Boundary
            let first = client
                .status
                .trace_events
                .list_with(StatusTraceEventsListOptions {
                    at: None,
                    page: Some(PageRequest {
                        page_size: 1,
                        max_bytes: 4096,
                        page_token: String::new(),
                    }),
                    ..Default::default()
                })
                .await?;
            assert_eq!(first.events.len(), 1);
            assert!(!first.next_page_token.is_empty());
            let second = client
                .status
                .trace_events
                .list_with(StatusTraceEventsListOptions {
                    at: first.resolved_at.clone(),
                    page: Some(PageRequest {
                        page_size: 1,
                        max_bytes: 4096,
                        page_token: first.next_page_token,
                    }),
                    ..Default::default()
                })
                .await?;
            assert_eq!(second.resolved_at, first.resolved_at);
            assert!(second.events[0].sequence > first.events[0].sequence);

            // Resume Bounded Archive Ranges and Verify Their Shared Representation
            let mut name = String::new();
            let mut bytes = Vec::new();
            let digest = loop {
                let range = client.status.diagnostics.get(name.clone(), bytes.len() as u64, 1024).await?;
                if name.is_empty() {
                    name = range.name.clone();
                }
                assert_eq!(range.name, name);
                bytes.extend(range.data);
                if bytes.len() as u64 == range.total_length {
                    break range.sha256;
                }
            };
            use sha2::Digest;
            assert_eq!(digest, sha2::Sha256::digest(&bytes).to_vec());
            let mut archive = tar::Archive::new(bytes.as_slice());
            let mut paths = Vec::new();
            for entry in archive.entries()? {
                use std::io::Read;
                let mut entry = entry?;
                let path = entry.path()?.into_owned();
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                assert!(!content.contains(secret));
                if path == Path::new("dokime/protocol.json") {
                    let protocol: serde_json::Value = serde_json::from_str(&content)?;
                    assert_eq!(protocol["descriptor_sha256"], serde_json::json!(sha2::Sha256::digest(FILE_DESCRIPTOR_SET).to_vec()));
                }
                paths.push(path);
            }
            assert!(paths.contains(&PathBuf::from("dokime/protocol.json")));
            secrets
                .delete_with(
                    vec!["diagnostics.access".into()],
                    saved.revision,
                    SecretsDeleteOptions {
                        request_id: Some("cleanup".into()),
                        ..Default::default()
                    },
                )
                .await?;
            process.close().await?;
            plugin.close().await?;

            // Refuse an Incompatible Root While Preserving Every Existing Byte
            let incompatible = super::support::tempdir()?;
            std::fs::write(incompatible.path().join("existing-data"), b"preserve")?;
            let output = Command::from(dokime_runtime::process::command(super::support::host_fixture()?))
                .args(["--endpoint", "127.0.0.1:0", "--storage"])
                .arg(incompatible.path())
                .output()
                .await?;
            assert!(!output.status.success());
            assert_eq!(std::fs::read(incompatible.path().join("existing-data"))?, b"preserve");
            assert!(matches!(Store::open(incompatible.path()), Err(DokimeStorageError::Incompatible(_))));
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
