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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
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
                    payload: dokime_protocol::context::content::PublishArtifact {
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
            dokime_protocol::client::content::ExecutionFileMetadata {
                relative_path: "capture.txt".into(),
                kind: dokime_protocol::client::content::ExecutionFileKind::Text as i32,
                state: dokime_protocol::client::content::ExecutionFileState::Streaming as i32,
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
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut executions = dokime_protocol::client::executions_client::ExecutionsClient::new(channel.clone());
        let mut files = dokime_protocol::client::content::execution_files_client::ExecutionFilesClient::new(channel.clone());
        let mut cycles = dokime_protocol::client::cycles_client::CyclesClient::new(channel.clone());
        let retained = executions
            .get(dokime_protocol::client::ExecutionsGetRequest {
                name: "imported-capture".into(),
                at: None,
            })
            .await?
            .into_inner();
        assert!(!retained.execution.as_ref().unwrap().seal.is_empty());
        let mut invalid = retained.resolved_at.clone().unwrap();
        invalid.history.as_mut().unwrap().store = "foreign".into();
        assert!(
            executions
                .get(dokime_protocol::client::ExecutionsGetRequest {
                    name: "imported-capture".into(),
                    at: Some(invalid)
                })
                .await
                .is_err()
        );
        let file = files
            .get(dokime_protocol::client::content::ExecutionFilesGetRequest {
                parent: "imported-capture".into(),
                name: "capture.txt".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let reference = file.file.unwrap();
        assert_eq!(reference.length, 6);
        assert_eq!(Store::open(root.path())?.content_window(&reference.metadata.unwrap().id, 2, 3)?, b"cde");
        assert!(
            files
                .get(dokime_protocol::client::content::ExecutionFilesGetRequest {
                    parent: "imported-capture".into(),
                    name: "../capture.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );

        // Complete a Plugin Run and Inspect Each Retained Resource at Its Own Cut.
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        client.plugins.add(plugin.endpoint()).await?;
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::new()).await?;
        let created = cycles
            .create(dokime_protocol::client::CyclesCreateRequest {
                origins: cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                request_id: uuid::Uuid::now_v7().to_string(),
                continue_automatically: true,
                ..Default::default()
            })
            .await?
            .into_inner();
        let cycle = created.cycle.unwrap().cycle_id;
        let admitted = executions
            .list(dokime_protocol::client::ExecutionsListRequest {
                parent: cycle.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let identifier = admitted.executions[0].summary.as_ref().unwrap().execution_id.clone();
        let mut progress = dokime_protocol::client::executions::progress_client::ProgressClient::new(channel.clone())
            .watch(dokime_protocol::client::executions::ProgressWatchRequest { name: identifier.clone(), at: None })
            .await?
            .into_inner();
        while let Some(frame) = progress.message().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        assert!(
            executions
                .get(dokime_protocol::client::ExecutionsGetRequest { name: identifier.clone(), at: None })
                .await?
                .into_inner()
                .execution
                .unwrap()
                .seal
                .is_empty()
        );
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        client.prompts.respond(prompt, "yes", true).await?;
        while progress.message().await?.is_some() {}
        let saved =
            executions.get(dokime_protocol::client::ExecutionsGetRequest { name: identifier.clone(), at: None }).await?.into_inner();
        let steps = executions
            .steps(dokime_protocol::client::ExecutionsStepsRequest {
                name: identifier.clone(),
                at: saved.resolved_at.clone(),
            })
            .await?
            .into_inner();
        assert_eq!(steps.steps.len(), 2);
        let mut prompts = dokime_protocol::client::prompts_client::PromptsClient::new(channel.clone());
        assert_eq!(
            prompts
                .list(dokime_protocol::client::PromptsListRequest {
                    execution_id: Some(identifier.clone()),
                    ..Default::default()
                })
                .await?
                .into_inner()
                .prompts
                .len(),
            1
        );
        let mut records = dokime_protocol::client::evidence::records_client::RecordsClient::new(channel.clone());
        let page = records
            .list(dokime_protocol::client::evidence::RecordsListRequest {
                selection: Some(dokime_protocol::client::evidence::CapturedRecordSelection {
                    scope: Some(v1::EvidenceScope {
                        execution_id: Some(identifier.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(page.records.len(), 1);
        let mut telemetry = dokime_protocol::client::evidence::samples_client::SamplesClient::new(channel);
        let mut request = dokime_protocol::client::evidence::SamplesListRequest {
            scope: Some(v1::EvidenceScope {
                execution_id: Some(identifier.clone()),
                ..Default::default()
            }),
            streams: vec!["thermal.temperature".into()],
            page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
            ..Default::default()
        };
        let mut samples = Vec::new();
        loop {
            let page = telemetry.list(request.clone()).await?.into_inner();
            samples.extend(page.samples);
            if page.next_page_token.is_empty() {
                break;
            }
            request.at = page.resolved_at;
            request.page.as_mut().unwrap().page_token = page.next_page_token;
        }
        assert_eq!(samples.len(), 3);

        // Cycle and Execution Pages Retain Their Selection Across Later Presentation Edits.
        let first = cycles
            .list(dokime_protocol::client::CyclesListRequest {
                page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(!first.next_page_token.is_empty());
        let current = cycles.get(dokime_protocol::client::CyclesGetRequest { name: cycle.clone(), at: None }).await?.into_inner();
        let mut value = current.value.unwrap();
        value.label = uuid::Uuid::now_v7().to_string();
        cycles
            .set(dokime_protocol::client::CyclesSetRequest {
                name: cycle.clone(),
                value: Some(value),
                expected_revision: current.revision,
                request_id: "rename".into(),
            })
            .await?;
        let continued = cycles
            .list(dokime_protocol::client::CyclesListRequest {
                at: first.resolved_at.clone(),
                page: Some(harmos_protocol::v1::PageRequest {
                    page_size: 1,
                    page_token: first.next_page_token,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(continued.cycles.len(), 1);
        let first = executions
            .list(dokime_protocol::client::ExecutionsListRequest {
                page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(first.outcome_counts.as_ref().unwrap().evaluated_count, 1);
        assert_eq!(first.outcome_counts.as_ref().unwrap().declared_count, 1);
        let next = dokime_protocol::client::ExecutionsListRequest {
            at: first.resolved_at,
            page: Some(harmos_protocol::v1::PageRequest {
                page_size: 1,
                page_token: first.next_page_token,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut changed = next.clone();
        changed.filter = Some(dokime_protocol::client::ExecutionFilter {
            text: identifier.clone(),
            ..Default::default()
        });
        assert!(executions.list(changed).await.is_err());
        assert_eq!(executions.list(next.clone()).await?.into_inner().executions.len(), 1);

        // A Sealed Deletion Replays Its Original Receipt After Restart and Invalidates Cuts.
        let deletion = dokime_protocol::client::ExecutionsDeleteRequest {
            name: "imported-capture".into(),
            request_id: "delete-capture".into(),
        };
        let deleted = executions.delete(deletion.clone()).await?.into_inner();
        assert!(deleted.receipt.as_ref().unwrap().completed);
        assert!(executions.list(next).await.is_err());
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut executions = dokime_protocol::client::executions_client::ExecutionsClient::connect(process.url.clone()).await?;
        assert_eq!(
            executions
                .get(dokime_protocol::client::ExecutionsGetRequest {
                    name: identifier,
                    at: saved.resolved_at.clone()
                })
                .await?
                .into_inner(),
            saved
        );
        assert_eq!(executions.delete(deletion).await?.into_inner(), deleted);
        assert!(
            executions
                .get(dokime_protocol::client::ExecutionsGetRequest {
                    name: "imported-capture".into(),
                    at: None
                })
                .await
                .is_err()
        );
        process.close().await?;
        assert!(Store::open(root.path())?.content("history-file").is_err());
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
