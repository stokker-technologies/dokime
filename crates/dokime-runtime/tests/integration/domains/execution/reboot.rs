/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

#[tokio::test]
async fn native_history_index() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle_id = start_cycle(&handle, suite).await;
    handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_outcome(&handle, &cycle_id, ResultStatus::Passed).await;
    let execution_id = cycle.test_executions[0].test_execution_id.clone();
    let checkpoint = crate::service_support::native::checkpoint(&handle, &execution_id).await;
    drop(handle);
    owner.shutdown().await.unwrap();

    let mut expected = None;
    for _ in 0..2 {
        let (mut owner, handle) = booted(root.path()).await;
        let history = handle.browse_executions(Default::default()).await.unwrap();
        assert!(history.discovery_complete && history.conflicts.is_empty());
        assert_eq!(history.entries.len(), 1);
        let current = format!("{:?}", history.entries[0]);
        if let Some(expected) = &expected {
            assert_eq!(&current, expected);
        }
        expected = Some(current);
        drop(history);
        let view = crate::service_support::native::open(&handle, &checkpoint).await;
        assert_eq!(view.execution().test_execution_id, execution_id);
        assert!(view.execution().saved.is_some());
        assert!(!root.path().join("execution-history-index.json").exists());
        assert!(!root.path().join("execution-history-index.session").exists());
        drop(handle);
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn stored_telemetry_reboot() {
    const FIRST: u64 = 30;
    const SECOND: u64 = 20;

    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;

    // Unscoped Originals Belong To Durable Primary Storage.
    register_python_suite(&handle, fixture_suite(&["passed"])).await;

    let before = handle.publish_telemetry_batch(unscoped_tracking_batch(FIRST)).await.unwrap();
    assert_eq!(before.stream_cursors.len(), 1);
    assert_eq!(before.stream_cursors.values().copied().next(), Some(FIRST));
    handle.storage().flush().await.unwrap();

    let selection = TelemetrySelection {
        streams: vec![Identifier::new("tracking").unwrap().into()],
        ..TelemetrySelection::default()
    };

    let first_positions =
        handle.selected_telemetry(selection.clone()).await.unwrap().iter().map(|sample| sample.sequence).collect::<Vec<_>>();
    assert_eq!(first_positions.len(), FIRST as usize);

    // Reopen The Same Durable History.
    drop(handle);
    runtime.shutdown().await.unwrap();

    let (mut rebooted, handle) = booted(root.path()).await;

    assert_eq!(
        handle.selected_telemetry(selection.clone()).await.unwrap().iter().map(|sample| sample.sequence).collect::<Vec<_>>(),
        first_positions
    );
    register_python_suite(&handle, fixture_suite(&["passed"])).await;
    let after = handle.publish_telemetry_batch(unscoped_tracking_batch(SECOND)).await.unwrap();
    handle.storage().flush().await.unwrap();
    assert_eq!(after.stream_cursors.len(), 1);
    assert_eq!(after.stream_cursors.values().copied().next(), Some(SECOND), "the second process starts a fresh stream cursor",);

    // The full run of sequences the two sessions claimed is strictly increasing
    // with no collision.
    let mut claimed: Vec<u64> = handle.selected_telemetry(selection).await.unwrap().iter().map(|sample| sample.sequence).collect();
    claimed.sort_unstable();
    assert!(
        claimed.windows(2).all(|pair| pair[1] > pair[0]),
        "sequences claimed across the reboot are strictly increasing, no collision: {claimed:?}",
    );
    assert_eq!(claimed.len(), (FIRST + SECOND) as usize);
    assert_eq!(&claimed[..FIRST as usize], first_positions.as_slice());
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

fn unscoped_tracking_batch(count: u64) -> TelemetryBatch {
    let stream = Identifier::new("tracking").unwrap();
    let samples = (0..count)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                stream.clone(),
                0,
                dokime_runtime::model::time::now_utc(),
                "python.fixture",
            );
            sample.values.insert("elapsed_s".to_owned(), Value::Number(index as f64));
            sample.values.insert("target_locked".to_owned(), Value::Bool(true));
            crate::service_support::contracts::typed_minimal_tracking_sample(sample)
        })
        .collect();
    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream, samples)
}
