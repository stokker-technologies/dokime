/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod callbacks;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
// Third Party Imports

// Local Imports
use dokime_domain::model;
use dokime_runtime::PromptId;
use dokime_runtime::Value;
use dokime_runtime::definition::TestCaseTemplate;
use dokime_runtime::definition::authoring::{criterion as authored_criterion, metadata as authored_metadata};
use dokime_runtime::definition::{DeclaredTemplate, Step, TemplateContext};
use dokime_runtime::execution::{Caller, SampleOutput, Workflow};
use dokime_runtime::execution::{CleanupContext, CriterionContext, InitializationContext, StepContext, TraceabilityContext};
use dokime_runtime::execution::{DeclaredObserver, InitializationFuture, ObserverContext, SidecarTemplate, StepFuture, TraceabilityFuture};
use dokime_runtime::plugin::Diagnostic;
use dokime_runtime::prelude::StreamExt as _;
use dokime_runtime::prompts::{PromptRequest, PromptResponse};
use dokime_runtime::transport::SidecarCriterionStatus;
use dokime_runtime::transport::*;
use {dokime_domain::model::traceability::TraceabilityRow, dokime_domain::model::traceability::TraceabilitySection};

/* === Constants === */
const WORKFLOW: &str = "probe.checks";
const TEMPLATE: &str = "probe.checks.pulse";
const STREAM: &str = "probe.pulse";
const CRITERION: &str = "probe.checks.acceptance";
const SOURCE: &str = "probe";
const ARTIFACT_EVIDENCE_BYTES: &[u8] = b"metric,value\npulse,1\n";

const WATCHED_REWRITE: &[u8] = b"rewritten\n";

const WATCH_SETTLE: Duration = Duration::from_millis(500);
const INTERVAL_MS: u64 = 5;
static ARTIFACT_GATE_FILLED: AtomicBool = AtomicBool::new(false);
static CLEANUP_RELEASE: tokio::sync::Notify = tokio::sync::Notify::const_new();
static ARTIFACT_GATE_REACHED: tokio::sync::Notify = tokio::sync::Notify::const_new();
static ARTIFACT_PUSH_FINISHED: AtomicBool = AtomicBool::new(false);
static INITIALIZATION_RELEASED: AtomicBool = AtomicBool::new(false);
static EXECUTION_RELEASED: AtomicBool = AtomicBool::new(false);
// Set when an observer samples after cleanup began; checked once the run settles.
static LATE_SAMPLE: AtomicBool = AtomicBool::new(false);

/* === Macros === */

/* === Type Aliases === */
type PublishedRow = (String, String, BTreeMap<String, Value>);

/* === Structs === */
struct Probe;

struct ProbeResources {
    produced: AtomicU64,
    produced_changed: tokio::sync::Notify,
    cleaning: AtomicBool,
    observer_wait: bool,
    sampling: AtomicBool,
    _resource: ExecutionResource,
}

struct PublishedExecutionRows(std::sync::Mutex<Vec<PublishedRow>>, Workflow<Probe>, std::sync::Mutex<dokime_runtime::RecordScope>);

// A host that refuses every row the way `host.publish` answers a refusal.
struct RefusedPublication(String);

struct SamplingGuard<'a>(&'a AtomicBool);

struct ExecutionResource {
    track: bool,
}

struct InitializationResource;

/* === Streams === */

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct PulseRow {
    index: i64,
    value: f64,
}

/* === Records === */

struct UnitConnected {
    serial: String,
}

#[dokime_runtime::prompt(id = "probe.confirm", label = "Confirm")]
struct Confirm;

/* === Enums === */
#[derive(Debug)]
enum ProbeError {
    Declined(String),
    Unmeasurable,
    Sdk(dokime_runtime::Error),
}

/* === Traits === */

/* === Implementations === */
impl dokime_runtime::execution::EmbeddedPublisher for PublishedExecutionRows {
    fn publish(
        &self, execution: &str, stream: &dokime_runtime::Identifier, values: BTreeMap<String, Value>, storage_payload: Vec<u8>,
    ) -> Result<(), String> {
        let sequence = {
            let mut rows = self.0.lock().unwrap();
            let sequence = rows.len() as u64;
            rows.push((execution.into(), stream.to_string(), values.clone()));
            sequence
        };
        let scope = self.2.lock().unwrap().clone();
        let mut sample = dokime_runtime::telemetry::TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            stream.clone(),
            sequence,
            dokime_runtime::model::time::now_utc(),
            SOURCE,
        );
        sample.values = values;
        sample.storage_payload = Some(storage_payload);
        sample.scope = Some(scope.clone());
        sample.received_at = sample.timestamp.clone();
        self.1.deliver_samples(&scope, &[sample], &[]).map_err(|error| error.to_string())?;
        Ok(())
    }
}

impl dokime_runtime::execution::EmbeddedPublisher for RefusedPublication {
    fn publish(&self, _: &str, _: &dokime_runtime::Identifier, _: BTreeMap<String, Value>, _: Vec<u8>) -> Result<(), String> {
        Err(self.0.clone())
    }
}

impl Drop for SamplingGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Drop for ExecutionResource {
    fn drop(&mut self) {
        if self.track {
            EXECUTION_RELEASED.store(true, Ordering::Release);
        }
    }
}

impl Drop for InitializationResource {
    fn drop(&mut self) {
        INITIALIZATION_RELEASED.store(true, Ordering::Release);
    }
}

impl dokime_runtime::telemetry::TelemetryRow for PulseRow {
    fn columns() -> Vec<dokime_runtime::telemetry::TelemetryColumn> {
        vec![
            dokime_runtime::telemetry::TelemetryColumn::declared(
                "index",
                "Index",
                dokime_runtime::ValueType::Integer,
                dokime_runtime::telemetry::TelemetryRole::Counter,
            ),
            dokime_runtime::telemetry::TelemetryColumn::declared(
                "value",
                "Value",
                dokime_runtime::ValueType::Number,
                dokime_runtime::telemetry::TelemetryRole::Value,
            ),
        ]
    }

    fn into_row(self) -> BTreeMap<String, Value> {
        BTreeMap::from([("index".to_owned(), Value::Integer(self.index)), ("value".to_owned(), Value::Number(self.value))])
    }

    fn from_row(values: &BTreeMap<String, Value>) -> Result<Self, dokime_runtime::FieldError> {
        Ok(Self { index: dokime_runtime::values::field(values, "index")?, value: dokime_runtime::values::field(values, "value")? })
    }
}

impl dokime_runtime::StreamSpec for PulseRow {
    type Row = Self;
    const ID: &'static str = STREAM;
    const VERSION: u32 = 1;
    const WINDOW: usize = 4096;
}

impl dokime_runtime::telemetry::TelemetryStream for PulseRow {
    fn contract() -> dokime_runtime::telemetry::TelemetryContract {
        dokime_runtime::telemetry::TelemetryContract::declared(<Self as dokime_runtime::StreamSpec>::ID, "Probe Pulse")
            .typed::<Self>()
            .columns(<Self as dokime_runtime::telemetry::TelemetryRow>::columns())
            .strict()
    }
}

impl dokime_runtime::telemetry::OwnedStream for PulseRow {}

impl dokime_runtime::Publishable for PulseRow {
    type Payload = Self;

    fn publication(self) -> Self {
        self
    }
}

impl From<PulseRow> for dokime_runtime::publication::Publication {
    fn from(row: PulseRow) -> Self {
        let payload = dokime_runtime::Message::encode_to_vec(&row);
        Self::Row {
            payload,
            stream: <PulseRow as dokime_runtime::telemetry::TelemetryStream>::stream_id(),
            values: <PulseRow as dokime_runtime::telemetry::TelemetryRow>::into_row(row),
        }
    }
}

impl dokime_runtime::records::DeclaredRecord for UnitConnected {
    const RECORD_TYPE: &'static str = "probe.unit.connected";
    const SEVERITY: dokime_runtime::records::RecordSeverity = dokime_runtime::records::RecordSeverity::Info;

    fn contract() -> dokime_runtime::records::RecordContract {
        dokime_runtime::records::RecordContract::declared(Self::RECORD_TYPE, "Unit connected").strict()
    }

    fn message(&self) -> String {
        "Unit connected".to_owned()
    }

    fn into_details(self) -> std::collections::BTreeMap<String, Value> {
        std::collections::BTreeMap::from([("serial".to_owned(), Value::from(self.serial))])
    }

    fn from_record(record: &model::records::evidence::Record) -> dokime_runtime::Result<Self> {
        Self::from_details(&record.details).map_err(|error| dokime_runtime::Error::record_field(Self::RECORD_TYPE, error))
    }

    fn from_details(values: &std::collections::BTreeMap<String, Value>) -> Result<Self, dokime_runtime::FieldError> {
        Ok(Self { serial: dokime_runtime::values::field(values, "serial")? })
    }
}

impl dokime_runtime::execution::Listenable for UnitConnected {
    type Item = Self;
    type Listening = dokime_runtime::execution::ListeningRecords<Self>;

    fn subscription() -> dokime_runtime::definition::Subscription {
        dokime_runtime::definition::Subscription::Record(<Self as dokime_runtime::records::DeclaredRecord>::contract().record_type)
    }

    fn listening(_streams: &dokime_runtime::telemetry::Streams, deliveries: &dokime_runtime::records::Deliveries) -> Self::Listening {
        dokime_runtime::execution::ListeningRecords::new(deliveries)
    }
}

impl fmt::Display for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declined(action) => write!(formatter, "the operator answered {action:?}"),
            Self::Unmeasurable => formatter.write_str("the probe could not measure"),
            Self::Sdk(error) => write!(formatter, "SDK operation failed: {error}"),
        }
    }
}

impl From<dokime_runtime::Error> for ProbeError {
    fn from(error: dokime_runtime::Error) -> Self {
        Self::Sdk(error)
    }
}

impl std::error::Error for ProbeError {}

impl DeclaredTemplate for Probe {
    const IDENTIFIER: &'static str = TEMPLATE;

