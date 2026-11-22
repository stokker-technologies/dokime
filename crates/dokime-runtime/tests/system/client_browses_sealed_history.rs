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
use std::collections::HashMap;
use std::error::Error;

// Third Party Imports
use dokime_protocol::host::plugin_placement;
use harmos_protocol::model::PageRequest;
use prost_types::Timestamp;
use tokio::time;
use tonic::transport::Endpoint;
use uuid::Uuid;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support::CredentialStore;
use dokime::CapturedRecordSelection;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::Metadata;
use dokime::model::Window;
use dokime_client_internal::Client;
use dokime_client_internal::CyclesCreateOptions;
use dokime_client_internal::CyclesGetOptions;
use dokime_client_internal::CyclesListOptions;
use dokime_client_internal::CyclesSetOptions;
use dokime_client_internal::EvidenceRecordsListOptions;
use dokime_client_internal::EvidenceSamplesListOptions;
use dokime_client_internal::ExecutionFilter;
use dokime_client_internal::ExecutionsDeleteOptions;
use dokime_client_internal::ExecutionsGetOptions;
use dokime_client_internal::ExecutionsListOptions;
use dokime_client_internal::ExecutionsProgressWatchOptions;
use dokime_client_internal::ExecutionsStepsOptions;
use dokime_client_internal::PromptsListOptions;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::PluginState;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_runtime::Host;
use dokime_runtime::PluginProcess;

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
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare an Isolated Storage Root for Sealed History
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = CredentialStore::start(credential_root.path()).await?;
            // Complete a Real Execution Through the In-Process Host
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("execution", &super::support::execution_fixture()?, &[]).await?;
            let origin = host.materialize(&entry, "execution.execution.capture", super::support::parameters(23.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let retained_id = cycle.executions[0].clone();
            let prompt = host.next_prompt(&retained_id).await?;
            super::support::answer(&host, &retained_id, prompt)?;
            cycle.wait().await?;
            host.shutdown().await?;
            drop(host);

            // Read the Saved Seal at Its Retained Position
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let executions = &client.executions;
            let cycles = &client.cycles;
            let retained = executions.get(retained_id.clone()).await?;
            assert!(!retained.execution.as_ref().unwrap().seal.is_empty());
            let mut invalid = retained.resolved_at.clone().unwrap();
            invalid.history.as_mut().unwrap().store = "foreign".into();
            assert!(
                executions.get_with(retained_id.clone(), ExecutionsGetOptions { at: Some(invalid), ..Default::default() }).await.is_err()
            );

            // Complete a Plugin Run and Inspect Each Retained Resource at Its Own Cut.
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let entry = client
                .plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            super::support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
            let cases = super::support::materialize(&client, "thermal.thermal", "thermal.thermal.cycle", HashMap::new()).await?;
            let created = cycles
                .create_with(
                    cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                    Metadata {
                        label: "History Run".into(),
                        ..Default::default()
                    },
                    true,
                    String::new(),
                    CyclesCreateOptions {
                        request_id: Some(Uuid::now_v7().to_string()),
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
            client.prompts.answers.create(prompt, "0", MapValue::default(), true).await?;
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
                    String::new(),
                    EvidenceRecordsListOptions {
                        scope: Some(Default::default()),
                        versions: Some(Default::default()),
                        between: Some(Default::default()),
                        step: Some(String::new()),
                        selection: Some(CapturedRecordSelection {
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
                streams: vec!["thermal.thermal.cycle.temperature".into()],
                between: Some(Window {
                    start: Some(Timestamp { seconds: 1, nanos: 0 }),
                    end: Some(Timestamp { seconds: 2, nanos: 0 }),
                }),
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
            assert_eq!(samples.len(), 2, "Sample the Hold at 1 s and 2 s: {samples:?}");

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
            value.label = Uuid::now_v7().to_string();
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
            assert_eq!(first.outcome_counts.as_ref().unwrap().evaluated_count, 2);
            assert_eq!(first.outcome_counts.as_ref().unwrap().declared_count, 0);
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
                name: retained_id.clone(),
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
            assert!(executions.get_with(retained_id.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await.is_err());
            process.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
