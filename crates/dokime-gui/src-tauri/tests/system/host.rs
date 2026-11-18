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
use std::{collections::BTreeSet, env, fs};
use std::{error::Error, num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports

use dokime_sdk::client::DocumentScope;
use dokime_sdk::client::PluginPlacement;
use dokime_sdk::client::PluginRemote;
use dokime_sdk::client::PluginValue;
use dokime_sdk::client::PluginsCreateOptions;
use dokime_sdk::client::plugin_placement;
use dokime_sdk::client::{PluginState, PluginsGetOptions, PluginsWatchOptions};

use base64::{Engine, engine::general_purpose::STANDARD};
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::PromptsListRequest;
use dokime_protocol::host::ReportCapture;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::SuitesGetRequest;
use dokime_protocol::host::SuitesListRequest;
use dokime_protocol::host::backend::OverviewGetRequest;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::executions::ProgressWatchRequest;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::prompts::AnswersCreateRequest;
use dokime_sdk::CapturedRecordSelection;
use dokime_sdk::Downsample;
use dokime_sdk::EvidenceScope;
use dokime_sdk::FieldReference;
use dokime_sdk::FilesGetResponse;
use dokime_sdk::FilesReadResponse;
use dokime_sdk::Import;
use dokime_sdk::Metadata;
use dokime_sdk::Origin;
use dokime_sdk::RecordsListResponse;
use dokime_sdk::Termination;
use dokime_sdk::client::AnswersCreateResponse;
use dokime_sdk::client::CasesCreateResponse;
use dokime_sdk::client::CyclesCreateResponse;
use dokime_sdk::client::DocumentsGetResponse;
use dokime_sdk::client::ExecutionOutcome;
use dokime_sdk::client::ExecutionState;
use dokime_sdk::client::ExecutionsGetResponse;
use dokime_sdk::client::ExecutionsListResponse;
use dokime_sdk::client::OverviewGetResponse;
use dokime_sdk::client::PlotRequest;
use dokime_sdk::client::ProgressWatchResponse;
use dokime_sdk::client::PromptsListResponse;
use dokime_sdk::client::RegisteredSelection;
use dokime_sdk::client::ReportSubject;
use dokime_sdk::client::ReportsCreateResponse;
use dokime_sdk::client::ReportsGetResponse;
use dokime_sdk::client::ReportsPublishResponse;
use dokime_sdk::client::SamplesListResponse;
use dokime_sdk::client::SelectionPlan;
use dokime_sdk::client::SnapshotsListResponse;
use dokime_sdk::client::SnapshotsWatchResponse;
use dokime_sdk::client::SuitesGetResponse;
use dokime_sdk::client::SuitesListResponse;
use dokime_sdk::client::document_frame_frame;
use dokime_sdk::client::report_subject;
use dokime_sdk::value;
use dokime_sdk::{card, origin};
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use tonic::Code;

use dokime_gui::connection::host::{self, Connection, ConnectionSettings};
use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
use dokime_gui::{test_support, testing};
use dokime_protocol::host::plots::PlotDomain;
use dokime_protocol::host::{DocumentsWatchRequest, DocumentsWatchResponse};
use dokime_runtime::{ControlledClock, HostSettings, PlacementSettings, PluginProcess, services, storage_defaults};

use dokime_sdk::StorageTransferSettings;
use dokime_sdk::client::document_binding_result;
use dokime_sdk::client::{ClientSettings, FrozenSchemaCache};
use dokime_sdk::model::Section;
use dokime_sdk::{Card, Document, FftWindow, Plot, PlotKind, Suite, Suites, TestCaseTemplate};
use dokime_storage::{Execution, Sample, Store};
use harmos_protocol::model;
use harmos_protocol::model::{Declarations, StreamDeclaration};
use prost_types::FileDescriptorSet;

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
            credentials: test_support::credentials(),
            capture: dokime_runtime::CaptureSettings::default(),
            clock: Arc::new(ControlledClock::new(0)),
            storage: storage_defaults(),
            placement: PlacementSettings {
                reconnect_delay: Duration::from_millis(25),
                stream_window: NonZeroUsize::new(8).unwrap(),
                interval_window: dokime_runtime::INTERVAL_WINDOW,
                interval_lag: dokime_runtime::INTERVAL_LAG,
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
fn large_document_and_long_history_frequency_use_bounded_native_reads() -> Result<(), Box<dyn Error>> {
    test_support::run(async {
        // Freeze a Large Authored Document and Three Days of Evidence.
        let root = test_support::tempdir()?;
        let store = Store::open(root.path())?;
        let plots = (0..65)
            .map(|index| Plot {
                metadata: Some(Metadata {
                    identifier: format!("plot-{index}"),
                    label: format!("Plot {index}"),
                    ..Default::default()
                }),
                kind: if index == 64 { PlotKind::FrequencySeries } else { PlotKind::TimeSeries } as i32,
                y: vec![FieldReference::new("wave").field("min")],
                input_samples: (index == 64).then_some(16),
                sample_rate_hz: (index == 64).then_some(1.0),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let document = Document {
            metadata: Some(Metadata {
                identifier: "large".into(),
                label: "Large Document".into(),
                ..Default::default()
            }),
            sections: vec![Section {
                cards: plots
                    .into_iter()
                    .enumerate()
                    .map(|(index, plot)| Card {
                        metadata: Some(Metadata {
                            identifier: format!("card-{index}"),
                            ..Default::default()
                        }),
                        body: Some(card::Body::Plot(plot)),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let catalog = Declarations {
            types: Some(FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![StreamDeclaration {
                id: "wave".into(),
                row: "dokime.host.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            sections: vec![model::Section {
                r#type: "dokime.model.v1.Suites".into(),
                payload: Suites {
                    suites: vec![Suite {
                        templates: vec![TestCaseTemplate {
                            documents: vec![document],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                }
                .encode_to_vec(),
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "long".into(),
            cycle: "cycle".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            ..Default::default()
        })?;
        store.open_stream("long", "wave")?;
        let samples = (0..2048)
            .map(|index| Sample {
                stream: "wave".into(),
                source_ns: if index < 2032 { index * 120_000_000_000 } else { (259_200 + index - 2032) * 1_000_000_000 },
                sequence: index as u64 + 1,
                row: PlotDomain {
                    min: [0.0, 1.0, 0.0, -1.0][index as usize % 4],
                    max: 0.0,
                }
                .encode_to_vec(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        store.sample_batch("long", &samples)?;
        store.close_streams("long")?;
        store.close("long", Termination::Completed, ExecutionOutcome::default())?;
        store.seal("long")?;
        drop(store);

        // Assemble the Entire Document Without Rendering Invisible Plot Payloads.
        let mut owner = Connection::new(settings())?;
        let result = async {
            let session = owner.start(root.path()).await?;
            let scope = DocumentScope {
                cycle_id: "cycle".into(),
                execution_id: "long".into(),
            };
            let mut request = DocumentsGetRequest {
                scope: Some(scope.clone()),
                document_id: "large".into(),
                max_frames: 32,
                ..Default::default()
            };
            let mut requests = Vec::new();
            let mut completed = false;
            loop {
                let reply: DocumentsGetResponse = session.unary("/dokime.host.v1.Documents/Get", request.clone()).await?;
                for frame in reply.frames {
                    match frame.frame.and_then(|frame| frame.kind) {
                        Some(document_frame_frame::Kind::Binding(binding)) => {
                            if let Some(document_binding_result::Kind::Plot(plot)) = binding.result.and_then(|result| result.kind) {
                                requests.push(plot.request.ok_or("Read Document: Missing Plot Selection")?);
                            }
                        }
                        Some(document_frame_frame::Kind::Completion(_)) => completed = true,
                        _ => (),
                    }
                }
                if reply.next_page_token.is_empty() {
                    break;
                }
                request.page_token = reply.next_page_token;
                request.at = reply.resolved_at;
            }
            assert!(completed);
            assert_eq!(requests.len(), 65);

            // Read Independent Visible Groups Through the Native Watch Adapter.
            for group in requests.chunks(32) {
                let mut watch = session
                    .watch_raw(
                        "/dokime.host.v1.plots.Snapshots/Watch",
                        &SnapshotsWatchRequest {
                            requests: group.to_vec(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                    )
                    .await?;
                let mut received = Vec::new();
                while let Some(frame) = watch.next().await {
                    let frame = SnapshotsWatchResponse::decode(frame?.as_slice())?;
                    assert!(frame.resync_required.is_none());
                    received.extend(frame.updates.into_iter().map(|update| update.envelope.unwrap()));
                }
                assert_eq!(received.len(), group.len());
                for (envelope, request) in received.iter().zip(group) {
                    assert_eq!(Some(&envelope.plot_id), request.plot_id.as_ref());
                    let frame = envelope
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.frame.as_ref())
                        .ok_or("Draw Native Plot: Restore Its Data Frame")?;
                    assert!(frame.point_count > 0);
                    assert!(frame.series.iter().all(|column| !column.y.is_empty()));
                }
            }
            let mut frequency = requests.pop().unwrap();
            frequency.fft.as_mut().unwrap().window = FftWindow::Rectangular as i32;
            let response: SnapshotsListResponse = session
                .unary(
                    "/dokime.host.v1.plots.Snapshots/List",
                    SnapshotsListRequest {
                        requests: vec![frequency],
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(response.envelopes[0].diagnostics.as_ref().unwrap().examined_rows, 16);
            let column = &response.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0];
            let peak = column.y.iter().enumerate().max_by(|(_, a), (_, b)| a.value.unwrap().total_cmp(&b.value.unwrap())).unwrap().0;
            assert_eq!(column.x[peak].kind, Some(value::Kind::Number(0.25)));

            // A Saved Document Watch Completes With the Same Plot Definitions.
            let mut watch = session
                .watch_raw(
                    "/dokime.host.v1.Documents/Watch",
                    &DocumentsWatchRequest {
                        scope: Some(scope),
                        document_id: "large".into(),
                        max_frames: 32,
                        ..Default::default()
                    }
                    .encode_to_vec(),
                )
                .await?;
            let mut completed = false;
            while let Some(frame) = watch.next().await {
                let frame = DocumentsWatchResponse::decode(frame?.as_slice())?;
                assert!(frame.resync_required.is_none());
                completed |= frame.frames.iter().any(|frame| {
                    matches!(frame.frame.as_ref().and_then(|frame| frame.kind.as_ref()), Some(document_frame_frame::Kind::Completion(_)))
                });
            }
            assert!(completed);
            Ok::<(), Box<dyn Error>>(())
        }
        .await;
        let closed = owner.close().await;
        result?;
        closed?;
        Ok(())
    })
}

#[test]
fn bench_execution_prompts_evidence_and_report() -> Result<(), Box<dyn Error>> {
    test_support::run(async {
        let executable = env::var_os("DOKIME_OBC_PLUGIN").map(PathBuf::from).unwrap_or_else(|| test_support::fixture("plugin-obc"));
        let configuration = settings();
        let mut plugin = PluginProcess::start(&executable, true, &configuration.host.placement).await?;
        let root = test_support::tempdir()?;
        let mut owner = Connection::new(configuration)?;
        let result = async {
            // Admit the Prepared Plugin and Read Its Top-Level Suite Resource.
            let session = owner.start(root.path()).await?;
            let overview: OverviewGetResponse =
                session.unary("/dokime.host.v1.backend.Overview/Get", OverviewGetRequest::default()).await?;
            let process_epoch = overview.overview.ok_or("Read Backend: Missing Overview")?.session_id;
            let options = PluginsCreateOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let placement = PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
            };
            let admitted = session
                .execute(session.client().plugins.create_with("desktop_plot", placement, PluginValue { enabled: true }, options))
                .await?;
            let mut entry = admitted.entry.ok_or("Admit Desktop Fixture: Missing Entry")?;
            let options = PluginsWatchOptions {
                name: Some(entry.name.clone()),
                context: session.call_options()?,
                ..Default::default()
            };
            let mut watching = session.execute(session.client().plugins.watch_with(options)).await?;
            while entry.state != PluginState::Ready as i32 {
                if entry.failure.is_some() || entry.state == PluginState::Stopped as i32 {
                    return Err(format!("Admit Desktop Fixture: {:?}", entry.failure).into());
                }
                session.execute(watching.next()).await?.ok_or("Admit Desktop Fixture: Watch Closed")?;
                let options = PluginsGetOptions {
                    context: session.call_options()?,
                    ..Default::default()
                };
                entry = session
                    .execute(session.client().plugins.get_with(entry.name, options))
                    .await?
                    .entry
                    .ok_or("Admit Desktop Fixture: Entry Removed")?;
            }
            drop(watching);
            let suites: SuitesListResponse = session.unary("/dokime.host.v1.Suites/List", SuitesListRequest::default()).await?;
            let suite = suites
                .values
                .iter()
                .find(|suite| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "obc.benchmark"))
                .ok_or("Read Suite: Missing Prepared Bench Suite")?;
            let suite_id = suite.metadata.as_ref().ok_or("Read Suite: Missing Identity")?.identifier.clone();
            let suite: SuitesGetResponse = session
                .unary(
                    "/dokime.host.v1.Suites/Get",
                    SuitesGetRequest {
                        name: suite_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let suite = suite.value.ok_or("Read Suite: Missing Definition")?;
            let template = suite
                .templates
                .iter()
                .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "obc.benchmark.plots"))
                .and_then(|template| template.metadata.as_ref())
                .ok_or("Read Suite: Missing Template")?;
            let plan: CasesCreateResponse = session
                .unary(
                    "/dokime.host.v1.Cases/Create",
                    CasesCreateRequest {
                        plan: Some(SelectionPlan {
                            selections: vec![RegisteredSelection {
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
            let cycle: CyclesCreateResponse = session
                .unary(
                    "/dokime.host.v1.Cycles/Create",
                    CyclesCreateRequest {
                        origins,
                        continue_automatically: true,
                        request_id: "desktop-cycle".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let cycle = cycle.cycle.ok_or("Create Cycle: Missing Value")?;
            let executions: ExecutionsListResponse = session
                .unary(
                    "/dokime.host.v1.Executions/List",
                    ExecutionsListRequest {
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
            let request = ProgressWatchRequest { name: execution.clone(), at: None }.encode_to_vec();
            let mut progress = session.watch_raw("/dokime.host.v1.executions.Progress/Watch", &request).await?;
            loop {
                let frame = progress.next().await.ok_or("Observe Execution: Closed Before Prompt")??;
                let frame = ProgressWatchResponse::decode(frame.as_slice())?;
                assert!(!frame.finished);
                if frame.phase == "prompt" {
                    break;
                }
            }
            let prompts: PromptsListResponse = session
                .unary(
                    "/dokime.host.v1.Prompts/List",
                    PromptsListRequest {
                        execution_id: Some(execution.clone()),
                        pending_only: true,
                        ..Default::default()
                    },
                )
                .await?;
            let prompt = prompts.prompts.first().and_then(|view| view.prompt.as_ref()).ok_or("Answer Prompt: Missing Question")?;

            // Refuse the Same Report Request While Its Execution Is Waiting for an Answer.
            let report_request = ReportsCreateRequest {
                subject: Some(ReportSubject {
                    kind: Some(report_subject::Kind::Execution(execution.clone())),
                }),
                request_id: "desktop-prepare-report".into(),
                ..Default::default()
            };
            let refused =
                session.unary::<_, ReportsCreateResponse>("/dokime.host.v1.Reports/Create", report_request.clone()).await.unwrap_err();
            let host::Error::Rpc(refused) = refused else {
                return Err("Create Report: Expected a Precondition Refusal".into());
            };
            assert_eq!(refused.code(), Code::FailedPrecondition);

            // Observe a Plot Before Answering, Then Join the Terminal Progress.
            let plot_request = PlotRequest {
                plot_id: Some("module-voltage".into()),
                stream: "obc.measurement".into(),
                y_keys: vec!["voltage_v".into()],
                scope: Some(EvidenceScope {
                    execution_id: Some(execution.clone()),
                    ..Default::default()
                }),
                technique: Downsample::Raw as i32,
                max_points: Some(512),
                ..Default::default()
            };
            let request = SnapshotsWatchRequest {
                requests: vec![plot_request.clone()],
                ..Default::default()
            }
            .encode_to_vec();
            let mut plots = session.watch_raw("/dokime.host.v1.plots.Snapshots/Watch", &request).await?;
            let opening = plots.next().await.ok_or("Watch Plot: Missing Opening")??;
            let opening = SnapshotsWatchResponse::decode(opening.as_slice())?;
            assert!(opening.opening);
            assert_eq!(opening.updates.len(), 1);
            // Open Every Authored Card From Two Independent Native Clients.
            let mut second_owner = Connection::new(settings())?;
            let second = second_owner.attach_local(session.endpoint()).await?;
            let mut document_request = DocumentsGetRequest {
                scope: Some(DocumentScope {
                    cycle_id: cycle.cycle_id.clone(),
                    execution_id: execution.clone(),
                }),
                document_id: "obc.benchmark.plots.report".into(),
                max_frames: 32,
                ..Default::default()
            };
            let mut producing = Vec::new();
            loop {
                let (first, other): (DocumentsGetResponse, DocumentsGetResponse) = tokio::try_join!(
                    session.unary("/dokime.host.v1.Documents/Get", document_request.clone()),
                    second.unary("/dokime.host.v1.Documents/Get", document_request.clone()),
                )?;
                assert_eq!(first.frames, other.frames);
                for frame in first.frames {
                    if let Some(document_frame_frame::Kind::Binding(binding)) = frame.frame.and_then(|frame| frame.kind)
                        && let Some(document_binding_result::Kind::Plot(plot)) = binding.result.and_then(|result| result.kind)
                    {
                        let mut request = plot.request.ok_or("Read Producing Document: Restore Its Plot Selection")?;
                        request.at = None;
                        producing.push(request);
                    }
                }
                if first.next_page_token.is_empty() {
                    break;
                }
                document_request.page_token = first.next_page_token;
                document_request.at = first.resolved_at;
            }
            assert!(producing.len() > 100);
            let mut readers = JoinSet::new();
            for viewer in [&session, &second] {
                for group in producing.chunks(32) {
                    let viewer = viewer.clone();
                    let group = group.to_vec();
                    readers.spawn(async move {
                        let expected = group.iter().map(|plot| plot.plot_id.clone().unwrap()).collect::<BTreeSet<_>>();
                        let mut data = BTreeSet::new();
                        let mut watch = viewer
                            .watch_raw(
                                "/dokime.host.v1.plots.Snapshots/Watch",
                                &SnapshotsWatchRequest { requests: group, ..Default::default() }.encode_to_vec(),
                            )
                            .await?;
                        while let Some(frame) = watch.next().await {
                            let frame = SnapshotsWatchResponse::decode(frame?.as_slice())?;
                            assert!(frame.resync_required.is_none());
                            for update in frame.updates {
                                let envelope = update.envelope.unwrap();
                                assert!(envelope.warnings.is_empty());
                                assert!(expected.contains(&envelope.plot_id));
                                if envelope.diagnostics.is_some_and(|diagnostics| diagnostics.point_count > 0) {
                                    assert!(envelope.payload.is_some());
                                    data.insert(envelope.plot_id);
                                }
                            }
                            if data == expected {
                                break;
                            }
                        }
                        assert_eq!(data, expected);
                        Ok::<_, Box<dyn Error + Send + Sync>>(data.len())
                    });
                }
            }
            let _: AnswersCreateResponse = session
                .unary(
                    "/dokime.host.v1.prompts.Answers/Create",
                    AnswersCreateRequest {
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
                terminal = ProgressWatchResponse::decode(frame?.as_slice())?.finished;
            }
            assert!(terminal);
            let mut cards_with_data = 0;
            while let Some(result) = readers.join_next().await {
                cards_with_data += result?.map_err(|error| error.to_string())?;
            }
            assert_eq!(cards_with_data, producing.len() * 2);
            second_owner.close().await?;
            drop(plots);

            // Decode Sealed Evidence With the Digest-Cached Descriptor Set.
            let detail: ExecutionsGetResponse = session
                .unary(
                    "/dokime.host.v1.Executions/Get",
                    ExecutionsGetRequest {
                        name: execution.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let detail = detail.execution.ok_or("Get Execution: Missing Value")?;
            assert_eq!(detail.summary.as_ref().unwrap().state, ExecutionState::Sealed as i32);
            assert!(!detail.seal.is_empty());
            let mut cache = FrozenSchemaCache::default();
            let descriptors = cache.get(session.client(), &detail).await?;
            assert!(Arc::ptr_eq(&descriptors, &cache.get(session.client(), &detail).await?));
            let records: RecordsListResponse = session
                .unary(
                    "/dokime.host.v1.evidence.Records/List",
                    RecordsListRequest {
                        selection: Some(CapturedRecordSelection {
                            scope: Some(EvidenceScope {
                                execution_id: Some(execution.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(records.records.iter().any(|record| record.record_type == "obc.benchmark.plots.confirmation_record"));
            let samples: SamplesListResponse = session
                .unary(
                    "/dokime.host.v1.evidence.Samples/List",
                    SamplesListRequest {
                        scope: Some(EvidenceScope {
                            execution_id: Some(execution.clone()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!samples.samples.is_empty());
            let snapshots: SnapshotsListResponse = session
                .unary(
                    "/dokime.host.v1.plots.Snapshots/List",
                    SnapshotsListRequest {
                        requests: vec![plot_request],
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(snapshots.envelopes.len(), 1);

            // Consume Every Ordered Document Page at the Returned Cut.
            let mut request = DocumentsGetRequest {
                max_frames: 32,
                scope: Some(DocumentScope {
                    cycle_id: cycle.cycle_id.clone(),
                    execution_id: execution.clone(),
                }),
                ..Default::default()
            };
            let mut header = false;
            let mut completed = false;
            let mut document_cut = None;
            loop {
                let reply: DocumentsGetResponse = session.unary("/dokime.host.v1.Documents/Get", request.clone()).await?;
                for frame in reply.frames {
                    assert!(!completed);
                    match frame.frame.and_then(|frame| frame.kind) {
                        Some(document_frame_frame::Kind::Header(value)) => {
                            assert!(!header);
                            assert_eq!(value.frontiers.unwrap().process_epoch, process_epoch);
                            header = true;
                        }
                        Some(document_frame_frame::Kind::EvidenceCut(value)) => document_cut = value.cut,
                        Some(document_frame_frame::Kind::Completion(_)) => completed = true,
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
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(execution.clone())),
            };
            let prepared: ReportsCreateResponse = session.unary("/dokime.host.v1.Reports/Create", report_request).await?;
            let document_cut = document_cut.ok_or("Read Document: Missing Final Evidence Cut")?;
            let resolved_at = prepared.resolved_at.as_ref().ok_or("Create Report: Missing Final Evidence Cut")?;
            assert_eq!(resolved_at.positions.len(), 1);
            assert!(resolved_at.revisions.is_empty());
            let position = &resolved_at.positions[0];
            let history = position.history.as_ref().ok_or("Create Report: Missing Execution History")?;
            assert_eq!(history.store, document_cut.store);
            assert_eq!(history.epoch, execution);
            assert_eq!(position.sequence, document_cut.position);
            let file = prepared.file.ok_or("Prepare Report: Missing File Reference")?;
            let _: ReportsPublishResponse = session
                .unary(
                    "/dokime.host.v1.Reports/Publish",
                    ReportsPublishRequest {
                        name: file.metadata.ok_or("Publish Report: Missing File Identity")?.id,
                        subject: Some(subject),
                        expected_revision: file.sha256,
                        request_id: "desktop-publish-report".into(),
                    },
                )
                .await?;
            let report: ReportsGetResponse = session
                .unary(
                    "/dokime.host.v1.Reports/Get",
                    ReportsGetRequest {
                        subject: Some(ReportSubject {
                            kind: Some(report_subject::Kind::Execution(execution.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            let reference = report.file.ok_or("Read Report: Missing File Reference")?;
            let selected: FilesGetResponse =
                session.unary("/dokime.host.v1.Files/Get", FilesGetRequest { file: Some(reference.clone()), at: None }).await?;
            let mut bytes = Vec::new();
            let mut at = selected.resolved_at;
            while bytes.len() < reference.length as usize {
                let reply: FilesReadResponse = session
                    .unary(
                        "/dokime.host.v1.Files/Read",
                        FilesReadRequest {
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
            let exported = testing::save_verified(
                active.clone(),
                connected.generation,
                testing::ArtifactSource::ExecutionReport { execution_id: execution },
                "native-export.html".into(),
                async { Ok(Some((destination.clone(), false))) },
            )
            .await?;
            assert!(exported.is_some());
            let html = fs::read(destination)?;
            let html = str::from_utf8(&html)?;
            let marker =
                html.find("id=\"hub-report-data\"").or_else(|| html.find("id=hub-report-data")).ok_or("Open Report: Missing Capture")?;
            let data = html[marker..]
                .split_once('>')
                .ok_or("Open Report: Missing Data")?
                .1
                .split_once("</script>")
                .ok_or("Open Report: Incomplete Data")?
                .0;
            let data = STANDARD.decode(data.trim())?;
            let capture = ReportCapture::decode(data.as_slice())?;
            let detail = capture.executions[0].detail.as_ref().unwrap().execution.as_ref().unwrap();
            let summary = detail.summary.as_ref().unwrap();
            assert!(
                capture
                    .cases
                    .iter()
                    .any(|case| case.detail.as_ref().unwrap().value.as_ref().unwrap().summary.as_ref().unwrap().case_id == summary.case_id)
            );
            assert!(
                capture.executions[0]
                    .plots
                    .as_ref()
                    .unwrap()
                    .envelopes
                    .iter()
                    .any(|plot| plot.diagnostics.as_ref().is_some_and(|diagnostics| diagnostics.point_count > 0))
            );
            active.disconnect().map_err(|error| format!("{error:?}"))?;
            Ok::<(), Box<dyn Error>>(())
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