    fn template_id() -> dokime_runtime::TemplateId {
        dokime_runtime::TemplateId::from_seed(TEMPLATE)
    }

    fn declaration() -> TestCaseTemplate {
        let mut ctx = TemplateContext::new();
        ctx.set_template_id(<Self as DeclaredTemplate>::template_id());
        ctx.set_metadata(authored_metadata(TEMPLATE, "Probe Pulse", None::<String>));
        ctx.set_subscriptions(vec![<UnitConnected as dokime_runtime::execution::Listenable>::subscription()]);
        ctx.set_steps(vec![Step::new("start", "Start"), Step::new("stream", "Stream"), Step::new("complete", "Complete")]);
        ctx.set_criteria(vec![authored_criterion(CRITERION, "Probe Accepted", Some(Value::from(true)))]);

        ctx.into_test_case_template().expect("the fixture template is declarable")
    }
}

impl SidecarTemplate for Probe {
    const RUN: &'static str = "probe.checks.pulse.run";
    const SIGNAL: &'static str = "probe.checks.pulse.signal";

    fn definition() -> dokime_runtime::execution::TemplateDefinition {
        dokime_runtime::execution::TemplateDefinition {
            template: <Self as DeclaredTemplate>::declaration(),
            records: <Self as DeclaredTemplate>::record_contracts(),
            streams: <Self as DeclaredTemplate>::stream_contracts(),
        }
    }

    type Own = ProbeResources;
    // Every case here carries whatever its own test set, with no declaration
    // behind it, so this fixture reads the raw map through `raw_parameters`.
    type Parameters = ();

    fn initialize<'a>(
        ctx: &'a mut InitializationContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> InitializationFuture<'a, (Self, Self::Own)> {
        Box::pin(async move {
            let state = dokime_runtime::plugin::captured(Self::initialized(ctx).await)?;
            let shared = dokime_runtime::plugin::captured(Self::opened(ctx).await)?;
            Ok((state, shared))
        })
    }

    fn traceability<'a>(&'a self, ctx: &'a TraceabilityContext<'_, Self, Self::Own, Self::Parameters>) -> TraceabilityFuture<'a> {
        Box::pin(async move { dokime_runtime::plugin::captured(Self::stated(ctx)) })
    }

    fn declared_observers() -> Vec<DeclaredObserver> {
        vec![
            DeclaredObserver::new("pulse", "Probe Pulse", INTERVAL_MS)
                .publishing(vec![<PulseRow as dokime_runtime::telemetry::TelemetryStream>::contract()], Vec::new()),
        ]
    }

    fn cleanup<'a>(&'a mut self, ctx: &'a mut CleanupContext<'_, Self, Self::Own, Self::Parameters>) -> StepFuture<'a> {
        Box::pin(async move { dokime_runtime::plugin::captured(self.cleaned_up(ctx).await) })
    }

    fn run_step<'a>(&'a mut self, step: &'a str, ctx: &'a mut StepContext<'_, Self, Self::Own, Self::Parameters>) -> StepFuture<'a> {
        Box::pin(async move { dokime_runtime::plugin::captured(self.ran(step, ctx).await) })
    }

    fn evaluate<'a>(
        criterion: &'a str, ctx: &'a CriterionContext<'_, Self, Self::Parameters>,
    ) -> dokime_runtime::execution::EvaluationFuture<'a, bool> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            if flag(ctx.raw_parameters(), "criterion_wait") {
                std::future::pending::<()>().await;
            }
            match criterion {
                CRITERION => dokime_runtime::plugin::captured(Self::accepted(ctx)).map(Some),
                _ => Ok(None),
            }
        })
    }

    fn observe<'a>(observer: &'a str, ctx: &'a mut ObserverContext<'_, Self, Self::Own, Self::Parameters>) -> StepFuture<'a> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            if observer != "pulse" {
                return Ok(());
            }
            let resources = ctx.resources();
            if resources.cleaning.load(Ordering::Acquire) {
                LATE_SAMPLE.store(true, Ordering::Release);
            }
            resources.sampling.store(true, Ordering::Release);
            let _sampling = SamplingGuard(&resources.sampling);
            let index = resources.produced.load(Ordering::Acquire);
            if resources.observer_wait {
                // The pending-observer case deliberately finishes its step before publication.
                resources.produced.fetch_add(1, Ordering::AcqRel);
                resources.produced_changed.notify_waiters();
                std::future::pending::<()>().await;
            }
            ctx.publish(PulseRow { index: index as i64, value: index as f64 * 0.5 }).await;
            resources.produced.fetch_add(1, Ordering::AcqRel);
            resources.produced_changed.notify_waiters();

            Ok(())
        })
    }
}

impl Probe {
    async fn initialized(ctx: &mut InitializationContext<'_, Probe, ProbeResources>) -> Result<Self, ProbeError> {
        ctx.info("probe-initializing");
        if flag(ctx.raw_parameters(), "diagnostics") {
            ctx.trace("initializing trace");
            ctx.debug("initializing debug");
            ctx.info("initializing info");
            ctx.warn("initializing warn");
            ctx.error("initializing error");
        }
        if flag(ctx.raw_parameters(), "read_context") {
            assert_eq!(ctx.parameter::<String>("images.directory").unwrap(), "/captured");
            assert_eq!(ctx.secret("api_token").unwrap().expose(), "runtime-only-token");
            assert!(flag(ctx.raw_parameters(), "read_context"));
        }
        if flag(ctx.raw_parameters(), "initialize_error") {
            return Err(ProbeError::Unmeasurable);
        }
        if flag(ctx.raw_parameters(), "initialize_wait") {
            let _resource = InitializationResource;
            std::future::pending::<()>().await;
        }

        Ok(Self)
    }

    async fn opened(ctx: &InitializationContext<'_, Probe, ProbeResources>) -> Result<ProbeResources, ProbeError> {
        if flag(ctx.raw_parameters(), "resources_error") {
            return Err(ProbeError::Unmeasurable);
        }

        Ok(ProbeResources {
            produced: AtomicU64::new(0),
            produced_changed: tokio::sync::Notify::new(),
            cleaning: AtomicBool::new(false),
            observer_wait: flag(ctx.raw_parameters(), "observer_wait"),
            sampling: AtomicBool::new(false),
            _resource: ExecutionResource { track: flag(ctx.raw_parameters(), "observe_release") },
        })
    }

    fn stated(ctx: &TraceabilityContext<'_, Probe, ProbeResources>) -> Result<Vec<TraceabilitySection>, ProbeError> {
        if flag(ctx.raw_parameters(), "traceability_error") {
            return Err(ProbeError::Unmeasurable);
        }
        if !flag(ctx.raw_parameters(), "traceability") {
            return Ok(Vec::new());
        }

        let mut dut = TraceabilitySection::new("dut", "Device Under Test");
        dut.row(TraceabilityRow::text("serial_number", "Serial number", "SN-7"));
        dut.row(TraceabilityRow::number("target_c", "Target", 60.0).unit("°C"));
        Ok(vec![dut])
    }

    async fn cleaned_up(&mut self, ctx: &mut CleanupContext<'_, Probe, ProbeResources>) -> Result<(), ProbeError> {
        assert!(!ctx.resources().sampling.load(Ordering::Acquire), "observers must settle before cleanup");
        ctx.resources().cleaning.store(true, Ordering::Release);
        ctx.info(format!("cleanup-reason:{:?}", ctx.reason()));
        if flag(ctx.raw_parameters(), "diagnostics") {
            ctx.trace("cleanup trace");
            ctx.debug("cleanup debug");
            ctx.info("cleanup info");
            ctx.warn("cleanup warn");
            ctx.error("cleanup error");
        }
        if flag(ctx.raw_parameters(), "read_context") {
            assert_eq!(ctx.parameter::<String>("images.directory").unwrap(), "/captured");
            assert_eq!(ctx.secret("api_token").unwrap().expose(), "runtime-only-token");
            assert!(flag(ctx.raw_parameters(), "read_context"));
        }
        if flag(ctx.raw_parameters(), "cleanup_wait") {
            ctx.info("cleanup-waiting");
            CLEANUP_RELEASE.notified().await;
        }
        if flag(ctx.raw_parameters(), "artifact_directory") {
            // The sweep runs after cleanup, so what teardown writes is evidence
            // like anything else the run produced.
            let directory = ctx.artifacts().directory("capture").map_err(|error| ProbeError::Declined(error.to_string()))?;
            std::fs::write(directory.join("released.txt"), b"released\n").map_err(|error| ProbeError::Declined(error.to_string()))?;
        }
        ctx.info("cleanup-finished");
        if ctx.raw_parameters().get("cleanup_error") == Some(&Value::Bool(true)) {
            return Err(ProbeError::Unmeasurable);
        }

        Ok(())
    }

    async fn ran(&mut self, step: &str, ctx: &mut StepContext<'_, Probe, ProbeResources>) -> Result<(), ProbeError> {
        match step {
            "start" => self.start(ctx).await,
            "stream" => self.stream(ctx).await,
            "complete" => self.complete(ctx).await,
            _ => Ok(()),
        }
    }

