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
use std::future::Future;
use std::time::Duration;

// Third Party Imports
use futures_util::{FutureExt, StreamExt};

// Local Imports
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_runtime::RuntimeHandle;
use dokime_runtime::TestCycleId;
use dokime_runtime::runtime::query::{
    RuntimeResolution, RuntimeStateSnapshot, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow,
};
use dokime_runtime::state::RuntimeState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Re-Runs `check` After Every Committed State Revision Or Operation Change Until It Answers.
///
/// The watch subscribes before the first check, so a change between the check and
/// the wait wakes it at once. The lane deadline is the only timer.
pub async fn settle<T, F, Fut>(runtime: &RuntimeHandle, mut check: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let mut watch = runtime.watch_state();
    loop {
        if let Some(value) = check().await {
            return value;
        }
        watch.next_change().await.expect("the runtime state watch closes only with the runtime");
    }
}

/// Re-Runs `check` After Every Row The Durable Telemetry Stream Appends Until It Answers.
pub async fn until_telemetry<T, F, Fut>(runtime: &RuntimeHandle, mut check: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let mut rows = runtime.watch_plot_telemetry(&[]).await.unwrap().pop().expect("the durable telemetry stream is watchable");
    loop {
        if let Some(value) = check().await {
            return value;
        }
        rows.next().await.expect("the telemetry stream closes only with the runtime");
        // Coalesce Rows That Arrived Together Into One Re-Check.
        while let Some(Some(_)) = rows.next().now_or_never() {}
    }
}

/// Waits For The First Committed Runtime State `predicate` Accepts.
pub async fn until_state(runtime: &RuntimeHandle, predicate: impl Fn(&RuntimeState) -> bool) -> RuntimeStateSnapshot {
    runtime
        .watch_state()
        .wait_until(|snapshot| predicate(snapshot.state()))
        .await
        .expect("the runtime state watch closes only with the runtime")
}

pub async fn wait_for_record_type(runtime: &RuntimeHandle, record_type: &str) {
    let mut subscription = runtime.subscribe_records();
    let evidence = runtime.evidence_snapshot().await.unwrap();
    if evidence.records.iter().any(|record| record.record_type.as_str() == record_type) {
        return;
    }
    wait_for_subscription_record(&mut subscription, record_type).await;
}

pub async fn wait_for_subscription_record(subscription: &mut dokime_runtime::runtime::RecordSubscription, record_type: &str) {
    while let Some(record) = subscription.next().await {
        let record = record.expect("Harmos event subscription stays readable");
        if record.record_type.as_str() == record_type {
            return;
        }
    }
    panic!("Harmos event subscription closed before {record_type}");
}

pub async fn cycle_snapshot(runtime: &RuntimeHandle, test_cycle_id: &TestCycleId) -> dokime_domain::model::execution::result::CycleView {
    runtime.cycles().await.into_iter().find(|cycle| &cycle.test_cycle_id == test_cycle_id).unwrap()
}

pub async fn wait_for_cycle_lifecycle(
    runtime: &RuntimeHandle, test_cycle_id: &TestCycleId, lifecycle: CycleStatus,
) -> dokime_domain::model::execution::result::CycleView {
    let snapshot = until_state(runtime, |state| {
        state.cycle_views().any(|cycle| &cycle.test_cycle_id == test_cycle_id && cycle.lifecycle == lifecycle)
    })
    .await;
    snapshot.state().cycle_views().find(|cycle| &cycle.test_cycle_id == test_cycle_id).unwrap()
}

/// Waits Until A Run Receipt Is Terminal, Which Follows Its Seals Or Unsealed Cause.
pub async fn wait_for_run_settled(runtime: &RuntimeHandle, run: &dokime_runtime::CommandId) {
    // TODO(source-hook): RequestStorage publishes no receipt change; a watch on the durable run receipt would replace this poll.
    loop {
        if let Ok(Some(receipt)) = runtime.run_request(run).await
            && receipt.summary.phase.terminal()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub async fn wait_for_execution_document_payload(runtime: &RuntimeHandle, scope: dokime_runtime::RecordScope) {
    settle(runtime, || async {
        let query = runtime
            .query_runtime_view(RuntimeViewQuery {
                scope: Some(scope.clone()),
                topic: RuntimeViewTopic::Documents,
                window: RuntimeWindow::Full,
                resolution: RuntimeResolution::Raw,
                cursor: None,
            })
            .await
            .unwrap();
        let RuntimeViewPayload::Documents { frame } = query.payload else {
            panic!("expected document query frame");
        };
        frame.documents.iter().any(|document| document.has_payload).then_some(())
    })
    .await;
}

pub fn latest_execution_for_case<'a>(
    cycle: &'a dokime_domain::model::execution::result::CycleView, test_case_id: &dokime_runtime::CaseId,
) -> Option<&'a TestExecution> {
    cycle.test_executions.iter().rev().find(|execution| &execution.test_case_id == test_case_id)
}

/* === Private Functions === */

/* === Tests === */
