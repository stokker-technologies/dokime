/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio_stream::StreamExt as _;

use super::*;
use dokime::domain::model;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::host::ToolConnectionAcquired;
use dokime::runtime::command::CommandContext;

const SOURCE: &str = "fixture_bench";
const SUITE: &str = "fixture.bench";

struct Expectation {
    budget: Duration,
    listening: Notify,
    heard: std::sync::Mutex<Option<String>>,
}

struct Bench {
    expectation: Arc<Expectation>,
}

struct BenchSuite;

struct Waiting {
    expectation: Arc<Expectation>,
}

#[dokime::catalog]
enum Templates {
    Waiting(Waiting),
}

#[dokime::catalog]
enum Subscriptions {
    ToolConnectionAcquired(ToolConnectionAcquired),
}

#[dokime::suite(id = "fixture.bench", label = "Fixture Bench", templates = Templates)]
impl BenchSuite {}

#[dokime::template(
    id = "fixture.bench.waiting",
    label = "Wait For The Unit",
    subscribes = Subscriptions
)]
impl Waiting {
    async fn initialize(ctx: &mut dokime::execution::InitializationContext<'_>) -> dokime::Result<Self> {
        let expectation = Arc::clone(ctx.environment().binding::<Arc<Expectation>>());
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

#[async_trait::async_trait]
impl dokime::runtime::EmbeddedProvider for Bench {
    fn catalog(&self) -> dokime::InstalledPlugin {
        dokime::InstalledPlugin {
            source: SOURCE.into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime::definition::SuiteSet::new([BenchSuite::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<dokime::harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }

    async fn start(&self) -> dokime::runtime::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        let binding: Arc<dyn std::any::Any + Send + Sync> = Arc::new(Arc::clone(&self.expectation));
        Ok(
            vec![
                Arc::new(dokime::execution::Workflow::<Waiting>::new(SUITE, SOURCE).bound(binding)) as Arc<dyn dokime::execution::Runnable>
            ],
        )
    }

    async fn stop(&self) -> dokime::runtime::Result<()> {
        Ok(())
    }
}

async fn waiting_cycle(root: &std::path::Path, expectation: Arc<Expectation>) -> (Runtime, dokime::TestCycleId) {
    let config =
        support::budgets::runtime_config(root, support::metadata("grpc.records", "Records Host")).register(Arc::new(Bench { expectation }));
    let owner = Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    let suite = BenchSuite::definition().suite;
    let cycle =
        support::cycle_with_cases(&handle, support::metadata("records.cycle", "Records Cycle"), support::materialize_cases(SUITE, &suite))
            .await;
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

// A step waiting on a host record proceeds when the host publishes it from
// outside the execution.
#[tokio::test]
async fn step_awaits_record() {
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
#[tokio::test]
async fn record_wait_expires() {
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
