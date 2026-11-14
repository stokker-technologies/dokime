/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SDK-Owned Three-Day History and Condition-Driven Live Publications

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use dokime_protocol::client;

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use harmos_protocol::v1::Sample;
use prost::Message;
use prost_types::Timestamp;
use serde_json::Value;
use tokio_stream::StreamExt;

// Local Imports
use super::sample_carrier::Producer;
use dokime_gui::connection::host::{Connection, ConnectionSettings, Session};
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings, services, storage_defaults};
use dokime_sdk::client::{ClientSettings, StorageTransferSettings};
use dokime_sdk::{Result, clock::TimeSleepRequest, prelude::*, protocol::plugin::host::v1::PromptsCreateRequest, v1};

use dokime_protocol::v1::{
    AxisScale, Card, ContourRender, Document, EvidenceScope, FieldReference, GridAxisEntry, GridChart, Metadata, Plot, PlotKind,
    PlotRequest, PlotStyle, Reduction, Section,
};

use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/* === Constants === */

pub const ROWS: usize = 3 * 24 * 60 * 60 * 8;
pub const ORIGIN_S: f64 = 1_767_225_600.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime_sdk::Row)]
#[harmos(package = "desktop.benchmark.v1")]
struct Row {
    timestamp: f64,
    y0: f64,
    y1: f64,
    y2: f64,
}

