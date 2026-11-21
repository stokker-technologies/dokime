/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Captures Device Files Into Its Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, HashMap};
use std::error;
use std::path::Path;

// Third Party Imports
use dokime_protocol::host::plugin_placement;
use prost::Message;
use tonic::transport::Endpoint;

// Local Imports
use dokime::RecordScope;
use dokime::Termination;
use dokime::Value;
use dokime::record_scope;
use dokime_client_internal::Client;
use dokime_client_internal::EvidenceRecordsListOptions;
use dokime_client_internal::ExecutionsCancelOptions;
use dokime_client_internal::ExecutionsProgressWatchOptions;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::model::ArtifactScope;
use dokime_protocol::model::value;
use dokime_runtime::PluginProcess;
use dokime_runtime::services;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(PartialEq, Message)]
struct Captured {
    #[prost(uint32, tag = "1")]
    files: u32,

    #[prost(uint64, tag = "2")]
    bytes: u64,

    #[prost(uint32, tag = "3")]
    gaps: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn admit(client: &Client, template: &str, device: &Path) -> Result<String, Box<dyn error::Error>> {
    let text = |path: &Path| {
        vec![Value {
            kind: Some(value::Kind::Text(path.display().to_string())),
        }]
    };
    let parameters = HashMap::from([("root".into(), text(device))]);
    let cases = super::support::materialize(client, "capture", template, parameters).await?;
    Ok(super::support::admit(client, cases).await?.remove(0))
}

async fn settle(client: &Client, execution: &str) -> Result<i32, Box<dyn error::Error>> {
    let mut progress = client.executions.progress.watch(execution.to_owned()).await?;
    while let Some(frame) = progress.next().await? {
        if frame.sealed {
            break;
        }
    }
    let outcome = client.executions.get(execution.to_owned()).await?.execution.and_then(|execution| execution.outcome);
    match outcome.and_then(|outcome| outcome.attribution) {
        Some(Attribution::NotEvaluated(outcome)) => Ok(outcome.termination),
        other => Err(format!("Settle Capture Execution: Unexpected Outcome {other:?}").into()),
    }
}

async fn record<T: Message + Default>(client: &Client, execution: &str) -> Result<T, Box<dyn error::Error>> {
    let records = client
        .evidence
        .records
        .list_with(
            String::new(),
            EvidenceRecordsListOptions {
                scope: Some(RecordScope {
                    kind: Some(record_scope::Kind::Execution(execution.to_owned())),
                }),
                versions: Some(Vec::new()),
                between: Some(Default::default()),
                step: Some(String::new()),
                selection: Some(Default::default()),
                ..Default::default()
            },
        )
        .await?;
    let [record] = records.records.as_slice() else {
        return Err(format!("Read Capture Record: Expected One, Found {}", records.records.len()).into());
    };
    Ok(T::decode(record.payload.as_slice())?)
}

// Every Artifact the Execution Holds, by Name, With Its Exact Bytes.
async fn artifacts(client: &Client, execution: &str) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn error::Error>> {
    let scope = ArtifactScope {
        execution_id: Some(execution.to_owned()),
        ..Default::default()
    };
    let mut contents = BTreeMap::new();
    for artifact in client.content.artifacts.list(scope).await?.artifacts {
        let file = client.content.artifacts.get(artifact.artifact_id).await?.file.ok_or("Read Captured Artifact: Missing File")?;
        let mut bytes = Vec::new();
        while (bytes.len() as u64) < file.length {
            bytes.extend(client.files.read(file.clone(), bytes.len() as u64, 1024).await?.data);
        }
        contents.insert(artifact.filename, bytes);
    }
    Ok(contents)
}

fn expected(entries: &[(&str, &[u8])]) -> BTreeMap<String, Vec<u8>> {
    entries.iter().map(|(name, bytes)| ((*name).to_owned(), bytes.to_vec())).collect()
}

/* === Tests === */

#[test]
fn plugin_captures_device_files() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Allow One Running Capture
            let root = super::support::tempdir()?;
            let device = super::support::tempdir()?;
            std::fs::create_dir(device.path().join("nested"))?;
            std::fs::write(device.path().join("device.log"), b"boot\n")?;
            std::fs::write(device.path().join("nested/trace.bin"), [0_u8, 1, 2])?;
            let mut settings = super::support::settings();
            settings.capture.maximum_captures = 1;
            let server = services::serve("127.0.0.1:0".parse()?, root.path(), settings.clone(), super::support::service_settings()).await?;
            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let mut plugin = PluginProcess::start(&super::support::capture_fixture()?, true, &settings.placement).await?;
            let placement = PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
            };
            let created = client.plugins.create("capture", placement, PluginValue { enabled: true }).await?;
            super::support::plugin_state(&client.plugins, created.entry.unwrap(), dokime_protocol::host::PluginState::Ready).await?;

            // Copy the Device Tree Once, Preserving Relative Paths
            let copied = admit(&client, "capture.copy", device.path()).await?;
            assert_eq!(settle(&client, &copied).await?, Termination::Completed as i32);
            assert_eq!(record::<Captured>(&client, &copied).await?, Captured { files: 2, bytes: 8, gaps: 0 });
            assert_eq!(
                artifacts(&client, &copied).await?,
                expected(&[("device/device.log", b"boot\n"), ("device/nested/trace.bin", &[0, 1, 2])])
            );

            // Capture Only What the Device Wrote Between Start and Stop
            let window = admit(&client, "capture.window", device.path()).await?;
            assert_eq!(settle(&client, &window).await?, Termination::Completed as i32);
            assert_eq!(record::<Captured>(&client, &window).await?, Captured { files: 2, bytes: 11, gaps: 0 });
            assert_eq!(artifacts(&client, &window).await?, expected(&[("window/device.log", b"late\n"), ("window/fresh.log", b"fresh\n")]));

            // A Cancelled Execution Still Seals Its Running Capture
            std::fs::remove_file(device.path().join("fresh.log"))?;
            let cancelled = admit(&client, "capture.cancelled", device.path()).await?;
            let mut progress = client
                .executions
                .progress
                .watch_with(cancelled.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    break;
                }
            }
            let options = ExecutionsCancelOptions {
                request_id: Some("cancel-capture".into()),
                ..Default::default()
            };
            client.executions.cancel_with(cancelled.clone(), options).await?;
            assert_eq!(settle(&client, &cancelled).await?, Termination::Cancelled as i32);
            assert_eq!(
                artifacts(&client, &cancelled).await?,
                expected(&[("window/device.log", b"late\n"), ("window/fresh.log", b"fresh\n")])
            );

            // The Sealed Capture Released Its Only Slot for the Next Execution
            std::fs::remove_file(device.path().join("fresh.log"))?;
            let again = admit(&client, "capture.window", device.path()).await?;
            assert_eq!(settle(&client, &again).await?, Termination::Completed as i32);
            assert_eq!(record::<Captured>(&client, &again).await?, Captured { files: 2, bytes: 11, gaps: 0 });

            server.close().await?;
            plugin.close().await?;
            Ok::<(), Box<dyn error::Error>>(())
        })
        .await?
    }))
}
