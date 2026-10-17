/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "protocol/publications.rs"]
mod publications;
#[path = "protocol/telemetry.rs"]
mod telemetry;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};
// Third Party Imports
use dokime_runtime::harmos::operations::{
    Host, OperationDeclaration, OperationPolicy, Response, RestartPolicy, RetryPolicy,
    ServingError, Visibility, WireOperation, WireTable,
};
use dokime_runtime::harmos::plugin::Contribution;
use dokime_runtime::harmos::plugin::{Registrar, RegistrationError, inventory::Section};
// Local Imports
use dokime_runtime::core::{
    Identifier, Metadata, StepAttemptId, StepId, TemplateId, Timestamp, Value, ValueType,
};
use dokime_runtime::definition::{
    ExecutionPhaseDefinition, Subscription, SuiteSet, TestCaseTemplate, TestSuite,
};
use dokime_runtime::domains::TestSuiteDefinition;
use dokime_runtime::execution::{StepAttemptStart, TerminationReason, TestStepAttempt};
use dokime_runtime::plugin::registration::{SUITES_NAMESPACE, domain_schema};
use dokime_runtime::records::{RecordContract, RecordField, RecordPublishRequest, RecordSeverity};
use dokime_runtime::transport::{
    ArtifactChunk, EvidenceBatch, ExecutionSignal, PushEvidence, RUN_BOUND, RunExecutionRequest,
    SIGNAL_BOUND, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome,
    SignalReply,
};
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */
const POLL: Duration = Duration::from_millis(10);

const SETTLE_DELAY: Duration = Duration::from_millis(1_600);

/* === Macros === */
macro_rules! operation {
    ($name:ident, $request:ty, $response:ty, $id:literal, $seconds:literal, $visibility:ident, $method:ident) => {
        struct $name {
            resources: Arc<Resources>,
        }

        impl OperationDeclaration for $name {
            type Request = $request;
            type Response = $response;
            const ID: &'static str = $id;
            const VISIBILITY: Visibility = Visibility::$visibility;
            const POLICY: OperationPolicy = OperationPolicy {
                timeout: Duration::from_secs($seconds),
                attempt_timeout: Duration::from_secs($seconds),
                retries: RetryPolicy::Never,
                restart: RestartPolicy::Never,
            };
        }

        impl WireOperation for $name {
            const VERSION: u32 = 1;
        }

        impl dokime_runtime::harmos::operations::Operation for $name {
            type Declaration = Self;
            type Resources = Resources;
            type Endpoints = ();
            type Error = Refusal;
            type Context<'a> = &'a Supervisor;

            async fn initialize<'ctx>(
                resources: Arc<Resources>,
                _: Self::Context<'ctx>,
            ) -> Result<Self, Refusal> {
                Ok(Self { resources })
            }

            async fn run<'ctx>(
                &mut self,
                context: Self::Context<'ctx>,
                request: &mut $request,
                _: &mut (),
            ) -> Result<Response<$response>, Refusal> {
                let request = std::mem::take(request);
                self.resources
                    .$method(context, request)
                    .await
                    .map(Response::new)
            }
        }
    };
}

macro_rules! modes {
    ($($name:ident, $run:ident, $signal:ident => $mode:literal),* $(,)?) => {
        $(
            struct $name;

            impl Mode for $name {
                const MODE: &'static str = $mode;
                const RUN: &'static str = concat!("fixture.sidecar.", $mode, ".run");
                const SIGNAL: &'static str = concat!("fixture.sidecar.", $mode, ".signal");
                const SIGNAL_TIMEOUT: Duration = signal_timeout($mode);
            }
        )*

                #[dokime_runtime::catalog]
        enum Templates {
            $(
                $run(Running<$name>),
                $signal(Signalling<$name>),
            )*
        }
    };
}

/* === Type Aliases === */

/* === Structs === */
struct Protocol {
    state: Arc<Resources>,
}

#[derive(Default)]
struct Resources {
    state: Mutex<Active>,
}

