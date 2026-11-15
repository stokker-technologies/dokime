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
use dokime::Client;
use dokime::EvidenceCapturedRecordSelection;
use dokime::EvidenceScope;
use dokime::Import;
use dokime::MapValue;
use dokime::Metadata;
use dokime::Origin;
use dokime::Termination;
use dokime::client::ContentExecutionFileKind;
use dokime::client::ContentExecutionFileMetadata;
use dokime::client::ContentExecutionFileState;
use dokime::client::CyclesCreateOptions;
use dokime::client::CyclesGetOptions;
use dokime::client::CyclesListOptions;
use dokime::client::CyclesSetOptions;
use dokime::client::DeclaredOutcome;
use dokime::client::EvidenceRecordsListOptions;
use dokime::client::EvidenceSamplesListOptions;
use dokime::client::ExecutionFilter;
use dokime::client::ExecutionOutcome;
use dokime::client::ExecutionsDeleteOptions;
use dokime::client::ExecutionsGetOptions;
use dokime::client::ExecutionsListOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::ExecutionsStepsOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::PromptsListOptions;
use dokime::client::ResultStatus;
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::execution_outcome::Attribution as ExecutionOutcomeAttribution;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_runtime::PluginProcess;
use dokime_storage::Execution;
use dokime_storage::Store;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use harmos_protocol::model::PageRequest;
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

