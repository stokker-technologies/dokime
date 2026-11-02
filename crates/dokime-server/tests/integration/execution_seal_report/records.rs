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
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
// Third Party Imports
use tokio::sync::Notify;
use tokio_stream::StreamExt as _;
// Local Imports
use super::*;
use dokime::domain::model;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::host::ToolConnectionAcquired;
use dokime::runtime::command::CommandContext;

/* === Constants === */

const SUITE: &str = "fixture.bench";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Expectation {
    budget: Duration,
    listening: Notify,
    heard: std::sync::Mutex<Option<String>>,
}

struct Bench;

struct BenchSuite;

struct Waiting {
    expectation: Arc<Expectation>,
}

#[dokime::parameters]
struct WaitingParameters {
    #[parameter(label = "Expectation fixture", default = "unregistered")]
    token: String,
}

/* === Enums === */

#[dokime::catalog]
enum Templates {
    Waiting(Waiting),
}

#[dokime::catalog]
enum Suites {
    Bench(BenchSuite),
}

#[dokime::catalog]
enum Subscriptions {
    ToolConnectionAcquired(ToolConnectionAcquired),
}

/* === Traits === */

/* === Implementations === */

#[dokime::suite(id = "fixture.bench", label = "Fixture Bench", templates = Templates)]
impl BenchSuite {}

#[dokime::template(
    id = "fixture.bench.waiting",
    label = "Wait For The Unit",
    parameters = WaitingParameters,
    subscribes = Subscriptions
)]
impl Waiting {
    async fn initialize(parameters: &WaitingParameters, _: &mut dokime::execution::InitializationContext<'_>) -> dokime::Result<Self> {
        let expectation = expectations().lock().unwrap().remove(&parameters.token).expect("the case names its registered fixture");
        Ok(Self { expectation })
    }

    #[dokime::step(label = "Wait")]
    async fn wait(&mut self, ctx: &mut dokime::execution::StepContext<'_>) -> dokime::Result<()> {
        let mut connected = ctx.listen::<ToolConnectionAcquired>().timeout(self.expectation.budget);
        self.expectation.listening.notify_one();
        let connected = connected.next().await.transpose()?.expect("a live subscription yields until it is dropped");
        *self.expectation.heard.lock().unwrap() = Some(connected.resource_id);
        Ok(())
    }
}

impl dokime::harmos::builtins::lifecycle::Policy for Bench {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
        timeout: Duration::from_secs(120),
        attempt_timeout: Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime::plugin(id = "fixture_bench", requires(), contributions(dokime::Suites<Suites>), restart = "never")]
impl dokime::Plugin for Bench {
    const CATALOGS: &'static [dokime::harmos::Catalog] = &[];
}

impl dokime::harmos::Lifecycle for Bench {
    type Config = dokime::plugin::Configuration;
    type Error = dokime::Refusal;
    type Context = dokime::Supervisor;

