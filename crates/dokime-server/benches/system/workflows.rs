/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Workflow Benchmarks: Operator Paths Over gRPC Against The Built Server And OBC Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    path::Path,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};
use harmos::domain::Portable;
use harmos::grpc::{Transcoded, Wire};
use harmos_packaging::bundle::{Bundle, Limits, Source};
use sha2::{Digest, Sha256};

// Local Imports
use super::server::{Server, StorageCounters};
use dokime::Value;
use dokime::client::{Client, ResponseStream, Target, contracts};
use dokime::domain::model;

/* === Constants === */
const CAPTURE_SUITE: &str = "obc.benchmarks";
const CAPTURE_TEMPLATE: &str = "obc.benchmarks.capture";
const RECOVERY_SUITE: &str = "obc.charger.recovery";
const RECOVERY_TEMPLATE: &str = "obc.charger.recovery.power";
const RECOVERY_STREAM: &str = "obc.charger.recovery.samples";
const SAMPLES: usize = 10;
const SETTLE: Duration = Duration::from_secs(900);

/* === Macros === */

/* === Type Aliases === */
type Changes = ResponseStream<model::observation::wire::RuntimeStateFrame>;

/* === Structs === */
struct Session {
    client: Client,
    changes: Changes,
    epoch: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Session {
    async fn open(server: &Server) -> Self {
        let client = Client::connect(Target::local(server.endpoint()).unwrap(), None, None).await.unwrap();
        let changes = client.observation_watch_runtime(Default::default()).await.unwrap();
        let first = changes.message().await.unwrap().expect("the runtime watch opens with a frame");
        let epoch = first.cursor.as_ref().expect("runtime frames carry their cursor").epoch.clone();
        Self { client, changes, epoch }
    }

    async fn on_change<T>(&mut self, mut probe: impl AsyncFnMut(&Client) -> Option<T>) -> T {
        tokio::time::timeout(SETTLE, async {
            loop {
                if let Some(value) = probe(&self.client).await {
                    return value;
                }
                self.changes.message().await.unwrap().expect("the runtime watch stays open");
            }
        })
        .await
        .expect("the benchmark workflow settles")
    }

    async fn executions(&self, cycle_id: &str) -> Vec<model::execution::wire::ExecutionSummary> {
        let request = contracts::execution::ListExecutions { cycle_id: cycle_id.to_owned(), ..Default::default() };
        self.client.execution_list_executions(request).await.unwrap().executions
    }

    // The Prompt Of One Member Once It Waits, Answering Every Earlier Member's.
    async fn prompt_of(&mut self, cycle_id: &str, execution_id: &str) -> String {
        self.on_change(async |client| {
            let request = contracts::prompts::ListPending { cycle_id: Some(cycle_id.to_owned()), ..Default::default() };
            for prompt in client.prompts_list_pending(request).await.unwrap().prompts {
                let owner = prompt.scope.as_ref().and_then(|scope| scope.execution_id.as_deref());
                if owner == Some(execution_id) {
                    return Some(prompt.prompt_id);
                }
                respond(client, prompt.prompt_id).await;
            }
            None
        })
        .await
    }

    // Answers Every Prompt Until The Cycle Completes.
    async fn complete(&mut self, cycle_id: &str) {
        self.on_change(async |client| {
            let request = contracts::prompts::ListPending { cycle_id: Some(cycle_id.to_owned()), ..Default::default() };
            for prompt in client.prompts_list_pending(request).await.unwrap().prompts {
                respond(client, prompt.prompt_id).await;
            }
            let cycle = client.execution_get_cycle(contracts::execution::GetCycle { cycle_id: cycle_id.to_owned() }).await.unwrap();
            use model::execution::wire::CycleStatus;
            assert!(!matches!(cycle.status, CycleStatus::Failed | CycleStatus::Stopped), "benchmark cycle failed: {:?}", cycle.status);
            (cycle.status == CycleStatus::Completed).then_some(())
        })
        .await;
    }

    async fn sealed(&mut self, cycle_id: &str, execution_id: &str) {
        self.on_change(async |client| {
            let request = contracts::execution::ListExecutions { cycle_id: cycle_id.to_owned(), ..Default::default() };
            let page = client.execution_list_executions(request).await.unwrap();
            page.executions.iter().any(|member| member.execution_id == execution_id && member.sealed).then_some(())
        })
        .await;
    }

    async fn close_cycle(&mut self, cycle_id: &str) {
        self.complete(cycle_id).await;
        let id = model::identity::TestCycleId::from_uuid(cycle_id.to_owned()).unwrap();
        self.client.cycle(id).seal().await.unwrap();
    }

    async fn template(&self, suite_id: &str, identifier: &str) -> String {
        let suite = self.client.catalog_get_suite(contracts::catalog::GetSuite { suite_id: suite_id.to_owned() }).await.unwrap();
        suite
            .templates
            .into_iter()
            .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier))
            .unwrap_or_else(|| panic!("{identifier} is registered"))
            .template_id
    }

    // One Cycle Of The Given Plans, Run In Order Without Further Input.
    async fn cycle(&self, label: &str, plans: Vec<model::planning::wire::SelectionPlan>) -> (String, Vec<String>) {
        let metadata = model::identity::wire::Metadata { identifier: label.to_owned(), label: label.to_owned(), ..Default::default() };
        let created = contracts::execution::CreateCycle { metadata: Some(metadata), ..Default::default() };
        let cycle_id = self.client.execution_create_cycle(created).await.unwrap().cycle_id;
        for plan in plans {
            let register = contracts::planning::RegisterSelection { plan: Some(plan), idempotency_key: uuid::Uuid::now_v7().to_string() };
            let cases = self.client.planning_register_selection(register).await.unwrap().cases;
            let executions = cases
                .into_iter()
                .map(|case| contracts::execution::ExecutionBinding {
                    execution_id: uuid::Uuid::now_v7().to_string(),
                    case_id: case.case_id,
                })
                .collect();
            let append = contracts::execution::AppendExecutions { cycle_id: cycle_id.clone(), executions };
            self.client.execution_append_executions(append).await.unwrap();
        }
        let policy =
            contracts::execution::UpdateCyclePolicy { cycle_id: cycle_id.clone(), continue_automatically: true, ..Default::default() };
        self.client.execution_update_cycle_policy(policy).await.unwrap();
        let members = self.executions(&cycle_id).await.into_iter().map(|member| member.execution_id).collect::<Vec<_>>();
        let start = contracts::execution::StartExecution {
            execution_id: members[0].clone(),
            mode: model::execution::wire::ExecutionMode::Automatic,
        };
        let reply = self.client.execution_start_execution(start).await.unwrap();
        assert!(matches!(reply.outcome, Some(model::execution::wire::RunReplyOutcome::Accepted(_))), "{reply:?}");
        (cycle_id, members)
    }

    async fn capture(&self, rows: u64, members: u64) -> model::planning::wire::SelectionPlan {
        let template = self.template(CAPTURE_SUITE, CAPTURE_TEMPLATE).await;
        let parameters = [("rows", vec![number(rows as f64)]), ("member", (0..members).map(|value| number(value as f64)).collect())];
        plan(CAPTURE_SUITE, &template, parameters)
    }

    async fn note(&self, cycle_id: &str, execution_id: &str) -> (Duration, usize) {
        use model::notes::wire::{AnnotationOwner, AnnotationOwnerOwner, ExecutionNoteOwner};
        let owner =
            AnnotationOwnerOwner::Execution(ExecutionNoteOwner { cycle_id: cycle_id.to_owned(), execution_id: execution_id.to_owned() });
        let request = contracts::notes::CreateNote {
            epoch: self.epoch.clone(),
            mutation_id: uuid::Uuid::now_v7().to_string(),
            owner: Some(AnnotationOwner { owner: Some(owner) }),
            message: "benchmark note".into(),
        };
        let request_bytes = encoded_size(&request);
        let started = Instant::now();
        let reply = self.client.notes_create_note(request).await.unwrap();
        let duration = started.elapsed();
        (duration, request_bytes + encoded_size(&reply))
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(4).enable_all().build().unwrap();
    let bundle = super::server::plugin_bundle().expect("the bench build stages the OBC plugin bundle");
    install(criterion, &executor, &bundle);
    let mut server = Server::start(Some(&bundle));
    server.reload(&bundle);
    let mut session = executor.block_on(Session::open(&server));
    running_report(criterion, &executor, &mut session, &mut server);
    cycle_report(criterion, &executor, &mut session, &mut server);
    seal(criterion, &executor, &mut session, &mut server);
    note(criterion, &executor, &mut session, &mut server);
    plot_stream(criterion, &executor, &mut session, &mut server);
    drop(session);
    server.stop();
}