    async fn start(&mut self, ctx: &mut StepContext<'_, Probe, ProbeResources>) -> Result<(), ProbeError> {
        if flag(ctx.raw_parameters(), "panic") {
            panic!("the probe blew up");
        }
        if flag(ctx.raw_parameters(), "diagnostics") {
            ctx.trace("step trace");
            ctx.debug("step debug");
            ctx.info("step info");
            ctx.warn("step warn");
            ctx.error("step error");
        }
        if flag(ctx.raw_parameters(), "read_context") {
            let folder: String = ctx.parameter("images.directory").map_err(|error| ProbeError::Declined(error.to_string()))?;
            let token = ctx.secret("api_token").map_err(|error| ProbeError::Declined(error.to_string()))?;
            if folder != "/captured" || token.expose() != "runtime-only-token" {
                return Err(ProbeError::Declined("incorrect execution context".into()));
            }
            let diagnostic = format!("credential {token:?}");
            ctx.info(diagnostic);
        }
        if flag(ctx.raw_parameters(), "artifact_backpressure") {
            let mut writer =
                ctx.artifacts().create("evidence/backpressure.bin").map_err(|error| ProbeError::Declined(error.to_string()))?;
            // Fill the 16 MiB backlog with protocol-sized pieces before asking
            // the public writer to emit its terminal byte.
            let chunk = vec![7; MAX_ARTIFACT_CHUNK_BYTES];
            for _ in 0..(16 * 1024 * 1024 / MAX_ARTIFACT_CHUNK_BYTES) {
                writer.write_all(&chunk).map_err(|error| ProbeError::Declined(error.to_string()))?;
            }
            ARTIFACT_GATE_FILLED.store(true, Ordering::Release);
            ARTIFACT_GATE_REACHED.notify_one();
            writer.write_all(&[1]).map_err(|error| ProbeError::Declined(error.to_string()))?;
            writer.finish().map_err(|error| ProbeError::Declined(error.to_string()))?;
            ARTIFACT_PUSH_FINISHED.store(true, Ordering::Release);
        }
        if flag(ctx.raw_parameters(), "artifact_evidence") {
            let artifacts = ctx.artifacts();
            artifacts.write("evidence/pulse.csv", ARTIFACT_EVIDENCE_BYTES).map_err(|error| ProbeError::Declined(error.to_string()))?;
            let mut writer = artifacts.create("evidence/trace.bin").map_err(|error| ProbeError::Declined(error.to_string()))?;
            for slice in streamed_evidence().chunks(4096) {
                writer.write_all(slice).map_err(|error| ProbeError::Declined(error.to_string()))?;
            }
            writer.finish().map_err(|error| ProbeError::Declined(error.to_string()))?;
        }
        if flag(ctx.raw_parameters(), "artifact_directory") {
            // Plain `std::fs`, because the producer a directory exists for is
            // the one that knows nothing about Dokime: it is handed a path and
            // writes files, and the SDK is what notices.
            let directory = ctx.artifacts().directory("capture").map_err(|error| ProbeError::Declined(error.to_string()))?;
            ctx.info(format!("directory:{}", directory.display()));
            let write = |relative: &str, bytes: &[u8]| {
                std::fs::write(directory.join(relative), bytes).map_err(|error| ProbeError::Declined(error.to_string()))
            };
            write("chamber.log", &watched_evidence())?;
            std::fs::create_dir_all(directory.join("frames")).map_err(|error| ProbeError::Declined(error.to_string()))?;
            write("frames/first.txt", b"nested\n")?;
            // A file whose name does not say what it is stays where it is.
            write("notes", b"unreadable media type\n")?;
            // TODO(source-hook): an artifact directory event that reports the files it has taken in.
            tokio::time::sleep(WATCH_SETTLE).await;
        }
        if flag(ctx.raw_parameters(), "await_record") {
            // The whole of the authored form: name the type, bound the wait with
            // the standard adapter, and let `?` carry its deadline into this
            // plugin's own error.
            let connected = ctx
                .listen::<UnitConnected>()
                .timeout(Duration::from_millis(count(ctx.raw_parameters(), "await_record_ms")))
                .next()
                .await
                .transpose()?
                .expect("a live subscription yields until it is dropped");
            ctx.info(format!("connected:{}", connected.serial));
        }
        if !flag(ctx.raw_parameters(), "ask") {
            return Ok(());
        }
        match ctx.ask(Confirm).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(ProbeError::Declined("no".into())),
            // A wait that ended without an answer ended because the run did.
            Err(_) => Ok(()),
        }
    }

    async fn complete(&mut self, ctx: &mut StepContext<'_, Probe, ProbeResources>) -> Result<(), ProbeError> {
        if !flag(ctx.raw_parameters(), "artifact_directory") {
            return Ok(());
        }
        let directory = ctx.artifacts().directory("capture").map_err(|error| ProbeError::Declined(error.to_string()))?;
        ctx.info(format!("directory:{}", directory.display()));
        std::fs::write(directory.join("chamber.log"), WATCHED_REWRITE).map_err(|error| ProbeError::Declined(error.to_string()))?;

        Ok(())
    }

    async fn stream(&mut self, ctx: &mut StepContext<'_, Probe, ProbeResources>) -> Result<(), ProbeError> {
        let before = if ctx.resources().observer_wait { 0 } else { ctx.resources().produced.load(Ordering::Acquire) };
        let wanted = before + count(ctx.raw_parameters(), "samples");
        loop {
            let produced = ctx.resources().produced_changed.notified();
            tokio::pin!(produced);
            produced.as_mut().enable();
            if ctx.resources().produced.load(Ordering::Acquire) >= wanted {
                break;
            }
            produced.await;
        }

        Ok(())
    }

    fn accepted(ctx: &CriterionContext<'_, Probe>) -> Result<bool, ProbeError> {
        if flag(ctx.raw_parameters(), "observe_release") {
            assert!(EXECUTION_RELEASED.load(Ordering::Acquire), "all instance resources must release before criterion evaluation");
        }
        if flag(ctx.raw_parameters(), "unmeasurable") {
            return Err(ProbeError::Unmeasurable);
        }

        Ok(ctx.samples::<PulseRow>().len() as u64 >= count(ctx.raw_parameters(), "samples"))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn streamed_evidence() -> Vec<u8> {
    (0..dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES + 321).map(|index| (index % 251) as u8).collect()
}

fn watched_evidence() -> Vec<u8> {
    (0..200_000).map(|index| b"0123456789\n"[index % 11]).collect()
}

fn flag(parameters: &BTreeMap<String, Value>, name: &str) -> bool {
    matches!(parameters.get(name), Some(Value::Bool(true)))
}

fn count(parameters: &BTreeMap<String, Value>, name: &str) -> u64 {
    match parameters.get(name) {
        Some(Value::Integer(value)) => u64::try_from(*value).unwrap_or(1),
        _ => 1,
    }
}

fn cycle() -> dokime_runtime::TestCycleId {
    dokime_runtime::TestCycleId::from_seed("probe-cycle")
}

fn execution() -> dokime_runtime::TestExecutionId {
    dokime_runtime::TestExecutionId::from_seed("probe-execution")
}

#[tokio::test]
async fn interactive_commands_downstream() {
    let mut definition = Probe::definition();
    for step in definition.template.phases.iter_mut().take(2) {
        step.repeatable = true;
    }
    let step_ids: BTreeMap<_, _> =
        definition.template.phases.iter().map(|step| (step.metadata.identifier.to_string(), step.step_id.clone())).collect();
    let workflow = Workflow::<Probe>::declaring(WORKFLOW, SOURCE, definition, Probe::declared_observers());
    let token = "interactive-probe";
    let mut start = start_request(token, serde_json::json!({"samples": 1}));
    start.mode = "interactive".into();
    workflow.start(start.clone()).unwrap();
    workflow.start(start).unwrap();
    let mut cursor = 0;
    let mut replies = Vec::new();
    let mut commands = Vec::new();
    for (action, step) in [("next", ""), ("next", ""), ("run_step", "start"), ("next", ""), ("next", "")] {
        let command = CommandExecutionRequest {
            execution_token: token.into(),
            command_id: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>().to_string(),
            action: action.into(),
            step: step.into(),
        };
        let receipt = settle_command(&workflow, &command, &mut cursor, &mut replies).await;
        assert_eq!(receipt.state, "completed");
        assert!(!receipt.attempt_id.is_empty());
        commands.push((command, receipt));
    }
    assert_ne!(commands[0].1.attempt_id, commands[2].1.attempt_id);
    let blocked = CommandExecutionRequest {
        execution_token: token.into(),
        command_id: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>().to_string(),
        action: "run_step".into(),
        step: "start".into(),
    };
    let refusal = settle_command(&workflow, &blocked, &mut cursor, &mut replies).await;
    assert_eq!(refusal.state, "refused");
    assert!(refusal.attempt_id.is_empty());
    assert!(refusal.message.contains("non-repeatable downstream"));
    assert!(!replies.iter().any(|reply| reply.outcome.is_some()));
    for (command, receipt) in &commands {
        assert_eq!(&workflow.command(command.clone()).unwrap(), receipt);
    }
    let finish = CommandExecutionRequest {
        execution_token: token.into(),
        command_id: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>().to_string(),
        action: "finish".into(),
        step: String::new(),
    };
    let receipt = settle_command(&workflow, &finish, &mut cursor, &mut replies).await;
    assert_eq!(receipt.state, "completed");
    if !replies.iter().any(|reply| reply.outcome.is_some()) {
        replies.extend(drain_from(&workflow, token, None, cursor).await);
    }
    let attempts: Vec<dokime_domain::model::execution::result::TestStepAttempt> =
        replies.iter().flat_map(|reply| &reply.step_attempts).map(|encoded| serde_json::from_str(encoded).unwrap()).collect();
    assert_eq!(attempts.len(), 5);
    let expected = ["start", "stream", "start", "stream", "complete"];
    for (attempt, identifier) in attempts.iter().zip(expected) {
        assert_eq!(attempt.step_id, step_ids[identifier]);
    }
    let logs: Vec<_> = replies.iter().flat_map(|reply| &reply.diagnostics).collect();
    assert_eq!(logs.iter().filter(|entry| entry.message == "probe-initializing").count(), 1);
    assert_eq!(logs.iter().filter(|entry| entry.message == "cleanup-finished").count(), 1);
    assert!(
        workflow
            .command(CommandExecutionRequest {
                command_id: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>().to_string(),
                ..blocked
            })
            .is_err()
    );
}

#[tokio::test]
async fn interactive_idle_stop() {
    let workflow = workflow();
    let token = "interactive-idle";
    let mut start = start_request(token, serde_json::json!({"samples": 1}));
    start.mode = "interactive".into();
    workflow.start(start).unwrap();
    let mut cursor = 0;
    let mut replies = Vec::new();
    let mut observed = 0;
    while observed < 2 {
        let reply = next(&workflow, token, cursor).await;
        cursor = reply.cursor;
        assert!(reply.step_begins.is_empty());
        assert!(reply.step_attempts.is_empty());
        assert!(reply.outcome.is_none());
        assert!(!reply.diagnostics.iter().any(|entry| entry.message == "cleanup-finished"));
        replies.push(reply);
        observed += workflow.take_local_samples(token).unwrap().len();
    }
    workflow.cancel(token);
    replies.extend(drain_from(&workflow, token, None, cursor).await);
    assert!(replies.iter().all(|reply| reply.step_attempts.is_empty()));
    assert_eq!(replies.iter().flat_map(|reply| &reply.diagnostics).filter(|entry| entry.message == "cleanup-finished").count(), 1);
    assert!(
        workflow
            .command(CommandExecutionRequest {
                execution_token: token.into(),
                command_id: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>().to_string(),
                action: "next".into(),
                step: String::new(),
            })
            .is_err()
    );
}

async fn settle_command(
    workflow: &Workflow<Probe>, command: &CommandExecutionRequest, cursor: &mut u64, replies: &mut Vec<EvidenceBatch>,
) -> CommandExecutionReply {
    workflow.command(command.clone()).unwrap();
    loop {
        let reply = next(workflow, &command.execution_token, *cursor).await;
        *cursor = reply.cursor;
        if reply.record_subscriptions_settling {
            workflow.settle_records(&command.execution_token);
        }
        replies.push(reply);
        let receipt = workflow.command(command.clone()).unwrap();
        if !matches!(receipt.state.as_str(), "accepted" | "running") {
            return receipt;
        }
    }
}

fn workflow() -> Workflow<Probe> {
    Workflow::<Probe>::new(WORKFLOW, SOURCE)
}

fn encoded_parameters(parameters: serde_json::Value) -> String {
    let Value::Object(values) = Value::from_plain_json(parameters).expect("fixture values") else {
        panic!("fixture parameters are a map");
    };

    serde_json::to_string(&values).unwrap()
}

fn start_request(token: &str, parameters: serde_json::Value) -> RunExecution {
    RunExecution {
        mode: "automatic".into(),
        execution_token: token.to_owned(),
        test_cycle_id: cycle().to_string(),
        test_execution_id: execution().to_string(),
        template_identifier: TEMPLATE.to_owned(),
        workflow: WORKFLOW.to_owned(),

        secrets_json: "{}".to_owned(),
        parameters_json: encoded_parameters(parameters),
    }
}

fn drain<T: SidecarTemplate>(workflow: &Workflow<T>, token: &str, cursor: u64) -> EvidenceBatch {
    let reply = workflow.drain(token, cursor).expect("the execution remains drainable");
    for encoded in &reply.step_begins {
        let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
        workflow.admit(token, &StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: true, message: String::new() });
    }
    reply
}

