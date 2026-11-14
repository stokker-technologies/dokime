/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Desktop Host Connection Journeys

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error as StdError, num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use prost::Message;
use sha2::{Digest, Sha256};
use tokio_stream::StreamExt;

use dokime_gui::connection::host::{Connection, ConnectionSettings};
use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
use dokime_protocol::{client, v1};
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings, PluginProcess, services};
use dokime_sdk::client::{ClientSettings, FrozenSchemaCache, StorageTransferSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn settings() -> ConnectionSettings {
    ConnectionSettings {
        host: HostSettings {
            clock: Arc::new(ControlledClock::new(0)),
            placement: PlacementSettings {
                connect_timeout: Duration::from_secs(30),
                reconnect_delay: Duration::from_millis(25),
                stop_grace: Duration::from_secs(30),
                stream_window: NonZeroUsize::new(8).unwrap(),
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            },
        },
        services: services::ClientSettings {
            read_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(30),
            watch_timeout: Duration::from_secs(3600),
            max_cases: 1000,
            page_items: 1000,
            message_bytes: 4 * 1024 * 1024,
            stream_buffer: 8,
        },
        client: ClientSettings {
            transfer: StorageTransferSettings { chunk_bytes: Some(64 * 1024) },
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            input_items: NonZeroUsize::new(8).unwrap(),
            input_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        connect_timeout: Duration::from_secs(30),
    }
}

/* === Tests === */

#[test]
fn thermal_execution_prompts_evidence_and_report() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| dokime_gui::test_support::fixture("dokime-rust-thermal"));
        let configuration = settings();
        let mut plugin = PluginProcess::start(&executable, true, &configuration.host.placement).await?;
        let root = dokime_gui::test_support::tempdir()?;
        let mut owner = Connection::new(configuration)?;
        let result = async {
            // Admit the Prepared Plugin and Read Its Top-Level Suite Resource.
            let session = owner.start(root.path()).await?;
            let overview: client::backend::OverviewGetResponse =
                session.unary("/dokime.client.v1.backend.Overview/Get", client::backend::OverviewGetRequest::default()).await?;
            let process_epoch = overview.overview.ok_or("Read Backend: Missing Overview")?.session_id;
            let admitted: client::Plugin =
                session.unary("/dokime.client.v1.Plugins/Add", client::AddRemotePlugin { url: plugin.endpoint().into() }).await?;
            assert!(!admitted.entry.is_empty());
            let suites: client::SuitesListResponse =
                session.unary("/dokime.client.v1.Suites/List", client::SuitesListRequest::default()).await?;
            let suite = suites.values.first().ok_or("Read Suite: Missing Prepared Fixture")?;
            let suite_id = suite.metadata.as_ref().ok_or("Read Suite: Missing Identity")?.identifier.clone();
            let suite: client::SuitesGetResponse = session
                .unary(
                    "/dokime.client.v1.Suites/Get",
                    client::SuitesGetRequest {
                        name: suite_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let suite = suite.value.ok_or("Read Suite: Missing Definition")?;
            let template = suite.templates.first().and_then(|template| template.metadata.as_ref()).ok_or("Read Suite: Missing Template")?;
            let plan: client::CasesCreateResponse = session
                .unary(
                    "/dokime.client.v1.Cases/Create",
                    client::CasesCreateRequest {
                        plan: Some(client::SelectionPlan {
                            selections: vec![client::RegisteredSelection {
                                suite_id,
                                template_id: template.identifier.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        validate_only: true,
                        request_id: "desktop-preview".into(),
                    },
                )
                .await?;
            let origins =
                plan.cases.into_iter().map(|case| case.origin.ok_or("Materialize Case: Missing Origin")).collect::<Result<Vec<_>, _>>()?;
            let cycle: client::CyclesCreateResponse = session
                .unary(
                    "/dokime.client.v1.Cycles/Create",
                    client::CyclesCreateRequest {
                        origins,
                        continue_automatically: true,
                        request_id: "desktop-cycle".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let cycle = cycle.cycle.ok_or("Create Cycle: Missing Value")?;
            let executions: client::ExecutionsListResponse = session
                .unary(
                    "/dokime.client.v1.Executions/List",
                    client::ExecutionsListRequest {
                        parent: cycle.cycle_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let execution = executions
                .executions
                .first()
                .and_then(|item| item.summary.as_ref())
                .ok_or("Start Execution: Missing Identity")?
                .execution_id
                .clone();

            // Await the Operator Question Through the Owned Progress Watch.
            let request = client::executions::ProgressWatchRequest { name: execution.clone(), at: None }.encode_to_vec();
            let mut progress = session.watch_raw("/dokime.client.v1.executions.Progress/Watch", &request).await?;
            loop {
                let frame = progress.next().await.ok_or("Observe Execution: Closed Before Prompt")??;
                let frame = client::executions::ProgressWatchResponse::decode(frame.as_slice())?;
                assert!(!frame.finished);
                if frame.phase == "prompt" {
                    break;
                }
            }
            let prompts: client::PromptsListResponse = session
                .unary(
                    "/dokime.client.v1.Prompts/List",
                    client::PromptsListRequest {
                        execution_id: Some(execution.clone()),
                        pending_only: true,
                        ..Default::default()
                    },
                )
                .await?;
            let prompt = prompts.prompts.first().and_then(|view| view.prompt.as_ref()).ok_or("Answer Prompt: Missing Question")?;

            // Observe a Plot Before Answering, Then Join the Terminal Progress.
            let plot_request = v1::PlotRequest {
                plot_id: Some("temperature".into()),
                stream: "thermal.temperature".into(),
                y_keys: vec!["celsius".into()],
                scope: Some(v1::EvidenceScope {
                    execution_id: Some(execution.clone()),
                    ..Default::default()
                }),
                technique: v1::Downsample::Raw as i32,
                max_points: Some(128),
                ..Default::default()
            };
            let request = client::plots::SnapshotsWatchRequest {
                requests: vec![plot_request.clone()],
                ..Default::default()
            }
            .encode_to_vec();
            let mut plots = session.watch_raw("/dokime.client.v1.plots.Snapshots/Watch", &request).await?;
            let opening = plots.next().await.ok_or("Watch Plot: Missing Opening")??;
            let opening = client::plots::SnapshotsWatchResponse::decode(opening.as_slice())?;
            assert!(opening.opening);
            assert_eq!(opening.updates.len(), 1);
            let _: client::prompts::AnswersCreateResponse = session
                .unary(
                    "/dokime.client.v1.prompts.Answers/Create",
                    client::prompts::AnswersCreateRequest {
                        parent: prompt.prompt_id.clone(),
                        action: "yes".into(),
                        accepted: true,
                        request_id: "desktop-answer".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let mut terminal = false;
            while let Some(frame) = progress.next().await {
                assert!(!terminal);
                terminal = client::executions::ProgressWatchResponse::decode(frame?.as_slice())?.finished;
            }
            assert!(terminal);
            drop(plots);

            // Decode Sealed Evidence With the Digest-Cached Descriptor Set.
            let detail: client::ExecutionsGetResponse = session
                .unary(
                    "/dokime.client.v1.Executions/Get",
                    client::ExecutionsGetRequest {
                        name: execution.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let detail = detail.execution.ok_or("Get Execution: Missing Value")?;
            assert_eq!(detail.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
            assert!(!detail.seal.is_empty());
            let mut cache = FrozenSchemaCache::default();
            let descriptors = cache.get(session.client(), &detail).await?;
            assert!(Arc::ptr_eq(&descriptors, &cache.get(session.client(), &detail).await?));
            let records: client::evidence::RecordsListResponse = session
                .unary(
                    "/dokime.client.v1.evidence.Records/List",
                    client::evidence::RecordsListRequest {
                        selection: Some(client::evidence::CapturedRecordSelection {
                            scope: Some(v1::EvidenceScope {
                                execution_id: Some(execution.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(records.records.iter().any(|record| record.record_type == "thermal.door"));
            let samples: client::evidence::SamplesListResponse = session
                .unary(
                    "/dokime.client.v1.evidence.Samples/List",
                    client::evidence::SamplesListRequest {
                        scope: Some(v1::EvidenceScope {
                            execution_id: Some(execution.clone()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!samples.samples.is_empty());
            let snapshots: client::plots::SnapshotsListResponse = session
                .unary(
                    "/dokime.client.v1.plots.Snapshots/List",
                    client::plots::SnapshotsListRequest {
                        requests: vec![plot_request],
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(snapshots.envelopes.len(), 1);

            // Consume Every Ordered Document Page at the Returned Cut.
            let mut request = client::DocumentsGetRequest {
                scope: Some(v1::DocumentScope {
                    cycle_id: cycle.cycle_id.clone(),
                    execution_id: execution.clone(),
                }),
                ..Default::default()
            };
            let mut header = false;
            let mut completed = false;
            loop {
                let reply: client::DocumentsGetResponse = session.unary("/dokime.client.v1.Documents/Get", request.clone()).await?;
                for frame in reply.frames {
                    assert!(!completed);
                    match frame.frame.and_then(|frame| frame.kind) {
                        Some(client::document_frame_frame::Kind::Header(value)) => {
                            assert!(!header);
                            assert_eq!(value.frontiers.unwrap().process_epoch, process_epoch);
                            header = true;
                        }
                        Some(client::document_frame_frame::Kind::Completion(_)) => completed = true,
                        _ => {}
                    }
                }
                if reply.next_page_token.is_empty() {
                    break;
                }
                assert_ne!(request.page_token, reply.next_page_token);
                request.page_token = reply.next_page_token;
                request.at = reply.resolved_at;
            }
            assert!(header && completed);

            // Deliver the Report Through Product File Ranges and the Native Verified Writer.
            let subject = client::ReportSubject {
                kind: Some(client::report_subject::Kind::Execution(execution.clone())),
            };
            let prepared: client::ReportsCreateResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Create",
                    client::ReportsCreateRequest {
                        subject: Some(subject.clone()),
                        request_id: "desktop-prepare-report".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let file = prepared.file.ok_or("Prepare Report: Missing File Reference")?;
            let _: client::ReportsPublishResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Publish",
                    client::ReportsPublishRequest {
                        name: file.metadata.ok_or("Publish Report: Missing File Identity")?.id,
                        subject: Some(subject),
                        expected_revision: file.sha256,
                        request_id: "desktop-publish-report".into(),
                    },
                )
                .await?;
            let report: client::ReportsGetResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Get",
                    client::ReportsGetRequest {
                        subject: Some(client::ReportSubject {
                            kind: Some(client::report_subject::Kind::Execution(execution.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            let reference = report.file.ok_or("Read Report: Missing File Reference")?;
            let selected: client::FilesGetResponse =
                session.unary("/dokime.client.v1.Files/Get", client::FilesGetRequest { file: Some(reference.clone()), at: None }).await?;
            let mut bytes = Vec::new();
            let mut at = selected.resolved_at;
            while bytes.len() < reference.length as usize {
                let reply: client::FilesReadResponse = session
                    .unary(
                        "/dokime.client.v1.Files/Read",
                        client::FilesReadRequest {
                            file: Some(reference.clone()),
                            offset: bytes.len() as u64,
                            max_bytes: selected.chunk_bytes,
                            at: at.clone(),
                        },
                    )
                    .await?;
                assert_eq!(reply.total_length, reference.length);
                assert_eq!(reply.sha256, reference.sha256);
                assert!(!reply.data.is_empty());
                if at.is_some() {
                    assert_eq!(at, reply.resolved_at);
                }
                at = reply.resolved_at;
                bytes.extend(reply.data);
            }
            assert_eq!(Sha256::digest(&bytes).as_slice(), reference.sha256);
            let active = ConnectionSession::default();
            let connected = active
                .connect(ConnectRequest {
                    target: ConnectionTarget::Local { endpoint: session.endpoint().to_owned() },
                    expected_backend_id: None,
                })
                .await
                .map_err(|error| format!("{error:?}"))?;
            let destination = root.path().join("native-export.html");
            let exported = dokime_gui::testing::save_verified(
                active.clone(),
                connected.generation,
                dokime_gui::testing::ArtifactSource::ExecutionReport { execution_id: execution },
                "native-export.html".into(),
                async { Ok(Some((destination.clone(), false))) },
            )
            .await?;
            assert!(exported.is_some());
            assert_eq!(std::fs::read(destination)?, bytes);
            active.disconnect().map_err(|error| format!("{error:?}"))?;
            Ok::<(), Box<dyn StdError>>(())
        }
        .await;
        let host_close = owner.close().await;
        let plugin_close = plugin.close().await;
        result?;
        host_close?;
        plugin_close?;
        Ok(())
    })
}