/* === Private Functions === */

// (a) Staging And Consenting The Built Bundle Through The Client Install Stream.
fn install(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, bundle: &Path) {
    let bytes = padded_bundle(bundle, 50 * 1024 * 1024);
    let mut server = Server::start(None);
    let client = executor.block_on(Client::connect(Target::local(server.endpoint()).unwrap(), None, None)).unwrap();
    let mut group = criterion.benchmark_group("workflow_installs_plugin");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function(BenchmarkId::from_parameter(format!("{}MiB", bytes.len() >> 20)), |bencher| {
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let before = server.storage_counters();
                    let started = Instant::now();
                    let installation = stage(&client, &bytes).await;
                    elapsed += started.elapsed();
                    counters("plugin_install", before, server.storage_counters(), bytes.len());
                    client.plugins_remove(contracts::plugins::Remove { install_id: installation.install_id }).await.unwrap();
                }
                elapsed
            })
        });
    });
    group.finish();
    drop(client);
    server.stop();
}

// (b) An Active 100k-Row Execution With Five Plots And Two Mid-Run Prompt Answers.
fn running_report(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, session: &mut Session, server: &mut Server) {
    let mut group = criterion.benchmark_group("workflow_reports_running_execution");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    for rows in [100_000_u64, 200_000] {
        group.throughput(Throughput::Elements(rows));
        group.bench_function(BenchmarkId::new("rows", rows), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let plan = session.capture(rows, 1).await;
                        let (cycle_id, members) = session.cycle(&format!("bench.report.{rows}"), vec![plan]).await;
                        let execution_id = &members[0];
                        let first = session.prompt_of(&cycle_id, execution_id).await;
                        let client = session.client.clone();
                        let progress = client.observation_watch_runtime(Default::default()).await.unwrap();
                        progress.message().await.unwrap().unwrap();
                        let before = server.storage_counters();
                        let report = async {
                            let started = Instant::now();
                            let html = execution_report(&client, execution_id).await;
                            (html, started.elapsed())
                        };
                        let answers = async {
                            report_started(progress, "Materializing Execution Report").await;
                            respond(&session.client, first).await;
                            let second = session.prompt_of(&cycle_id, execution_id).await;
                            respond(&session.client, second).await;
                        };
                        let ((html, duration), ()) = tokio::join!(report, answers);
                        elapsed += duration;
                        counters("running_report", before, server.storage_counters(), html.len());
                        session.close_cycle(&cycle_id).await;
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();
}

// (c) The Current Report Of A Completed Ten-Member Cycle.
fn cycle_report(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, session: &mut Session, server: &mut Server) {
    let mut group = criterion.benchmark_group("workflow_reports_cycle");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    let (cycle_id, bytes) = executor.block_on(async {
        let plan = session.capture(2_000, 10).await;
        let (cycle_id, members) = session.cycle("bench.cycle-report", vec![plan]).await;
        assert_eq!(members.len(), 10);
        session.complete(&cycle_id).await;
        let bytes = download_cycle_report(&session.client, &cycle_id).await.len();
        (cycle_id, bytes)
    });
    group.throughput(Throughput::Bytes(bytes as u64));
    group.bench_function(BenchmarkId::new("members", 10), |bencher| {
        bencher.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let before = server.storage_counters();
                let started = Instant::now();
                let html = executor.block_on(download_cycle_report(&session.client, &cycle_id));
                elapsed += started.elapsed();
                counters("cycle_report", before, server.storage_counters(), html.len());
            }
            elapsed
        });
    });
    group.finish();
    executor.block_on(session.close_cycle(&cycle_id));
}

