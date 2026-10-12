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
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
// Third Party Imports

// Local Imports
use super::evidence::{
    activate_fixture_execution, custom_outcome, custom_record, host_system_batch,
    provider_execution_scope, python_outcome, runnable_executions, tracking_batch,
    tracking_derived_record, tracking_transition_batch,
};
use dokime::domains::ExecutionProviderTarget;
use dokime::transport::ProviderExecutionFailure;
use dokime_app::capabilities::execution::provider::{
    ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture,
};

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
    entered: AtomicBool,
    released: AtomicBool,
    release_signal: tokio::sync::Notify,
}

#[derive(Default)]
pub struct BlockingErrorExecutor {
    entered: AtomicBool,
    released: AtomicBool,
    release_signal: tokio::sync::Notify,
}

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

impl BlockingPublishingExecutor {
    pub fn new() -> Self {
        Self {
            entered: AtomicBool::new(false),
            released: AtomicBool::new(false),
            release_signal: tokio::sync::Notify::new(),
        }
    }

    pub async fn wait_until_entered(&self) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        while tokio::time::Instant::now() < deadline {
            if self.entered.load(Ordering::SeqCst) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("blocking executor did not enter within timeout");
    }

    pub fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
        self.release_signal.notify_waiters();
    }
}

impl BlockingErrorExecutor {
    pub fn new() -> Self {
        Self {
            entered: AtomicBool::new(false),
            released: AtomicBool::new(false),
            release_signal: tokio::sync::Notify::new(),
        }
    }

    pub async fn wait_until_entered(&self) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        while tokio::time::Instant::now() < deadline {
            if self.entered.load(Ordering::SeqCst) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("blocking error executor did not enter within timeout");
    }

    pub fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
        self.release_signal.notify_waiters();
    }
}

impl ExecutionFixture for FakePythonExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for execution in runnable_executions(cycle) {
                activate_fixture_execution(&context, execution).await;
            }
            Ok(python_outcome(
                cycle,
                &self.status,
                self.case_parameter_status,
            ))
        })
    }
}

impl ExecutionFixture for BlockingPublishingExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if let Some(execution) = runnable_executions(cycle).next() {
                activate_fixture_execution(&context, execution).await;
            }
            self.entered.store(true, Ordering::SeqCst);
            while !self.released.load(Ordering::SeqCst) {
                if context.is_cancelled() {
                    return Err(dokime::transport::ProviderExecutionFailure::new(
                        "cancel",
                        "fixture cancelled",
                    ));
                }
                tokio::select! {
                    _ = self.release_signal.notified() => {},
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {},
                }
            }

            Ok(python_outcome(cycle, "passed", false))
        })
    }
}

impl ExecutionFixture for BlockingErrorExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if let Some(execution) = runnable_executions(cycle).next() {
                activate_fixture_execution(&context, execution).await;
            }
            self.entered.store(true, Ordering::SeqCst);
            while !self.released.load(Ordering::SeqCst) {
                self.release_signal.notified().await;
            }

            Err(ProviderExecutionFailure::new(
                "worker",
                "worker was cancelled",
            ))
        })
    }
}

impl ExecutionFixture for PanickingExecutor {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        _cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            panic!("fixture provider panic");
        })
    }
}

impl ExecutionFixture for PublishingExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
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

            tokio::time::sleep(Duration::from_millis(5)).await;

            Ok(custom_outcome(cycle))
        })
    }
}

impl ExecutionFixture for DocumentAddingExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution_scope = provider_execution_scope(cycle);

            let mut discovered =
                dokime_app::document::DocumentSection::new("run.discovered", "Discovered Streams");
            discovered.note("The bench answered on one stream.");
            context
                .add_document_parts(
                    execution_scope.clone(),
                    vec![dokime::document::DocumentAddition::section(discovered)],
                )
                .await
                .unwrap();
            // The same id a second time changes nothing, so a re-delivered
            // drain page cannot duplicate what the host already merged.
            context
                .add_document_parts(
                    execution_scope,
                    vec![dokime::document::DocumentAddition::section(
                        dokime_app::document::DocumentSection::new("run.discovered", "Replaced"),
                    )],
                )
                .await
                .unwrap();

            Ok(custom_outcome(cycle))
        })
    }
}

impl ExecutionFixture for TrackingTransitionExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution_scope = provider_execution_scope(cycle);

            let mut batch = tracking_transition_batch();
            for sample in &mut batch.samples {
                sample.scope = Some(execution_scope.clone());
            }
            context.publish_telemetry_batch(batch).await.unwrap();
            for record_type in [
                "target.lost",
                "target.acquired",
                "tool.connection.lost",
                "tool.connection.acquired",
            ] {
                context
                    .publish_record(tracking_derived_record(record_type, &execution_scope))
                    .await
                    .unwrap();
            }

            Ok(custom_outcome(cycle))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
