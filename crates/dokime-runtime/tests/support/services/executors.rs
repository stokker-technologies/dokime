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
// Third Party Imports

// Local Imports
use super::evidence::{
    activate_fixture_execution, custom_outcome, custom_record, host_system_batch, provider_execution_scope, python_outcome,
    runnable_executions, tracking_batch, tracking_derived_record, tracking_transition_batch,
};
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};
use dokime_runtime::transport::ProviderExecutionFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct FakePythonExecutor {
    status: String,
    case_parameter_status: bool,
}

#[derive(Default)]
pub struct BlockingPublishingExecutor {
    entered: Entered,
    released: tokio::sync::watch::Sender<bool>,
}

#[derive(Default)]
pub struct BlockingErrorExecutor {
    entered: Entered,
    released: tokio::sync::watch::Sender<bool>,
}

#[derive(Default)]
pub struct MemberStopExecutor {
    entered: Entered,
}

#[derive(Default)]
pub struct SecondExecutionHoldExecutor {
    executions: std::sync::atomic::AtomicUsize,
    entered: Entered,
    released: tokio::sync::watch::Sender<bool>,
}

#[derive(Default)]
struct Entered(tokio::sync::watch::Sender<bool>);

pub struct PanickingExecutor;

pub struct PublishingExecutor;

pub struct DocumentAddingExecutor;

pub struct TrackingTransitionExecutor;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl FakePythonExecutor {
    pub fn fixed(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            case_parameter_status: false,
        }
    }

    pub fn case_parameter() -> Self {
        Self {
            status: "passed".to_owned(),
            case_parameter_status: true,
        }
    }
}

impl Entered {
    fn signal(&self) {
        self.0.send_replace(true);
    }

    async fn wait(&self) {
        let mut entered = self.0.subscribe();
        entered.wait_for(|entered| *entered).await.expect("the executor owns its entry signal");
    }
}

impl BlockingPublishingExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn wait_until_entered(&self) {
        self.entered.wait().await;
    }

    pub fn release(&self) {
        self.released.send_replace(true);
    }
}

impl BlockingErrorExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn wait_until_entered(&self) {
        self.entered.wait().await;
    }

    pub fn release(&self) {
        self.released.send_replace(true);
    }
}

impl SecondExecutionHoldExecutor {
    pub async fn wait_until_entered(&self) {
        self.entered.wait().await;
    }

    pub fn release(&self) {
        self.released.send_replace(true);
    }
}

impl MemberStopExecutor {
    pub async fn wait_until_entered(&self) {
        self.entered.wait().await;
    }
}

impl ExecutionFixture for FakePythonExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for execution in runnable_executions(cycle) {
                activate_fixture_execution(&context, execution).await;
            }
            Ok(python_outcome(cycle, &self.status, self.case_parameter_status))
        })
    }
}

impl ExecutionFixture for BlockingPublishingExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if let Some(execution) = runnable_executions(cycle).next() {
                activate_fixture_execution(&context, execution).await;
            }
            let mut released = self.released.subscribe();
            self.entered.signal();
            while !*released.borrow_and_update() {
                if context.is_cancelled() {
                    return Err(dokime_runtime::transport::ProviderExecutionFailure::new("cancel", "fixture cancelled"));
                }
                tokio::select! {
                    _ = released.changed() => {},
                    () = context.cancelled() => {},
                }
            }

            Ok(python_outcome(cycle, "passed", false))
        })
    }
}

impl ExecutionFixture for SecondExecutionHoldExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let ordinal = self.executions.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            for execution in runnable_executions(cycle) {
                activate_fixture_execution(&context, execution).await;
            }
            if ordinal > 0 {
                let mut released = self.released.subscribe();
                self.entered.signal();
                tokio::select! {
                    released = released.wait_for(|released| *released) => {
                        released.expect("the executor owns its release signal");
                    },
                    () = context.cancelled() => {
                        return Err(ProviderExecutionFailure::new("cancel", "fixture cancelled"));
                    }
                }
            }
            Ok(python_outcome(cycle, "passed", true))
        })
    }
}