#[derive(Default)]
struct Active {
    mode: String,
    token: String,
    starts: u64,
    cancelled: bool,
    cancelled_at: Option<std::time::Instant>,
    released: bool,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct Signals {
    starts: u64,
    cancelled: bool,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Empty {}

struct Run<M>(PhantomData<fn() -> M>);

struct Signal<M>(PhantomData<fn() -> M>);

struct Running<M> {
    resources: Arc<Resources>,
    mode: PhantomData<fn() -> M>,
}

struct Signalling<M> {
    resources: Arc<Resources>,
    mode: PhantomData<fn() -> M>,
}

struct Domains;

/* === Enums === */
#[dokime_runtime::catalog]
enum Streams {
    Readings(telemetry::Readings),
}

#[dokime_runtime::catalog]
enum Operations {
    #[nested]
    Templates(Templates),
    Status(Status),
    Release(Release),
    PublishTelemetry(PublishTelemetry),
    PublishRecord(PublishRecord),
}

/* === Traits === */
trait Mode: Send + Sync + 'static {
    const MODE: &'static str;
    const RUN: &'static str;
    const SIGNAL: &'static str;
    const SIGNAL_TIMEOUT: Duration;
}

/* === Implementations === */
modes!(
    Pass, PassRun, PassSignal => "pass",
    Fail, FailRun, FailSignal => "fail",
    Fail2, Fail2Run, Fail2Signal => "fail2",
    Fail3, Fail3Run, Fail3Signal => "fail3",
    Abandon, AbandonRun, AbandonSignal => "abandon",
    CancelComplete, CancelCompleteRun, CancelCompleteSignal => "cancel_complete",
    CancelDelayed, CancelDelayedRun, CancelDelayedSignal => "cancel_delayed",
    CancelRefused, CancelRefusedRun, CancelRefusedSignal => "cancel_refused",
    CancelIgnored, CancelIgnoredRun, CancelIgnoredSignal => "cancel_ignored",
    CancelSignalIgnored, CancelSignalIgnoredRun, CancelSignalIgnoredSignal => "cancel_signal_ignored",
    ScopeError, ScopeErrorRun, ScopeErrorSignal => "scope_error",
    Capture, CaptureRun, CaptureSignal => "capture",
    StreamLog, StreamLogRun, StreamLogSignal => "stream_log",
);

operation!(
    Status,
    Empty,
    Signals,
    "fixture-sidecar.status",
    30,
    Shared,
    status
);

operation!(
    Release,
    Empty,
    Empty,
    "fixture-sidecar.release",
    30,
    Shared,
    release
);

operation!(
    PublishTelemetry,
    telemetry::Publish,
    Empty,
    "fixture-sidecar.publish-telemetry",
    30,
    Shared,
    publish_telemetry
);

operation!(
    PublishRecord,
    dokime_runtime::records::RecordPublication,
    publications::Published,
    "fixture-sidecar.publish-record",
    30,
    Shared,
    publish_record
);

impl<M: Mode> OperationDeclaration for Run<M> {
    type Request = RunExecutionRequest;
    type Response = SidecarExecutionOutcome;
    const ID: &'static str = M::RUN;
    const POLICY: OperationPolicy = OperationPolicy {
        timeout: RUN_BOUND,
        attempt_timeout: RUN_BOUND,
        retries: RetryPolicy::Never,
        restart: RestartPolicy::Never,
    };
    const VISIBILITY: Visibility = Visibility::Host;
}

impl<M: Mode> WireOperation for Run<M> {
    const VERSION: u32 = 1;
}

impl<M: Mode> OperationDeclaration for Signal<M> {
    type Request = ExecutionSignal;
    type Response = SignalReply;
    const ID: &'static str = M::SIGNAL;
    const POLICY: OperationPolicy = OperationPolicy {
        timeout: M::SIGNAL_TIMEOUT,
        attempt_timeout: M::SIGNAL_TIMEOUT,
        retries: RetryPolicy::Never,
        restart: RestartPolicy::Never,
    };
    const VISIBILITY: Visibility = Visibility::Host;
}

impl<M: Mode> WireOperation for Signal<M> {
    const VERSION: u32 = 1;
}

impl<M: Mode> dokime_runtime::harmos::operations::Declares for Running<M> {
    type Declaration = Run<M>;
}

impl<M: Mode> dokime_runtime::harmos::operations::Operation for Running<M> {
    type Declaration = Run<M>;
    type Resources = Resources;
    type Endpoints = ();
    type Error = Refusal;
    type Context<'a> = &'a Supervisor;

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Refusal> {
        Ok(Self {
            resources,
            mode: PhantomData,
        })
    }

    async fn run<'ctx>(
        &mut self,
        host: Self::Context<'ctx>,
        request: &mut RunExecutionRequest,
        _: &mut (),
    ) -> Result<Response<SidecarExecutionOutcome>, Refusal> {
        self.resources
            .execute(host, M::MODE, std::mem::take(request))
            .await
            .map(Response::new)
    }
}

impl<M: Mode> dokime_runtime::harmos::operations::Declares for Signalling<M> {
    type Declaration = Signal<M>;
}