async fn next<T: SidecarTemplate>(workflow: &Workflow<T>, token: &str, cursor: u64) -> EvidenceBatch {
    let reply = workflow.next(token, cursor).await.expect("the execution remains retained");
    for encoded in &reply.step_begins {
        let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
        workflow.admit(token, &StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: true, message: String::new() });
    }
    reply
}

async fn drain_until_terminal(workflow: &Workflow<Probe>, token: &str, answer: Option<(&str, bool)>) -> Vec<EvidenceBatch> {
    drain_from(workflow, token, answer, 0).await
}

async fn drain_from(workflow: &Workflow<Probe>, token: &str, answer: Option<(&str, bool)>, mut cursor: u64) -> Vec<EvidenceBatch> {
    let mut replies = Vec::new();
    loop {
        let reply = next(workflow, token, cursor).await;
        cursor = reply.cursor;
        if reply.record_subscriptions_settling {
            workflow.settle_records(token);
        }
        if let Some((action, accepted)) = answer {
            for request in &reply.prompt_requests {
                respond(workflow, token, &decode(request).prompt_id, action, accepted);
            }
        }
        let terminal = reply.outcome.is_some();
        replies.push(reply);
        if terminal {
            assert!(!LATE_SAMPLE.load(Ordering::Acquire), "an observer must join before cleanup");
            return replies;
        }
    }
}

async fn drain_until_prompt(workflow: &Workflow<Probe>, token: &str) -> (u64, EvidenceBatch) {
    let mut cursor = 0;
    loop {
        let reply = next(workflow, token, cursor).await;
        if !reply.prompt_requests.is_empty() {
            return (cursor, reply);
        }
        cursor = reply.cursor;
    }
}

async fn drain_until_line(workflow: &Workflow<Probe>, token: &str, message: &str) {
    let mut cursor = 0;
    loop {
        let reply = next(workflow, token, cursor).await;
        cursor = reply.cursor;
        if reply.diagnostics.iter().any(|line| line.message == message) {
            return;
        }
    }
}

fn decode(request: &str) -> PromptRequest {
    serde_json::from_str(request).expect("the engine buffers a decodable operator question")
}

fn respond(workflow: &Workflow<Probe>, token: &str, prompt_id: &PromptId, action: &str, accepted: bool) {
    let response = PromptResponse {
        prompt_id: prompt_id.clone(),
        action: action.to_owned(),
        responder: "test".to_owned(),
        submitted_at: dokime_runtime::model::time::now_utc(),
        values: BTreeMap::new(),
        accepted,
    };
    workflow
        .answer(token, &prompt_id.to_string(), &serde_json::to_string(&response).expect("a fixture answer encodes"))
        .expect("delivering an operator answer is accepted");
}

fn outcome(
    replies: &[EvidenceBatch],
) -> (&dokime_runtime::transport::SidecarExecutionOutcome, &dokime_runtime::transport::SidecarCriterionOutcome) {
    let outcome = replies.iter().find_map(|reply| reply.outcome.as_ref()).expect("the drain stream terminates once");
    let criterion = outcome
        .criteria
        .iter()
        .find(|criterion| criterion.identifier == CRITERION)
        .expect("the terminal outcome decides the declared criterion");

    (outcome, criterion)
}

fn diagnostics(replies: &[EvidenceBatch]) -> impl Iterator<Item = &dokime_runtime::execution::log::Entry> {
    replies.iter().flat_map(|reply| reply.diagnostics.iter())
}

fn assembled(chunks: &[&dokime_runtime::transport::ArtifactChunk]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for chunk in chunks {
        if chunk.replace {
            bytes.clear();
        }
        bytes.extend_from_slice(&chunk.bytes);
    }

    bytes
}

fn status(criterion: &dokime_runtime::transport::SidecarCriterionOutcome) -> dokime_runtime::transport::SidecarCriterionStatus {
    dokime_runtime::transport::SidecarCriterionStatus::try_from(criterion.status).expect("the engine reports a known criterion status")
}

fn detail(replies: &[EvidenceBatch]) -> dokime_runtime::diagnostics::Diagnostic {
    let encoded = outcome(replies).0.detail_json.as_deref().expect("a failed execution carries its detail");

    serde_json::from_str(encoded).expect("the engine encodes a decodable diagnostic")
}

// The paused clock lets the 30 ms window below elapse only once every task
// is idle, so an unadmitted step has had every chance to run.
#[tokio::test(start_paused = true)]
async fn unacknowledged_invoke_callback() {
    for cancel in [false, true] {
        let workflow = workflow();
        let token = "admission";
        workflow.start(start_request(token, serde_json::json!({ "samples": 1, "diagnostics": true }))).unwrap();
        let mut cursor = 0;
        let start = loop {
            let reply = workflow.next(token, cursor).await.unwrap();
            cursor = reply.cursor;
            if let Some(encoded) = reply.step_begins.first() {
                break serde_json::from_str::<dokime_domain::model::execution::result::StepAttemptStart>(encoded).unwrap();
            }
        };
        tokio::time::advance(Duration::from_millis(30)).await;
        let waiting = workflow.drain(token, cursor).unwrap();
        assert!(waiting.outcome.is_none());
        assert!(waiting.step_attempts.is_empty());
        assert!(!waiting.diagnostics.iter().any(|line| line.message == "step info"));
        cursor = waiting.cursor;
        if cancel {
            workflow.cancel(token);
        } else {
            workflow.admit(
                token,
                &StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: false, message: "host refused step".into() },
            );
        }
        let replies = drain_from(&workflow, token, None, cursor).await;
        assert!(!outcome(&replies).0.passed);
        assert!(replies.iter().all(|reply| reply.step_attempts.is_empty()));
        assert!(!replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message == "step info"));
        workflow.shutdown().await;
    }
}

#[tokio::test]
async fn embedded_execution_output() {
    let workflow = workflow();
    let rows = Arc::new(PublishedExecutionRows(
        std::sync::Mutex::new(Vec::new()),
        workflow.clone(),
        std::sync::Mutex::new(dokime_runtime::RecordScope {
            test_cycle_id: Some(cycle()),
            test_execution_id: Some(execution()),
            origin: dokime_runtime::EvidenceOrigin::Initialization,
            ..Default::default()
        }),
    ));
    let output = SampleOutput::Embedded(rows.clone());
    let start = start_request("embedded", serde_json::json!({"samples": 2}));
    for _ in 0..2 {
        workflow.start_on(start.clone(), Some(output.clone()), Caller::default()).unwrap();
    }

    let mut replies = Vec::new();
    let mut cursor = 0;
    loop {
        let reply = workflow.next("embedded", cursor).await.unwrap();
        cursor = reply.cursor;
        for encoded in &reply.capture_phases {
            let phase: dokime_runtime::CapturePhase = serde_json::from_str(encoded).unwrap();
            rows.2.lock().unwrap().origin = phase.into();
        }
        for encoded in &reply.step_begins {
            let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            rows.2.lock().unwrap().origin = dokime_runtime::EvidenceOrigin::Step { attempt_id: start.attempt_id.clone() };
            workflow.admit("embedded", &StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: true, message: String::new() });
        }
        if reply.record_subscriptions_settling {
            workflow.settle_records("embedded");
        }
        let terminal = reply.outcome.is_some();
        replies.push(reply);
        if terminal {
            break;
        }
    }
    assert!(outcome(&replies).0.passed, "{:?}", outcome(&replies).0);
    // A repeated cursor re-delivers the terminal outcome.
    let replayed = workflow.drain("embedded", replies[..replies.len() - 1].last().map_or(0, |reply| reply.cursor)).unwrap();
    assert!(replayed.outcome.unwrap().passed);
    assert!(workflow.take_local_samples("embedded").unwrap().is_empty(), "hosted output must not use the fixture buffer");
    {
        let rows = rows.0.lock().unwrap();
        assert!(rows.len() >= 2);
        for (index, (token, stream, values)) in rows.iter().enumerate() {
            assert_eq!(token, "embedded");
            assert_eq!(stream, STREAM);
            assert_eq!(values["index"], Value::Integer(index as i64));
        }
    }
    workflow.shutdown().await;
}

