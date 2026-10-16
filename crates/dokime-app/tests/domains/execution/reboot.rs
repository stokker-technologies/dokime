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
use std::process::Command;
// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn native_history_index() {
    let root = tempdir().unwrap();
    let (mut owner, handle) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle_id = start_cycle(&handle, suite, false).await;
    handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Completed).await;
    let execution_id = cycle.test_executions[0].test_execution_id.clone();
    let checkpoint = crate::support::native::checkpoint(&handle, &execution_id).await;
    drop(handle);
    owner.shutdown().await.unwrap();

    let mut expected = None;
    for attempt in 0..2 {
        let (mut owner, handle) = booted(root.path()).await;
        let history = handle
            .browse_executions(Default::default(), |page, _| Ok(page.clone()))
            .await
            .unwrap();
        assert!(history.discovery_complete && history.conflicts.is_empty());
        assert_eq!(history.entries.len(), 1);
        let current = format!("{:?}", history.entries[0]);
        if let Some(expected) = &expected {
            assert_eq!(&current, expected);
        }
        expected = Some(current);
        drop(history);
        let view = if attempt == 0 {
            let reservation = crate::support::budgets::hold_all_but(
                handle.storage(),
                std::num::NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            );
            let mut pending = Box::pin(handle.checkpoint_execution(&checkpoint));
            assert!(
                tokio::time::timeout(Duration::from_millis(20), pending.as_mut())
                    .await
                    .is_err(),
                "checkpoint admission should wait while another reader holds the budget",
            );
            drop(reservation);
            tokio::time::timeout(Duration::from_secs(2), pending)
                .await
                .expect("checkpoint should resume after the reader releases its budget")
                .unwrap()
        } else {
            crate::support::native::open(&handle, &checkpoint).await
        };
        assert_eq!(view.execution().test_execution_id, execution_id);
        assert!(view.execution().saved.is_some());
        assert!(!root.path().join("execution-history-index.json").exists());
        assert!(!root.path().join("execution-history-index.session").exists());
        drop(handle);
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn checkpointed_telemetry_kill() {
    const PUBLISHED: usize = 25;

    if std::env::var_os("DOKIME_TELEMETRY_KILL_CHILD").is_some() {
        run_telemetry_kill_child(PUBLISHED).await;
        return;
    }

    let root = tempdir().unwrap();
    let marker = root.path().join("terminal-telemetry-ready");
    let mut child = Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--exact")
        .arg("execution::reboot::checkpointed_telemetry_kill")
        .arg("--nocapture")
        .env("DOKIME_TELEMETRY_KILL_CHILD", "1")
        .env("DOKIME_TELEMETRY_KILL_ROOT", root.path())
        .spawn()
        .expect("spawn child telemetry writer");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);

    while tokio::time::Instant::now() < deadline {
        if marker.is_file() {
            break;
        }
        if let Some(status) = child.try_wait().expect("read child status") {
            panic!("child telemetry writer exited before marker: {status}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(marker.is_file(), "child did not finish telemetry capture");
    child.kill().expect("kill child telemetry writer");
    let _ = child.wait();

    let (mut rebooted, handle) = booted(root.path()).await;
    // A Sealed Cycle Leaves Live State At Boot. A Kill Before The Seal Reached
    // The Journal Leaves It Resident And Stopped; Its Saved Evidence Survives.
    assert!(
        handle
            .cycles()
            .await
            .iter()
            .all(|cycle| cycle.saved.is_none() && cycle.lifecycle == CycleStatus::Stopped)
    );
    let checkpoint = serde_json::from_slice(&std::fs::read(&marker).unwrap()).unwrap();
    let view = crate::support::native::open(&handle, &checkpoint).await;
    assert_eq!(view.execution().status, ExecutionStatus::Passed);
    let after = crate::support::native::samples(&handle, &checkpoint).await;
    let after_elapsed = elapsed_seconds(&after);
    for index in 0..PUBLISHED {
        assert!(
            after_elapsed.contains(&(index as f64).to_bits().to_string()),
            "completed sample {index} was lost across the process kill",
        );
    }
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

async fn run_telemetry_kill_child(published: usize) {
    let root = PathBuf::from(
        std::env::var_os("DOKIME_TELEMETRY_KILL_ROOT").expect("telemetry child root env"),
    );
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_runtime, handle) = booted_with_fixture(&root, executor.clone()).await;

    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let test_cycle_id = start_cycle(&handle, suite, false).await;
    handle
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;

    let cycle = cycle_snapshot(&handle, &test_cycle_id).await;
    let execution = cycle
        .test_executions
        .iter()
        .find(|execution| execution.status == ExecutionStatus::Running)
        .expect("running execution exists");
    let case = cycle
        .cases()
        .find(|case| case.test_case_id == execution.test_case_id)
        .expect("execution case exists");
    let scope = RecordScope::default()
        .with_cycle(test_cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone())
        .with_template(case.test_case_template_id.clone());
    let samples = (0..published)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                Identifier::new("tracking").unwrap(),
                0,
                Timestamp::new(format!("2026-01-01T00:00:{index:02}Z")),
                "python.fixture",
            );
            sample.scope = Some(scope.clone());
            sample
                .values
                .insert("elapsed_s".to_owned(), Value::Number(index as f64));
            sample
                .values
                .insert("target_locked".to_owned(), Value::Bool(true));
            crate::support::contracts::typed_minimal_tracking_sample(sample)
        })
        .collect();
    handle
        .publish_telemetry_batch(TelemetryBatch::new(
            Identifier::new("tracking").unwrap(),
            samples,
        ))
        .await
        .unwrap();

    executor.release();
    wait_for_cycle_lifecycle(&handle, &test_cycle_id, CycleStatus::Completed).await;
    wait_for_record_type(&handle, "provider.execution.completed").await;
    let selection = TelemetrySelection {
        streams: vec![Identifier::new("tracking").unwrap()],
        scope: ScopeSelection {
            test_cycle_id: Some(test_cycle_id),
            ..ScopeSelection::default()
        },
        ..TelemetrySelection::default()
    };
    let before = wait_for_stable_telemetry(&handle, &selection, published).await;
    assert!(before.len() >= published);
    // The checkpoint is published a moment after the telemetry settles; on a
    // loaded runner that moment can outlast the read, so pin on a deadline.
    let checkpoint =
        crate::support::native::checkpoint(&handle, &execution.test_execution_id).await;
    std::fs::write(
        root.join("terminal-telemetry-ready"),
        serde_json::to_vec(&checkpoint).unwrap(),
    )
    .expect("write telemetry marker");

    std::future::pending::<()>().await;
}