// (d) From The Last Answer To The Durable Seal Of A Member With Large Telemetry.
fn seal(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, session: &mut Session, server: &mut Server) {
    let mut group = criterion.benchmark_group("workflow_seals_execution");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    for rows in [100_000_u64, 200_000] {
        group.throughput(Throughput::Elements(rows));
        group.bench_function(BenchmarkId::new("rows", rows), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let plan = session.capture(rows, 1).await;
                        let (cycle_id, members) = session.cycle(&format!("bench.seal.{rows}"), vec![plan]).await;
                        let prompt = session.prompt_of(&cycle_id, &members[0]).await;
                        respond(&session.client, prompt).await;
                        let prompt = session.prompt_of(&cycle_id, &members[0]).await;
                        let before = server.storage_counters();
                        let started = Instant::now();
                        let bytes = respond(&session.client, prompt).await;
                        session.sealed(&cycle_id, &members[0]).await;
                        elapsed += started.elapsed();
                        counters("execution_seal", before, server.storage_counters(), bytes);
                        session.close_cycle(&cycle_id).await;
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();
}

// (e) An Operator Note On A Running Member, Idle And While A Cycle Report Renders.
fn note(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, session: &mut Session, server: &mut Server) {
    let mut group = criterion.benchmark_group("workflow_adds_note");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    let (mut cycle_id, mut execution_id) = executor.block_on(note_fixture(session));
    let mut notes = 0;
    group.bench_function(BenchmarkId::new("idle_rows", 100_000), |bencher| {
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    renew_note_fixture(session, server, &mut cycle_id, &mut execution_id, &mut notes).await;
                    let before = server.storage_counters();
                    let (duration, bytes) = session.note(&cycle_id, &execution_id).await;
                    elapsed += duration;
                    counters("note", before, server.storage_counters(), bytes);
                }
                elapsed
            })
        });
    });
    group.bench_function(BenchmarkId::new("during_cycle_report_rows", 100_000), |bencher| {
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    renew_note_fixture(session, server, &mut cycle_id, &mut execution_id, &mut notes).await;
                    let progress = session.client.observation_watch_runtime(Default::default()).await.unwrap();
                    progress.message().await.unwrap().unwrap();
                    let before = server.storage_counters();
                    let report = download_cycle_report(&session.client, &cycle_id);
                    let note = async {
                        report_started(progress, "Materializing Cycle Report").await;
                        session.note(&cycle_id, &execution_id).await
                    };
                    let (html, (duration, bytes)) = tokio::join!(report, note);
                    elapsed += duration;
                    counters("note_during_report", before, server.storage_counters(), html.len() + bytes);
                }
                elapsed
            })
        });
    });
    group.finish();
    executor.block_on(session.close_cycle(&cycle_id));
}

