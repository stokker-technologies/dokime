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
use std::{path::Path, time::Duration};

// Third Party Imports
use tokio_stream::StreamExt;

// Local Imports
use dokime_gui::{
    connection::{ConnectionSession, host::Session},
    native_session::{BackendSelection, DesktopSession, backend::Backend},
};
use dokime_sdk::{prelude::*, v1};

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
    pub owner: DesktopSession,
    pub session: Session,
    pub cycle: String,
    pub execution: String,
    plugin: Option<dokime_runtime_next::PluginProcess>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn start(root: &Path, restoring: bool) -> Self {
        // Start the Desktop's Current Embedded Host
        let owner = DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(root.to_owned()));
        owner.select(BackendSelection::OwnedLocal).await.unwrap();
        let session = owner.local_client().await.unwrap();
        let manifest = root.join("bench-fixture.json");
        if restoring {
            let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
            return Self {
                owner,
                session,
                cycle: saved["cycle"].as_str().unwrap().into(),
                execution: saved["execution"].as_str().unwrap().into(),
                plugin: None,
            };
        }

        // Admit the SDK Fixture as a Remote Plugin Through the Host Adapter
        let placement = dokime_runtime_next::PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stop_grace: Duration::from_secs(30),
            stream_window: std::num::NonZeroUsize::new(8).unwrap(),
            message_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let plugin = dokime_runtime_next::PluginProcess::start(&std::env::current_exe().unwrap(), true, &placement).await.unwrap();
        let _: v1::Plugin = session.unary("/dokime.v1.Plugins/Add", v1::AddRemotePlugin { url: plugin.endpoint().into() }).await.unwrap();
        let plan: v1::PreviewSelectionReply = session
            .unary(
                "/dokime.v1.Planning/MaterializeTemplate",
                v1::PlanningMaterializeTemplateRequest {
                    selection: Some(v1::RegisteredSelection {
                        suite_id: "bench.pipeline".into(),
                        template_id: "bench.pipeline.capture".into(),
                        ..Default::default()
                    }),
                },
            )
            .await
            .unwrap();
        let created: v1::CreateCycleReply = session
            .unary(
                "/dokime.v1.Execution/CreateCycle",
                v1::ExecutionCreateCycleRequest {
                    origins: plan.cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                    continue_automatically: true,
                    metadata: Some(v1::Metadata {
                        label: "Pipeline Bench".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let cycle = created.cycle_id;
        let execution = created.execution_ids[0].clone();
        std::fs::write(manifest, serde_json::to_vec(&serde_json::json!({"cycle": cycle, "execution": execution})).unwrap()).unwrap();
        Self {
            owner,
            session,
            cycle,
            execution,
            plugin: Some(plugin),
        }
    }

    pub async fn publish(&self, action: &str) {
        // Await the Producer's Question Before Releasing Its Next Publication
        let mut changes = self.session.watch_raw("/dokime.v1.Observation/WatchRuntime", &[]).await.unwrap();
        loop {
            let pending: v1::ListPendingPromptsReply = self
                .session
                .unary(
                    "/dokime.v1.Prompts/ListPending",
                    v1::PromptsListPendingRequest {
                        execution_id: Some(self.execution.clone()),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            if let Some(prompt) = pending.prompts.first() {
                let _: v1::PromptResponseReceipt = self
                    .session
                    .unary(
                        "/dokime.v1.Prompts/Respond",
                        v1::PromptsRespondRequest {
                            prompt_id: prompt.prompt_id.clone(),
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
        let request = v1::PlotsWatchRequest {
            requests: vec![v1::PlotRequest {
                stream: "bench.pipeline".into(),
                y_keys: vec!["y1".into()],
                scope: Some(v1::EvidenceScope {
                    execution_id: Some(self.execution.clone()),
                    ..Default::default()
                }),
                max_points: Some(32),
                ..Default::default()
            }],
        };
        let bytes = prost::Message::encode_to_vec(&request);
        let mut watch = self.session.watch_raw("/dokime.v1.Plots/Watch", &bytes).await.unwrap();
        while let Some(frame) = watch.next().await {
            let frame = <v1::PlotsWatchOutput as prost::Message>::decode(frame.unwrap().as_slice()).unwrap();
            if let Some(v1::plots_watch_output::Value::Item(frame)) = frame.value
                && frame
                    .updates
                    .iter()
                    .filter_map(|update| update.envelope.as_ref())
                    .filter_map(|envelope| envelope.payload.as_ref())
                    .flat_map(|payload| &payload.series)
                    .any(|series| series.raw_points >= ROWS as u64)
            {
                return;
            }
        }
        panic!("Benchmark History Must Become Visible Before Drawing");
    }

    pub async fn close(mut self) {
        self.owner.shutdown().await.unwrap();
        if let Some(plugin) = &mut self.plugin {
            plugin.close().await.unwrap();
        }
    }
}

/* === Public Functions === */

pub fn serve() -> dokime_sdk::Result<()> {
    let stream = Stream::<Row>::new("bench.pipeline", "Pipeline Samples").publish_on(publish);
    let template = Template::new("bench.pipeline.capture", "Pipeline Capture")
        .step(Step::new("Capture", async |_: &Context| std::future::pending().await))
        .stream(stream);
    Plugin::new("desktop-benchmark", "1.0.0")
        .suite(Suite::new("bench.pipeline", "Pipeline Bench").template(template))
        .prompt(Prompt::new("bench.next", "Next Publication", "Publish the Next Samples?").action("live", "Live").action("late", "Late"))
        .requires_host::<dokime_sdk::protocol::context::v1::PromptsCreateRequest>()
        .remote()
        .serve()
}

/* === Private Functions === */

async fn publish(context: &Context, publisher: Publisher<Row>) -> dokime_sdk::Result<()> {
    // Produce History Through the Same SDK Sample Carrier as Product Plugins
    let mut sequence = 0;
    let mut first = 0;
    let mut count = ROWS;
    let mut late = false;
    loop {
        for index in first..first + count {
            let source_ns = ORIGIN_S as i64 * 1_000_000_000 + index as i64 * 125_000_000 + if late { 500_000_000 } else { 0 };
            publisher
                .publish_at(
                    Row {
                        timestamp: source_ns as f64 / 1_000_000_000.0,
                        y0: (index % 8) as f64,
                        y1: if late { 150.0 } else { (index % 776) as f64 / 8.0 },
                        y2: ((index / 8) % 8) as f64,
                    },
                    Some(prost_types::Timestamp {
                        seconds: source_ns.div_euclid(1_000_000_000),
                        nanos: source_ns.rem_euclid(1_000_000_000) as i32,
                    }),
                )
                .await?;
        }
        sequence += count;
        let answer = context
            .prompt(dokime_sdk::protocol::context::v1::PromptsCreateRequest {
                prompt: format!("next-{sequence}"),
                prompt_type: "bench.next".into(),
                ..Default::default()
            })
            .await?;
        late = answer.action == "late";
        first = if late { ROWS - 480 + (sequence - ROWS) % 400 } else { sequence };
        count = 8;
    }
}

/* === Tests === */