#[test]
fn client_browses_sealed_history() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare Imported Artifact Evidence Without Inventing a Plugin Run
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            store.create(&Execution {
                identifier: "imported-capture".into(),
                cycle: "imported-cycle".into(),
                origin: Some(Origin {
                    kind: Some(OriginKind::Import(Import {
                        metadata: Some(Metadata {
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
                ContentExecutionFileMetadata {
                    relative_path: "capture.txt".into(),
                    kind: ContentExecutionFileKind::Text as i32,
                    state: ContentExecutionFileState::Streaming as i32,
                    ..Default::default()
                },
            )?;
            store.append_content("history-file", 0, b"abcdef")?;
            store.finish_execution_file("history-file", false)?;
            store.close_streams("imported-capture")?;
            store.close(
                "imported-capture",
                Termination::Completed,
                ExecutionOutcome {
                    attribution: Some(ExecutionOutcomeAttribution::Declared(DeclaredOutcome {
                        status: ResultStatus::Passed as i32,
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
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let executions = Client::from_channel(channel.clone(), super::support::client_settings())?.executions.clone();
            let files = Client::from_channel(channel.clone(), super::support::client_settings())?.content.execution_files.clone();
            let cycles = Client::from_channel(channel.clone(), super::support::client_settings())?.cycles.clone();
            let retained = executions.get_with("imported-capture", ExecutionsGetOptions { at: None, ..Default::default() }).await?;
            assert!(!retained.execution.as_ref().unwrap().seal.is_empty());
            let mut invalid = retained.resolved_at.clone().unwrap();
            invalid.history.as_mut().unwrap().store = "foreign".into();
            assert!(
                executions.get_with("imported-capture", ExecutionsGetOptions { at: Some(invalid), ..Default::default() }).await.is_err()
            );
            let file = files.get("imported-capture", "capture.txt").await?;
            let reference = file.file.unwrap();
            assert_eq!(reference.length, 6);
            assert_eq!(Store::open(root.path())?.content_window(&reference.metadata.unwrap().id, 2, 3)?, b"cde");
            assert!(files.get("imported-capture", "../capture.txt").await.is_err());

            // Complete a Plugin Run and Inspect Each Retained Resource at Its Own Cut.
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let entry = client
                .plugins
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
            super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            let cases = super::support::materialize(&client, "thermal", "thermal.cycle", HashMap::new()).await?;
            let created = cycles
                .create_with(
                    cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                    Default::default(),
                    true,
                    String::new(),
                    CyclesCreateOptions {
                        request_id: Some(uuid::Uuid::now_v7().to_string()),
                        ..Default::default()
                    },
                )
                .await?;
            let cycle = created.cycle.unwrap().cycle_id;
            let admitted = executions.list(cycle.clone(), Default::default(), Default::default(), Default::default()).await?;
            let identifier = admitted.executions[0].summary.as_ref().unwrap().execution_id.clone();
            let mut progress = Client::from_channel(channel.clone(), super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(identifier.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    break;
                }
            }
            assert!(
                executions
                    .get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() })
                    .await?
                    .execution
                    .unwrap()
                    .seal
                    .is_empty()
            );
            let prompt = client.prompts.list(identifier.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
            client.prompts.answers.create(prompt, "yes", MapValue::default(), true).await?;
            while progress.next().await?.is_some() {}
            let saved = executions.get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await?;
            let steps = executions
                .steps_with(
                    identifier.clone(),
                    ExecutionsStepsOptions {
                        at: saved.resolved_at.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(steps.steps.len(), 2);
            let prompts = Client::from_channel(channel.clone(), super::support::client_settings())?.prompts.clone();
            assert_eq!(
                prompts
                    .list_with(
                        String::new(),
                        Default::default(),
                        PromptsListOptions {
                            execution_id: Some(identifier.clone()),
                            ..Default::default()
                        }
                    )
                    .await?
                    .prompts
                    .len(),
                1
            );
            let records = Client::from_channel(channel.clone(), super::support::client_settings())?.evidence.records.clone();
            let page = records
                .list_with(
                    Default::default(),
                    String::new(),
                    EvidenceRecordsListOptions {
                        versions: Some(Default::default()),
                        between: Some(Default::default()),
                        step: Some(String::new()),
                        selection: Some(EvidenceCapturedRecordSelection {
                            scope: Some(EvidenceScope {
                                execution_id: Some(identifier.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        page: Some(PageRequest { page_size: 1, ..Default::default() }),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(page.records.len(), 1);
            let telemetry = Client::from_channel(channel, super::support::client_settings())?.evidence.samples.clone();
            let mut request = SamplesListRequest {
                scope: Some(EvidenceScope {
                    execution_id: Some(identifier.clone()),
                    ..Default::default()
                }),
                streams: vec!["thermal.temperature".into()],
                page: Some(PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            };
            let mut samples = Vec::new();
            loop {
                let page = telemetry
                    .list_with(
                        ((request.clone()).scope).unwrap_or_default(),
                        (request.clone()).streams,
                        EvidenceSamplesListOptions {
                            versions: Some((request.clone()).versions),
                            fields: Some((request.clone()).fields),
                            sources: Some((request.clone()).sources),
                            signals: Some((request.clone()).signals),
                            between: Some(((request.clone()).between).unwrap_or_default()),
                            bucket: Some(((request.clone()).bucket).unwrap_or_default()),
                            latest: Some((request.clone()).latest),
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        },
                    )
                    .await?;
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
                .list_with(
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    CyclesListOptions {
                        page: Some(PageRequest { page_size: 1, ..Default::default() }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!first.next_page_token.is_empty());
            let current = cycles.get_with(cycle.clone(), CyclesGetOptions { at: None, ..Default::default() }).await?;
            let mut value = current.value.unwrap();
            value.label = uuid::Uuid::now_v7().to_string();
            cycles
                .set_with(
                    cycle.clone(),
                    value,
                    current.revision,
                    CyclesSetOptions {
                        request_id: Some("rename".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let continued = cycles
                .list_with(
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    CyclesListOptions {
                        at: first.resolved_at.clone(),
                        page: Some(PageRequest {
                            page_size: 1,
                            page_token: first.next_page_token,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(continued.cycles.len(), 1);
            let first = executions
                .list_with(
                    String::new(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    ExecutionsListOptions {
                        page: Some(PageRequest { page_size: 1, ..Default::default() }),
                        ..Default::default()
                    },
                )
                .await?;
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
            assert!(
                executions
                    .list_with(
                        (changed.clone()).parent,
                        ((changed.clone()).selection).unwrap_or_default(),
                        ((changed.clone()).filter).unwrap_or_default(),
                        ((changed.clone()).order).unwrap_or_default(),
                        ExecutionsListOptions {
                            at: (changed.clone()).at,
                            page: (changed.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert_eq!(
                executions
                    .list_with(
                        (next.clone()).parent,
                        ((next.clone()).selection).unwrap_or_default(),
                        ((next.clone()).filter).unwrap_or_default(),
                        ((next.clone()).order).unwrap_or_default(),
                        ExecutionsListOptions {
                            at: (next.clone()).at,
                            page: (next.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await?
                    .executions
                    .len(),
                1
            );

            // A Sealed Deletion Replays Its Original Receipt After Restart and Invalidates Cuts.
            let deletion = ExecutionsDeleteRequest {
                name: "imported-capture".into(),
                request_id: "delete-capture".into(),
            };
            let deleted = executions
                .delete_with(
                    (deletion.clone()).name,
                    ExecutionsDeleteOptions {
                        request_id: Some((deletion.clone()).request_id),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(deleted.receipt.as_ref().unwrap().completed);
            assert!(
                executions
                    .list_with(
                        (next.clone()).parent,
                        ((next.clone()).selection).unwrap_or_default(),
                        ((next.clone()).filter).unwrap_or_default(),
                        ((next.clone()).order).unwrap_or_default(),
                        ExecutionsListOptions {
                            at: (next.clone()).at,
                            page: (next.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let executions =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .executions
                    .clone();
            assert_eq!(
                executions
                    .get_with(
                        identifier,
                        ExecutionsGetOptions {
                            at: saved.resolved_at.clone(),
                            ..Default::default()
                        }
                    )
                    .await?,
                saved
            );
            assert_eq!(
                executions
                    .delete_with(
                        (deletion.clone()).name,
                        ExecutionsDeleteOptions {
                            request_id: Some((deletion.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                deleted
            );
            assert!(executions.get_with("imported-capture", ExecutionsGetOptions { at: None, ..Default::default() }).await.is_err());
            process.close().await?;
            assert!(Store::open(root.path())?.content("history-file").is_err());
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    })
}
