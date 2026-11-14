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

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{client::HostClient, v1};
use dokime_protocol::client::CyclesCreateRequest;
use dokime_protocol::client::CyclesGetRequest;
use dokime_protocol::client::CyclesListRequest;
use dokime_protocol::client::CyclesSetRequest;
use dokime_protocol::client::ExecutionFilter;
use dokime_protocol::client::ExecutionsDeleteRequest;
use dokime_protocol::client::ExecutionsGetRequest;
use dokime_protocol::client::ExecutionsListRequest;
use dokime_protocol::client::ExecutionsStepsRequest;
use dokime_protocol::client::PromptsListRequest;
use dokime_protocol::client::content::ExecutionFileKind;
use dokime_protocol::client::content::ExecutionFileMetadata;
use dokime_protocol::client::content::ExecutionFileState;
use dokime_protocol::client::content::ExecutionFilesGetRequest;
use dokime_protocol::client::content::execution_files_client::ExecutionFilesClient;
use dokime_protocol::client::cycles_client::CyclesClient;
use dokime_protocol::client::evidence::CapturedRecordSelection;
use dokime_protocol::client::evidence::RecordsListRequest;
use dokime_protocol::client::evidence::SamplesListRequest;
use dokime_protocol::client::evidence::records_client::RecordsClient;
use dokime_protocol::client::evidence::samples_client::SamplesClient;
use dokime_protocol::client::executions::ProgressWatchRequest;
use dokime_protocol::client::executions::progress_client::ProgressClient;
use dokime_protocol::client::executions_client::ExecutionsClient;
use dokime_protocol::client::prompts_client::PromptsClient;
use dokime_runtime_next::PluginProcess;
use dokime_storage::{Execution, Store};
use harmos_protocol::v1::PageRequest;
use std::collections::HashMap;
use std::error::Error;
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

#[tokio::test]
async fn client_browses_sealed_history() -> Result<(), Box<dyn Error>> {
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
        // Retain Registered Chunked Content with Its Execution
        store.register_execution_file(
            "history-file",
            "imported-capture",
            ExecutionFileMetadata {
                relative_path: "capture.txt".into(),
                kind: ExecutionFileKind::Text as i32,
                state: ExecutionFileState::Streaming as i32,
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
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut executions = ExecutionsClient::new(channel.clone());
        let mut files = ExecutionFilesClient::new(channel.clone());
        let mut cycles = CyclesClient::new(channel.clone());
        let retained = executions
            .get(ExecutionsGetRequest {
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
                .get(ExecutionsGetRequest {
                    name: "imported-capture".into(),
                    at: Some(invalid)
                })
                .await
                .is_err()
        );
        let file = files
            .get(ExecutionFilesGetRequest {
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
                .get(ExecutionFilesGetRequest {
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
        let cases = client.templates.materialize("thermal", "thermal.cycle", HashMap::new()).await?;
        let created = cycles
            .create(CyclesCreateRequest {
                origins: cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                request_id: uuid::Uuid::now_v7().to_string(),
                continue_automatically: true,
                ..Default::default()
            })
            .await?
            .into_inner();
        let cycle = created.cycle.unwrap().cycle_id;
        let admitted = executions
            .list(ExecutionsListRequest {
                parent: cycle.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let identifier = admitted.executions[0].summary.as_ref().unwrap().execution_id.clone();
        let mut progress =
            ProgressClient::new(channel.clone()).watch(ProgressWatchRequest { name: identifier.clone(), at: None }).await?.into_inner();
        while let Some(frame) = progress.message().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        assert!(
            executions
                .get(ExecutionsGetRequest { name: identifier.clone(), at: None })
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
        let saved = executions.get(ExecutionsGetRequest { name: identifier.clone(), at: None }).await?.into_inner();
        let steps = executions
            .steps(ExecutionsStepsRequest {
                name: identifier.clone(),
                at: saved.resolved_at.clone(),
            })
            .await?
            .into_inner();
        assert_eq!(steps.steps.len(), 2);
        let mut prompts = PromptsClient::new(channel.clone());
        assert_eq!(
            prompts
                .list(PromptsListRequest {
                    execution_id: Some(identifier.clone()),
                    ..Default::default()
                })
                .await?
                .into_inner()
                .prompts
                .len(),
            1
        );
        let mut records = RecordsClient::new(channel.clone());
        let page = records
            .list(RecordsListRequest {
                selection: Some(CapturedRecordSelection {
                    scope: Some(v1::EvidenceScope {
                        execution_id: Some(identifier.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                page: Some(PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(page.records.len(), 1);
        let mut telemetry = SamplesClient::new(channel);
        let mut request = SamplesListRequest {
            scope: Some(v1::EvidenceScope {
                execution_id: Some(identifier.clone()),
                ..Default::default()
            }),
            streams: vec!["thermal.temperature".into()],
            page: Some(PageRequest { page_size: 1, ..Default::default() }),
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
            .list(CyclesListRequest {
                page: Some(PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(!first.next_page_token.is_empty());
        let current = cycles.get(CyclesGetRequest { name: cycle.clone(), at: None }).await?.into_inner();
        let mut value = current.value.unwrap();
        value.label = uuid::Uuid::now_v7().to_string();
        cycles
            .set(CyclesSetRequest {
                name: cycle.clone(),
                value: Some(value),
                expected_revision: current.revision,
                request_id: "rename".into(),
            })
            .await?;
        let continued = cycles
            .list(CyclesListRequest {
                at: first.resolved_at.clone(),
                page: Some(PageRequest {
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
            .list(ExecutionsListRequest {
                page: Some(PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(first.outcome_counts.as_ref().unwrap().evaluated_count, 1);
        assert_eq!(first.outcome_counts.as_ref().unwrap().declared_count, 1);
        let next = ExecutionsListRequest {
            at: first.resolved_at,
            page: Some(PageRequest {
                page_size: 1,
                page_token: first.next_page_token,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut changed = next.clone();
        changed.filter = Some(ExecutionFilter {
            text: identifier.clone(),
            ..Default::default()
        });
        assert!(executions.list(changed).await.is_err());
        assert_eq!(executions.list(next.clone()).await?.into_inner().executions.len(), 1);

        // A Sealed Deletion Replays Its Original Receipt After Restart and Invalidates Cuts.
        let deletion = ExecutionsDeleteRequest {
            name: "imported-capture".into(),
            request_id: "delete-capture".into(),
        };
        let deleted = executions.delete(deletion.clone()).await?.into_inner();
        assert!(deleted.receipt.as_ref().unwrap().completed);
        assert!(executions.list(next).await.is_err());
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut executions = ExecutionsClient::connect(process.url.clone()).await?;
        assert_eq!(
            executions
                .get(ExecutionsGetRequest {
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
                .get(ExecutionsGetRequest {
                    name: "imported-capture".into(),
                    at: None
                })
                .await
                .is_err()
        );
        process.close().await?;
        assert!(Store::open(root.path())?.content("history-file").is_err());
        plugin.close().await?;
        Ok::<_, Box<dyn Error>>(())
    })
    .await?
}
