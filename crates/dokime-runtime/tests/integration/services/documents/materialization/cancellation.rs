/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stopped Report Boundary Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

// Third Party Imports
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::telemetry::{SamplingScope, TelemetrySamplingRequest, TelemetrySourceKind};
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use dokime_runtime::domains::telemetry::{Sampler, SamplerSession, SettlementError};

// Local Imports
use crate::service_support::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct QueuedSamples {
    ready: tokio::sync::watch::Sender<bool>,
    queue: Mutex<Option<TelemetryBatch>>,
    dropped: AtomicU64,
}

#[derive(Clone, Default)]
struct CollectedQueue {
    state: Arc<CollectedQueueState>,
}
#[derive(Default)]
struct CollectedQueueState {
    queued: tokio::sync::watch::Sender<bool>,
    delivered: AtomicU64,
    samples: Mutex<Option<TelemetryBatch>>,
}
struct CollectedSession {
    state: Arc<CollectedQueueState>,
    settling: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for QueuedSamples {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_domain::model::execution::result::CycleView,
        _: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = runnable_executions(cycle).next().unwrap();
            activate_fixture_execution(&context, execution).await;
            let mut first = tracking_batch();
            for sample in &mut first.samples {
                sample.scope = Some(execution.scope.clone());
            }
            let receipt = context.publish_telemetry_batch(first).await.unwrap();
            assert_eq!(receipt.accepted, 1);
            // Record A Completed Phase Before Stop: Its Outcome Must Survive Evidence Truncation.
            context
                .update_execution_phase(
                    execution.scope.clone(),
                    identifier("fixture.phase"),
                    "Fixture Phase".into(),
                    "passed".into(),
                )
                .await
                .unwrap();
            let mut waiting = tracking_batch();
            for sample in &mut waiting.samples {
                sample.scope = Some(execution.scope.clone());
                sample.sequence = 999;
                sample
                    .values
                    .insert("distance_m".into(), Value::Number(999.0));
                *sample = typed_fixture_sample(sample.clone());
            }
            *self.queue.lock().unwrap() = Some(waiting);
            self.ready.send_replace(true);
            context.cancelled().await;
            // These Are Real Producer-Owned Rows Waiting To Be Submitted, Not Storage-Accepted Rows.
            let waiting = self.queue.lock().unwrap().take().unwrap();
            let receipt = context.publish_telemetry_batch(waiting).await.unwrap();
            assert_eq!(receipt.accepted, 0);
            assert_eq!(receipt.dropped, 1);
            self.dropped.store(receipt.dropped, Ordering::SeqCst);
            Err(dokime_runtime::transport::ProviderExecutionFailure::new(
                "cancel",
                "fixture cancelled with unsent samples",
            ))
        })
    }
}