// A host that cannot take a row in losslessly refuses it; the execution fails
// naming the stream and the refusal instead of losing rows or the plugin.
#[tokio::test]
async fn refused_publication_fails() {
    for refusal in [harmos::streams::StreamPublicationError::Backlog, harmos::streams::StreamPublicationError::Capacity] {
        let cause =
            harmos::Error::<std::convert::Infallible>::OperationFailed { operation: "host.publish".into(), source: Arc::new(refusal) }
                .to_string();
        let workflow = workflow();
        let output = SampleOutput::Embedded(Arc::new(RefusedPublication(cause.clone())));
        workflow.start_on(start_request("refused", serde_json::json!({"samples": 1})), Some(output), Caller::default()).unwrap();

        let replies = drain_until_terminal(&workflow, "refused", None).await;

        let detail = detail(&replies);

        assert!(!outcome(&replies).0.passed);
        assert_eq!(detail.message, format!("step `publishing stream evidence` failed in {TEMPLATE}"));
        assert_eq!(detail.chain, vec![format!("stream {STREAM} publication failed: {cause}")]);
        workflow.shutdown().await;
    }
}

#[tokio::test]
async fn runtime_bindings_metadata() {
    for identifier in ["registered.first", "registered.second"] {
        let mut definition = Probe::declaration();
        definition.metadata.identifier = dokime_runtime::Identifier::new(identifier).unwrap();
        definition.test_case_template_id = dokime_runtime::TemplateId::from_seed(identifier);
        let workflow = Workflow::<Probe>::declaring(
            WORKFLOW,
            SOURCE,
            dokime_runtime::execution::TemplateDefinition {
                template: definition,
                records: <Probe as DeclaredTemplate>::record_contracts(),
                streams: <Probe as DeclaredTemplate>::stream_contracts(),
            },
            <Probe as SidecarTemplate>::declared_observers(),
        );
        let mut request = start_request(identifier, serde_json::json!({"samples": 1}));
        assert!(workflow.start(request.clone()).is_err(), "the type's static identity is not registered");
        request.template_identifier = identifier.into();
        workflow.start(request).unwrap();

        let replies = drain_until_terminal(&workflow, identifier, None).await;
        assert!(outcome(&replies).0.passed);
        assert_eq!(workflow.template_identifier(), identifier);
        workflow.shutdown().await;
    }
}

#[tokio::test]
async fn stated_section_step() {
    let workflow = workflow();
    workflow.start(start_request("stated", serde_json::json!({"samples": 1, "traceability": true}))).unwrap();

    let replies = drain_until_terminal(&workflow, "stated", None).await;

    let stated = replies
        .iter()
        .flat_map(|reply| reply.traceability.iter())
        .map(|encoded| serde_json::from_str::<TraceabilitySection>(encoded).expect("the engine encodes a decodable section"))
        .collect::<Vec<_>>();
    assert_eq!(stated.len(), 1);
    assert_eq!(stated[0].identifier.as_str(), "dut");
    assert_eq!(stated[0].label, "Device Under Test");
    assert_eq!(stated[0].rows[1].unit.as_deref(), Some("°C"));
    // Stated before the run measured anything, which is the whole point of the
    // hook: an operator can read what is on the bench while it is still going.
    let stated_cursor = replies.iter().position(|reply| !reply.traceability.is_empty()).expect("a reply carries the section");
    let first_step = replies.iter().position(|reply| reply.phase == "start").expect("the run entered its first step");
    assert!(stated_cursor < first_step);
    assert!(outcome(&replies).0.passed);
}

#[tokio::test]
async fn traceability_failure_up() {
    let workflow = workflow();
    workflow.start(start_request("unstated", serde_json::json!({"samples": 1, "traceability_error": true}))).unwrap();

    let replies = drain_until_terminal(&workflow, "unstated", None).await;

    let (outcome, _) = outcome(&replies);
    assert!(!outcome.passed);
    assert!(outcome.message.as_deref().is_some_and(|message| message.contains("traceability")), "{outcome:?}");
    // No step ran, and the handles initialization constructed were still released.
    assert!(workflow.take_local_samples("unstated").unwrap().is_empty());
    assert!(replies.iter().flat_map(|reply| reply.diagnostics.iter()).any(|entry| entry.message.contains("cleanup-finished")));
}

#[tokio::test]
async fn failing_step_backtrace() {
    // The layer is what makes `SpanTrace::capture` see anything at all; without
    // it the span half of every diagnostic in this process is silently empty.
    dokime_runtime::plugin::install_subscriber();
    let workflow = workflow();
    workflow.start(start_request("failing", serde_json::json!({"samples": 1, "ask": true}))).unwrap();

    let replies = drain_until_terminal(&workflow, "failing", Some(("no", true))).await;
    let detail = detail(&replies);

    assert_eq!(detail.message, format!("step `start` failed in {TEMPLATE}"));
    // The author's own words survive as the cause, under the engine's framing.
    assert_eq!(detail.chain, vec![r#"the operator answered "no""#]);
    // The location is the generated call into authored code, in the file the
    // template was written in — not a line in the engine that ran it.
    assert!(detail.location.replace('\\', "/").contains("tests/integration/engine/execution/workflow.rs"), "{}", detail.location);
    assert!(detail.span_trace.iter().any(|span| span.contains("step=start")), "{:?}", detail.span_trace);
    assert!(detail.span_trace.iter().any(|span| span.contains(TEMPLATE) && span.contains(SOURCE)), "{:?}", detail.span_trace);
    assert!(detail.backtrace.contains("0:"), "{}", detail.backtrace);
    // The rendered form is the console layout, headline first.
    assert!(detail.to_string().starts_with(&format!("error: step `start` failed in {TEMPLATE}\n  caused by: ")));
    assert!(diagnostics(&replies).any(|entry| {
        entry.level == dokime_runtime::execution::log::Level::Error as i32 && entry.message == detail.redacted().to_string()
    }));
    workflow.shutdown().await;
}

#[tokio::test]
async fn lifecycle_logs_failure() {
    for (flag, stage) in [
        ("initialize_error", "initialization"),
        ("resources_error", "initialization"),
        ("traceability_error", "traceability"),
        ("cleanup_error", "cleanup"),
    ] {
        let workflow = workflow();
        workflow.start(start_request("stage-error", serde_json::json!({"samples": 1, (flag): true}))).unwrap();
        let replies = drain_until_terminal(&workflow, "stage-error", None).await;
        let errors = diagnostics(&replies)
            .filter(|entry| entry.level == dokime_runtime::execution::log::Level::Error as i32 && entry.message.starts_with("error: "))
            .collect::<Vec<_>>();

        assert!(!outcome(&replies).0.passed);
        assert_eq!(errors.len(), 1, "{stage}");
        assert_eq!(errors[0].message, detail(&replies).redacted().to_string());
        assert!(errors[0].message.contains(&format!("{stage} failed")));
        assert!(errors[0].message.contains("backtrace:"));
        workflow.shutdown().await;
    }

    let workflow = workflow();
    workflow.start(start_request("two-errors", serde_json::json!({"samples": 1, "ask": true, "cleanup_error": true}))).unwrap();
    let replies = drain_until_terminal(&workflow, "two-errors", Some(("no", true))).await;
    let errors = diagnostics(&replies)
        .filter(|entry| entry.level == dokime_runtime::execution::log::Level::Error as i32 && entry.message.starts_with("error: "))
        .collect::<Vec<_>>();

    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].message, detail(&replies).redacted().to_string());
    assert!(errors[1].message.contains("cleanup failed"));
    assert!(errors[1].message.contains("backtrace:"));
    workflow.shutdown().await;
}

#[tokio::test]
async fn panicking_step_diagnostic() {
    dokime_runtime::plugin::install_panic_hook();
    let workflow = workflow();
    workflow.start(start_request("panicking", serde_json::json!({"samples": 1, "panic": true}))).unwrap();

    // The run reaches a terminal outcome at all, which is the point: a panic
    // used to unwind the task and leave the host waiting on a dead execution.
    let replies = drain_until_terminal(&workflow, "panicking", None).await;
    let detail = detail(&replies);

    assert_eq!(detail.message, format!("step `start` failed in {TEMPLATE}"));
    assert_eq!(detail.chain, vec!["plugin panicked: the probe blew up"]);
    // Captured at the panic site by the hook, so the location is the `panic!`
    // itself rather than the boundary that caught the unwind.
    assert!(detail.location.replace('\\', "/").contains("tests/integration/engine/execution/workflow.rs"), "{}", detail.location);
    assert!(!outcome(&replies).0.passed);
    workflow.shutdown().await;
}

#[tokio::test]
async fn context_scoped_stable() {
    let workflow = workflow();
    let mut request = start_request("context", serde_json::json!({"samples": 1, "read_context": true}));
    request.parameters_json = encoded_parameters(serde_json::json!({"read_context": true, "images.directory": "/captured"}));
    request.secrets_json = serde_json::json!({"api_token": "runtime-only-token"}).to_string();
    assert!(!format!("{request:?}").contains("runtime-only-token"));
    workflow.start(request.clone()).unwrap();
    workflow.start(request).unwrap();
    let replies = drain_until_terminal(&workflow, "context", None).await;
    assert!(outcome(&replies).0.passed);
    assert!(replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message.contains("<redacted>")));
    assert!(!format!("{replies:?}").contains("runtime-only-token"));
    workflow.shutdown().await;
}

