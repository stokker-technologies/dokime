/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Browses Sealed History

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{client::HostClient, v1};
use dokime_runtime_next::PluginProcess;
use dokime_storage::{Evidence, Execution, Store};

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

#[tokio::test]
async fn client_browses_sealed_history() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Imported Artifact Evidence Without Inventing a Plugin Run
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&Execution {
            identifier: "imported-capture".into(),
            cycle: "imported-cycle".into(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import {
                    metadata: Some(v1::Metadata {
                        identifier: "capture".into(),
                        label: "Captured File".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })?;
        for (sequence, data) in [b"abc".as_slice(), b"def".as_slice()].into_iter().enumerate() {
            store.append(
                "imported-capture",
                Evidence {
                    kind: "artifact".into(),
                    payload: v1::PublishArtifact {
                        transfer_id: "capture".into(),
                        name: "capture.txt".into(),
                        mime: "text/plain".into(),
                        sequence: sequence as u64,
                        data: data.to_vec(),
                        last: sequence == 1,
                        ..Default::default()
                    }
                    .encode_to_vec(),
                    ..Default::default()
                },
            )?;
        }
        // Retain Chunked Content Alongside the History Artifact Evidence
        store.register_execution_file(
            "history-file",
            "imported-capture",
            v1::ExecutionFileMetadata {
                relative_path: "capture.txt".into(),
                kind: v1::ExecutionFileKind::Text as i32,
                state: v1::ExecutionFileState::Streaming as i32,
                ..Default::default()
            },
        )?;
        store.append_content("history-file", 0, b"abcdef")?;
        store.finish_execution_file("history-file", false)?;
        store.close_streams("imported-capture")?;
        store.close(
            "imported-capture",
            v1::Termination::Completed,
            v1::ExecutionOutcome {
                attribution: Some(v1::execution_outcome::Attribution::Declared(v1::DeclaredOutcome {
                    status: v1::ResultStatus::Passed as i32,
                    importer: "operator".into(),
                    ..Default::default()
                })),
            },
        )?;
        store.seal("imported-capture")?;
        drop(store);

        // Read an Existing Saved Seal and Its Bounded Artifact Window
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone());
        let mut history = v1::history_client::HistoryClient::new(channel.clone());
        let checkpoint = history
            .get_execution_checkpoint(v1::HistoryGetExecutionCheckpointRequest {
                execution_identifier: "imported-capture".into(),
            })
            .await?
            .into_inner();
        let mut forged = checkpoint.clone();
        forged.version[0] ^= 1;
        assert_eq!(
            history.get_checkpoint_context(v1::HistoryGetCheckpointContextRequest { checkpoint: Some(forged) }).await.unwrap_err().code(),
            tonic::Code::FailedPrecondition
        );
        let evidence = history
            .get_checkpoint_evidence(v1::HistoryGetCheckpointEvidenceRequest {
                checkpoint: Some(checkpoint.clone()),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(evidence.artifacts.len(), 1);
        let artifact_id = evidence.artifacts[0].artifact_id.clone();
        let window = history
            .read_checkpoint_output(v1::HistoryReadCheckpointOutputRequest {
                checkpoint: Some(checkpoint.clone()),
                relative_path: "capture.txt".into(),
                offset: 2,
                max_bytes: 3,
            })
            .await?
            .into_inner();
        assert_eq!(window.content.unwrap().data, b"cde");
        assert!(
            history
                .read_checkpoint_output(v1::HistoryReadCheckpointOutputRequest {
                    checkpoint: Some(checkpoint.clone()),
                    relative_path: "../capture.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let mut download = history
            .download_checkpoint_artifact(v1::HistoryDownloadCheckpointArtifactRequest {
                checkpoint: Some(checkpoint.clone()),
                artifact_id,
            })
            .await?
            .into_inner();
        let mut bytes = Vec::new();
        let mut terminal = false;
        while let Some(frame) = download.message().await? {
            match frame.value.unwrap() {
                v1::history_download_checkpoint_artifact_output::Value::Item(frame) => match frame.payload.unwrap().kind.unwrap() {
                    v1::artifact_frame_payload::Kind::Chunk(chunk) => {
                        assert_eq!(chunk.offset, bytes.len() as u64);
                        bytes.extend(chunk.data);
                    }
                    v1::artifact_frame_payload::Kind::Complete(complete) => assert_eq!(complete.size_bytes, 6),
                    _ => {}
                },
                v1::history_download_checkpoint_artifact_output::Value::Response(_) => terminal = true,
            }
        }
        assert!(terminal);
        assert_eq!(bytes, b"abcdef");

        // Complete a Real Thermal Run and Inspect Its Frozen History
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        client.plugins.add(plugin.endpoint()).await?;
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::new()).await?;
        let cycle = client.cycles.start(cases).await?;
        let identifier = cycle.executions[0].clone();
        let mut progress = cycle.watch();
        while let Some(frame) = progress.next().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        assert_eq!(
            history
                .get_execution_checkpoint(v1::HistoryGetExecutionCheckpointRequest { execution_identifier: identifier.clone() })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::NotFound
        );
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt_id.clone();
        client.prompts.respond(prompt, "yes", true).await?;
        while progress.next().await?.is_some() {}
        let saved = history
            .get_execution_checkpoint(v1::HistoryGetExecutionCheckpointRequest { execution_identifier: identifier.clone() })
            .await?
            .into_inner();
        let document = history
            .get_checkpoint_document(v1::HistoryGetCheckpointDocumentRequest { checkpoint: Some(saved.clone()) })
            .await?
            .into_inner();
        assert!(document.acquisition_finished);
        let document = document.document.unwrap().execution.unwrap();
        assert_eq!(document.origin, client.executions.get(identifier.clone()).await?.origin);
        assert_eq!(document.steps.len(), 2);
        let context =
            history.get_checkpoint_context(v1::HistoryGetCheckpointContextRequest { checkpoint: Some(saved.clone()) }).await?.into_inner();
        assert_eq!(context.prompts.len(), 1);
        let records = history
            .list_checkpoint_records(v1::HistoryListCheckpointRecordsRequest {
                checkpoint: Some(saved.clone()),
                limit: 1,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(records.records.len(), 1);
        let mut after = String::new();
        let mut samples = Vec::new();
        loop {
            let page = history
                .list_checkpoint_telemetry(v1::HistoryListCheckpointTelemetryRequest {
                    checkpoint: Some(saved.clone()),
                    table: "thermal.temperature".into(),
                    limit: 1,
                    after,
                })
                .await?
                .into_inner();
            samples.extend(page.rows);
            let Some(next) = page.next_after else {
                break;
            };
            after = next;
        }
        assert_eq!(samples.len(), 3);

        // Keep the Captured Storage Frontier Across Later Cycle Changes
        let first = history.browse_cycles(v1::HistoryBrowseCyclesRequest { limit: 1, ..Default::default() }).await?.into_inner();
        v1::execution_client::ExecutionClient::new(channel.clone())
            .rename_cycle(v1::ExecutionRenameCycleRequest {
                cycle_id: cycle.identifier.clone(),
                label: "Later Label".into(),
                ..Default::default()
            })
            .await?;
        let second = history
            .browse_cycles(v1::HistoryBrowseCyclesRequest {
                limit: 1,
                at: first.at,
                after_sequence: first.next_after_sequence.unwrap(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(second.entries.len(), 1);
        assert!(second.entries[0].metadata.as_ref().is_none_or(|metadata| metadata.label != "Later Label"));
        let first = history.browse_executions(v1::HistoryBrowseExecutionsRequest { limit: 1, ..Default::default() }).await?.into_inner();
        assert_eq!(first.cycles.iter().map(|cycle| cycle.execution_count).sum::<u64>(), 2);
        assert_eq!(first.cycles.iter().map(|cycle| cycle.passed_count).sum::<u64>(), 1);
        assert_eq!(first.cycles.iter().map(|cycle| cycle.declared_count).sum::<u64>(), 1);
        let next = first.next_after_execution_id.clone().unwrap();
        let changed = history
            .browse_executions(v1::HistoryBrowseExecutionsRequest {
                limit: 1,
                after_execution_id: next.clone(),
                cursor: first.cursor.clone(),
                filter: Some(v1::ExecutionFilter {
                    text: identifier.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert_eq!(changed.code(), tonic::Code::FailedPrecondition);
        assert_eq!(
            history
                .browse_executions(v1::HistoryBrowseExecutionsRequest {
                    limit: 1,
                    after_execution_id: next,
                    cursor: first.cursor,
                    ..Default::default()
                })
                .await?
                .into_inner()
                .entries
                .len(),
            1
        );

        // Delete Only the Inspected Import and Resolve the Same Deletion After Restart
        let deletion = v1::HistoryDeleteExecutionRequest {
            checkpoint: Some(checkpoint),
            request_identifier: "delete-capture".into(),
        };
        assert!(history.delete_execution(deletion.clone()).await?.into_inner().local_complete);
        process.close().await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut history = v1::history_client::HistoryClient::new(channel);
        assert_eq!(
            history
                .get_execution_checkpoint(v1::HistoryGetExecutionCheckpointRequest { execution_identifier: identifier })
                .await?
                .into_inner(),
            saved
        );
        assert!(history.delete_execution(deletion).await?.into_inner().local_complete);
        assert_eq!(
            history
                .get_execution_checkpoint(v1::HistoryGetExecutionCheckpointRequest {
                    execution_identifier: "imported-capture".into()
                })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::NotFound
        );
        process.close().await?;
        // Deleted Execution Content Does Not Survive the History Deletion
        let store = Store::open(root.path())?;
        assert!(matches!(store.content("history-file"), Err(dokime_storage::Error::NotFound(_))));
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
