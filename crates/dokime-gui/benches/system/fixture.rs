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

// Standard Library Imports
use std::{num::NonZeroUsize, path::Path, sync::Arc, time::Duration};

// Third Party Imports
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_stream::StreamExt;

// Local Imports

use dokime_sdk::client::PluginPlacement;
use dokime_sdk::client::PluginRemote;
use dokime_sdk::client::PluginValue;
use dokime_sdk::client::PluginsCreateOptions;
use dokime_sdk::client::plugin_placement::Kind as PluginPlacementKind;
use dokime_sdk::client::{PluginState, PluginsGetOptions, PluginsWatchOptions};

use dokime_gui::connection::host::{Connection, ConnectionSettings, Session};
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_runtime::{ControlledClock, HostSettings, PlacementSettings, PluginProcess, services, storage_defaults};
use dokime_sdk as model;
use dokime_sdk::StorageTransferSettings;
use dokime_sdk::client::CasesCreateResponse;
use dokime_sdk::client::CyclesCreateResponse;
use dokime_sdk::client::ExecutionsGetResponse;
use dokime_sdk::client::ExecutionsListResponse;
use dokime_sdk::client::PlotsSnapshotsWatchResponse;
use dokime_sdk::client::RegisteredSelection;
use dokime_sdk::client::SelectionPlan;
use dokime_sdk::client::{ClientSettings, PlotsPlotRequest};
use dokime_sdk::plugin::HostOperation;
use dokime_sdk::{Result, prelude::*};

use dokime_sdk::{
    AxisScale, Card, ContourRender, Document, EvidenceScope, FieldReference, GridAxisEntry, GridChart, Metadata, Plot, PlotKind, PlotStyle,
    Reduction, Section,
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
    plugin: Option<PluginProcess>,
    control: Option<Mutex<BufReader<TcpStream>>>,
}