pub struct Fixture {
    pub owner: Connection,
    pub session: Session,
    pub cycle: String,
    pub execution: String,
    plugin: Option<Producer>,
    mode: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn start(root: &Path, restoring: bool) -> Self {
        // Start the Desktop's Current Embedded Host
        let settings = ConnectionSettings {
            host: HostSettings {
                clock: Arc::new(ControlledClock::new(ORIGIN_S as i64 * 1_000_000_000)),
                storage: storage_defaults(),
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
        };
        let mut owner = Connection::new(settings).unwrap();
        let session = owner.start(root).await.unwrap();
        let manifest = root.join("bench-fixture.json");
        if restoring {
            let saved: Value = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
            return Self {
                owner,
                session,
                cycle: saved["cycle"].as_str().unwrap().into(),
                execution: saved["execution"].as_str().unwrap().into(),
                plugin: None,
                mode: root.join("bench-mode"),
            };
        }

        // Admit the SDK Fixture as a Remote Plugin Through the Host Adapter
        let placement = PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let mode = root.join("bench-mode");
        let plugin = Producer::start(&placement, &[("DOKIME_BENCH_MODE".into(), mode.to_str().unwrap().into())]).await.unwrap();
        let _: client::Plugin =
            session.unary("/dokime.client.v1.Plugins/Add", client::AddRemotePlugin { url: plugin.endpoint().into() }).await.unwrap();
        let plan: client::CasesCreateResponse = session
            .unary(
                "/dokime.client.v1.Cases/Create",
                client::CasesCreateRequest {
                    plan: Some(client::SelectionPlan {
                        selections: vec![client::RegisteredSelection {
                            suite_id: "bench.pipeline".into(),
                            template_id: "bench.pipeline.capture".into(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    validate_only: true,
                    request_id: "bench-preview".into(),
                },
            )
            .await
            .unwrap();
        let created: client::CyclesCreateResponse = session
            .unary(
                "/dokime.client.v1.Cycles/Create",
                client::CyclesCreateRequest {
                    origins: plan.cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                    continue_automatically: true,
                    request_id: "bench-cycle".into(),
                    metadata: Some(Metadata {
                        label: "Pipeline Bench".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let cycle = created.cycle.unwrap().cycle_id;
        let executions: client::ExecutionsListResponse = session
            .unary(
                "/dokime.client.v1.Executions/List",
                client::ExecutionsListRequest {
                    parent: cycle.clone(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let execution = executions.executions[0].summary.as_ref().unwrap().execution_id.clone();
        std::fs::write(manifest, serde_json::to_vec(&serde_json::json!({"cycle": cycle, "execution": execution})).unwrap()).unwrap();
        Self {
            owner,
            session,
            cycle,
            execution,
            plugin: Some(plugin),
            mode,
        }
    }

    pub async fn publish(&self, action: &str) {
        std::fs::write(&self.mode, action).unwrap();
        // Await the Producer's Question Before Releasing Its Next Publication
        let mut changes = self.session.watch_raw("/dokime.client.v1.backend.Overview/Watch", &[]).await.unwrap();
        loop {
            let pending: client::PromptsListResponse = self
                .session
                .unary(
                    "/dokime.client.v1.Prompts/List",
                    client::PromptsListRequest {
                        execution_id: Some(self.execution.clone()),
                        pending_only: true,
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            if let Some(prompt) = pending.prompts.first() {
                let _: client::prompts::AnswersCreateResponse = self
                    .session
                    .unary(
                        "/dokime.client.v1.prompts.Answers/Create",
                        client::prompts::AnswersCreateRequest {
                            parent: prompt.prompt.as_ref().unwrap().prompt_id.clone(),
                            request_id: format!("bench-answer-{}", prompt.prompt.as_ref().unwrap().prompt_id),
                            action: action.into(),
                            accepted: true,
                            ..Default::default()
                        },
                    )
                    .await
                    .unwrap();
                return;
            }
            changes.next().await.expect("Benchmark Host Watch Remains Open").unwrap();
        }
    }

    pub async fn prepared(&self) {
        // Sample Visibility, Rather Than an Elapsed Delay, Establishes Readiness
        let request = client::plots::SnapshotsWatchRequest {
            requests: vec![PlotRequest {
                stream: "bench.pipeline".into(),
                y_keys: vec!["y1".into()],
                scope: Some(EvidenceScope {
                    execution_id: Some(self.execution.clone()),
                    ..Default::default()
                }),
                max_points: Some(32),
                ..Default::default()
            }],
            ..Default::default()
        };
        let bytes = prost::Message::encode_to_vec(&request);
        let mut watch = self.session.watch_raw("/dokime.client.v1.plots.Snapshots/Watch", &bytes).await.unwrap();
        while let Some(frame) = watch.next().await {
            let frame = <client::plots::SnapshotsWatchResponse as prost::Message>::decode(frame.unwrap().as_slice()).unwrap();
            if frame
                .updates
                .iter()
                .filter_map(|update| update.envelope.as_ref())
                .filter_map(|envelope| envelope.payload.as_ref())
                .flat_map(|payload| &payload.series)
                .any(|series| series.raw_points >= rows() as u64)
            {
                return;
            }
        }
        panic!("Benchmark History Must Become Visible Before Drawing");
    }

    pub async fn close(mut self) {
        let host_close = self.owner.close().await;
        let plugin_close = if let Some(plugin) = &mut self.plugin { plugin.close().await } else { Ok(()) };
        host_close.unwrap();
        plugin_close.unwrap();
    }
}

/* === Public Functions === */

pub fn serve() -> Result<()> {
    let stream = Stream::<Row>::new("bench.pipeline", "Pipeline Samples");
    let observer = Observer::new(&stream, observe).every_ms(125);
    let template = Template::new("bench.pipeline.capture", "Pipeline Capture")
        .initialize(async |context: &Context| {
            context.set(History::default());
            Ok(())
        })
        .step(Step::new("Capture", publish))
        .stream(stream)
        .observer(observer)
        .document(document(30))
        .document(document(100));
    Plugin::new("desktop.benchmark", "1.0.0")
        .suite(Suite::new("bench.pipeline", "Pipeline Bench").template(template))
        .prompt(Prompt::new("bench.next", "Next Publication", "Publish the Next Samples?").action("live", "Live").action("late", "Late"))
        .requires_host::<PromptsCreateRequest>()
        .requires::<TimeSleepRequest>()
        .remote()
        .serve()
}

pub fn rows() -> usize {
    std::env::var("DOKIME_DESKTOP_HISTORY_ROWS").ok().map(|value| value.parse().expect("Benchmark Row Count")).unwrap_or(ROWS)
}
pub fn sample_ns() -> i64 {
    259_200_000_000_000 / rows() as i64
}

/* === Private Functions === */

fn document(count: usize) -> Document {
    let kinds = [
        PlotKind::FrequencySeries,
        PlotKind::Xy,
        PlotKind::Trajectory,
        PlotKind::Contour,
        PlotKind::GridChart,
        PlotKind::Bar,
        PlotKind::Pie,
        PlotKind::Histogram,
        PlotKind::BoxPlot,
    ];
    let field = |name: &str| FieldReference::new("bench.pipeline").field(name);
    let cards = (0..count)
        .map(|index| {
            let kind = if index < count - 9 { PlotKind::TimeSeries } else { kinds[index - count + 9] };
            let plot = Plot {
                metadata: Some(Metadata {
                    identifier: format!("desktop.plot.{index}"),
                    label: format!("Plot {}", index + 1),
                    ..Default::default()
                }),
                kind: kind as i32,
                style: PlotStyle::Line as i32,
                x_scale: AxisScale::Linear as i32,
                y_scale: AxisScale::Linear as i32,
                x: Some(field(if index <= count - 9 { "timestamp" } else { "y0" })),
                y: vec![if kind == PlotKind::Pie { field("y1").mean() } else { field("y1") }],
                z: Some(field("y2")),
                reduction: Some(Reduction::Mean as i32),
                render: ContourRender::Cells as i32,
                input_samples: (kind == PlotKind::FrequencySeries).then_some(512),
                sample_rate_hz: (kind == PlotKind::FrequencySeries).then_some(1e9 / sample_ns() as f64),
                histogram_bins: Some(32),
                grid: (kind == PlotKind::GridChart).then(|| GridChart {
                    row: Some(field("y0")),
                    column: Some(field("y2")),
                    frame: Some(field("timestamp")),
                    rows: (0..8)
                        .map(|index| GridAxisEntry {
                            key: index.to_string(),
                            label: index.to_string(),
                            order: index,
                            ..Default::default()
                        })
                        .collect(),
                    columns: (0..8)
                        .map(|index| GridAxisEntry {
                            key: index.to_string(),
                            label: index.to_string(),
                            order: index,
                            ..Default::default()
                        })
                        .collect(),
                    summary_label: "Value".into(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            Card {
                metadata: Some(Metadata {
                    identifier: format!("card-{index}"),
                    label: format!("Plot {}", index + 1),
                    ..Default::default()
                }),
                body: Some(v1::card::Body::Plot(plot)),
                ..Default::default()
            }
        })
        .collect();
    Document {
        metadata: Some(Metadata {
            identifier: format!("desktop-{count}"),
            label: "Mixed Plots".into(),
            ..Default::default()
        }),
        sections: vec![Section {
            metadata: Some(Metadata {
                identifier: "plots".into(),
                label: "Mixed Plots".into(),
                ..Default::default()
            }),
            cards,
            ..Default::default()
        }],
        ..Default::default()
    }
}

async fn publish(context: &Context) -> Result<()> {
    // Advance the Controlled Host Clock Through Real Observer Ticks.
    let mut sequence = 0;
    let mut first = 0;
    let mut count = rows();
    let mut late = false;
    loop {
        context.get::<History>()?.next.store(first, Ordering::SeqCst);
        context.get::<History>()?.late.store(late, Ordering::SeqCst);
        let chunks = count.div_ceil(4096);
        let ticks = if sequence == 0 { chunks - 1 } else { chunks };
        context.sleep(Duration::from_millis(ticks as u64 * 125)).await?;
        sequence += count;
        let answer = context
            .prompt(PromptsCreateRequest {
                prompt: format!("next-{sequence}"),
                prompt_type: "bench.next".into(),
                ..Default::default()
            })
            .await?;
        late = answer.action == "late";
        first = if late { rows() - 480 + (sequence - rows()) % 400 } else { sequence };
        count = 8;
    }
}

#[derive(Default)]
struct History {
    next: AtomicUsize,
    late: AtomicBool,
}

async fn observe(context: &Context) -> Result<Row> {
    let history = context.get::<History>()?;
    let index = history.next.fetch_add(1, Ordering::SeqCst);
    let late = history.late.load(Ordering::SeqCst);
    let source_ns = ORIGIN_S as i64 * 1_000_000_000 + index as i64 * sample_ns() + if late { 4 * sample_ns() } else { 0 };
    Ok(Row {
        timestamp: source_ns as f64 / 1_000_000_000.0,
        y0: (index % 8) as f64,
        y1: if late { 150.0 } else { (index % 776) as f64 / 8.0 },
        y2: ((index / 8) % 8) as f64,
    })
}

pub fn samples(_stream: &str, first: u64, _at: &Timestamp) -> Vec<Sample> {
    let first = first as usize;
    let history_rows = rows();
    let sample_ns = sample_ns();
    let count = if first < history_rows { (history_rows - first).min(4096) } else { 8 };
    let late = std::env::var_os("DOKIME_BENCH_MODE").and_then(|path| std::fs::read_to_string(path).ok()).is_some_and(|mode| mode == "late");
    (first..first + count)
        .map(|sequence| {
            let index = if late { history_rows - 480 + (sequence - history_rows) % 400 } else { sequence };
            let source_ns = ORIGIN_S as i64 * 1_000_000_000 + index as i64 * sample_ns + if late { 4 * sample_ns } else { 0 };
            let row = Row {
                timestamp: source_ns as f64 / 1_000_000_000.0,
                y0: (index % 8) as f64,
                y1: if late { 150.0 } else { (index % 776) as f64 / 8.0 },
                y2: ((index / 8) % 8) as f64,
            };
            Sample {
                sequence: sequence as u64 + 1,
                at: Some(Timestamp {
                    seconds: source_ns.div_euclid(1_000_000_000),
                    nanos: source_ns.rem_euclid(1_000_000_000) as i32,
                }),
                row: row.encode_to_vec(),
            }
        })
        .collect()
}

/* === Tests === */
