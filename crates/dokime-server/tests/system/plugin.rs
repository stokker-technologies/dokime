/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
// Third Party Imports
use sha2::{Digest, Sha256};
// Local Imports
use crate::support::{self, host::Host};
use dokime_runtime::Value;
use dokime_client::{Client, ResponseStream, contracts};

/* === Constants === */
const STARTUP_SUITE: &str = "obc.charger.startup";
const STARTUP_TEMPLATE: &str = "obc.charger.startup.sequence";
const STARTUP_SAMPLES: &str = "obc.charger.startup.samples";
const STARTUP_STREAM_LOG: &str = "obc.charger.startup/stream.log";
const STARTUP_SAMPLE_COUNT: u64 = 40;
const STEADY_SUITE: &str = "obc.charger.steady";
const STEADY_TEMPLATE: &str = "obc.charger.steady.measurement";
const STEADY_SAMPLES: &str = "obc.charger.steady.samples";
const STEADY_EVENTS: &str = "obc.charger.steady.events";
const STEADY_STREAM_LOG: &str = "obc.charger.steady/stream.log";
const STEADY_HOST_SYSTEM: &str = "host.system";
const STEADY_SAMPLE_COUNT: u64 = 240;
const STEADY_PROMPT: &str = "obc.charger.steady.check";
const STEADY_ACCEPTANCE: &str = "obc.charger.steady.acceptance";
const STEADY_SUMMARY: &str = "charger/steady-state-summary.json";
const HOST_FAST_STREAM: &str = "host.system";
const HOST_STREAMS: &[&str] = &[
    "host.system",
    "host.network",
    "host.wifi",
    "host.wifi.latency",
    "host.wifi.channel_occupancy",
    "host.bluetooth",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
type Changes = ResponseStream<contracts::observation::RuntimeStateFrame>;

/// Re-runs `probe` after every runtime frame the host pushes until it yields.
async fn on_change<T>(changes: &mut Changes, mut probe: impl AsyncFnMut() -> Option<T>) -> T {
    loop {
        if let Some(value) = probe().await {
            return value;
        }
        changes
            .message()
            .await
            .unwrap()
            .expect("the runtime watch stays open");
    }
}

async fn wait_prompt(
    changes: &mut Changes,
    prompts: &mut Client,
    cycle_id: &str,
) -> contracts::prompts::PendingPromptIdentity {
    on_change(changes, async || {
        let page = prompts
            .prompts_list_pending(contracts::prompts::ListPending {
                cycle_id: Some(cycle_id.to_owned()),
                ..Default::default()
            })
            .await
            .unwrap();
        page.prompts.into_iter().next()
    })
    .await
}

async fn wait_completed(
    changes: &mut Changes,
    execution: &mut Client,
    cycle_id: &str,
) -> contracts::execution::CycleSummary {
    on_change(changes, async || {
        let cycle = execution
            .execution_get_cycle(contracts::execution::GetCycle {
                cycle_id: cycle_id.to_owned(),
            })
            .await
            .unwrap();
        matches!(
            cycle.status,
            contracts::execution::CycleStatus::Completed
                | contracts::execution::CycleStatus::Failed
                | contracts::execution::CycleStatus::Stopped
        )
        .then_some(cycle)
    })
    .await
}

async fn execution_id(changes: &mut Changes, execution: &mut Client, cycle_id: &str) -> String {
    on_change(changes, async || {
        let page = execution
            .execution_list_executions(contracts::execution::ListExecutions {
                cycle_id: cycle_id.to_owned(),
                ..Default::default()
            })
            .await
            .unwrap();
        page.executions
            .into_iter()
            .next()
            .map(|item| item.execution_id)
    })
    .await
}

fn field(sample: &contracts::telemetry::TelemetrySample, name: &str) -> String {
    let Value::Object(mut fields) = sample.values.as_ref().unwrap().to_core().unwrap() else {
        panic!("sample values must be an object");
    };
    match fields.remove(name) {
        Some(Value::Text(text)) => text,
        other => panic!("sample field {name:?} is not text: {other:?}"),
    }
}

fn number(value: f64) -> contracts::value::Value {
    contracts::value::Value::from(&Value::Number(value))
}

fn text(value: &str) -> contracts::value::Value {
    contracts::value::Value::from(&Value::Text(value.to_owned()))
}

fn boolean(value: bool) -> contracts::value::Value {
    contracts::value::Value::from(&Value::Bool(value))
}

fn selection(
    suite_id: &str,
    template_id: &str,
    parameters: impl IntoIterator<Item = (&'static str, contracts::value::Value)>,
) -> contracts::planning::SelectionPlan {
    contracts::planning::SelectionPlan {
        selections: vec![contracts::planning::RegisteredSelection {
            suite_id: suite_id.to_owned(),
            template_id: template_id.to_owned(),
            parameters: parameters
                .into_iter()
                .map(
                    |(name, value)| contracts::planning::RegisteredSelectionParametersEntry {
                        key: name.to_owned(),
                        value: Some(contracts::planning::SelectionValues {
                            values: vec![value],
                        }),
                    },
                )
                .collect(),
            files: vec![],
        }],
        max_cases: 0,
        case_ids: vec![],
        cycle_id: String::new(),
    }
}

fn scope(cycle_id: &str, execution_id: &str) -> contracts::common::EvidenceScope {
    contracts::common::EvidenceScope {
        cycle_id: Some(cycle_id.to_owned()),
        execution_id: Some(execution_id.to_owned()),
        ..Default::default()
    }
}

async fn suite(catalog: &mut Client, suite_id: &str) -> contracts::catalog::SuiteDefinition {
    catalog
        .catalog_get_suite(contracts::catalog::GetSuite {
            suite_id: suite_id.to_owned(),
        })
        .await
        .unwrap()
}

async fn create_cycle(
    planning: &mut Client,
    execution: &mut Client,
    plan: contracts::planning::SelectionPlan,
    identifier: &str,
    expected_cases: usize,
) -> String {
    let preview = planning
        .planning_preview_selection(contracts::planning::PreviewSelection {
            plan: Some(plan.clone()),
        })
        .await
        .unwrap();
    assert_eq!(preview.cases.len(), expected_cases);
    let cycle_id = execution
        .execution_create_cycle(contracts::execution::CreateCycle {
            metadata: Some(contracts::common::Metadata {
                identifier: identifier.to_owned(),
                label: identifier.to_owned(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .cycle_id;
    execution
        .execution_append_planned_cases(contracts::execution::AppendPlannedCases {
            cycle_id: cycle_id.clone(),
            plan: Some(plan),
            context: None,
        })
        .await
        .unwrap();
    cycle_id
}

async fn run_cycle(execution: &mut Client, cycle_id: &str) {
    let reply = execution
        .execution_run_cycle(contracts::execution::RunCycle {
            cycle_id: cycle_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(matches!(
        reply.outcome,
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
}

async fn read_file(
    content: &mut Client,
    execution_id: &str,
    relative_path: &str,
) -> contracts::content::ExecutionFileWindow {
    content
        .content_read_execution_file(contracts::content::ReadExecutionFile {
            execution_id: execution_id.to_owned(),
            relative_path: relative_path.to_owned(),
            offset: 0,
            max_bytes: 8192,
        })
        .await
        .unwrap()
}

async fn list_file(
    content: &mut Client,
    execution_id: &str,
    relative_path: &str,
) -> contracts::content::ExecutionFileMetadata {
    let page = content
        .content_list_execution_files(contracts::content::ListExecutionFiles {
            execution_id: execution_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap();
    page.files
        .into_iter()
        .find(|file| file.relative_path == relative_path)
        .unwrap_or_else(|| panic!("execution file {relative_path} was not present"))
}

fn value_object(
    fields: impl IntoIterator<Item = (&'static str, contracts::value::Value)>,
) -> contracts::value::Value {
    let values = fields
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_core().unwrap()))
        .collect();
    contracts::value::Value::from(&Value::Object(values))
}

async fn serves_startup(client: &Client) -> bool {
    client
        .catalog_get_suite(contracts::catalog::GetSuite {
            suite_id: STARTUP_SUITE.to_owned(),
        })
        .await
        .is_ok()
}

async fn status_ready(client: &Client) {
    let frame = client.settings_get_settings(Default::default()).await.unwrap();
    let card = frame.cards.iter().find(|card| card.plugin_id == "obc-plugin").expect("the OBC settings card is registered");
    assert_eq!(card.state, contracts::settings::SettingsCardState::Ready, "the native status operation serves the installed plugin");
}

async fn installations(client: &Client) -> Vec<contracts::plugins::PluginInstallation> {
    client
        .plugins_list_installations(Default::default())
        .await
        .unwrap()
        .installations
}

/// Stages `bytes` in the plugin store and consents to them.
async fn install(client: &Client, bytes: &[u8]) -> contracts::plugins::PluginInstallation {
    use contracts::plugins::{StagePluginFrame, StagePluginFrameFrame as Frame};
    let sha256 = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut frames = vec![StagePluginFrame {
        frame: Some(Frame::Header(contracts::plugins::StagePluginHeader {
            size_bytes: bytes.len() as u64,
            sha256: sha256.clone(),
        })),
    }];
    for (index, chunk) in bytes.chunks(64 * 1024).enumerate() {
        frames.push(StagePluginFrame {
            frame: Some(Frame::Chunk(contracts::plugins::StagePluginChunk {
                offset: (index * 64 * 1024) as u64,
                data: chunk.to_vec(),
            })),
        });
    }
    let staged = client
        .plugins_stage(tokio_stream::iter(frames))
        .await
        .unwrap();
    client
        .plugins_consent(contracts::plugins::Consent {
            install_id: staged.install_id,
            expected_sha256: sha256,
        })
        .await
        .unwrap()
}

/// The file under `root` the store holds with exactly `bytes`.
fn installed_copy(root: &Path, bytes: &[u8]) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() {
            if let Some(found) = installed_copy(&path, bytes) {
                return Some(found);
            }
        } else if std::fs::read(&path).is_ok_and(|content| content == bytes) {
            return Some(path);
        }
    }
    None
}

/// Scene: a steady sweep runs through its prompt and leaves its execution
/// evidence: criteria, stream log, traceability, paged samples, pushed rows,
/// host streams, the document plot, records and the summary artifact.
async fn steady_sweep(client: &Client, changes: &mut Changes) {
    let mut catalog = client.clone();
    let steady = suite(&mut catalog, STEADY_SUITE).await;
    let template = steady
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STEADY_TEMPLATE)
        })
        .expect("steady template is registered");
    let mut planning = client.clone();
    let plan = selection(
        STEADY_SUITE,
        &template.template_id,
        [
            ("ac_voltage_v", number(230.0)),
            ("load_ohm", number(100.0)),
            ("frequency_hz", number(1.0)),
            ("duration_s", number(STEADY_SAMPLE_COUNT as f64 * 0.1)),
            ("sample_interval_s", number(0.1)),
        ],
    );
    let mut execution = client.clone();
    let cycle_id = create_cycle(
        &mut planning,
        &mut execution,
        plan,
        "obc.charger.steady.sweep",
        1,
    )
    .await;
    run_cycle(&mut execution, &cycle_id).await;
    let execution_id = execution_id(changes, &mut execution, &cycle_id).await;
    let execution_scope = scope(&cycle_id, &execution_id);

    // The sidecar observer runs independently of the prompt step. A non-empty
    // execution-scoped page proves that evidence is visible while the run is
    // still live, before the prompt is answered or the report is finalized.
    let telemetry = client.clone();
    // Bounded readiness poll: the plugin process exposes no signal for its
    // first observer sample.
    let live_deadline = Instant::now() + Duration::from_secs(10);
    let live_samples = loop {
        let page = telemetry
            .telemetry_list_samples(contracts::telemetry::ListSamples {
                streams: vec![STEADY_SAMPLES.into()],
                scope: Some(execution_scope.clone()),
                // Headroom, not a count: the stream measures at its declared
                // cadence for as long as the prompt is pending, so this page is
                // sized to hold everything rather than to predict how much.
                limit: 5_000,
                ..Default::default()
            })
            .await
            .unwrap();
        if !page.samples.is_empty() {
            break page;
        }
        assert!(
            Instant::now() < live_deadline,
            "sidecar telemetry was not visible while execution was running"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    assert!(live_samples.samples.len() < STEADY_SAMPLE_COUNT as usize);

    let mut prompts = client.clone();
    let pending = wait_prompt(changes, &mut prompts, &cycle_id).await;
    let definition = pending.definition.as_ref().expect("prompt definition");
    assert_eq!(definition.prompt_type, STEADY_PROMPT);
    let prompt_scope = pending.scope.as_ref().expect("prompt scope");
    assert_eq!(prompt_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(
        prompt_scope.execution_id.as_deref(),
        Some(execution_id.as_str())
    );

    // Hold the prompt until two host ticks are observable. Charger sample
    // time can advance faster than the host collector's wall-clock cadence.
    // Bounded readiness poll: host collectors tick on wall time and expose no
    // signal per row.
    let host_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let page = telemetry
            .telemetry_list_samples(contracts::telemetry::ListSamples {
                streams: vec![STEADY_HOST_SYSTEM.into()],
                scope: Some(execution_scope.clone()),
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();
        if page.samples.len() >= 2 {
            break;
        }
        assert!(
            Instant::now() < host_deadline,
            "declared host telemetry did not recur while prompt was pending"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    prompts
        .prompts_respond(contracts::prompts::Respond {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([
                ("operator_note", text("embedded-sidecar-confirmed")),
                ("accepted", boolean(true)),
            ])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(changes, &mut execution, &cycle_id).await;
    assert_eq!(cycle.status, contracts::execution::CycleStatus::Completed);
    let summary = execution
        .execution_list_executions(contracts::execution::ListExecutions {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .executions
        .into_iter()
        .next()
        .expect("steady execution summary");
    assert_eq!(summary.execution_id, execution_id);
    assert_eq!(
        summary.status,
        contracts::execution::ExecutionStatus::Passed
    );

    let steps = execution
        .execution_list_execution_steps(contracts::execution::ListExecutionSteps {
            execution_id: execution_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .steps;
    let mut criteria = Vec::new();
    for step in steps {
        criteria.extend(
            execution
                .execution_list_step_criteria(contracts::execution::ListStepCriteria {
                    execution_id: execution_id.clone(),
                    step_id: step.step_id,
                    ..Default::default()
                })
                .await
                .unwrap()
                .criteria,
        );
    }
    let acceptance = criteria
        .iter()
        .find(|criterion| {
            criterion
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STEADY_ACCEPTANCE)
        })
        .expect("steady acceptance criterion is present");
    assert_eq!(
        acceptance.status,
        contracts::execution::CriterionStatus::Passed
    );
    assert_eq!(
        acceptance
            .observed
            .as_ref()
            .map(|value| value.to_core().unwrap()),
        Some(Value::Text("passed".into()))
    );

    let mut content = client.clone();
    let stream_log = list_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    assert_eq!(stream_log.kind, contracts::content::ExecutionFileKind::Text);
    assert_eq!(
        stream_log.state,
        contracts::content::ExecutionFileState::Finalized
    );
    let log = read_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    let log = String::from_utf8_lossy(&log.data);
    assert!(log.contains("record=sidecar_started"));
    assert!(log.contains("record=sidecar_prompt_answered action=continue accepted=true"));
    assert!(log.contains("Charger resources released"));

    // What the template stated about the DUT before its first step, read back
    // through the typed query an operator's Traceability card reads.
    let observation = client.clone();
    let traceability = observation
        .observation_get_traceability(contracts::observation::GetTraceability {
            scope: Some(execution_scope.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    let dut = traceability
        .snapshots
        .iter()
        .find(|snapshot| {
            snapshot
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == "dut")
        })
        .expect("the steady template states the device it energized");
    assert_eq!(
        dut.metadata
            .as_ref()
            .map(|metadata| metadata.label.as_str()),
        Some("Device Under Test")
    );
    let supply = dut
        .entries
        .iter()
        .find(|entry| {
            entry
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == "nominal_voltage_rms_v")
        })
        .expect("the stated section carries the DUT supply it read");
    assert_eq!(supply.unit.as_deref(), Some("V"));
    assert_eq!(
        supply.source.as_deref(),
        Some("traceability.template.device")
    );

    // The stream measures at its declared cadence for as long as the prompt
    // is pending, so read every page: an oversized limit is clamped to what
    // fits one message and continues from the returned cursors.
    let mut final_samples = Vec::new();
    let mut after_stream_cursors = Vec::new();
    loop {
        let page = telemetry
            .telemetry_list_samples(contracts::telemetry::ListSamples {
                streams: vec![STEADY_SAMPLES.into()],
                scope: Some(execution_scope.clone()),
                limit: u32::MAX,
                after_stream_cursors,
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            !page.samples.is_empty() || page.complete,
            "an incomplete page always makes progress"
        );
        final_samples.extend(page.samples);
        if page.complete {
            break;
        }
        after_stream_cursors = page
            .through_stream_cursors
            .into_iter()
            .map(
                |entry| contracts::telemetry::ListTelemetrySamplesRequestAfterStreamCursorsEntry {
                    key: entry.key,
                    value: entry.value,
                },
            )
            .collect();
    }
    assert!(final_samples.len() >= STEADY_SAMPLE_COUNT as usize);
    assert!(
        final_samples
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence),
        "pages continue in order without repeats"
    );
    assert!(final_samples.iter().all(|sample| {
        sample.stream == STEADY_SAMPLES
            && sample.scope.as_ref().is_some_and(|sample_scope| {
                sample_scope.cycle_id.as_deref() == Some(cycle_id.as_str())
                    && sample_scope.execution_id.as_deref() == Some(execution_id.as_str())
            })
    }));

    // The stream the measuring step pushes into: declared by its row type, with
    // nothing measuring it, and written by hand around the wait on the observer.
    let events = telemetry
        .telemetry_list_samples(contracts::telemetry::ListSamples {
            streams: vec![STEADY_EVENTS.into()],
            scope: Some(execution_scope.clone()),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        events
            .samples
            .iter()
            .map(|sample| field(sample, "event"))
            .collect::<Vec<_>>(),
        ["hold_started", "hold_reached"],
        "a step pushes into a stream no observer measures"
    );
    assert!(
        events.samples.iter().all(|sample| {
            sample
                .scope
                .as_ref()
                .is_some_and(|scope| scope.execution_id.as_deref() == Some(execution_id.as_str()))
        }),
        "a pushed row is scoped to the execution that pushed it"
    );

    // A template naming every host stream in `subscribes(...)` leases them all.
    // Due collectors start on the shared tick and publish independently.
    // Alignment is carried by the timestamp rather than by waiting, so a wifi
    // scan that takes seconds costs only its own stream's ticks.
    let mut host_instants: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    for stream in HOST_STREAMS {
        let page = telemetry
            .telemetry_list_samples(contracts::telemetry::ListSamples {
                streams: vec![(*stream).into()],
                scope: Some(execution_scope.clone()),
                limit: 500,
                ..Default::default()
            })
            .await
            .unwrap();
        for sample in page.samples {
            host_instants
                .entry(*stream)
                .or_default()
                .push(sample.timestamp.expect("a host row carries its instant"));
        }
    }
    assert!(
        host_instants.len() >= 2,
        "a declared host request captures more than one stream: {:?}",
        host_instants.keys().collect::<Vec<_>>()
    );
    // CPU observations recur while other streams collect independently.
    let ticks = host_instants
        .get(HOST_FAST_STREAM)
        .map(|instants| {
            instants
                .iter()
                .map(|at| (at.seconds, at.nanos))
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    assert!(
        ticks.len() >= 2,
        "`{HOST_FAST_STREAM}` produces recurring observations: {} rows",
        host_instants
            .get(HOST_FAST_STREAM)
            .map(Vec::len)
            .unwrap_or_default()
    );
    // Each collector has its own cadence and can miss ticks while busy,
    // including host.system. Host capture also starts before the first step
    // reports its execution start. Check the admitted cycle's capture window;
    // controlled observer tests assert exact cross-stream tick alignment.
    let started = cycle.started_at.as_ref().expect("cycle started");
    let finished = cycle.updated_at.as_ref().expect("cycle completed");
    // The lake stores microseconds while lifecycle timestamps retain nanoseconds.
    let micros = |at: &contracts::common::Timestamp| {
        i128::from(at.seconds) * 1_000_000 + i128::from(at.nanos) / 1_000
    };
    for (stream, instants) in &host_instants {
        for instant in instants {
            assert!(
                (micros(started)..=micros(finished)).contains(&micros(instant)),
                "`{stream}` row at {instant:?} is outside {started:?}..={finished:?}"
            );
        }
    }

    let documents = client.clone();
    let stream = documents
        .documents_capture_current_document(contracts::documents::CaptureCurrentDocument {
            scope: Some(contracts::documents::DocumentScope {
                cycle_id: cycle_id.clone(),
                execution_id: execution_id.clone(),
            }),
            document_id: "default".into(),
        })
        .await
        .unwrap();
    let mut plot_found = false;
    let mut document_complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.frame {
            Some(contracts::documents::DocumentFrameFrame::Card(card)) => {
                if let Some(contracts::documents::DocumentCardBody::Owner(reference)) = card.body
                    && reference.plot_id.as_deref()
                        == Some("obc.charger.steady.measurement.execution.signals")
                {
                    let plot = reference.plot.expect("plot definition");
                    let request = plot.request.expect("plot request");
                    assert_eq!(request.stream, STEADY_SAMPLES);
                    // A time series draws against the row's automatic timestamp
                    // rather than an authored x expression — the wire carries no
                    // x expression for it, and a reader defaults the absent x to
                    // the "timestamp" field itself.
                    assert!(
                        request.x_expression.is_none(),
                        "a time_series plot's x is the automatic timestamp, not an authored expression"
                    );
                    assert!(request.y_expressions.iter().any(|expression| {
                        matches!(
                            expression.source,
                            Some(contracts::plots::DataExpressionSource::Telemetry(
                                contracts::plots::TelemetryDataSource { ref stream, ref field }
                            )) if stream == STEADY_SAMPLES && field == "voltage_v"
                        )
                    }));
                    plot_found = true;
                }
            }
            Some(contracts::documents::DocumentFrameFrame::Completion(_)) => {
                document_complete = true
            }
            _ => {}
        }
    }
    assert!(
        plot_found,
        "steady telemetry document exposes its signal plot"
    );
    assert!(document_complete, "document stream has a completion frame");

    let evidence = client.clone();
    let records = evidence
        .evidence_list_records(contracts::evidence::ListRecords {
            selection: Some(contracts::evidence::CapturedRecordSelection {
                scope: Some(execution_scope.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .records;
    for record_type in ["obc.charger.steady.started", "obc.charger.steady.completed"] {
        let record = records
            .iter()
            .find(|record| record.record_type == record_type)
            .unwrap_or_else(|| panic!("record {record_type} was not journaled"));
        let record_scope = record.scope.as_ref().expect("record scope");
        assert_eq!(record_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
        assert_eq!(
            record_scope.execution_id.as_deref(),
            Some(execution_id.as_str())
        );
    }
    let artifact_created = records
        .iter()
        .find(|record| record.record_type == "artifact.created")
        .expect("sidecar artifact creation is journaled");
    let artifact_scope = artifact_created.scope.as_ref().expect("artifact scope");
    assert_eq!(artifact_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(
        artifact_scope.execution_id.as_deref(),
        Some(execution_id.as_str())
    );
    let artifact_id = artifact_created
        .related_artifact_id
        .clone()
        .expect("artifact.created carries its artifact ID");
    let details = artifact_created.details.as_ref().expect("artifact details");
    let Value::Object(details) = details.to_core().unwrap() else {
        panic!("artifact details must be an object");
    };
    assert_eq!(details["artifact_id"], Value::Text(artifact_id.clone()));
    assert_eq!(details["filename"], Value::Text(STEADY_SUMMARY.into()));

    let artifact_stream = content
        .content_download_artifact(contracts::content::DownloadArtifact { artifact_id })
        .await
        .unwrap();
    let Some(contracts::content::ArtifactFramePayload::Metadata(metadata)) =
        artifact_stream.message().await.unwrap().unwrap().payload
    else {
        panic!("an artifact download opens with its metadata");
    };
    assert_eq!(metadata.filename, STEADY_SUMMARY);
    assert_eq!(metadata.content_type.as_deref(), Some("application/json"));
    assert_eq!(metadata.state, contracts::content::ArtifactState::Complete);
    let mut artifact_bytes = Vec::new();
    let mut artifact_complete = false;
    while let Some(frame) = artifact_stream.message().await.unwrap() {
        match frame.payload.unwrap() {
            contracts::content::ArtifactFramePayload::Chunk(chunk) => {
                artifact_bytes.extend(chunk.data)
            }
            contracts::content::ArtifactFramePayload::Complete(completion) => {
                assert_eq!(completion.size_bytes, metadata.size_bytes);
                artifact_complete = true;
            }
            contracts::content::ArtifactFramePayload::Metadata(_) => {
                panic!("artifact metadata repeated")
            }
        }
    }
    assert!(artifact_complete);
    let summary: serde_json::Value = serde_json::from_slice(&artifact_bytes).unwrap();
    assert_eq!(summary["accepted"], true);
    assert_eq!(summary["prompt_action"], "continue");
    assert_eq!(summary["supply"]["voltage_rms_v"], 230.0);
    assert_eq!(summary["supply"]["load_ohm"], 100.0);
    assert!(
        summary["sample_count"]
            .as_u64()
            .is_some_and(|count| count >= STEADY_SAMPLE_COUNT)
    );
}

/// Scene: Every Execution In The Nine-Case Steady Sweep Seals Its Report.
async fn steady_reports(client: &Client, changes: &mut Changes) {
    let mut catalog = client.clone();
    let steady = suite(&mut catalog, STEADY_SUITE).await;
    let template = steady
        .templates
        .iter()
        .find(|template| {
            template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == STEADY_TEMPLATE)
        })
        .expect("steady template is registered");
    let mut plan = selection(
        STEADY_SUITE,
        &template.template_id,
        [
            ("ac_voltage_v", number(230.0)),
            ("load_ohm", number(100.0)),
            ("frequency_hz", number(1.0)),
            ("duration_s", number(0.2)),
            ("sample_interval_s", number(0.1)),
        ],
    );
    for parameter in &mut plan.selections[0].parameters {
        let values = match parameter.key.as_str() {
            "ac_voltage_v" => vec![number(220.0), number(230.0), number(240.0)],
            "load_ohm" => vec![number(50.0), number(100.0), number(200.0)],
            _ => continue,
        };
        parameter.value.as_mut().unwrap().values = values;
    }
    let mut planning = client.clone();
    let mut execution = client.clone();
    let cycle_id = create_cycle(
        &mut planning,
        &mut execution,
        plan,
        "obc.charger.steady.reports",
        9,
    )
    .await;
    run_cycle(&mut execution, &cycle_id).await;

    // Answer Every Case, Then Observe The Durable Cycle Seal.
    let prompts = client.clone();
    on_change(changes, async || {
        let pending = prompts
            .prompts_list_pending(contracts::prompts::ListPending {
                cycle_id: Some(cycle_id.clone()),
                ..Default::default()
            })
            .await
            .unwrap();
        for prompt in pending.prompts {
            prompts
                .prompts_respond(contracts::prompts::Respond {
                    prompt_id: prompt.prompt_id,
                    action: "submit".into(),
                    values: Some(value_object([
                        ("operator_note", text("steady-report-sweep")),
                        ("accepted", boolean(true)),
                    ])),
                    accepted: true,
                    ..Default::default()
                })
                .await
                .unwrap();
        }
        let cycle = execution
            .execution_get_cycle(contracts::execution::GetCycle {
                cycle_id: cycle_id.clone(),
            })
            .await
            .unwrap();
        assert_ne!(cycle.status, contracts::execution::CycleStatus::Failed);
        cycle.sealed.then_some(())
    })
    .await;
    let executions = execution
        .execution_list_executions(contracts::execution::ListExecutions {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .executions;
    assert_eq!(executions.len(), 9);
    assert!(
        executions.iter().all(|execution| execution.sealed),
        "all nine execution reports seal before the cycle report: {executions:?}"
    );
}

/* === Tests === */
// The Plugin-And-Host Walkthrough Over The Built Bundle, One Scene At A Time:
// a configured bundle serves without an installation, reloads keep or refuse
// bytes, a startup suite runs through its prompt, evidence and sealed report, a
// steady sweep leaves its execution evidence, the host stops, and the plugin store installs, removes and skips a broken bundle
// across restarts.
#[tokio::test]
async fn plugin_host_walkthrough() {
    let root = tempfile::tempdir().unwrap();
    let bytes = std::fs::read(support::artifacts::obc_plugin_bundle()).unwrap();
    let bundle = root.path().join("bundle/obc-plugin.dokime");
    std::fs::create_dir_all(bundle.parent().unwrap()).unwrap();
    std::fs::write(&bundle, &bytes).unwrap();

    // Scene: A Configured Bundle Serves Its Suites Without An Installation
    let mut host = Host::launch_in(&root.path().join("configured"), Some(&bundle)).await;
    let client = host.connect().await;
    assert!(serves_startup(&client).await);
    status_ready(&client).await;
    assert!(installations(&client).await.is_empty());

    // Scene: Reloading Unchanged Bytes Keeps The Suites
    let reload = serde_json::json!({"command": "reload", "path": bundle});
    assert_eq!(host.command(reload.clone()).await["event"], "reloaded");
    assert!(serves_startup(&client).await);

    // Scene: A Reload Refuses Bytes That Are Not A Distribution
    std::fs::write(&bundle, b"not a distribution").unwrap();
    assert_eq!(host.command(reload.clone()).await["event"], "reload_failed");
    assert!(
        serves_startup(&client).await,
        "a refused reload keeps the suites"
    );
    std::fs::write(&bundle, &bytes).unwrap();
    assert_eq!(host.command(reload).await["event"], "reloaded");

    // Scene: A Suite Runs Through Its Prompt, Evidence And Sealed Report
    let mut changes = client
        .observation_watch_runtime(Default::default())
        .await
        .unwrap();
    let mut catalog = client.clone();
    let startup = suite(&mut catalog, STARTUP_SUITE).await;
    let template = startup
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE)
        })
        .unwrap_or_else(|| panic!("startup template is registered: {:?}", startup.templates));
    let mut planning = client.clone();
    let plan = selection(
        STARTUP_SUITE,
        &template.template_id,
        [
            ("duration_s", number(STARTUP_SAMPLE_COUNT as f64 * 0.1)),
            ("frequency_hz", number(1.0)),
            ("sample_interval_s", number(0.1)),
            ("realtime", boolean(true)),
        ],
    );
    let mut execution = client.clone();
    let cycle_id = create_cycle(
        &mut planning,
        &mut execution,
        plan,
        "native.charger.startup",
        1,
    )
    .await;
    run_cycle(&mut execution, &cycle_id).await;

    let mut prompts = client.clone();
    let pending = wait_prompt(&mut changes, &mut prompts, &cycle_id).await;
    assert_eq!(
        pending
            .definition
            .as_ref()
            .expect("prompt definition")
            .prompt_type,
        "obc.charger.startup.check"
    );
    let prompt_scope = pending.scope.expect("prompt scope");
    let execution_id = prompt_scope.execution_id.clone().expect("execution scope");

    let mut content = client.clone();

    // The Viewer's Live Path: Follow The Same Log Mid-Run From Offset Zero
    let follow = content
        .content_follow_execution_file(contracts::content::FollowExecutionFile {
            execution_id: execution_id.clone(),
            relative_path: STARTUP_STREAM_LOG.into(),
            identity: None,
            offset: 0,
            max_bytes: 64 * 1024,
        })
        .await
        .unwrap();
    let Some(contracts::content::ExecutionFileFollowFramePayload::Metadata(metadata)) = follow
        .message()
        .await
        .unwrap()
        .and_then(|frame| frame.payload)
    else {
        panic!("a live follow begins with its metadata frame");
    };
    assert_eq!(
        metadata.phase,
        contracts::content::ExecutionFilePhase::Producing
    );
    assert_eq!(metadata.offset, 0);
    let identity = metadata
        .identity
        .expect("a live follow names its retained identity");
    assert_eq!(identity.relative_path, STARTUP_STREAM_LOG);
    let followed = {
        let mut followed = Vec::new();
        loop {
            match follow
                .message()
                .await
                .unwrap()
                .and_then(|frame| frame.payload)
            {
                Some(contracts::content::ExecutionFileFollowFramePayload::Chunk(chunk)) => {
                    assert_eq!(chunk.offset, followed.len() as u64);
                    assert!(!chunk.data.is_empty() && chunk.data.len() <= 64 * 1024);
                    followed.extend_from_slice(&chunk.data);
                }
                _ => panic!("a producing follow delivers only contiguous chunks"),
            }
            let text = String::from_utf8_lossy(&followed).into_owned();
            if text.contains("record=prompt_requested") {
                break text;
            }
        }
    };
    assert!(followed.contains("record=started"));
    drop(follow);

    // The Followed Prefix Is Also What A Plain Read Serves While Still Streaming
    let file = list_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    assert_eq!(
        file.state,
        contracts::content::ExecutionFileState::Streaming
    );
    let window = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let running_log = String::from_utf8_lossy(&window.data).into_owned();
    assert!(running_log.contains("record=started"));
    assert!(running_log.contains("record=prompt_requested"));

    prompts
        .prompts_respond(contracts::prompts::Respond {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([
                ("operator_note", text("native-test-confirmed")),
                ("accepted", boolean(true)),
            ])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(&mut changes, &mut execution, &cycle_id).await;
    assert_eq!(cycle.status, contracts::execution::CycleStatus::Completed);
    let telemetry = client.clone();
    let telemetry = telemetry
        .telemetry_list_samples(contracts::telemetry::ListSamples {
            streams: vec![STARTUP_SAMPLES.into()],
            scope: Some(scope(&cycle_id, &execution_id)),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(telemetry.complete);
    assert_eq!(telemetry.samples.len() as u64, STARTUP_SAMPLE_COUNT);
    assert_eq!(
        telemetry
            .samples
            .iter()
            .map(|sample| sample.stream.as_str())
            .collect::<HashSet<_>>(),
        HashSet::from([STARTUP_SAMPLES])
    );

    let file = list_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    assert_eq!(file.kind, contracts::content::ExecutionFileKind::Text);
    assert_eq!(
        file.state,
        contracts::content::ExecutionFileState::Finalized
    );
    let final_log = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let final_text = String::from_utf8_lossy(&final_log.data);
    assert!(final_text.contains("record=prompt_answered action=continue accepted=true"));
    assert!(final_text.contains("Charger resources released"));

    // The Cycle Seals And Serves Its Report
    on_change(&mut changes, async || {
        execution
            .execution_get_cycle(contracts::execution::GetCycle {
                cycle_id: cycle_id.clone(),
            })
            .await
            .unwrap()
            .sealed
            .then_some(())
    })
    .await;
    let report = client
        .reports_download_cycle_report(contracts::reports::DownloadCycleReport {
            cycle_id: cycle_id.clone(),
            identifier: String::new(),
        })
        .await
        .unwrap();
    let mut metadata = None;
    let mut html = Vec::new();
    let mut complete = false;
    while let Some(frame) = report.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            contracts::reports::ReportHtmlFramePayload::Report(header) => {
                assert!(metadata.replace(*header).is_none(), "one metadata frame");
            }
            contracts::reports::ReportHtmlFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, html.len() as u64);
                html.extend_from_slice(&chunk.data);
            }
            contracts::reports::ReportHtmlFramePayload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                complete = true;
            }
        }
    }
    assert!(complete, "the report stream completes");
    assert_eq!(metadata.expect("report metadata").test_cycle_id, cycle_id);
    assert!(!html.is_empty());

    steady_sweep(&client, &mut changes).await;
    steady_reports(&client, &mut changes).await;

    // Scene: The Host Stops Cleanly With Its Watch Still Open
    drop(report);
    host.stop(false).await;
    drop(changes);
    drop(client);

    // Scene: A Consented Installation Activates Only After A Restart
    let store = root.path().join("store");
    let host = Host::launch_in(&store, None).await;
    let client = host.connect().await;
    assert!(!serves_startup(&client).await);
    let installation = install(&client, &bytes).await;
    assert!(!installation.active && installation.restart_required);
    assert!(!serves_startup(&client).await);
    host.stop(false).await;
    let host = Host::launch_in(&store, None).await;
    let client = host.connect().await;
    let listed = installations(&client).await;
    assert_eq!(listed.len(), 1);
    assert!(listed[0].active && !listed[0].restart_required);
    assert!(serves_startup(&client).await);
    status_ready(&client).await;

    // Scene: Removal Keeps The Live Plugin Until The Next Boot Drops It
    let removed = client
        .plugins_remove(contracts::plugins::Remove {
            install_id: installation.install_id.clone(),
        })
        .await
        .unwrap()
        .installation
        .expect("a consented installation stays pending removal");
    assert_eq!(
        removed.state,
        contracts::plugins::PluginInstallationState::RemovalPending
    );
    assert!(removed.active && removed.restart_required);
    assert!(serves_startup(&client).await);
    host.stop(false).await;
    let host = Host::launch_in(&store, None).await;
    let client = host.connect().await;
    assert!(installations(&client).await.is_empty());
    assert!(!serves_startup(&client).await);

    // Scene: A Corrupted Installed Bundle Is Skipped And Boot Succeeds
    install(&client, &bytes).await;
    host.stop(false).await;
    let installed = installed_copy(&store, &bytes).expect("the store keeps the consented bytes");
    std::fs::write(&installed, b"corrupted after consent").unwrap();
    let host = Host::launch_in(&store, None).await;
    let client = host.connect().await;
    let listed = installations(&client).await;
    assert_eq!(listed.len(), 1);
    assert!(
        matches!(
            listed[0]
                .last_boot
                .as_ref()
                .and_then(|boot| boot.result.as_ref()),
            Some(contracts::plugins::PluginLastBootResult::Error(_))
        ),
        "the broken installation reports its failed boot: {:?}",
        listed[0].last_boot
    );
    assert!(!serves_startup(&client).await);
    host.stop(false).await;
}