impl ExecutionFixture for BlockingErrorExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if let Some(execution) = runnable_executions(cycle).next() {
                activate_fixture_execution(&context, execution).await;
            }
            let mut released = self.released.subscribe();
            self.entered.signal();
            released.wait_for(|released| *released).await.expect("the executor owns its release signal");

            Err(ProviderExecutionFailure::new("worker", "worker was cancelled"))
        })
    }
}

impl ExecutionFixture for MemberStopExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            // === Hold The First Execution Until It Alone Is Stopped === //

            let first = runnable_executions(cycle).next().expect("cycle has a runnable execution").clone();
            activate_fixture_execution(&context, &first).await;
            self.entered.signal();
            tokio::select! {
                () = context.execution_stopped(&first.test_execution_id) => {},
                () = context.cancelled() => {
                    return Err(ProviderExecutionFailure::new("cancel", "fixture cancelled"));
                }
            }

            // === Run The Siblings To Completion === //

            for execution in runnable_executions(cycle).skip(1) {
                assert!(
                    !context.execution_stop_requested(&execution.test_execution_id),
                    "stopping one execution must not mark its siblings"
                );
                activate_fixture_execution(&context, execution).await;
            }
            let mut outcome = python_outcome(cycle, "passed", false);
            let stopped = outcome
                .executions
                .iter_mut()
                .find(|execution| execution.test_execution_id == first.test_execution_id.as_str())
                .expect("stopped execution has an outcome");
            stopped.status = "cancelled".to_owned();
            stopped.criteria.clear();
            for phase in &mut stopped.phases {
                phase.status = "cancelled".to_owned();
            }
            Ok(outcome)
        })
    }
}

impl ExecutionFixture for PanickingExecutor {
    fn run_cycle<'a>(
        &'a self, _context: ExecutionFixtureContext, _cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            panic!("fixture provider panic");
        })
    }
}

impl ExecutionFixture for PublishingExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution_scope = provider_execution_scope(cycle);

            context.publish_record(custom_record()).await.unwrap();
            let mut tracking = tracking_batch();
            for sample in &mut tracking.samples {
                sample.scope = Some(execution_scope.clone());
            }
            context.publish_telemetry_batch(tracking).await.unwrap();
            let mut host_batch = host_system_batch();
            for sample in &mut host_batch.samples {
                sample.scope = Some(execution_scope.clone());
            }
            context.publish_telemetry_batch(host_batch).await.unwrap();

            Ok(custom_outcome(cycle))
        })
    }
}

impl ExecutionFixture for DocumentAddingExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution_scope = provider_execution_scope(cycle);

            let mut discovered = dokime_runtime::document::DocumentSection::new("Discovered Streams");
            discovered.id("run.discovered");
            discovered.note("The bench answered on one stream.");
            context
                .add_document_parts(execution_scope.clone(), vec![dokime_runtime::document::DocumentAddition::section(discovered)])
                .await
                .unwrap();
            // The same id a second time changes nothing, so a re-delivered
            // drain page cannot duplicate what the host already merged.
            let mut replaced = dokime_runtime::document::DocumentSection::new("Replaced");
            replaced.id("run.discovered");
            context.add_document_parts(execution_scope, vec![dokime_runtime::document::DocumentAddition::section(replaced)]).await.unwrap();

            Ok(custom_outcome(cycle))
        })
    }
}

impl ExecutionFixture for TrackingTransitionExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution_scope = provider_execution_scope(cycle);

            let mut batch = tracking_transition_batch();
            for sample in &mut batch.samples {
                sample.scope = Some(execution_scope.clone());
            }
            context.publish_telemetry_batch(batch).await.unwrap();
            for record_type in ["target.lost", "target.acquired", "tool.connection.lost", "tool.connection.acquired"] {
                context.publish_record(tracking_derived_record(record_type, &execution_scope)).await.unwrap();
            }

            Ok(custom_outcome(cycle))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