#[tokio::test]
async fn declared_stream_decides() {
    let workflow = workflow();
    let request = start_request("streaming", serde_json::json!({ "samples": 3 }));
    workflow.start(request.clone()).expect("the execution starts");
    workflow.start(request).expect("an identical start is idempotent");

    let replies = drain_until_terminal(&workflow, "streaming", None).await;
    let samples = workflow.take_local_samples("streaming").unwrap();
    let (outcome, criterion) = outcome(&replies);

    assert!(samples.len() >= 3, "the stream sampled for the run");
    assert!(samples.iter().all(|sample| {
        sample
            .scope
            .as_ref()
            .is_some_and(|scope| scope.test_cycle_id.as_ref() == Some(&cycle()) && scope.test_execution_id.as_ref() == Some(&execution()))
    }));
    assert_eq!(samples[0].stream.as_str(), STREAM);
    // The engine fills in the canonical column a stream's contract requires
    // when the row itself leaves it out.
    assert!(samples[0].values.contains_key("timestamp"));
    assert_eq!(samples[0].values.get("index"), Some(&Value::Integer(0)));
    assert!(outcome.passed);
    assert_eq!(status(criterion), dokime_runtime::transport::SidecarCriterionStatus::Passed);
    workflow.shutdown().await;
}

#[tokio::test]
async fn streamed_artifact_chunks() {
    let workflow = workflow();
    let request = start_request("artifact-evidence", serde_json::json!({ "samples": 1, "artifact_evidence": true }));
    workflow.start(request).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "artifact-evidence", None).await;
    assert!(outcome(&replies).0.passed);

    // Through the real `FrameTransport` the engine drives, not a stand-in: the
    // step's writes become cursor-addressed chunks on the drain itself.
    let chunks: Vec<_> = replies.iter().flat_map(|reply| reply.artifact_chunks.iter()).collect();
    let named = |name: &str| chunks.iter().filter(|chunk| chunk.name == name).collect::<Vec<_>>();

    let small = named("evidence/pulse.csv");
    assert_eq!(small.len(), 1, "a short artifact is one terminal piece");
    assert_eq!(small[0].mime, "text/csv");
    assert!(small[0].last);
    assert_eq!(small[0].bytes, ARTIFACT_EVIDENCE_BYTES);

    let streamed = named("evidence/trace.bin");
    assert!(streamed.len() > 1, "a large artifact spans several pieces");
    assert_eq!(streamed.iter().map(|chunk| chunk.sequence).collect::<Vec<_>>(), (0..streamed.len() as u64).collect::<Vec<_>>());
    assert_eq!(streamed.iter().filter(|chunk| chunk.last).count(), 1, "exactly one piece terminates the artifact");
    assert!(streamed.last().expect("pieces were drained").last);
    assert_eq!(streamed.iter().flat_map(|chunk| chunk.bytes.iter().copied()).collect::<Vec<_>>(), streamed_evidence());

    // Every artifact is fully handed over before the terminal outcome, so the
    // host never has to decide what to do with a transfer the run left open.
    let terminal = replies.iter().position(|reply| reply.outcome.is_some()).expect("the run reports its outcome");
    let last_chunk = replies.iter().rposition(|reply| !reply.artifact_chunks.is_empty()).expect("the run streams artifacts");
    assert!(last_chunk <= terminal);
    assert!(
        replies
            .iter()
            .flat_map(|reply| reply.artifact_chunks.iter())
            .all(|chunk| chunk.transfer_id.starts_with("artifact-evidence:artifact:")),
        "identity is derived from the execution token"
    );

    workflow.shutdown().await;
}

#[tokio::test]
async fn watched_directory_in() {
    let workflow = workflow();
    let request = start_request("artifact-directory", serde_json::json!({ "samples": 1, "artifact_directory": true }));
    workflow.start(request).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "artifact-directory", None).await;
    assert!(outcome(&replies).0.passed);
    let chunks: Vec<_> = replies.iter().flat_map(|reply| reply.artifact_chunks.iter()).collect();
    let named = |name: &str| chunks.iter().filter(|chunk| chunk.name == name).copied().collect::<Vec<_>>();

    // Written with `std::fs` by a step that never named an artifact, filed
    // under the directory's name and the path within it.
    let nested = named("capture/frames/first.txt");
    assert_eq!(nested.len(), 1, "a small file is one terminal piece");
    assert_eq!(nested[0].mime, "text/plain");
    assert_eq!(nested[0].bytes, b"nested\n");
    assert!(nested[0].last);

    // The sweep runs after cleanup, so teardown's own evidence is evidence.
    assert_eq!(assembled(&named("capture/released.txt")), b"released\n");

    // The rewrite in the third step replaces what the scan had already sent.
    let rewritten = named("capture/chamber.log");
    assert!(rewritten.len() > 1, "the watch streamed the file before the rewrite: {}", rewritten.len());
    assert_eq!(rewritten.iter().filter(|chunk| chunk.replace).count(), 1, "one piece starts the content over");
    assert_eq!(assembled(&rewritten), WATCHED_REWRITE, "a replaced artifact ends with the new content alone");
    assert_eq!(
        rewritten.iter().map(|chunk| chunk.sequence).collect::<Vec<_>>(),
        (0..rewritten.len() as u64).collect::<Vec<_>>(),
        "sequences carry on across the replacement rather than restarting"
    );
    assert_eq!(rewritten.iter().filter(|chunk| chunk.last).count(), 1);

    // A file whose name does not say what it is stays in the directory, named
    // once in the run's own log rather than filed as something it is not.
    assert!(named("capture/notes").is_empty());
    let notes = diagnostics(&replies).filter(|entry| entry.message.contains("capture/notes")).count();
    assert_eq!(notes, 1, "refused once, then skipped");

    // Every piece is handed over before the outcome, and the directory the run
    // owned is gone once it has been swept.
    let terminal = replies.iter().position(|reply| reply.outcome.is_some()).expect("the run reports its outcome");
    let last_chunk = replies.iter().rposition(|reply| !reply.artifact_chunks.is_empty()).expect("the run streams artifacts");
    assert!(last_chunk <= terminal);
    let directory = diagnostics(&replies)
        .find_map(|entry| entry.message.strip_prefix("directory:").map(str::to_owned))
        .expect("the step says where it wrote");
    assert!(!std::path::Path::new(&directory).exists(), "the execution's directory is removed after the sweep: {directory}");

    workflow.shutdown().await;
}

#[tokio::test]
async fn one_directory_execution() {
    let workflow = workflow();
    let request = start_request("artifact-directory-identity", serde_json::json!({ "samples": 1, "artifact_directory": true }));
    workflow.start(request).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "artifact-directory-identity", None).await;

    let paths = diagnostics(&replies).filter_map(|entry| entry.message.strip_prefix("directory:").map(str::to_owned)).collect::<Vec<_>>();
    assert_eq!(paths.len(), 2, "two steps asked for the same name");
    assert_eq!(paths[0], paths[1], "asking twice answers the same path, so a step and a later step write into one place");

    workflow.shutdown().await;
}

#[tokio::test]
async fn token_reused_refused() {
    let workflow = workflow();
    workflow.start(start_request("reused", serde_json::json!({ "samples": 1 }))).expect("the execution starts");

    let refusal = workflow
        .start(start_request("reused", serde_json::json!({ "samples": 9 })))
        .expect_err("a reused token with different inputs is refused");

    assert!(refusal.to_string().contains("reused"));
    workflow.shutdown().await;
}

#[tokio::test]
async fn unknown_workflow_starts() {
    let workflow = workflow();
    let mut request = start_request("wrong", serde_json::json!({ "samples": 1 }));
    request.workflow = "someone.elses.suite".to_owned();

    let refusal = workflow.start(request).expect_err("an unknown workflow is refused");

    assert!(refusal.to_string().contains("someone.elses.suite"));
    assert!(workflow.drain("wrong", 0).is_err());
    workflow.shutdown().await;
}

#[tokio::test]
async fn cancelled_run_produced() {
    let workflow = workflow();
    workflow.start(start_request("cancelled", serde_json::json!({ "samples": 10_000 }))).expect("the execution starts");
    drain_until_line(&workflow, "cancelled", "execution initialization completed").await;
    // Cancelling is a no-op for a token that already stopped, so repeating it is safe.
    workflow.cancel("cancelled");
    workflow.cancel("cancelled");

    let replies = drain_until_terminal(&workflow, "cancelled", None).await;
    let (outcome, criterion) = outcome(&replies);

    assert!(!outcome.passed);
    assert!(outcome.message.as_deref().is_some_and(|message| message.contains("cancelled")));
    // Measured, not blanked: the run produced fewer rows than it needed, and
    // that is a verdict rather than an absence of one.
    assert_eq!(status(criterion), dokime_runtime::transport::SidecarCriterionStatus::Failed);
    workflow.shutdown().await;
}

#[tokio::test]
async fn criterion_could_failed() {
    let workflow = workflow();
    workflow.start(start_request("blind", serde_json::json!({ "samples": 1, "unmeasurable": true }))).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "blind", None).await;
    let (outcome, criterion) = outcome(&replies);

    assert!(!outcome.passed);
    assert_eq!(status(criterion), dokime_runtime::transport::SidecarCriterionStatus::Indeterminate);
    assert!(criterion.message.as_deref().is_some_and(|message| message.contains("could not measure")));
    workflow.shutdown().await;
}