impl<M: Mode> dokime_runtime::harmos::operations::Operation for Signalling<M> {
    type Declaration = Signal<M>;
    type Resources = Resources;
    type Endpoints = ();
    type Error = Refusal;
    type Context<'a> = &'a Supervisor;

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Refusal> {
        Ok(Self {
            resources,
            mode: PhantomData,
        })
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        signal: &mut ExecutionSignal,
        _: &mut (),
    ) -> Result<Response<SignalReply>, Refusal> {
        let cancel = signal.cancel;
        let reply = self.resources.signal(std::mem::take(signal))?;
        if cancel && M::MODE == "cancel_signal_ignored" {
            while !self.resources.lock().released {
                tokio::time::sleep(POLL).await;
            }
        }
        Ok(Response::new(reply))
    }
}

impl Contribution for Domains {
    const NAMESPACE: &'static str = SUITES_NAMESPACE;
    const SECTION: Section = Section::new(SUITES_NAMESPACE, r#"{"schema":2}"#);

    fn register(registrar: &mut Registrar) -> Result<(), RegistrationError> {
        let mut suite = TestSuiteDefinition::new("fixture.sidecar", suite()).unwrap();
        suite.workflow = Some("fixture.sidecar".into());
        suite.record_contracts = vec![record_contract()];
        let capture = suite
            .suite
            .test_case_templates
            .iter()
            .find(|template| template.metadata.identifier.as_str() == "fixture.sidecar.capture")
            .unwrap();
        suite.telemetry_contracts =
            vec![telemetry::contract(capture.test_case_template_id.clone())];
        registrar.register(
            SUITES_NAMESPACE,
            domain_schema(SUITES_NAMESPACE),
            &SuiteSet::new([suite]),
        )
    }
}

impl dokime_runtime::harmos::operations::lifecycle::Policy for Protocol {
    const INITIALIZATION_POLICY: OperationPolicy = <Status as OperationDeclaration>::POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime_runtime::plugin(
    id = "fixture-sidecar",
    provides = Operations,
    requires(
        PushEvidence,
        dokime_runtime::records::PublishRecord,
        dokime_runtime::harmos::operations::host::Publish,
    ),
    contributions(Domains, dokime_runtime::Evidence<Streams>),
    restart = "never", description = "Serves Controlled Execution Replies Through The Original Physical Owner.")]
impl dokime_runtime::Plugin for Protocol {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    fn prepare(_: dokime_runtime::plugin::Configuration) -> Result<Self, Refusal> {
        Ok(Self {
            state: Arc::new(Resources::default()),
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.state)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Operations, _>(self.resources())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

impl Resources {
    fn lock(&self) -> std::sync::MutexGuard<'_, Active> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    async fn execute(
        &self,
        host: &Supervisor,
        mode: &str,
        request: RunExecutionRequest,
    ) -> Result<SidecarExecutionOutcome, Refusal> {
        {
            let mut state = self.lock();
            state.mode = mode.into();
            state.token = request.execution_token;
            state.cancelled = false;
            state.cancelled_at = None;
            state.released = false;
        }
        if mode.starts_with("cancel_") || mode == "capture" {
            return self.settle(host, mode).await;
        }
        self.lock().starts += 1;
        if mode == "scope_error" {
            while !self.lock().released {
                tokio::time::sleep(POLL).await;
            }
            return Err(Refusal::of("fixture run failure"));
        }
        let mut pushed = 0;
        let mut push = async |mut batch: EvidenceBatch| {
            pushed += 1;
            batch.cursor = pushed;
            host.call_as::<PushEvidence>(batch)
                .await
                .map(|response| response.into_owned())
                .map_err(|error| Refusal::of(error.to_string()))
        };
        if mode == "stream_log" {
            // A log line the host persists while the run is still producing,
            // so a viewer can follow it before the step settles.
            use dokime_runtime::execution::log::{Entry, Level};
            push(EvidenceBatch {
                phase: "stream".into(),
                diagnostics: vec![Entry::new(Level::Info, "streamed before release")],
                ..Default::default()
            })
            .await?;
            while !self.lock().released {
                tokio::time::sleep(POLL).await;
            }
            push(EvidenceBatch {
                diagnostics: vec![Entry::new(Level::Info, "streamed after release")],
                ..Default::default()
            })
            .await?;
            return Ok(SidecarExecutionOutcome {
                passed: true,
                message: Some("stream log released".into()),
                criteria: Vec::new(),
                detail_json: None,
                failed_in: None,
            });
        }
        if mode == "abandon" {
            let opened = ArtifactChunk {
                transfer_id: "given-up".into(),
                name: "partial.bin".into(),
                mime: "application/octet-stream".into(),
                sequence: 0,
                bytes: b"a prefix".to_vec(),
                last: false,
                aborted: false,
                replace: false,
            };
            push(EvidenceBatch {
                phase: "stream".into(),
                artifact_chunks: vec![
                    opened.clone(),
                    ArtifactChunk {
                        sequence: 1,
                        bytes: Vec::new(),
                        last: true,
                        aborted: true,
                        ..opened
                    },
                ],
                ..Default::default()
            })
            .await?;
        } else {
            let mut record = RecordPublishRequest::new(
                Identifier::new("fixture.sidecar.observed").unwrap(),
                RecordSeverity::Info,
            );
            record.details.insert("value".into(), Value::Number(42.0));
            push(EvidenceBatch {
                phase: "stream".into(),
                records: vec![serde_json::to_string(&record).unwrap()],
                diagnostics: vec![dokime_runtime::execution::log::Entry::new(
                    dokime_runtime::execution::log::Level::Info,
                    "fixture complete",
                )],
                ..Default::default()
            })
            .await?;
            // One piece per page, the first one twice: a repeated piece is the
            // host's to recognize, not the producer's to avoid.
            let chunks = ArtifactChunk::split(
                "capture",
                "evidence.bin",
                "application/octet-stream",
                vec![23; dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES + 7],
            );
            for chunk in std::iter::once(chunks[0].clone()).chain(chunks) {
                push(EvidenceBatch {
                    artifact_chunks: vec![chunk],
                    ..Default::default()
                })
                .await?;
            }
        }
        Ok(SidecarExecutionOutcome {
            passed: !mode.starts_with("fail"),
            message: Some("fixture complete".into()),
            criteria: Vec::new(),
            detail_json: None,
            failed_in: None,
        })
    }

    async fn settle(
        &self,
        host: &Supervisor,
        mode: &str,
    ) -> Result<SidecarExecutionOutcome, Refusal> {
        // The one step this template declares, begun the way a run begins it:
        // the host admits the attempt before the step's body runs.
        let start = StepAttemptStart {
            attempt_id: dokime_runtime::model::identity::new_id::<StepAttemptId>(),
            step_id: StepId::from_seed(format!("fixture.sidecar.{mode}.stream")),
            started_at: dokime_runtime::model::time::now_utc(),
        };
        host.call_as::<PushEvidence>(EvidenceBatch {
            cursor: 1,
            phase: "stream".into(),
            step_begins: vec![serde_json::to_string(&start).unwrap()],
            ..Default::default()
        })
        .await
        .map(|response| response.into_owned())
        .map_err(|error| Refusal::of(error.to_string()))?;
        // Started once the stream phase is admitted, so a test that stops the
        // cycle as soon as it sees the start stops a run that is streaming.
        self.lock().starts += 1;
        let cancelled_at = loop {
            if let Some(at) = self.lock().cancelled_at {
                break at;
            }
            tokio::time::sleep(POLL).await;
        };
        if mode == "cancel_delayed" {
            tokio::time::sleep(SETTLE_DELAY.saturating_sub(cancelled_at.elapsed())).await;
        }
        // A run that never answers its stop, the way a step blocked on an
        // operator question does: only the host's settlement budget ends it.
        // The test releases it afterwards so shutdown has nothing left to wait on.
        if matches!(mode, "cancel_ignored" | "cancel_signal_ignored") {
            while !self.lock().released {
                tokio::time::sleep(POLL).await;
            }
            return Err(Refusal::of("run released after the host settled it"));
        }
        let ended = serde_json::to_string(&TestStepAttempt {
            attempt_id: start.attempt_id,
            step_id: start.step_id,
            started_at: start.started_at,
            finished_at: Some(dokime_runtime::model::time::now_utc()),
            termination: TerminationReason::Cancelled,
            detail: None,
        })
        .unwrap();
        let refused = mode == "cancel_refused";
        host.call_as::<PushEvidence>(EvidenceBatch {
            cursor: 2,
            step_attempts: vec![ended],
            // A refusing run reports its step's end and then no outcome at all.
            diagnostics: if refused {
                Vec::new()
            } else {
                cancellation_log()
            },
            cleanup_settled: !refused,
            ..Default::default()
        })
        .await
        .map(|response| response.into_owned())
        .map_err(|error| Refusal::of(error.to_string()))?;
        if refused {
            return Err(Refusal::of("run refused during cancellation"));
        }
        Ok(SidecarExecutionOutcome {
            passed: false,
            message: Some("cancelled".into()),
            criteria: vec![SidecarCriterionOutcome {
                identifier: "fixture.acceptance".into(),
                label: "Acceptance".into(),
                status: SidecarCriterionStatus::Indeterminate as i32,
                message: Some("cancelled before deciding".into()),
            }],
            detail_json: None,
            failed_in: None,
        })
    }

    fn signal(&self, signal: ExecutionSignal) -> Result<SignalReply, Refusal> {
        if signal.cancel {
            let mut state = self.lock();
            if state.token != signal.execution_token {
                return Err(Refusal::of("cancellation token does not own execution"));
            }
            state.cancelled = true;
            state
                .cancelled_at
                .get_or_insert_with(std::time::Instant::now);
        }
        Ok(SignalReply::default())
    }

    async fn status(&self, _: &Supervisor, _: Empty) -> Result<Signals, Refusal> {
        let state = self.lock();
        Ok(Signals {
            starts: state.starts,
            cancelled: state.cancelled,
        })
    }

    async fn release(&self, _: &Supervisor, _: Empty) -> Result<Empty, Refusal> {
        self.lock().released = true;
        Ok(Empty {})
    }
}

/* === Public Functions === */

/* === Private Functions === */
const fn signal_timeout(mode: &str) -> Duration {
    match mode.as_bytes() {
        b"cancel_signal_ignored" => RUN_BOUND,
        _ => SIGNAL_BOUND,
    }
}

fn metadata(id: &str, label: &str) -> Metadata {
    Metadata::new(Identifier::new(id).unwrap(), label, None)
}

fn suite() -> TestSuite {
    TestSuite {
        metadata: metadata("fixture.sidecar", "Fixture Sidecar"),
        tags: vec!["fixture".into()],
        parameter_groups: Vec::new(),
        test_case_templates: [
            "pass",
            "fail",
            "fail2",
            "fail3",
            "abandon",
            "cancel_complete",
            "cancel_delayed",
            "cancel_refused",
            "cancel_ignored",
            "cancel_signal_ignored",
            "scope_error",
            "capture",
            "stream_log",
        ]
        .into_iter()
        .map(|mode| TestCaseTemplate {
            test_case_template_id: TemplateId::from_seed(format!("fixture.sidecar.{mode}")),
            metadata: metadata(&format!("fixture.sidecar.{mode}"), mode),
            tags: Vec::new(),
            parameter_groups: Vec::new(),
            parameters: Vec::new(),
            files: Vec::new(),
            document: None,
            phases: vec![ExecutionPhaseDefinition {
                repeatable: false,
                step_id: StepId::from_seed(format!("fixture.sidecar.{mode}.stream")),
                metadata: metadata("stream", "Stream"),
            }],
            criteria: Vec::new(),
            continue_on_failure: false,
            prompt_definitions: Vec::new(),
            subscriptions: if mode == "scope_error" {
                vec![Subscription::Stream(
                    Identifier::new("host.system").unwrap(),
                )]
            } else {
                Vec::new()
            },
            attributes: BTreeMap::new(),
        })
        .collect(),
    }
}

fn record_contract() -> RecordContract {
    RecordContract {
        record_type: Identifier::new("fixture.sidecar.observed").unwrap(),
        schema_version: 1,
        metadata: metadata("fixture.sidecar.observed", "Fixture Sidecar Observed"),
        display: Default::default(),
        template_id: None,
        owner: None,
        default_severity: RecordSeverity::Info,
        fields: vec![RecordField::new(
            metadata("value", "Value"),
            ValueType::Number,
        )],
        validation_mode: None,
        allow_extra_fields: false,
    }
}

fn cancellation_log() -> Vec<dokime_runtime::execution::log::Entry> {
    use dokime_runtime::execution::log::{Entry, Level};
    vec![
        Entry::new(Level::Trace, "trace detail"),
        Entry::new(Level::Debug, "debug detail"),
        Entry::new(Level::Info, "fixture cancellation settled"),
        Entry::new(Level::Warn, "warning\ncontinued"),
        Entry::new(Level::Error, "error detail"),
        Entry::new(Level::Info, ""),
        Entry {
            level: 99,
            message: "unknown detail".into(),
            timestamp: dokime_runtime::execution::log::timestamp(),
        },
    ]
}

fn main() -> std::io::Result<()> {
    dokime_runtime::plugin::run::<Protocol>()
}

/* === Tests === */