pub(crate) async fn live_telemetry_state() {
    const FIRST: u64 = 30;
    const SECOND: u64 = 20;

    let root = tempdir().unwrap();
    let (mut runtime, handle) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;

    // No execution owns these samples; they are process-local live telemetry.
    register_python_suite(&handle, fixture_suite(&["passed"])).await;

    let before = handle
        .publish_telemetry_batch(unscoped_tracking_batch(FIRST))
        .await
        .unwrap();
    assert_eq!(before.stream_cursors.len(), 1);
    assert_eq!(before.stream_cursors.values().copied().next(), Some(FIRST));

    let selection = TelemetrySelection {
        streams: vec![Identifier::new("tracking").unwrap()],
        ..TelemetrySelection::default()
    };

    // Graceful shutdown seals the durable streams; reboot on the same root.
    drop(handle);
    runtime.shutdown().await.unwrap();

    let (mut rebooted, handle) = booted(root.path()).await;

    assert!(
        handle
            .selected_telemetry(selection.clone())
            .await
            .is_empty()
    );
    register_python_suite(&handle, fixture_suite(&["passed"])).await;
    let after = handle
        .publish_telemetry_batch(unscoped_tracking_batch(SECOND))
        .await
        .unwrap();
    assert_eq!(after.stream_cursors.len(), 1);
    assert_eq!(
        after.stream_cursors.values().copied().next(),
        Some(SECOND),
        "the second process starts a fresh stream cursor",
    );

    // The full run of sequences the two sessions claimed is strictly increasing
    // with no collision.
    let mut claimed: Vec<u64> = handle
        .selected_telemetry(selection)
        .await
        .iter()
        .map(|sample| sample.sequence)
        .collect();
    claimed.sort_unstable();
    assert!(
        claimed.windows(2).all(|pair| pair[1] > pair[0]),
        "sequences claimed across the reboot are strictly increasing, no collision: {claimed:?}",
    );
    assert_eq!(
        claimed,
        (1..=SECOND).collect::<Vec<_>>(),
        "only the current process appears in live telemetry",
    );
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

/* === Helpers === */

fn unscoped_tracking_batch(count: u64) -> TelemetryBatch {
    let stream = Identifier::new("tracking").unwrap();
    let samples = (0..count)
        .map(|index| {
            let mut sample =
                TelemetrySample::new(stream.clone(), 0, Timestamp::now_utc(), "python.fixture");
            sample
                .values
                .insert("elapsed_s".to_owned(), Value::Number(index as f64));
            sample
                .values
                .insert("target_locked".to_owned(), Value::Bool(true));
            crate::support::contracts::typed_minimal_tracking_sample(sample)
        })
        .collect();
    TelemetryBatch::new(stream, samples)
}

async fn wait_for_stable_telemetry(
    handle: &RuntimeHandle,
    selection: &TelemetrySelection,
    minimum: usize,
) -> Vec<TelemetrySample> {
    let mut previous = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        let samples = handle.selected_telemetry(selection.clone()).await;
        if samples.len() >= minimum && samples.len() == previous {
            return samples;
        }
        previous = samples.len();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("telemetry did not stabilize at >= {minimum} samples (last: {previous})");
}

fn elapsed_seconds(samples: &[TelemetrySample]) -> Vec<String> {
    let mut values: Vec<String> = samples
        .iter()
        .filter_map(|sample| match sample.values.get("elapsed_s") {
            Some(Value::Number(value)) => Some(value.to_bits().to_string()),
            _ => None,
        })
        .collect();
    values.sort();
    values
}

/* === Tests === */
