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
use std::sync::Arc;
use std::time::Duration;
// Third Party Imports
use tokio::sync::Notify;
use tokio_stream::StreamExt as _;
// Local Imports
use super::*;
use dokime::host::ToolConnectionAcquired;
use dokime_app::execution::{CycleStatus, ExecutionStatus};
use dokime_app::runtime::command::CommandContext;

/* === Constants === */
const SOURCE: &str = "fixture-bench";
const SUITE: &str = "fixture.bench";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// The waiting step's budget, the signal it gives once it listens, and the
/// unit it heard.
struct Expectation {
    budget: Duration,
    listening: Notify,
    heard: std::sync::Mutex<Option<String>>,
}

/// Serves the waiting suite in process.
struct Bench {
    expectation: Arc<Expectation>,
}

struct BenchSuite;

struct Waiting {
    expectation: Arc<Expectation>,
}

/* === Enums === */
#[dokime::catalog]
enum Templates {
    Waiting(Waiting),
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
    subscribes = Subscriptions
)]
impl Waiting {
    async fn initialize(
        ctx: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        let expectation = Arc::clone(ctx.environment().binding::<Arc<Expectation>>());
        Ok(Self { expectation })
    }

    #[dokime::step(label = "Wait")]
    async fn wait(&mut self, ctx: &mut dokime::execution::StepContext<'_>) -> dokime::Result<()> {
        let mut connected = ctx
            .listen::<ToolConnectionAcquired>()
            .timeout(self.expectation.budget);
        self.expectation.listening.notify_one();
        let connected = connected
            .next()
            .await
            .transpose()?
            .expect("a live subscription yields until it is dropped");
        *self.expectation.heard.lock().unwrap() = Some(connected.resource_id);
        Ok(())
    }
}

#[async_trait::async_trait]
impl dokime_app::runtime::EmbeddedProvider for Bench {
    fn catalog(&self) -> dokime_app::InstalledPlugin {
        dokime_app::InstalledPlugin {
            source: SOURCE.into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_app::definition::SuiteSet::new([BenchSuite::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        let binding: Arc<dyn std::any::Any + Send + Sync> = Arc::new(Arc::clone(&self.expectation));
        Ok(vec![
            Arc::new(dokime::execution::Workflow::<Waiting>::new(SUITE, SOURCE).bound(binding))
                as Arc<dyn dokime::execution::Runnable>,
        ])
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
/// Starts a host serving the waiting suite and runs one cycle of it.
async fn waiting_cycle(
    root: &std::path::Path,
    expectation: Arc<Expectation>,
) -> (Runtime, dokime_app::core::TestCycleId) {
    let config =
        support::budgets::runtime_config(root, support::metadata("grpc.records", "Records Host"))
            .register(Arc::new(Bench { expectation }));
    let owner = Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    let suite = BenchSuite::definition().suite;
    let cycle = support::cycle_with_cases(
        &handle,
        support::metadata("records.cycle", "Records Cycle"),
        support::materialize_cases(SUITE, &suite),
    )
    .await;
    handle
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    (owner, cycle)
}

fn connection(resource: &str) -> dokime_app::records::RecordPublishRequest {
    let mut record = dokime_app::records::RecordPublishRequest::new(
        Identifier::new("tool.connection.acquired").expect("a builtin record type"),
        dokime_app::records::RecordSeverity::Info,
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
#[tokio::test]
async fn step_awaits_record() {
    let root = tempfile::tempdir().unwrap();
    let expectation = Arc::new(Expectation {
        budget: Duration::from_secs(30),
        listening: Notify::new(),
        heard: Default::default(),
    });
    let (mut owner, cycle) = waiting_cycle(root.path(), Arc::clone(&expectation)).await;
    let handle = owner.handle();

    expectation.listening.notified().await;
    handle.publish_record(connection("unit-a")).await.unwrap();

    let ended = support::wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Completed).await;
    assert_eq!(ended.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(expectation.heard.lock().unwrap().as_deref(), Some("unit-a"));
    drop(handle);
    owner.shutdown().await.unwrap();
}

// The listen deadline is time semantics: with nobody publishing, the step's
// budget elapses on the paused clock and the run fails.
#[tokio::test(start_paused = true)]
async fn record_wait_expires() {
    let root = tempfile::tempdir().unwrap();
    let expectation = Arc::new(Expectation {
        budget: Duration::from_secs(30),
        listening: Notify::new(),
        heard: Default::default(),
    });
    let (mut owner, cycle) = waiting_cycle(root.path(), Arc::clone(&expectation)).await;
    let handle = owner.handle();

    expectation.listening.notified().await;
    tokio::time::advance(Duration::from_secs(31)).await;

    let ended = support::wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Failed).await;
    assert_eq!(ended.test_executions[0].status, ExecutionStatus::Failed);
    assert!(expectation.heard.lock().unwrap().is_none());
    drop(handle);
    owner.shutdown().await.unwrap();
}