async fn note_fixture(session: &mut Session) -> (String, String) {
    let measured = session.capture(100_000, 1).await;
    let (cycle_id, members) = session.cycle("bench.note", vec![measured]).await;
    let execution_id = members[0].clone();
    session.prompt_of(&cycle_id, &execution_id).await;
    (cycle_id, execution_id)
}

async fn renew_note_fixture(
    session: &mut Session, server: &mut Server, cycle_id: &mut String, execution_id: &mut String, notes: &mut usize,
) {
    // Bound Receipt Growth Even When Criterion Calibrates A Very Fast Note Path.
    // A New Private Server Root Keeps Every Measured Operation A New Note.
    if *notes == 1_024 {
        session.close_cycle(cycle_id).await;
        let bundle = super::server::plugin_bundle().unwrap();
        let previous = std::mem::replace(server, Server::start(Some(&bundle)));
        previous.stop();
        server.reload(&bundle);
        *session = Session::open(server).await;
        (*cycle_id, *execution_id) = note_fixture(session).await;
        *notes = 0;
    }
    *notes += 1;
}

// (f) Time To The First Frame Of A Plot Watch On A Recovery Execution.
fn plot_stream(criterion: &mut Criterion, executor: &tokio::runtime::Runtime, session: &mut Session, server: &mut Server) {
    let mut group = criterion.benchmark_group("workflow_streams_plot");
    group.sampling_mode(SamplingMode::Flat).sample_size(SAMPLES);
    let (cycle_id, execution_id) = executor.block_on(async {
        let template = session.template(RECOVERY_SUITE, RECOVERY_TEMPLATE).await;
        let (cycle_id, members) = session.cycle("bench.plot", vec![plan(RECOVERY_SUITE, &template, [])]).await;
        session.complete(&cycle_id).await;
        (cycle_id, members[0].clone())
    });
    let initial = executor.block_on(first_frame(&session.client, &cycle_id, &execution_id));
    let payload = initial.envelope.unwrap().payload.unwrap();
    let rows = payload.series.iter().map(|series| series.raw_points).sum::<u64>();
    group.throughput(Throughput::Elements(rows));
    group.bench_function(BenchmarkId::new("first_frame_rows", rows), |bencher| {
        bencher.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let before = server.storage_counters();
                let started = Instant::now();
                let frame = executor.block_on(first_frame(&session.client, &cycle_id, &execution_id));
                elapsed += started.elapsed();
                let bytes = encoded_size(&frame);
                counters("plot_first_frame", before, server.storage_counters(), bytes);
            }
            elapsed
        });
    });
    group.finish();
    executor.block_on(session.close_cycle(&cycle_id));
}