impl Sampler for CollectedQueue {
    fn open(&self) -> std::result::Result<Box<dyn SamplerSession>, String> {
        Ok(Box::new(CollectedSession {
            state: self.state.clone(),
            settling: false,
        }))
    }
}
impl SamplerSession for CollectedSession {
    fn tick(&mut self, _: Timestamp, _: &[String]) {
        if *self.state.queued.borrow() {
            return;
        }
        let mut batch = tracking_batch();
        for sample in &mut batch.samples {
            sample.sequence = 2000;
            sample
                .values
                .insert("distance_m".into(), Value::Number(2000.0));
            *sample = typed_fixture_sample(sample.clone());
        }
        *self.state.samples.lock().unwrap() = Some(batch);
        self.state.queued.send_replace(true);
    }
    fn collected(&mut self) -> std::result::Result<Vec<TelemetryBatch>, String> {
        // Hold A Real Tick's Collection Debt Until Stop Settles The Original Session.
        if !self.settling {
            return Ok(Vec::new());
        }
        let batch = self.state.samples.lock().unwrap().take();
        if batch.is_some() {
            self.state.delivered.fetch_add(1, Ordering::SeqCst);
        }
        Ok(batch.into_iter().collect())
    }
    fn settle(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>,
    > {
        Box::pin(async move {
            self.settling = true;
            Ok(())
        })
    }
}

/* === Public Functions === */
#[tokio::test]
async fn stopped_report_boundary() {
    let root = tempdir().unwrap();
    let executor = Arc::new(QueuedSamples::default());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    executor
        .ready
        .subscribe()
        .wait_for(|ready| *ready)
        .await
        .unwrap();
    assert_eq!(
        executor
            .queue
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .samples
            .len(),
        1
    );
    let running = cycle_snapshot(&runtime, &cycle_id).await;
    let member = &running.test_executions[0];
    let sampler = CollectedQueue::default();
    let lease = runtime
        .request_sampling(
            SamplingScope {
                test_cycle_id: Some(cycle_id.to_string()),
                test_case_id: Some(member.test_case_id.to_string()),
                test_execution_id: Some(member.test_execution_id.to_string()),
            },
            TelemetrySamplingRequest {
                request_id: "queued.report.stop".into(),
                source_kind: TelemetrySourceKind::InstalledPlugin,
                source_id: "fixture.stop.queue".into(),
                stream: "tracking".into(),
                interval: Some(dokime_runtime::DurationMillis(10)),
                min_interval: None,
                start_after: None,
            },
            sampler.clone(),
        )
        .await
        .unwrap();
    sampler
        .state
        .queued
        .subscribe()
        .wait_for(|queued| *queued)
        .await
        .unwrap();
    assert_eq!(sampler.state.delivered.load(Ordering::SeqCst), 0);
    runtime.storage().flush().await.unwrap();
    let persisted_before_stop = runtime.storage().cursor_saved();
    runtime
        .stop_cycle(cycle_id.clone(), CommandContext::default())
        .await
        .unwrap();
    let stopped = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Stopped).await;
    let execution_id = stopped.test_executions[0].test_execution_id.clone();
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &execution_id).await;
    let settled = cycle_snapshot(&runtime, &cycle_id).await;
    let seal = settled.test_executions[0].saved.as_ref().unwrap();
    assert!(
        seal.source
            .evidence_through
            .covers(&persisted_before_stop)
            .unwrap()
    );
    assert!(
        seal.dependencies_saved
            .covers(&seal.source.evidence_through)
            .unwrap()
    );
    assert!(
        seal.dependencies_saved
            .covers(&seal.source.context.position)
            .unwrap()
    );
    let captured = crate::service_support::native::open(&runtime, &checkpoint)
        .await
        .snapshot()
        .await
        .unwrap();
    let samples = crate::service_support::native::samples(&runtime, &checkpoint).await;
    assert_eq!(
        captured.evidence_through().unwrap(),
        &seal.source.evidence_through
    );
    assert_eq!(captured.execution().test_execution_id, execution_id);
    assert!(captured.execution().saved.is_some());
    assert_eq!(executor.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(
        sampler.state.delivered.load(Ordering::SeqCst),
        1,
        "stop must settle the same queued physical collection"
    );
    assert!(sampler.state.samples.lock().unwrap().is_none());
    // TODO(source-hook): telemetry capture release publishes no notification; a lease-closed signal would replace this poll.
    while runtime.telemetry_capture_is_active(&lease.lease_id).await {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(executor.queue.lock().unwrap().is_none());
    let tracking = samples
        .iter()
        .filter(|sample| sample.stream.as_str() == "tracking")
        .collect::<Vec<_>>();
    assert_eq!(
        tracking.len(),
        1,
        "only the persisted baseline belongs to the report"
    );
    assert_eq!(
        tracking[0].values.get("distance_m"),
        Some(&Value::Number(1.0))
    );
    assert!(
        samples
            .iter()
            .all(|sample| sample.sequence != 999 && sample.sequence != 2000)
    );
    assert!(
        captured
            .context()
            .unwrap()
            .health
            .iter()
            .all(|item| !format!("{item:?}").contains("Telemetry capture destination failed")),
        "discarded sampler debt must not become a destination failure"
    );
    let lifecycle = captured.execution();
    assert!(
        lifecycle
            .steps
            .iter()
            .any(|step| step.metadata.identifier.as_str() == "fixture.phase"
                && step.status == StepStatus::Passed),
        "stopping evidence collection must preserve a recorded phase outcome"
    );
    assert_eq!(captured.execution().status, ExecutionStatus::Cancelled);
    let original_html = crate::service_support::reports::bytes(runtime.storage(), &seal.report).await;
    assert_eq!(
        crate::service_support::reports::bytes(runtime.storage(), &seal.report).await,
        original_html
    );
    drop((captured, checkpoint, runtime));
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