#[tokio::test]
async fn unanswered_question_id() {
    let workflow = workflow();
    workflow.start(start_request("asked", serde_json::json!({ "samples": 1, "ask": true }))).expect("the execution starts");

    let (cursor, raised) = drain_until_prompt(&workflow, "asked").await;
    // The same cursor must re-deliver the unanswered question: that replay is
    // what keeps a timed-out drain from stranding the run.
    let replayed = drain(&workflow, "asked", cursor);
    assert_eq!(raised.prompt_requests, replayed.prompt_requests);

    let asked = decode(&raised.prompt_requests[0]);
    // Derived from the execution, not minted: a re-delivered request is the
    // same request.
    assert_eq!(asked.prompt_id, PromptId::from_seed("asked:prompt:0"));
    // An answer for a question this execution never asked changes nothing.
    respond(&workflow, "asked", &dokime_runtime::model::identity::new_id::<PromptId>(), "yes", true);
    assert!(drain(&workflow, "asked", raised.cursor).outcome.is_none());

    respond(&workflow, "asked", &asked.prompt_id, "yes", true);
    // A redelivery the host was not sure landed is a no-op, not a refusal.
    respond(&workflow, "asked", &asked.prompt_id, "cancel", false);

    let replies = drain_from(&workflow, "asked", None, raised.cursor).await;
    let (outcome, _) = outcome(&replies);

    assert!(outcome.passed);
    workflow.shutdown().await;
}

#[tokio::test]
async fn declined_answer_asked() {
    let workflow = workflow();
    workflow.start(start_request("declined", serde_json::json!({ "samples": 1, "ask": true }))).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "declined", Some(("no", true))).await;
    let (outcome, _) = outcome(&replies);

    assert!(!outcome.passed);
    assert!(outcome.message.as_deref().is_some_and(|message| message.contains("start")));
    workflow.shutdown().await;
}

#[tokio::test]
async fn cancelling_waiting_execution() {
    let workflow = workflow();
    workflow.start(start_request("abandoned", serde_json::json!({ "samples": 1, "ask": true }))).expect("the execution starts");
    let (_, raised) = drain_until_prompt(&workflow, "abandoned").await;
    let asked = decode(&raised.prompt_requests[0]);

    workflow.cancel("abandoned");
    // An answer the host had already dispatched cannot revive a cancelled wait,
    // and is still accepted rather than refused.
    respond(&workflow, "abandoned", &asked.prompt_id, "yes", true);

    let replies = drain_from(&workflow, "abandoned", None, raised.cursor).await;
    let (outcome, _) = outcome(&replies);

    assert!(!outcome.passed);
    // Teardown does not block on a cancelled question.
    workflow.shutdown().await;
}

#[tokio::test]
async fn phase_drain_reached() {
    let workflow = workflow();
    workflow.start(start_request("phased", serde_json::json!({ "samples": 2 }))).expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "phased", None).await;
    let phases = replies.iter().map(|reply| reply.phase.clone()).filter(|phase| !phase.is_empty()).collect::<Vec<_>>();

    assert_eq!(phases.last().map(String::as_str), Some("complete"));
    assert!(phases.iter().any(|phase| phase == "stream"));
    workflow.shutdown().await;
}

#[tokio::test]
async fn cleanup_runs_pass() {
    for fail in [false, true] {
        let workflow = workflow();
        workflow.start(start_request("cleanup", serde_json::json!({"samples": 100000, "cleanup_error": fail}))).unwrap();
        drain_until_line(&workflow, "cleanup", "execution initialization completed").await;
        workflow.cancel("cleanup");
        let replies = drain_until_terminal(&workflow, "cleanup", None).await;
        assert_eq!(
            replies.iter().flat_map(|reply| &reply.diagnostics).filter(|line| line.message.as_str() == "cleanup-finished").count(),
            1
        );
        let terminal = outcome(&replies).0;
        assert!(!terminal.passed);
        if fail {
            assert!(terminal.message.as_ref().unwrap().contains("cleanup"));
        }
        workflow.shutdown().await;
    }
}

#[tokio::test]
async fn initialize_failure_execution() {
    let workflow = workflow();
    workflow
        .start(start_request(
            "init-error",
            serde_json::json!({
                "initialize_error": true, "samples": 0,
            }),
        ))
        .unwrap();
    let replies = drain_until_terminal(&workflow, "init-error", None).await;
    let terminal = outcome(&replies).0;
    assert!(!terminal.passed);
    assert!(terminal.message.as_ref().unwrap().contains("initialization"));
    assert!(terminal.criteria.iter().all(|criterion| criterion.status == SidecarCriterionStatus::Indeterminate as i32));
    assert!(replies.iter().all(|reply| reply.phase.is_empty()));
    assert!(!replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message.starts_with("cleanup-")));
    workflow.shutdown().await;
}

#[tokio::test]
async fn initialize_cancellation_cleanup() {
    INITIALIZATION_RELEASED.store(false, Ordering::Release);
    let workflow = workflow();
    workflow
        .start(start_request(
            "init-cancel",
            serde_json::json!({
                "initialize_wait": true,
            }),
        ))
        .unwrap();
    let mut cursor = 0;
    loop {
        let reply = next(&workflow, "init-cancel", cursor).await;
        cursor = reply.cursor;
        assert!(reply.phase.is_empty());
        if reply.diagnostics.iter().any(|line| line.message == "probe-initializing") {
            break;
        }
    }
    workflow.cancel("init-cancel");
    let replies = drain_until_terminal(&workflow, "init-cancel", None).await;
    let terminal = outcome(&replies).0;
    assert!(INITIALIZATION_RELEASED.load(Ordering::Acquire));
    assert!(!terminal.passed);
    assert!(terminal.message.as_ref().unwrap().contains("cancelled during phase `initialization`"));
    assert!(!replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message.starts_with("cleanup-")));
    workflow.shutdown().await;
}

#[tokio::test]
async fn cleanup_procedure_failure() {
    let workflow = workflow();
    workflow
        .start(start_request(
            "failed-cleanup",
            serde_json::json!({
                "ask": true, "cleanup_error": true,
            }),
        ))
        .unwrap();
    let replies = drain_until_terminal(&workflow, "failed-cleanup", Some(("no", true))).await;
    let terminal = outcome(&replies).0;
    let message = terminal.message.as_ref().unwrap();
    assert!(!terminal.passed);
    assert!(message.contains("the operator answered"));
    assert!(message.contains("cleanup failed"));
    assert!(replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message == "cleanup-reason:Failed"));
    workflow.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn cleanup_joins_release() {
    let workflow = workflow();
    workflow
        .start(start_request(
            "cleanup-timeout",
            serde_json::json!({
                "samples": 1, "cleanup_wait": true,
            }),
        ))
        .unwrap();
    drain_until_line(&workflow, "cleanup-timeout", "cleanup-waiting").await;
    CLEANUP_RELEASE.notify_one();
    let replies = drain_until_terminal(&workflow, "cleanup-timeout", None).await;
    let terminal = outcome(&replies).0;
    assert!(terminal.passed, "{terminal:?}");
    assert!(replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message == "cleanup-reason:Completed"));
    assert!(terminal.criteria.iter().all(|criterion| criterion.status == SidecarCriterionStatus::Passed as i32));
    workflow.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn pending_sampler_cleanup() {
    let workflow = workflow();
    workflow
        .start(start_request(
            "pending-sample",
            serde_json::json!({
                "samples": 1, "observer_wait": true,
            }),
        ))
        .unwrap();

    let replies = drain_until_terminal(&workflow, "pending-sample", None).await;
    let (terminal, criterion) = outcome(&replies);
    assert!(!terminal.passed, "an interrupted measurement supplies no sample");
    assert_eq!(status(criterion), SidecarCriterionStatus::Failed);
    assert!(replies.iter().flat_map(|reply| &reply.diagnostics).any(|line| line.message == "cleanup-reason:Completed"));
    assert!(terminal.detail_json.is_none(), "cleanup must not observe a live observer");
    workflow.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn stalled_criterion_indeterminate() {
    let workflow = workflow();
    workflow
        .start(start_request(
            "pending-criterion",
            serde_json::json!({
                "samples": 1, "criterion_wait": true,
            }),
        ))
        .unwrap();

    let replies = drain_until_terminal(&workflow, "pending-criterion", None).await;
    let (terminal, criterion) = outcome(&replies);
    assert!(!terminal.passed);
    assert_eq!(status(criterion), SidecarCriterionStatus::Indeterminate);
    assert_eq!(criterion.message.as_deref(), Some("criterion evaluation exceeded one second"));
    workflow.shutdown().await;
}

#[tokio::test]
async fn resources_release_verdict() {
    for cleanup_error in [false, true] {
        EXECUTION_RELEASED.store(false, Ordering::Release);
        let workflow = workflow();
        workflow
            .start(start_request(
                "release",
                serde_json::json!({
                    "samples": 1, "observe_release": true, "cleanup_error": cleanup_error,
                }),
            ))
            .unwrap();
        let replies = drain_until_terminal(&workflow, "release", None).await;
        let (terminal, criterion) = outcome(&replies);
        assert_eq!(terminal.passed, !cleanup_error);
        assert_eq!(status(criterion), SidecarCriterionStatus::Passed);
        assert!(EXECUTION_RELEASED.load(Ordering::Acquire));
        workflow.shutdown().await;
    }
}

#[tokio::test]
async fn diagnostics_preserve_success() {
    use dokime_runtime::execution::log::Level;
    let workflow = workflow();
    workflow.start(start_request("levels", serde_json::json!({"samples": 1, "diagnostics": true}))).unwrap();
    let replies = drain_until_terminal(&workflow, "levels", None).await;
    assert!(outcome(&replies).0.passed);
    // A run that passed has no failure to detail.
    assert!(outcome(&replies).0.detail_json.is_none());
    let entries = replies.iter().flat_map(|reply| &reply.diagnostics).collect::<Vec<_>>();
    for phase in ["initializing", "step", "cleanup"] {
        for (level, suffix) in
            [(Level::Trace, "trace"), (Level::Debug, "debug"), (Level::Info, "info"), (Level::Warn, "warn"), (Level::Error, "error")]
        {
            let message = format!("{phase} {suffix}");
            let matching = entries.iter().filter(|entry| entry.message == message).collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{message}");
            assert_eq!(matching[0].level, level as i32, "{message}");
        }
    }
    workflow.shutdown().await;
}

fn captured(execution: dokime_runtime::TestExecutionId) -> dokime_runtime::RecordScope {
    dokime_runtime::RecordScope::default().with_cycle(cycle()).with_test_execution(execution)
}

fn admitted(record_type: &str, source: &str, serial: &str) -> model::records::evidence::Record {
    let mut record = model::records::evidence::Record::new(
        dokime_runtime::model::identity::new_id(),
        dokime_runtime::model::time::now_utc(),
        dokime_runtime::Identifier::new(record_type).expect("a fixture identifier is valid"),
        1,
        source,
        dokime_runtime::records::RecordSeverity::Info,
    );
    record.details.insert("serial".to_owned(), Value::from(serial));
    record.scope = Some(dokime_runtime::RecordScope::default().with_cycle(cycle()).with_test_execution(execution()));

    record
}

#[tokio::test]
async fn record_wait_publication() {
    let workflow = workflow();
    workflow
        .start(start_request("awaited", serde_json::json!({ "await_record": true, "await_record_ms": 5_000, "samples": 1 })))
        .expect("the execution starts");

    // The host admits a record from somewhere else and pushes it down. Nothing
    // about the delivery names this execution: it carries the scopes the host
    // captured it for, and the engine is what decides that this run is in one.
    assert_eq!(
        workflow.deliver_captured(&admitted("probe.unit.connected", "gateway", "SN-7"), SOURCE, &[captured(execution())],),
        1,
        "admission retains the record before the step begins listening"
    );
    let replies = drain_until_terminal(&workflow, "awaited", None).await;

    let outcome = replies.iter().find_map(|reply| reply.outcome.clone()).expect("the run terminates");
    assert!(outcome.passed, "{:?}", outcome.message);
    assert!(
        replies.iter().flat_map(|reply| &reply.diagnostics).any(|entry| entry.message == "connected:SN-7"),
        "the step read the record it was handed"
    );
    workflow.shutdown().await;
}

// The listen deadline is time semantics: it elapses on the paused clock.
#[tokio::test(start_paused = true)]
async fn record_nobody_error() {
    let workflow = workflow();
    workflow
        .start(start_request("unawaited", serde_json::json!({ "await_record": true, "await_record_ms": 25, "samples": 1 })))
        .expect("the execution starts");

    let replies = drain_until_terminal(&workflow, "unawaited", None).await;

    let outcome = replies.iter().find_map(|reply| reply.outcome.clone()).expect("the run terminates");
    assert!(!outcome.passed);
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("deadline elapsed"), "the deadline reaches the plugin's own error: {message}");
    workflow.shutdown().await;
}