async fn stage(client: &Client, bytes: &[u8]) -> model::plugins::wire::PluginInstallation {
    use {model::plugins::wire::StagePluginFrame, model::plugins::wire::StagePluginFrameFrame as Frame};
    let sha256 = Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let header = model::plugins::wire::StagePluginHeader { size_bytes: bytes.len() as u64, sha256: sha256.clone() };
    let mut frames = vec![StagePluginFrame { frame: Some(Frame::Header(header)) }];
    for (index, chunk) in bytes.chunks(64 * 1024).enumerate() {
        let chunk = model::plugins::wire::StagePluginChunk { offset: (index * 64 * 1024) as u64, data: chunk.to_vec() };
        frames.push(StagePluginFrame { frame: Some(Frame::Chunk(chunk)) });
    }
    let staged = client.plugins_stage(tokio_stream::iter(frames)).await.unwrap();
    client.plugins_consent(contracts::plugins::Consent { install_id: staged.install_id, expected_sha256: sha256 }).await.unwrap()
}

async fn respond(client: &Client, prompt_id: String) -> usize {
    let values = object([("operator_note", text("bench")), ("accepted", Value::Bool(true))]);
    let request =
        contracts::prompts::Respond { prompt_id, action: "submit".into(), values: Some(values), accepted: true, ..Default::default() };
    let request_bytes = encoded_size(&request);
    let reply = client.prompts_respond(request).await.unwrap();
    request_bytes + encoded_size(&reply)
}

fn encoded_size<T: Portable>(value: &T) -> usize {
    let mut bytes = Vec::new();
    <Transcoded as Wire<T>>::encode(value, &mut bytes).unwrap();
    bytes.len()
}

async fn report_bytes(report: ResponseStream<model::reports::wire::ReportHtmlFrame>) -> Vec<u8> {
    use model::reports::wire::ReportHtmlFramePayload as Payload;
    let mut html = Vec::new();
    while let Some(frame) = report.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            Payload::Report(_) => {}
            Payload::Chunk(chunk) => {
                assert_eq!(chunk.offset, html.len() as u64);
                html.extend_from_slice(&chunk.data);
            }
            Payload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                let digest = Sha256::digest(&html).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                assert_eq!(done.sha256, digest);
                assert!(report.message().await.unwrap().is_none(), "completion is the final frame with successful transport status");
                return html;
            }
        }
    }
    panic!("the report stream ends before it completes");
}

async fn execution_report(client: &Client, execution_id: &str) -> Vec<u8> {
    let request = contracts::reports::DownloadExecutionReport { execution_id: execution_id.to_owned() };
    report_bytes(client.reports_download_execution_report(request).await.unwrap()).await
}

async fn download_cycle_report(client: &Client, cycle_id: &str) -> Vec<u8> {
    let request = contracts::reports::DownloadCycleReport { cycle_id: cycle_id.to_owned(), identifier: "cycle.summary".into() };
    report_bytes(client.reports_download_cycle_report(request).await.unwrap()).await
}

