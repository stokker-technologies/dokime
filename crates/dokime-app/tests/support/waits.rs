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
use std::time::Duration;
// Third Party Imports

// Local Imports
use dokime::core::{Identifier, TestCycleId};
use dokime::execution::TestExecution;
use dokime_app::RuntimeHandle;
use dokime_app::domains::evidence::TelemetrySelection;
use dokime_app::execution::CycleStatus;
use dokime_app::runtime::query::{
    RuntimeResolution, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn wait_for_streamed_tracking_sample(runtime: &RuntimeHandle) -> Option<usize> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        let samples = runtime
            .selected_telemetry(TelemetrySelection {
                streams: vec![Identifier::new("tracking").unwrap()],
                ..TelemetrySelection::default()
            })
            .await;
        if !samples.is_empty() {
            return Some(samples.len());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    None
}

pub async fn wait_for_record_type(runtime: &RuntimeHandle, record_type: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        let evidence = runtime.evidence_snapshot().await.unwrap();
        if evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == record_type)
        {
            return;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!("event did not appear: {record_type}");
}

pub async fn wait_for_subscription_record(
    subscription: &mut dokime_app::runtime::RecordSubscription,
    record_type: &str,
) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(record) = subscription.next().await {
            let record = record.expect("Harmos event subscription stays readable");
            let matches = record.record_type.as_str() == record_type;
            if matches {
                return;
            }
        }
        panic!("Harmos event subscription closed before {record_type}");
    })
    .await
    .unwrap_or_else(|_| panic!("event did not appear: {record_type}"));
}

pub async fn cycle_snapshot(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
) -> dokime_app::execution::CycleView {
    runtime
        .cycles()
        .await
        .into_iter()
        .find(|cycle| &cycle.test_cycle_id == test_cycle_id)
        .unwrap()
}

pub async fn wait_for_cycle_lifecycle(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
    lifecycle: CycleStatus,
) -> dokime_app::execution::CycleView {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        let cycle = cycle_snapshot(runtime, test_cycle_id).await;
        if cycle.lifecycle == lifecycle {
            return cycle;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "cycle did not reach {lifecycle:?}: {:?}",
        cycle_snapshot(runtime, test_cycle_id).await
    );
}

/// Waits Until A Run Receipt Is Terminal, Which Follows Its Seals Or Unsealed Cause.
pub async fn wait_for_run_settled(runtime: &RuntimeHandle, run: &dokime::core::CommandId) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        if let Ok(Some(receipt)) = runtime.run_request(run).await
            && receipt.summary.phase.terminal()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("run {run:?} did not settle");
}

pub async fn wait_for_execution_document_payload(
    runtime: &RuntimeHandle,
    scope: dokime::core::RecordScope,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
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
        if frame.documents.iter().any(|document| document.has_payload) {
            return;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!("execution document payload was not finalized");
}

pub fn latest_execution_for_case<'a>(
    cycle: &'a dokime_app::execution::CycleView,
    test_case_id: &dokime::core::CaseId,
) -> Option<&'a TestExecution> {
    cycle
        .test_executions
        .iter()
        .rev()
        .find(|execution| &execution.test_case_id == test_case_id)
}

/* === Private Functions === */

/* === Tests === */