struct PublicationControl(Mutex<BufReader<TcpStream>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn start(root: &Path, restoring: bool) -> Self {
        // Start the Desktop's Current Embedded Host
        let settings = ConnectionSettings {
            host: HostSettings {
                capture: dokime_runtime::CaptureSettings::default(),
                clock: Arc::new(ControlledClock::new(ORIGIN_S as i64 * 1_000_000_000)),
                storage: storage_defaults(),
                placement: PlacementSettings {
                    connect_timeout: Duration::from_secs(30),
                    reconnect_delay: Duration::from_millis(25),
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
                control: None,
            };
        }

        // Admit the SDK Fixture as a Remote Plugin Through the Host Adapter
        let placement = PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let executable = std::env::current_exe().unwrap();
        let control = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plugin = PluginProcess::start_with_environment(
            &executable,
            true,
            &placement,
            &[("DOKIME_DESKTOP_BENCH_CONTROL".into(), control.local_addr().unwrap().to_string())],
        )
        .await
        .unwrap();
        let options = PluginsCreateOptions {
            context: session.call_options().unwrap(),
            ..Default::default()
        };
        let placement = PluginPlacement {
            kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
        };
        let admitted = session
            .execute(session.client().plugins.create_with("bench.pipeline", placement, PluginValue { enabled: true }, options))
            .await
            .unwrap();
        let mut entry = admitted.entry.expect("Admit Benchmark Fixture: Missing Entry");
        let options = PluginsWatchOptions {
            name: Some(entry.name.clone()),
            context: session.call_options().unwrap(),
            ..Default::default()
        };
        let mut watching = session.execute(session.client().plugins.watch_with(options)).await.unwrap();
        while entry.state != PluginState::Ready as i32 {
            assert!(entry.failure.is_none(), "Admit Benchmark Fixture: {:?}", entry.failure);
            assert_ne!(entry.state, PluginState::Stopped as i32);
            session.execute(watching.next()).await.unwrap().expect("Admit Benchmark Fixture: Watch Closed");
            let options = PluginsGetOptions {
                context: session.call_options().unwrap(),
                ..Default::default()
            };
            entry = session.execute(session.client().plugins.get_with(entry.name, options)).await.unwrap().entry.unwrap();
        }
        drop(watching);
        let plan: CasesCreateResponse = session
            .unary(
                "/dokime.host.v1.Cases/Create",
                CasesCreateRequest {
                    plan: Some(SelectionPlan {
                        selections: vec![RegisteredSelection {
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
        let created: CyclesCreateResponse = session
            .unary(
                "/dokime.host.v1.Cycles/Create",
                CyclesCreateRequest {
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
        let executions: ExecutionsListResponse = session
            .unary(
                "/dokime.host.v1.Executions/List",
                ExecutionsListRequest {
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
            control: Some(Mutex::new(BufReader::new(control.accept().await.unwrap().0))),
        }
    }

    pub async fn publish(&self, action: &str) {
        // Benchmark Synchronization Is Independent of an Operator's Answer Deadline.
        let mut control = self.control.as_ref().expect("Publish Active Benchmark History: Retain Its Producer").lock().await;
        control.get_mut().write_all(format!("{action}\n").as_bytes()).await.unwrap();
        let mut ready = String::new();
        assert_ne!(control.read_line(&mut ready).await.unwrap(), 0, "Publish Benchmark History: Await Producer Completion");
        assert!(ready.starts_with("ready "), "Publish Benchmark History: {ready}");
    }

    pub async fn prepared(&self) {
        let mut control = self.control.as_ref().expect("Prepare Active Benchmark History: Retain Its Producer").lock().await;
        let mut ready = String::new();
        assert_ne!(control.read_line(&mut ready).await.unwrap(), 0, "Prepare Benchmark History: Await Producer Completion");
        assert_eq!(ready.trim(), format!("ready {}", rows()), "Prepare Benchmark History: Retain Every Sample");
        drop(control);
        // Sample Visibility, Rather Than an Elapsed Delay, Establishes Readiness
        let request = SnapshotsWatchRequest {
            requests: vec![PlotsPlotRequest {
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
        let mut watch = self.session.watch_raw("/dokime.host.v1.plots.Snapshots/Watch", &bytes).await.unwrap();
        while let Some(frame) = watch.next().await {
            let frame = <PlotsSnapshotsWatchResponse as prost::Message>::decode(frame.unwrap().as_slice()).unwrap();
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
        let detail: ExecutionsGetResponse = self
            .session
            .unary(
                "/dokime.host.v1.Executions/Get",
                ExecutionsGetRequest {
                    name: self.execution.clone(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        panic!("Prepare Benchmark History: Watch Closed Before History Became Visible: {:?}", detail.execution);
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
    let observer = Observer::new(&stream, observe).interval_setting("sample_interval");
    let template = Template::new("bench.pipeline.capture", "Pipeline Capture")
        .initialize(async |context: &Context| {
            context.set(History::default());
            let address = std::env::var("DOKIME_DESKTOP_BENCH_CONTROL")
                .map_err(|error| model::Error::Invalid(format!("Connect Benchmark Producer Control: {error}")))?;
            let stream = TcpStream::connect(address)
                .await
                .map_err(|error| model::Error::Invalid(format!("Connect Benchmark Producer Control: {error}")))?;
            context.set(PublicationControl(Mutex::new(BufReader::new(stream))));
            Ok(())
        })
        .step(Step::new("Capture", publish))
        .stream(stream)
        .observer(observer)
        .document(document(30))
        .document(document(100));
    Plugin::new("desktop.benchmark", "1.0.0")
        .setting(model::SettingDeclaration {
            name: "sample_interval".into(),
            title: "Sample Interval".into(),
            description: "Canonical sample cadence for the three-day history".into(),
            default: Some(model::SettingValue {
                value: Some(model::setting_value::Value::Duration(prost_types::Duration {
                    seconds: sample_ns() / 1_000_000_000,
                    nanos: (sample_ns() % 1_000_000_000) as i32,
                })),
            }),
        })
        .suite(Suite::new("bench.pipeline", "Pipeline Bench").template(template))
        .requires_host(HostOperation::TimeSleep)
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
                body: Some(model::card::Body::Plot(plot)),
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
        let target = sequence + count;
        let history = context.get::<History>()?;
        let pending = target.saturating_sub(history.published.load(Ordering::SeqCst));
        // A Controlled Sleep Settles After Every Tick It Spans.
        context.sleep(Duration::from_nanos(pending as u64 * sample_ns() as u64)).await?;
        assert_eq!(history.published.load(Ordering::SeqCst), target, "Publish Benchmark History: Retain the Requested Row Count");
        sequence += count;
        let control = context.get::<PublicationControl>()?;
        let mut control = control.0.lock().await;
        control
            .get_mut()
            .write_all(format!("ready {sequence}\n").as_bytes())
            .await
            .map_err(|error| model::Error::Invalid(format!("Announce Benchmark Publication: {error}")))?;
        let mut action = String::new();
        if control.read_line(&mut action).await.map_err(|error| model::Error::Invalid(format!("Await Benchmark Publication: {error}")))?
            == 0
        {
            return Err(model::Error::Invalid("Await Benchmark Publication: Control Connection Closed".into()));
        }
        late = action.trim() == "late";
        first = if late { rows() - 480 + (sequence - rows()) % 400 } else { sequence };
        count = 8;
    }
}

#[derive(Default)]
struct History {
    next: AtomicUsize,
    late: AtomicBool,
    published: AtomicUsize,
}

async fn observe(context: &Context) -> Result<Row> {
    if context.execution_id().is_empty() {
        return Ok(Row {
            timestamp: ORIGIN_S,
            y0: 0.0,
            y1: 0.0,
            y2: 0.0,
        });
    }
    let history = context.get::<History>()?;
    let index = history.next.fetch_add(1, Ordering::SeqCst);
    let late = history.late.load(Ordering::SeqCst);
    let source_ns = ORIGIN_S as i64 * 1_000_000_000 + index as i64 * sample_ns() + if late { 4 * sample_ns() } else { 0 };
    history.published.fetch_add(1, Ordering::SeqCst);
    Ok(Row {
        timestamp: source_ns as f64 / 1_000_000_000.0,
        y0: (index % 8) as f64,
        y1: if late { 150.0 } else { (index % 776) as f64 / 8.0 },
        y2: ((index / 8) % 8) as f64,
    })
}

/* === Tests === */