async fn first_frame(client: &Client, cycle_id: &str, execution_id: &str) -> model::plots::wire::PlotStreamFrame {
    use model::{
        document::DownsampleTechnique,
        plots::wire::{AxisScale, PlotRequest, PlotType},
    };
    let request = PlotRequest {
        plot_id: Some("bench.recovery.voltage".into()),
        plot_type: PlotType::TimeSeries,
        x_scale: AxisScale::Linear,
        y_scale: AxisScale::Linear,
        stream: RECOVERY_STREAM.into(),
        y_keys: vec!["voltage_v".into()],
        marker_record_types: vec![],
        filters: vec![],
        x_key: None,
        series_by: None,
        x_expression: None,
        y_expressions: vec![],
        series_by_expression: None,
        z_stream: None,
        z_field: None,
        unit: None,
        max_points: Some(2_000),
        window_s: None,
        scope: Some(model::identity::wire::EvidenceScope {
            cycle_id: Some(cycle_id.to_owned()),
            execution_id: Some(execution_id.to_owned()),
            ..Default::default()
        }),
        cursor: None,
        technique: DownsampleTechnique::Unspecified,
        z_expression: None,
        fft: None,
        interval: None,
        histogram: None,
        analysis: None,
        summary_window: None,
    };
    let stream = client.plots_watch(contracts::plots::Watch { requests: vec![request] }).await.unwrap();
    tokio::time::timeout(SETTLE, async {
        loop {
            let frame = stream.message().await.unwrap().expect("the plot watch opens with a frame");
            assert_ne!(frame.kind, model::plots::wire::PlotStreamKind::Error, "the plot watch serves its snapshot");
            let payload = frame.envelope.as_ref().and_then(|envelope| envelope.payload.as_ref());
            if payload.is_some_and(|payload| payload.series.iter().any(|series| !series.points.is_empty())) {
                return frame;
            }
        }
    })
    .await
    .expect("the recovery plot delivers actual points, beyond its initial acknowledgement")
}

async fn report_started(progress: Changes, label: &str) {
    tokio::time::timeout(SETTLE, async {
        loop {
            let frame = progress.message().await.unwrap().expect("runtime progress remains available");
            if frame
                .operations
                .iter()
                .any(|operation| operation.label == label && operation.status == model::observation::wire::OperationStatus::Running)
            {
                return;
            }
        }
    })
    .await
    .expect("the measured report enters materialization before the competing mutation");
}

fn counters(scenario: &str, before: StorageCounters, after: StorageCounters, bytes: usize) {
    eprintln!(
        "{scenario} sqlite_committed_transactions={} sqlite_data_batches={} durable_files={} transferred_payload_bytes={bytes}",
        after.committed_transactions.checked_sub(before.committed_transactions).unwrap(),
        after.data_batches.checked_sub(before.data_batches).unwrap(),
        after.durable_files.checked_sub(before.durable_files).unwrap(),
    );
}

fn plan(
    suite_id: &str, template_id: &str, parameters: impl IntoIterator<Item = (&'static str, Vec<model::values::wire::Value>)>,
) -> model::planning::wire::SelectionPlan {
    model::planning::wire::SelectionPlan {
        selections: vec![model::planning::wire::RegisteredSelection {
            suite_id: suite_id.to_owned(),
            template_id: template_id.to_owned(),
            parameters: parameters
                .into_iter()
                .map(|(name, values)| model::planning::wire::RegisteredSelectionParametersEntry {
                    key: name.to_owned(),
                    value: Some(model::planning::wire::SelectionValues { values }),
                })
                .collect(),
            files: vec![],
        }],
        max_cases: 0,
        case_ids: vec![],
        cycle_id: String::new(),
    }
}

fn number(value: f64) -> model::values::wire::Value {
    model::values::wire::Value::from(&Value::Number(value))
}

fn padded_bundle(path: &Path, size: usize) -> Vec<u8> {
    let original = std::fs::read(path).unwrap();
    let bundle = Bundle::inspect(&original, Limits::default()).unwrap();
    assert!(original.len() < size, "the built fixture must fit the requested install size");
    let mut padding = vec![0; size - original.len()];
    for _ in 0..4 {
        let mut sources = bundle
            .manifest()
            .files
            .iter()
            .map(|file| Source { path: &file.path, bytes: bundle.file(&file.path).unwrap(), executable: file.executable.clone() })
            .collect::<Vec<_>>();
        sources.push(Source { path: "assets/benchmark-padding.bin", bytes: &padding, executable: None });
        let padded = Bundle::build(bundle.manifest().identity.clone(), &sources, Limits::default()).unwrap();
        let length = padded.bytes().len();
        if length == size {
            return padded.bytes().to_vec();
        }
        let next = padding.len().checked_add(size).unwrap().checked_sub(length).expect("bundle overhead fits");
        padding.resize(next, 0);
    }
    panic!("bundle padding did not converge on the requested byte count");
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn object(fields: impl IntoIterator<Item = (&'static str, Value)>) -> model::values::wire::Value {
    model::values::wire::Value::from(&Value::Object(fields.into_iter().map(|(key, value)| (key.to_owned(), value)).collect()))
}

/* === Tests === */