    async fn prepare(_: Self::Config, setup: &mut dokime::harmos::Setup<Self>) -> Result<Self, Self::Error> {
        setup.resource::<()>(Arc::new(()));
        dokime::execution::prepare::<Self, Suites>(setup)?;
        Ok(Self)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn expectations() -> &'static Mutex<BTreeMap<String, Arc<Expectation>>> {
    static EXPECTATIONS: OnceLock<Mutex<BTreeMap<String, Arc<Expectation>>>> = OnceLock::new();
    EXPECTATIONS.get_or_init(Mutex::default)
}

async fn waiting_cycle(root: &std::path::Path, expectation: Arc<Expectation>) -> (Runtime, dokime::TestCycleId) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let token = format!("record-listener-{}", NEXT.fetch_add(1, Ordering::Relaxed));
    expectations().lock().unwrap().insert(token.clone(), expectation);
    let config = support::budgets::runtime_config(root, support::metadata("grpc.records", "Records Host"));
    let config = dokime::testing::plugin::<Bench>(config, root).unwrap();
    let owner = Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    let mut suite = BenchSuite::definition().suite;
    suite.test_case_templates[0].parameters[0].expression = dokime::definition::ParameterExpr::Literal { value: token.into() };
    let cases = support::materialize_cases(SUITE, &suite);
    let cycle = support::cycle_with_cases(&handle, support::metadata("records.cycle", "Records Cycle"), cases).await;
    let execution_id = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    handle.start_execution(execution_id, model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    (owner, cycle)
}

fn connection(resource: &str) -> model::records::publication::RecordPublishRequest {
    let mut record = model::records::publication::RecordPublishRequest::new(
        Identifier::new("tool.connection.acquired").expect("a builtin record type"),
        dokime::records::RecordSeverity::Info,
    );
    record.message = Some("a unit reported itself on the bench".to_owned());
    record.details.extend([
        ("resource_id".to_owned(), resource.into()),
        ("resource_kind".to_owned(), "device".into()),
        ("connection_state".to_owned(), "connected".into()),
    ]);
    record
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn captured_record_watch() {
    tokio::time::timeout(Duration::from_secs(120), captured_watch()).await.unwrap();
}

async fn captured_watch() {
    let root = tempfile::tempdir().unwrap();
    let expectation = Arc::new(Expectation {
        budget: Duration::from_secs(120),
        listening: Notify::new(),
        heard: Default::default(),
    });
    let token = "captured-watch".to_owned();
    expectations().lock().unwrap().insert(token.clone(), expectation.clone());
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let config = support::budgets::runtime_config(root.path(), support::metadata("capture.watch", "Capture Watch"));
    let config = dokime::testing::plugin::<Bench>(config, root.path()).unwrap();
    let mut owner = Runtime::start(prepared.configuration(config)).await.unwrap();
    let handle = owner.handle();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut suite = BenchSuite::definition().suite;
    suite.test_case_templates[0].parameters[0].expression = dokime::definition::ParameterExpr::Literal { value: token.into() };
    let cycle =
        support::cycle_with_cases(&handle, support::metadata("capture.cycle", "Capture Cycle"), support::materialize_cases(SUITE, &suite))
            .await;
    let execution = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    let receipt = handle.start_execution(execution.clone(), model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    assert!(receipt.rejection.is_none(), "{receipt:?}");
    expectation.listening.notified().await;
    let selection = model::evidence::wire::CapturedRecordSelection {
        record_types: vec!["tool.connection.acquired".into()],
        scope: Some(model::identity::wire::EvidenceScope {
            execution_id: Some(execution.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut watch = client
        .download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords {
            selection: Some(selection.clone()),
            after: None,
        })
        .await
        .unwrap()
        .into_inner();
    let initial = watch.message().await.unwrap().unwrap();
    assert!(initial.records.is_empty());
    handle.publish_record(connection("captured-unit")).await.unwrap();
    let captured = loop {
        let page = watch.message().await.unwrap().unwrap();
        if let Some(record) = page.records.first() {
            break record.clone();
        }
    };
    assert!(captured.scope.as_ref().is_none_or(|scope| scope.execution_id.is_none()));
    let replay = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection.clone()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(replay.records.iter().map(|record| &record.record_id).collect::<Vec<_>>(), vec![&captured.record_id]);
    let cycle_page = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(model::evidence::wire::CapturedRecordSelection {
                scope: Some(model::identity::wire::EvidenceScope {
                    cycle_id: Some(cycle.to_string()),
                    ..Default::default()
                }),
                ..selection.clone()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(cycle_page.records.len(), 1);
    let unrelated = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(model::evidence::wire::CapturedRecordSelection {
                scope: Some(model::identity::wire::EvidenceScope {
                    execution_id: Some(dokime::TestExecutionId::from_seed("unrelated").to_string()),
                    ..Default::default()
                }),
                ..selection
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(unrelated.records.is_empty());
    let runtime_page = handle
        .selected_records(dokime::domains::evidence::RecordSelection {
            record_types: vec![Identifier::new("tool.connection.acquired").unwrap()],
            scope: dokime::domains::evidence::ScopeSelection {
                test_execution_id: Some(execution),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(runtime_page.len(), 1);
    assert!(runtime_page[0].scope.as_ref().is_none_or(|scope| scope.test_execution_id.is_none()));
    drop(watch);
    drop(client);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// A step waiting on a host record proceeds when the host publishes it from
// outside the execution.
pub(crate) async fn step_awaits_record() {
    let root = tempfile::tempdir().unwrap();
    let expectation = Arc::new(Expectation {
        budget: Duration::from_secs(120),
        listening: Notify::new(),
        heard: Default::default(),
    });
    let (mut owner, cycle) = waiting_cycle(root.path(), Arc::clone(&expectation)).await;
    let handle = owner.handle();

    expectation.listening.notified().await;
    handle.publish_record(connection("unit-a")).await.unwrap();

    let ended = support::wait_for_cycle_outcome(&handle, &cycle, dokime::domain::model::execution::result::ResultStatus::Passed).await;
    assert_eq!(ended.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(expectation.heard.lock().unwrap().as_deref(), Some("unit-a"));
    drop(handle);
    owner.shutdown().await.unwrap();
}

// With nobody publishing, the step's listen budget elapses and the run fails.
// The budget is short and real: advancing a paused clock would also expire
// the runtime's own 30 second call deadlines and race the outcome.
pub(crate) async fn record_wait_expires() {
    let root = tempfile::tempdir().unwrap();
    let expectation = Arc::new(Expectation {
        budget: Duration::from_millis(250),
        listening: Notify::new(),
        heard: Default::default(),
    });
    let (mut owner, cycle) = waiting_cycle(root.path(), Arc::clone(&expectation)).await;
    let handle = owner.handle();

    expectation.listening.notified().await;

    let ended = support::wait_for_cycle_outcome(&handle, &cycle, dokime::domain::model::execution::result::ResultStatus::Failed).await;
    assert_eq!(ended.test_executions[0].status, ExecutionStatus::Failed);
    assert!(expectation.heard.lock().unwrap().is_none());
    drop(handle);
    owner.shutdown().await.unwrap();
}
