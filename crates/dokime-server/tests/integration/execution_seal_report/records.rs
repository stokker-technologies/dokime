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