#[tokio::test]
async fn capture_declared_scope() {
    let workflow = workflow();
    workflow
        .start(start_request("filtered", serde_json::json!({ "await_record": true, "await_record_ms": 5_000, "samples": 1 })))
        .expect("the execution starts");
    assert!(workflow.owns("filtered"));

    // This sidecar's own record comes back through the host like any other. It
    // is kept for the run's criteria, but handing a step its own words back
    // would answer the question it asked somebody else.
    let mine = admitted("probe.unit.connected", SOURCE, "SN-mine");
    assert_eq!(workflow.deliver_captured(&mine, SOURCE, &[captured(execution())]), 1);
    // A record the host captured for a different run is not this run's.
    let mut elsewhere = admitted("probe.unit.connected", "gateway", "SN-elsewhere");
    assert_eq!(
        workflow.deliver_captured(&elsewhere, SOURCE, &[captured(dokime_runtime::TestExecutionId::from_seed("another-execution"))]),
        0
    );
    // The capture overrides the scope the record was filed under.
    elsewhere.scope = Some(captured(dokime_runtime::TestExecutionId::from_seed("another-execution")));
    assert_eq!(workflow.deliver_captured(&elsewhere, SOURCE, &[captured(execution())]), 1);
    assert_eq!(workflow.deliver_captured(&elsewhere, SOURCE, &[captured(execution())]), 0, "redelivery is deduplicated");
    // A record the host filed against nothing in particular is still captured
    // for the run the host names.
    let mut everyones = admitted("probe.unit.connected", "gateway", "SN-global");
    everyones.scope = None;
    assert_eq!(workflow.deliver_captured(&everyones, SOURCE, &[captured(execution())]), 1);

    let replies = drain_until_terminal(&workflow, "filtered", None).await;

    assert!(
        replies.iter().flat_map(|reply| &reply.diagnostics).any(|entry| entry.message == "connected:SN-elsewhere"),
        "the only record the step could take is the one it did"
    );
    workflow.shutdown().await;
}

#[tokio::test]
async fn settlement_record_progress() {
    let workflow = workflow();
    let token = "record-progress";
    let rows = Arc::new(PublishedExecutionRows(
        std::sync::Mutex::new(Vec::new()),
        workflow.clone(),
        std::sync::Mutex::new(captured(execution())),
    ));
    workflow
        .start_on(
            start_request(token, serde_json::json!({"samples": 1})),
            Some(SampleOutput::Embedded(rows)),
            Caller::default(),
        )
        .unwrap();
    let mut cursor = 0;
    loop {
        let reply = next(&workflow, token, cursor).await;
        cursor = reply.cursor;
        assert!(reply.outcome.is_none());
        if reply.record_subscriptions_settling {
            break;
        }
    }

    // Observers Have Joined Before This Clock Is Paused. An accepted record
    // extends settlement beyond the original idle window.
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(20)).await;
    let record = admitted("probe.unit.connected", "gateway", "SN-progress");
    assert_eq!(workflow.deliver_captured(&record, SOURCE, &[captured(execution())]), 1);
    tokio::time::advance(Duration::from_millis(10)).await;
    tokio::time::advance(Duration::from_secs(20)).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    workflow.settle_records(token);
    let replies = drain_until_terminal(&workflow, token, None).await;
    tokio::time::resume();
    assert!(outcome(&replies).0.passed, "accepted records keep settlement alive: {:?}", outcome(&replies).0);
    workflow.shutdown().await;
}

#[tokio::test]
async fn settlement_duplicate_idle() {
    let workflow = workflow();
    let token = "duplicate-idle";
    let rows = Arc::new(PublishedExecutionRows(
        std::sync::Mutex::new(Vec::new()),
        workflow.clone(),
        std::sync::Mutex::new(captured(execution())),
    ));
    workflow
        .start_on(
            start_request(token, serde_json::json!({"samples": 1})),
            Some(SampleOutput::Embedded(rows)),
            Caller::default(),
        )
        .unwrap();
    let mut cursor = 0;
    loop {
        let reply = next(&workflow, token, cursor).await;
        cursor = reply.cursor;
        assert!(reply.outcome.is_none());
        if reply.record_subscriptions_settling {
            break;
        }
    }

    tokio::time::pause();
    let record = admitted("probe.unit.connected", "gateway", "SN-duplicate");
    assert_eq!(workflow.deliver_captured(&record, SOURCE, &[captured(execution())]), 1);
    tokio::time::sleep(Duration::from_millis(10)).await;
    tokio::time::advance(Duration::from_secs(20)).await;
    assert_eq!(workflow.deliver_captured(&record, SOURCE, &[captured(execution())]), 0);
    tokio::time::advance(Duration::from_secs(20)).await;
    let replies = drain_until_terminal(&workflow, token, None).await;
    tokio::time::resume();
    let terminal = outcome(&replies).0;
    assert!(!terminal.passed);
    assert!(terminal.message.as_deref().unwrap_or_default().contains("host did not settle execution inputs before the deadline"));
    workflow.shutdown().await;
}

#[test]
fn signal_refuses_deliveries() {
    let workflow = workflow();
    let record = workflow.signal(ExecutionSignal {
        execution_token: "unknown".into(),
        record: Some(DeliverRecordRequest { record_json: "{".into(), captured_scopes: Vec::new() }),
        ..ExecutionSignal::default()
    });
    assert!(record.is_err_and(|refusal| refusal.to_string().contains("delivered record JSON")));
    let samples = workflow.signal(ExecutionSignal {
        execution_token: "unknown".into(),
        samples: Some(DeliverSamplesRequest { samples: Vec::new(), gaps: Vec::new(), scope_json: "not a scope".into() }),
        ..ExecutionSignal::default()
    });
    assert!(samples.is_err_and(|refusal| refusal.to_string().contains("sample scope JSON")));
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifact_backpressure() {
    ARTIFACT_GATE_FILLED.store(false, Ordering::Release);
    ARTIFACT_PUSH_FINISHED.store(false, Ordering::Release);
    let workflow = workflow();
    let token = "artifact-backpressure";
    workflow
        .start(start_request(
            token,
            serde_json::json!({
                "samples": 1, "artifact_backpressure": true
            }),
        ))
        .expect("the execution starts");
    let mut cursor = 0;
    loop {
        let reply = next(&workflow, token, cursor).await;
        cursor = reply.cursor;
        if !reply.step_begins.is_empty() {
            break;
        }
    }
    ARTIFACT_GATE_REACHED.notified().await;
    assert!(ARTIFACT_GATE_FILLED.load(Ordering::Acquire));
    let blocked = !ARTIFACT_PUSH_FINISHED.load(Ordering::Acquire);
    let first = drain(&workflow, token, cursor);
    let carried_chunks = !first.artifact_chunks.is_empty();
    let replies = drain_from(&workflow, token, None, first.cursor).await;
    let finished = ARTIFACT_PUSH_FINISHED.load(Ordering::Acquire);
    let passed = outcome(&replies).0.passed;
    workflow.shutdown().await;
    assert!(blocked, "the sink stays parked while the backlog is at the gate");
    assert!(carried_chunks, "one partial drain acknowledges at least one chunk");
    assert!(finished, "draining the backlog resumes the parked push");
    assert!(passed, "the resumed execution terminates successfully");
}
