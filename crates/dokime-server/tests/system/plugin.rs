/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod diagnosis;
mod disconnections;
mod shutdown;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
// Third Party Imports
// Local Imports
use crate::support::{self, host::Host};
use dokime::Value;
use dokime::client::{Client, ResponseStream, contracts};
use dokime::domain::model;

/* === Constants === */

const STARTUP_SUITE: &str = "obc.charger.startup";
const PROTECTION_SUITE: &str = "obc.charger.protection";
const PROTECTION_TEMPLATE: &str = "obc.charger.protection.overcurrent";
const STARTUP_TEMPLATE: &str = "obc.charger.startup.sequence";
const STARTUP_SAMPLES: &str = "obc.charger.startup.samples";
const STARTUP_STREAM_LOG: &str = "obc.charger.startup/stream.000000.log";
const STARTUP_SAMPLE_COUNT: u64 = 40;
const STEADY_SUITE: &str = "obc.charger.steady";
const STEADY_TEMPLATE: &str = "obc.charger.steady.measurement";
const STEADY_SAMPLES: &str = "obc.charger.steady.samples";
const STEADY_EVENTS: &str = "obc.charger.steady.events";
const STEADY_STREAM_LOG: &str = "obc.charger.steady/stream.000000.log";
const STEADY_HOST_SYSTEM: &str = "host.system";
const STEADY_SAMPLE_COUNT: u64 = 240;
const STEADY_PROMPT: &str = "obc.charger.steady.check";
const STEADY_ACCEPTANCE: &str = "obc.charger.steady.acceptance";
const STEADY_SUMMARY: &str = "charger/steady-state-summary.json";
const HOST_FAST_STREAM: &str = "host.system";
const HOST_STREAMS: &[&str] = &["host.system", "host.network", "host.wifi.link", "host.wifi.latency", "host.wifi.scan", "host.bluetooth"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The Shared Walkthrough Fixture: A Development Host Over A Private Copy Of The Built Bundle, With
// Its Runtime Watch Open. Every Scene Starts From `Session::ready` (the development session from
// `Session::launch`) and leaves the fixture ready, so scenes run alone or in walkthrough order.
struct Session {
    root: tempfile::TempDir,
    bytes: Vec<u8>,
    bundle: PathBuf,
    home: PathBuf,
    host: Host,
    client: Client,
    changes: Changes,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Session {
    // A Development Host Launched Over Its Bundle Copy, Before The First Reload.
    async fn launch() -> Self {
        let root = tempfile::tempdir().unwrap();
        let bytes = std::fs::read(support::artifacts::plugin_obc_bundle()).unwrap();
        let bundle = root.path().join("bundle/plugin-obc.dokime");
        std::fs::create_dir_all(bundle.parent().unwrap()).unwrap();
        std::fs::write(&bundle, &bytes).unwrap();
        let home = root.path().join("host");
        std::fs::create_dir_all(home.join("plugins")).unwrap();
        std::fs::copy(support::artifacts::host_plugin_bundle(), home.join("plugins/plugin-host.dokime")).unwrap();
        let host = Host::launch_in(&home, Some(&bundle)).await;
        let client = host.connect().await;
        let changes = client.stream(contracts::observation::WatchRuntime::default()).await.unwrap();
        Self {
            root,
            bytes,
            bundle,
            home,
            host,
            client,
            changes,
        }
    }

    // The Launched Host After Its First Reload, Serving The Bundle's Suites.
    async fn ready() -> Self {
        let mut session = Self::launch().await;
        session.reload().await;
        session
    }

    // A New Host Over The Same Home And Bundle, Reloaded, With A Fresh Watch.
    async fn relaunch(root: tempfile::TempDir, bytes: Vec<u8>, bundle: PathBuf, home: PathBuf) -> Self {
        let host = Host::launch_in(&home, Some(&bundle)).await;
        let client = host.connect().await;
        let changes = client.stream(contracts::observation::WatchRuntime::default()).await.unwrap();
        let mut session = Self {
            root,
            bytes,
            bundle,
            home,
            host,
            client,
            changes,
        };
        session.reload().await;
        session
    }

    async fn reload(&mut self) {
        let reloaded = self.host.command(serde_json::json!({"command": "reload", "path": self.bundle})).await;
        assert_eq!(reloaded["event"], "reloaded", "{reloaded}");
    }

    async fn stop(self) {
        drop(self.changes);
        drop(self.client);
        self.host.stop(false).await;
        drop(self.root);
    }
}

/* === Public Functions === */

/* === Private Functions === */

// The Development Session: The Selected Bundle Serves From The First Reload, Without An
// Installation; Reloading Unchanged Bytes Keeps The Suites; A Reload Refuses Bytes That Are Not A
// Distribution And Keeps The Suites. It Ends Ready, As `Session::ready` Leaves The Fixture.
async fn scene_development_session(session: &mut Session) {
    let client = session.client.clone();
    assert!(!serves_startup(&client).await, "a development host activates its bundle on the first reload");
    let reload = serde_json::json!({"command": "reload", "path": session.bundle});
    assert_eq!(session.host.command(reload.clone()).await["event"], "reloaded");
    assert!(serves_startup(&client).await);
    status_ready(&client).await;
    assert!(installations(&client).await.is_empty());

    // Reloading Unchanged Bytes Keeps The Suites
    assert_eq!(session.host.command(reload.clone()).await["event"], "reloaded");
    assert!(serves_startup(&client).await);

    // A Reload Refuses Bytes That Are Not A Distribution
    std::fs::write(&session.bundle, b"not a distribution").unwrap();
    assert_eq!(session.host.command(reload.clone()).await["event"], "reload_failed");
    assert!(serves_startup(&client).await, "a refused reload keeps the suites");
    std::fs::write(&session.bundle, &session.bytes).unwrap();
    assert_eq!(session.host.command(reload).await["event"], "reloaded");
}

// A Suite Runs Through Its Prompt, Evidence And Sealed Report.
async fn scene_startup(session: &mut Session) {
    let client = session.client.clone();
    let changes = &mut session.changes;
    let mut catalog = client.clone();
    let startup = suite(&mut catalog, STARTUP_SUITE).await;
    let template = startup
        .templates
        .iter()
        .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE))
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
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, "native.charger.startup", 1).await;
    run_cycle(&mut execution, &cycle_id).await;

    let mut prompts = client.clone();
    let pending = wait_prompt(changes, &mut prompts, &cycle_id).await;
    assert_eq!(pending.definition.as_ref().expect("prompt definition").prompt_type, "obc.charger.startup.check");
    let prompt_scope = pending.scope.expect("prompt scope");
    let execution_id = prompt_scope.execution_id.clone().expect("execution scope");

    let mut content = client.clone();

    // The Viewer's Live Path: Follow The Same Log Mid-Run From Offset Zero
    let follow = content
        .stream(contracts::content::FollowExecutionFile {
            execution_id: execution_id.clone(),
            relative_path: STARTUP_STREAM_LOG.into(),
            identity: None,
            offset: 0,
            max_bytes: 64 * 1024,
        })
        .await
        .unwrap();
    let Some(model::content::wire::ExecutionFileFollowFramePayload::Metadata(metadata)) =
        follow.message().await.unwrap().and_then(|frame| frame.payload)
    else {
        panic!("a live follow begins with its metadata frame");
    };
    assert_eq!(metadata.phase, model::content::wire::ExecutionFilePhase::Producing);
    assert_eq!(metadata.offset, 0);
    let identity = metadata.identity.expect("a live follow names its retained identity");
    assert_eq!(identity.relative_path, STARTUP_STREAM_LOG);
    let followed = {
        let mut followed = Vec::new();
        loop {
            match follow.message().await.unwrap().and_then(|frame| frame.payload) {
                Some(model::content::wire::ExecutionFileFollowFramePayload::Chunk(chunk)) => {
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
    assert_eq!(file.state, model::content::wire::ExecutionFileState::Streaming);
    let window = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let running_log = String::from_utf8_lossy(&window.data).into_owned();
    assert!(running_log.contains("record=started"));
    assert!(running_log.contains("record=prompt_requested"));

    prompts
        .call(contracts::prompts::Respond {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([("operator_note", text("native-test-confirmed")), ("accepted", boolean(true))])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(changes, &mut execution, &cycle_id).await;
    if !passed(&cycle) {
        let diagnosis = diagnosis::execution(&client, &execution_id, STARTUP_STREAM_LOG).await;
        assert!(passed(&cycle), "{diagnosis}");
    }
    let telemetry = client.clone();
    let telemetry = telemetry
        .call(contracts::telemetry::ListSamples {
            streams: vec![STARTUP_SAMPLES.into()],
            scope: Some(scope(&cycle_id, &execution_id)),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(telemetry.complete);
    assert_eq!(telemetry.samples.len() as u64, STARTUP_SAMPLE_COUNT);
    assert_eq!(telemetry.samples.iter().map(|sample| sample.stream.as_str()).collect::<HashSet<_>>(), HashSet::from([STARTUP_SAMPLES]));

    let file = list_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    assert_eq!(file.kind, model::content::wire::ExecutionFileKind::Text);
    assert_eq!(file.state, model::content::wire::ExecutionFileState::Finalized);
    let final_log = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let final_text = String::from_utf8_lossy(&final_log.data);
    assert!(final_text.contains("record=prompt_answered action=continue accepted=true"));
    assert!(final_text.contains("Charger resources released"));

    // The Explicitly Sealed Cycle Serves Its Report
    let id = model::identity::TestCycleId::from_uuid(&cycle_id).unwrap();
    execution.cycle(id).seal().await.unwrap();
    on_change(changes, "the startup cycle to seal", async || {
        execution.call(contracts::execution::GetCycle { cycle_id: cycle_id.clone() }).await.unwrap().sealed.then_some(())
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
            model::reports::wire::ReportHtmlFramePayload::Report(header) => {
                assert!(metadata.replace(*header).is_none(), "one metadata frame");
            }
            model::reports::wire::ReportHtmlFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, html.len() as u64);
                html.extend_from_slice(&chunk.data);
            }
            model::reports::wire::ReportHtmlFramePayload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                complete = true;
            }
        }
    }
    assert!(complete, "the report stream completes");
    assert_eq!(metadata.expect("report metadata").test_cycle_id, cycle_id);
    assert!(!html.is_empty());
}

// The Actual Remote Scripts Control The Built Plugin And Export Saved Evidence.
async fn scene_remote_scripts(session: &mut Session) {
    let root = session.root.path().join("remote scripts");
    Box::pin(super::remote::scripts(&session.client, session.host.endpoint(), &root)).await;
}

// The Actual Remote Scripts Are Interrupted Through Their Native Terminals Without Harming Other Clients.
async fn scene_interruptions(session: &mut Session) {
    let root = session.root.path().join("remote scripts/interruptions");
    Box::pin(super::remote::interruptions(&session.client, session.host.endpoint(), &root)).await;
}

// The Host Stops Cleanly With Its Watch Still Open.
async fn scene_clean_stop(session: Session) {
    let Session { root, host, changes, client, .. } = session;
    host.stop(false).await;
    drop(changes);
    drop(client);
    drop(root);
}

// A Cycle Report Opens On Every Request From The Measuring Member's Settlement, Through The
// Prompting Member's Admission, To Its Pending Question, And Both Reports Open While That
// Question Waits Without Answering It.
async fn scene_open_report_while_cycle_runs(session: &mut Session) {
    const BOUND: Duration = Duration::from_secs(120);
    let client = session.client.clone();
    let changes = &mut session.changes;

    // One Cycle: A Measuring Member, Then A Prompting One Queued Behind It
    let (mut catalog, mut planning, mut execution) = (client.clone(), client.clone(), client.clone());
    let mut plans = Vec::new();
    for (suite_id, template_id) in [(PROTECTION_SUITE, PROTECTION_TEMPLATE), (STARTUP_SUITE, STARTUP_TEMPLATE)] {
        let template = suite(&mut catalog, suite_id)
            .await
            .templates
            .into_iter()
            .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == template_id))
            .unwrap_or_else(|| panic!("{template_id} is registered"));
        plans.push(selection(suite_id, &template.template_id, []));
    }
    let prompting = plans.pop().unwrap();
    let cycle_id = create_cycle(&mut planning, &mut execution, plans.pop().unwrap(), "native.charger.open-report", 1).await;
    let registered = execution
        .call(contracts::planning::RegisterSelection {
            plan: Some(prompting),
            idempotency_key: uuid::Uuid::now_v7().to_string(),
        })
        .await
        .unwrap();
    let executions = registered
        .cases
        .into_iter()
        .map(|case| contracts::execution::ExecutionBinding {
            execution_id: uuid::Uuid::now_v7().to_string(),
            case_id: case.case_id,
        })
        .collect();
    execution.call(contracts::execution::AppendExecutions { cycle_id: cycle_id.clone(), executions }).await.unwrap();
    let members = execution
        .call(contracts::execution::ListExecutions {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .executions;
    for member in &members {
        let reply = execution
            .call(contracts::execution::StartExecution {
                execution_id: member.execution_id.clone(),
                mode: model::execution::wire::ExecutionMode::Automatic,
            })
            .await
            .unwrap();
        assert!(matches!(reply.outcome, Some(model::execution::wire::RunReplyOutcome::Accepted(_))), "{reply:?}");
    }

    // Back-To-Back Requests Cover The Measuring Member's Seal, The Next Admission And The Question
    on_change(changes, "the measuring member to pass", async || {
        let page = execution
            .call(contracts::execution::ListExecutions {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        let measuring = page.executions.iter().find(|member| member.execution_id == members[0].execution_id)?;
        (measuring.status == model::execution::wire::ExecutionStatus::Passed).then_some(())
    })
    .await;
    let prompts = client.clone();
    let mut opened = 0;
    let pending = loop {
        let html = tokio::time::timeout(BOUND, cycle_report(&client, &cycle_id))
            .await
            .unwrap_or_else(|_| panic!("cycle report {opened} did not open within {BOUND:?} while the cycle ran"));
        assert!(!html.is_empty());
        opened += 1;
        let page = prompts
            .call(contracts::prompts::ListPending {
                cycle_id: Some(cycle_id.clone()),
                ..Default::default()
            })
            .await
            .unwrap();
        if let Some(pending) = page.prompts.into_iter().next() {
            break pending;
        }
    };

    // Both Reports Open While The Question Waits, And It Still Waits
    let execution_id = pending.scope.as_ref().and_then(|scope| scope.execution_id.clone()).expect("execution scope");
    let cycle_html =
        tokio::time::timeout(BOUND, cycle_report(&client, &cycle_id)).await.expect("the cycle report opens while the prompt waits");
    assert!(!cycle_html.is_empty());
    let execution_report = async {
        let request = contracts::reports::DownloadExecutionReport { execution_id: execution_id.clone() };
        report_bytes(client.reports_download_execution_report(request).await.unwrap()).await
    };
    let execution_html = tokio::time::timeout(BOUND, execution_report).await.expect("the execution report opens while the prompt waits");
    assert!(!execution_html.is_empty());
    let still = prompts
        .call(contracts::prompts::ListPending {
            cycle_id: Some(cycle_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(still.prompts.iter().any(|prompt| prompt.prompt_id == pending.prompt_id), "opening a report leaves the prompt pending");

    // Answering The Question Completes The Cycle
    prompts
        .call(contracts::prompts::Respond {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([("operator_note", text("open-report")), ("accepted", boolean(true))])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();
    let cycle = wait_completed(changes, &mut execution, &cycle_id).await;
    assert!(passed(&cycle));
}

type Changes = ResponseStream<model::observation::wire::RuntimeStateFrame>;

async fn on_change<T>(changes: &mut Changes, boundary: &str, mut probe: impl AsyncFnMut() -> Option<T>) -> T {
    loop {
        if let Some(value) = probe().await {
            return value;
        }
        changes
            .message()
            .await
            .unwrap_or_else(|error| panic!("runtime watch failed while waiting for {boundary}: {error:?}"))
            .unwrap_or_else(|| panic!("runtime watch closed while waiting for {boundary}"));
    }
}

async fn wait_prompt(changes: &mut Changes, prompts: &mut Client, cycle_id: &str) -> model::prompts::wire::PendingPromptIdentity {
    on_change(changes, "a pending prompt", async || {
        let page = prompts
            .call(contracts::prompts::ListPending {
                cycle_id: Some(cycle_id.to_owned()),
                ..Default::default()
            })
            .await
            .unwrap();
        page.prompts.into_iter().next()
    })
    .await
}

async fn wait_completed(changes: &mut Changes, execution: &mut Client, cycle_id: &str) -> model::execution::wire::CycleSummary {
    on_change(changes, "cycle completion", async || {
        let cycle = execution.call(contracts::execution::GetCycle { cycle_id: cycle_id.to_owned() }).await.unwrap();
        // An Active cycle has settled once it is idle again with every member run.
        let settled = cycle.status == model::execution::wire::CycleStatus::Ready
            && cycle.running_count == 0
            && cycle.queued_count == 0
            && cycle.result.is_some();
        settled.then_some(cycle)
    })
    .await
}

fn passed(cycle: &model::execution::wire::CycleSummary) -> bool {
    cycle.result.as_ref().is_some_and(|result| result.status == model::execution::wire::ResultStatus::Passed)
}

async fn execution_id(changes: &mut Changes, execution: &mut Client, cycle_id: &str) -> String {
    on_change(changes, "the first execution", async || {
        let page = execution
            .call(contracts::execution::ListExecutions {
                cycle_id: cycle_id.to_owned(),
                ..Default::default()
            })
            .await
            .unwrap();
        page.executions.into_iter().next().map(|item| item.execution_id)
    })
    .await
}

fn field(sample: &model::telemetry::wire::TelemetrySample, name: &str) -> String {
    let Value::Object(mut fields) = sample.values.as_ref().unwrap().to_core().unwrap() else {
        panic!("sample values must be an object");
    };
    match fields.remove(name) {
        Some(Value::Text(text)) => text,
        other => panic!("sample field {name:?} is not text: {other:?}"),
    }
}

fn number(value: f64) -> model::values::wire::Value {
    model::values::wire::Value::try_from(&Value::Number(value)).unwrap()
}

fn text(value: &str) -> model::values::wire::Value {
    model::values::wire::Value::try_from(&Value::Text(value.to_owned())).unwrap()
}

fn boolean(value: bool) -> model::values::wire::Value {
    model::values::wire::Value::try_from(&Value::Bool(value)).unwrap()
}

fn selection(
    suite_id: &str, template_id: &str, parameters: impl IntoIterator<Item = (&'static str, model::values::wire::Value)>,
) -> model::planning::wire::SelectionPlan {
    model::planning::wire::SelectionPlan {
        selections: vec![model::planning::wire::RegisteredSelection {
            suite_id: suite_id.to_owned(),
            template_id: template_id.to_owned(),
            parameters: parameters
                .into_iter()
                .map(|(name, value)| model::planning::wire::RegisteredSelectionParametersEntry {
                    key: name.to_owned(),
                    value: Some(model::planning::wire::SelectionValues { values: vec![value] }),
                })
                .collect(),
            files: vec![],
        }],
        max_cases: 0,
        case_ids: vec![],
        cycle_id: String::new(),
    }
}

fn scope(cycle_id: &str, execution_id: &str) -> model::identity::wire::EvidenceScope {
    model::identity::wire::EvidenceScope {
        cycle_id: Some(cycle_id.to_owned()),
        execution_id: Some(execution_id.to_owned()),
        ..Default::default()
    }
}

async fn suite(catalog: &mut Client, suite_id: &str) -> model::catalog::wire::SuiteDefinition {
    catalog.call(contracts::catalog::GetSuite { suite_id: suite_id.to_owned() }).await.unwrap()
}

async fn create_cycle(
    planning: &mut Client, execution: &mut Client, plan: model::planning::wire::SelectionPlan, identifier: &str, expected_cases: usize,
) -> String {
    let preview = planning.call(contracts::planning::PreviewSelection { plan: Some(plan.clone()) }).await.unwrap();
    assert_eq!(preview.cases.len(), expected_cases);
    let cycle_id = execution
        .call(contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
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
    let registered = execution
        .call(contracts::planning::RegisterSelection {
            plan: Some(plan),
            idempotency_key: uuid::Uuid::now_v7().to_string(),
        })
        .await
        .unwrap();
    let executions = registered
        .cases
        .into_iter()
        .map(|case| contracts::execution::ExecutionBinding {
            execution_id: uuid::Uuid::now_v7().to_string(),
            case_id: case.case_id,
        })
        .collect();
    execution.call(contracts::execution::AppendExecutions { cycle_id: cycle_id.clone(), executions }).await.unwrap();
    cycle_id
}

async fn run_cycle(execution: &mut Client, cycle_id: &str) {
    let members = execution
        .call(contracts::execution::ListExecutions {
            cycle_id: cycle_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap();
    let reply = execution
        .call(contracts::execution::StartExecution {
            execution_id: members.executions[0].execution_id.clone(),
            mode: model::execution::wire::ExecutionMode::Automatic,
        })
        .await
        .unwrap();
    assert!(matches!(reply.outcome, Some(model::execution::wire::RunReplyOutcome::Accepted(_))));
}

async fn read_file(content: &mut Client, execution_id: &str, relative_path: &str) -> model::content::wire::ExecutionFileWindow {
    content
        .call(contracts::content::ReadExecutionFile {
            execution_id: execution_id.to_owned(),
            relative_path: relative_path.to_owned(),
            offset: 0,
            max_bytes: 8192,
        })
        .await
        .unwrap()
}

async fn list_file(content: &mut Client, execution_id: &str, relative_path: &str) -> model::content::wire::ExecutionFileMetadata {
    let page = content
        .call(contracts::content::ListExecutionFiles {
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

fn value_object(fields: impl IntoIterator<Item = (&'static str, model::values::wire::Value)>) -> model::values::wire::Value {
    let values = fields.into_iter().map(|(key, value)| (key.to_owned(), value.to_core().unwrap())).collect();
    model::values::wire::Value::try_from(&Value::Object(values)).unwrap()
}

async fn report_bytes(report: ResponseStream<model::reports::wire::ReportHtmlFrame>) -> Vec<u8> {
    let mut html = Vec::new();
    while let Some(frame) = report.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            model::reports::wire::ReportHtmlFramePayload::Report(_) => {}
            model::reports::wire::ReportHtmlFramePayload::Chunk(chunk) => html.extend_from_slice(&chunk.data),
            model::reports::wire::ReportHtmlFramePayload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                return html;
            }
        }
    }
    panic!("the report stream ends before it completes");
}

async fn cycle_report(client: &Client, cycle_id: &str) -> Vec<u8> {
    let request = contracts::reports::DownloadCycleReport {
        cycle_id: cycle_id.to_owned(),
        identifier: "cycle.summary".into(),
    };
    report_bytes(client.reports_download_cycle_report(request).await.unwrap()).await
}

async fn serves_startup(client: &Client) -> bool {
    client.call(contracts::catalog::GetSuite { suite_id: STARTUP_SUITE.to_owned() }).await.is_ok()
}

async fn status_ready(client: &Client) {
    let frame = client.call(contracts::settings::GetSettings::default()).await.unwrap();
    let card = frame.cards.iter().find(|card| card.plugin_id == "plugin_obc").expect("the OBC settings card is registered");
    assert_eq!(card.state, model::settings::wire::SettingsCardState::Ready, "the native status operation serves the installed plugin");
}

async fn installations(client: &Client) -> Vec<model::plugins::wire::PluginInstallation> {
    client.call(contracts::plugins::ListInstallations::default()).await.unwrap().installations
}

async fn install(client: &Client, bytes: &[u8]) -> model::plugins::wire::PluginInstallation {
    let sha256 = dokime::client::plugin_bundle_digest(bytes);
    client.plugins_install_bundle(bytes.to_vec(), sha256, |_, _| {}).await.unwrap()
}

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

async fn scene_steady_sweep(client: &Client, changes: &mut Changes) {
    let mut catalog = client.clone();
    let steady = suite(&mut catalog, STEADY_SUITE).await;
    let template = steady
        .templates
        .iter()
        .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == STEADY_TEMPLATE))
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
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, "obc.charger.steady.sweep", 1).await;
    run_cycle(&mut execution, &cycle_id).await;
    let execution_id = execution_id(changes, &mut execution, &cycle_id).await;
    let execution_scope = scope(&cycle_id, &execution_id);

    // The sidecar observer runs independently of the prompt step. A non-empty
    // execution-scoped page proves that evidence is visible while the run is
    // still live, before the prompt is answered or the report is finalized.
    let telemetry = client.clone();
    // Bounded readiness poll: the plugin process exposes no signal for its
    // first observer sample.
    let live_deadline = Instant::now() + Duration::from_secs(120);
    let live_samples = loop {
        let page = telemetry
            .call(contracts::telemetry::ListSamples {
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
        assert!(Instant::now() < live_deadline, "sidecar telemetry was not visible while execution was running");
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    assert!(live_samples.samples.len() < STEADY_SAMPLE_COUNT as usize);

    let mut prompts = client.clone();
    let pending = wait_prompt(changes, &mut prompts, &cycle_id).await;
    let definition = pending.definition.as_ref().expect("prompt definition");
    assert_eq!(definition.prompt_type, STEADY_PROMPT);
    let prompt_scope = pending.scope.as_ref().expect("prompt scope");
    assert_eq!(prompt_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(prompt_scope.execution_id.as_deref(), Some(execution_id.as_str()));

    // Hold the prompt until two host ticks are observable. Charger sample
    // time can advance faster than the host collector's wall-clock cadence.
    // Bounded readiness poll: host collectors tick on wall time and expose no
    // signal per row.
    let host_deadline = Instant::now() + Duration::from_secs(120);
    loop {
        let page = telemetry
            .call(contracts::telemetry::ListSamples {
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
        assert!(Instant::now() < host_deadline, "declared host telemetry did not recur while prompt was pending");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    prompts
        .call(contracts::prompts::Respond {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([("operator_note", text("embedded-sidecar-confirmed")), ("accepted", boolean(true))])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(changes, &mut execution, &cycle_id).await;
    if !passed(&cycle) {
        let diagnosis = diagnosis::execution(client, &execution_id, STEADY_STREAM_LOG).await;
        assert!(passed(&cycle), "{diagnosis}");
    }
    let id = model::identity::TestCycleId::from_uuid(&cycle_id).unwrap();
    execution.cycle(id).seal().await.unwrap();
    let summary = execution
        .call(contracts::execution::ListExecutions {
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
    assert_eq!(summary.status, model::execution::wire::ExecutionStatus::Passed);

    let steps = execution
        .call(contracts::execution::ListExecutionSteps {
            execution_id: execution_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .steps;
    let criteria = steps.into_iter().flat_map(|step| step.criteria).collect::<Vec<_>>();
    let acceptance = criteria
        .iter()
        .find(|criterion| criterion.metadata.as_ref().is_some_and(|metadata| metadata.identifier == STEADY_ACCEPTANCE))
        .expect("steady acceptance criterion is present");
    assert_eq!(acceptance.status, model::execution::wire::CriterionStatus::Passed);
    assert_eq!(acceptance.observed.as_ref().map(|value| value.to_core().unwrap()), Some(Value::Text("passed".into())));

    let mut content = client.clone();
    let stream_log = list_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    assert_eq!(stream_log.kind, model::content::wire::ExecutionFileKind::Text);
    assert_eq!(stream_log.state, model::content::wire::ExecutionFileState::Finalized);
    let log = read_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    let log = String::from_utf8_lossy(&log.data);
    assert!(log.contains("record=sidecar_started"));
    assert!(log.contains("record=sidecar_prompt_answered action=continue accepted=true"));
    assert!(log.contains("Charger resources released"));

    // What the first step captured about the DUT, read back
    // through the typed query an operator's Traceability card reads.
    let observation = client.clone();
    let traceability = observation
        .call(contracts::observation::GetTraceability {
            scope: Some(execution_scope.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    let dut = traceability
        .snapshots
        .iter()
        .find(|snapshot| snapshot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "dut"))
        .expect("the steady template states the device it energized");
    assert_eq!(dut.metadata.as_ref().map(|metadata| metadata.label.as_str()), Some("Device Under Test"));
    let supply = dut
        .entries
        .iter()
        .find(|entry| entry.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "nominal_voltage_rms_v"))
        .expect("the stated section carries the DUT supply it read");
    assert_eq!(supply.unit.as_deref(), Some("V"));
    assert_eq!(supply.source.as_deref(), Some("traceability.template.device"));

    // The stream measures at its declared cadence for as long as the prompt
    // is pending, so read every page: an oversized limit is clamped to what
    // fits one message and continues from the returned cursors.
    let mut final_samples = Vec::new();
    let mut after_stream_cursors = Vec::new();
    loop {
        let page = telemetry
            .call(contracts::telemetry::ListSamples {
                streams: vec![STEADY_SAMPLES.into()],
                scope: Some(execution_scope.clone()),
                limit: u32::MAX,
                after_stream_cursors,
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(!page.samples.is_empty() || page.complete, "an incomplete page always makes progress");
        final_samples.extend(page.samples);
        if page.complete {
            break;
        }
        after_stream_cursors = page
            .through_stream_cursors
            .into_iter()
            .map(|entry| model::telemetry::wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry { key: entry.key, value: entry.value })
            .collect();
    }
    assert!(final_samples.len() >= STEADY_SAMPLE_COUNT as usize);
    assert!(final_samples.windows(2).all(|pair| pair[0].sequence < pair[1].sequence), "pages continue in order without repeats");
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
        .call(contracts::telemetry::ListSamples {
            streams: vec![STEADY_EVENTS.into()],
            scope: Some(execution_scope.clone()),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        events.samples.iter().map(|sample| field(sample, "event")).collect::<Vec<_>>(),
        ["hold_started", "hold_reached"],
        "a step pushes into a stream no observer measures"
    );
    assert!(
        events
            .samples
            .iter()
            .all(|sample| { sample.scope.as_ref().is_some_and(|scope| scope.execution_id.as_deref() == Some(execution_id.as_str())) }),
        "a pushed row is scoped to the execution that pushed it"
    );

    // A template naming every host stream in `subscribes(...)` leases them all.
    // Demand observers call the host sidecar operations and publish independently.
    // Alignment is carried by the timestamp rather than by waiting, so a wifi
    // scan that takes seconds costs only its own stream's ticks.
    let mut host_instants: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    for stream in HOST_STREAMS {
        let page = telemetry
            .call(contracts::telemetry::ListSamples {
                streams: vec![(*stream).into()],
                scope: Some(execution_scope.clone()),
                limit: 500,
                ..Default::default()
            })
            .await
            .unwrap();
        for sample in page.samples {
            assert_eq!(sample.source, "plugin_host", "host rows arrive from the sidecar owner");
            host_instants.entry(*stream).or_default().push(sample.timestamp.expect("a host row carries its instant"));
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
        .map(|instants| instants.iter().map(|at| (at.seconds, at.nanos)).collect::<HashSet<_>>())
        .unwrap_or_default();
    assert!(
        ticks.len() >= 2,
        "`{HOST_FAST_STREAM}` produces recurring observations: {} rows",
        host_instants.get(HOST_FAST_STREAM).map(Vec::len).unwrap_or_default()
    );
    // Each collector has its own cadence and can miss ticks while busy,
    // including host.system. Host capture also starts before the first step
    // reports its execution start. Check the admitted cycle's capture window;
    // controlled observer tests assert exact cross-stream tick alignment.
    let started = cycle.started_at.as_ref().expect("cycle started");
    let finished = cycle.updated_at.as_ref().expect("cycle completed");
    // The lake stores microseconds while lifecycle timestamps retain nanoseconds.
    let micros = |at: &model::identity::wire::Timestamp| i128::from(at.seconds) * 1_000_000 + i128::from(at.nanos) / 1_000;
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
        .stream(contracts::documents::CaptureCurrentDocument {
            scope: Some(model::documents::wire::DocumentScope {
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
            Some(model::documents::wire::DocumentFrameFrame::Card(card)) => {
                if let Some(model::documents::wire::DocumentCardBody::Owner(reference)) = card.body
                    && reference.plot_id.as_deref() == Some("obc.charger.steady.measurement.execution.signals")
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
                            Some(model::plots::wire::DataExpressionSource::Telemetry(
                                model::plots::wire::TelemetryDataSource { ref stream, ref field }
                            )) if stream == STEADY_SAMPLES && field == "voltage_v"
                        )
                    }));
                    plot_found = true;
                }
            }
            Some(model::documents::wire::DocumentFrameFrame::Completion(_)) => document_complete = true,
            _ => {}
        }
    }
    assert!(plot_found, "steady telemetry document exposes its signal plot");
    assert!(document_complete, "document stream has a completion frame");

    let evidence = client.clone();
    let records = evidence
        .call(contracts::evidence::ListRecords {
            selection: Some(model::evidence::wire::CapturedRecordSelection {
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
        assert_eq!(record_scope.execution_id.as_deref(), Some(execution_id.as_str()));
    }
    let artifact_created =
        records.iter().find(|record| record.record_type == "artifact.created").expect("sidecar artifact creation is journaled");
    let artifact_scope = artifact_created.scope.as_ref().expect("artifact scope");
    assert_eq!(artifact_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(artifact_scope.execution_id.as_deref(), Some(execution_id.as_str()));
    let artifact_id = artifact_created.related_artifact_id.clone().expect("artifact.created carries its artifact ID");
    let details = artifact_created.details.as_ref().expect("artifact details");
    let Value::Object(details) = details.to_core().unwrap() else {
        panic!("artifact details must be an object");
    };
    assert_eq!(details["artifact_id"], Value::Text(artifact_id.clone()));
    assert_eq!(details["filename"], Value::Text(STEADY_SUMMARY.into()));

    let artifact_stream = content.stream(contracts::content::DownloadArtifact { artifact_id }).await.unwrap();
    let Some(model::content::wire::ArtifactFramePayload::Metadata(metadata)) = artifact_stream.message().await.unwrap().unwrap().payload
    else {
        panic!("an artifact download opens with its metadata");
    };
    assert_eq!(metadata.filename, STEADY_SUMMARY);
    assert_eq!(metadata.content_type.as_deref(), Some("application/json"));
    assert_eq!(metadata.state, model::content::wire::ArtifactState::Complete);
    let mut artifact_bytes = Vec::new();
    let mut artifact_complete = false;
    while let Some(frame) = artifact_stream.message().await.unwrap() {
        match frame.payload.unwrap() {
            model::content::wire::ArtifactFramePayload::Chunk(chunk) => artifact_bytes.extend(chunk.data),
            model::content::wire::ArtifactFramePayload::Complete(completion) => {
                assert_eq!(completion.size_bytes, metadata.size_bytes);
                artifact_complete = true;
            }
            model::content::wire::ArtifactFramePayload::Metadata(_) => {
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
    assert!(summary["sample_count"].as_u64().is_some_and(|count| count >= STEADY_SAMPLE_COUNT));
}

/* === Tests === */

// The Plugin-And-Host Walkthrough Over The Built Bundle Is One Story Of Independently Runnable
// Scenes On One Shared Fixture. Each Scene Is A Test That Reruns Alone (`-- plugin::<scene>`);
// `walkthrough` Replays Every Scene In Order On One Host.

#[dokime_tracing::trace_test]
#[tokio::test]
async fn development_session() {
    let mut session = Session::launch().await;
    scene_development_session(&mut session).await;
    session.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn startup() {
    let mut session = Session::ready().await;
    scene_startup(&mut session).await;
    session.stop().await;
}

// A Steady Sweep Leaves Its Execution Evidence.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn steady_sweep() {
    let mut session = Session::ready().await;
    scene_steady_sweep(&session.client.clone(), &mut session.changes).await;
    session.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn remote_scripts() {
    let mut session = Session::ready().await;
    scene_remote_scripts(&mut session).await;
    session.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn interruptions() {
    let mut session = Session::ready().await;
    scene_interruptions(&mut session).await;
    session.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn clean_stop() {
    scene_clean_stop(Session::ready().await).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn open_report_while_cycle_runs() {
    let mut session = Session::ready().await;
    scene_open_report_while_cycle_runs(&mut session).await;
    session.stop().await;
}

// The Plugin Store Installs, Removes And Skips A Broken Bundle Across Restarts.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn store() {
    let root = tempfile::tempdir().unwrap();
    let bytes = std::fs::read(support::artifacts::plugin_obc_bundle()).unwrap();

    // An Installed Plugin Activates Only After A Restart
    let store = root.path().join("store");
    std::fs::create_dir_all(store.join("plugins")).unwrap();
    std::fs::copy(support::artifacts::host_plugin_bundle(), store.join("plugins/plugin-host.dokime")).unwrap();
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
        .call(contracts::plugins::Remove {
            install_id: installation.install_id.clone(),
        })
        .await
        .unwrap()
        .installation
        .expect("a installed installation stays pending removal");
    assert_eq!(removed.state, model::plugins::wire::PluginInstallationState::RemovalPending);
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
    let installed = installed_copy(&store, &bytes).expect("the store keeps the installed bytes");
    std::fs::write(&installed, b"corrupted after installation").unwrap();
    let host = Host::launch_in(&store, None).await;
    let client = host.connect().await;
    let listed = installations(&client).await;
    assert_eq!(listed.len(), 1);
    assert!(
        matches!(
            listed[0].last_boot.as_ref().and_then(|boot| boot.result.as_ref()),
            Some(model::plugins::wire::PluginLastBootResult::Error(_))
        ),
        "the broken installation reports its failed boot: {:?}",
        listed[0].last_boot
    );
    assert!(!serves_startup(&client).await);
    host.stop(false).await;
}

// Every Scene In Walkthrough Order On One Host; The Lane Runs The Scenes Individually.
#[dokime_tracing::trace_test]
#[tokio::test]
#[ignore = "replays the scenes the lane already runs; select it with --run-ignored"]
async fn walkthrough() {
    // Each Scene Is Boxed: A Debug Poll Frame Keeps A Stack Temporary For Every Future It Awaits,
    // So Replaying Every Scene In One Function Would Outgrow The Test Thread's Stack.
    let mut session = Session::launch().await;
    Box::pin(scene_development_session(&mut session)).await;
    Box::pin(scene_startup(&mut session)).await;
    Box::pin(scene_steady_sweep(&session.client.clone(), &mut session.changes)).await;
    Box::pin(scene_open_report_while_cycle_runs(&mut session)).await;
    Box::pin(scene_remote_scripts(&mut session)).await;
    Box::pin(scene_interruptions(&mut session)).await;
    Box::pin(disconnections::scene(&mut session)).await;
    Box::pin(scene_clean_stop(session)).await;
}
