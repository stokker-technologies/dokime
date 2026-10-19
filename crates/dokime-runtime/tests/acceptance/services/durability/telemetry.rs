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

async fn wait_for_telemetry(handle: &RuntimeHandle, selection: &TelemetrySelection, minimum: usize) -> Vec<TelemetrySample> {
    crate::service_support::waits::until_telemetry(handle, || async {
        let samples = handle.selected_telemetry(selection.clone()).await;
        (samples.len() >= minimum).then_some(samples)
    })
    .await
}

/* === Tests === */
#[tokio::test]
async fn checkpointed_telemetry_kill() {
    const PUBLISHED: usize = 25;

    if std::env::var_os("DOKIME_TELEMETRY_KILL_CHILD").is_some() {
        run_telemetry_kill_child(PUBLISHED).await;
        return;
    }

    let root = tempdir().unwrap();
    let marker = root.path().join("terminal-telemetry-ready");
    let mut child = crate::service_support::children::ChildStory::spawn(
        "services::durability::telemetry::checkpointed_telemetry_kill",
        &[("DOKIME_TELEMETRY_KILL_CHILD", "1".as_ref()), ("DOKIME_TELEMETRY_KILL_ROOT", root.path().as_os_str())],
    );
    child.ready().await;
    child.kill();

    let (mut rebooted, handle) = booted(root.path()).await;
    // Recovery Seals Every Previous-Session Container Before New Admission.
    let resident = handle.cycles().await.into_iter().map(|cycle| (cycle.saved.is_some(), cycle.lifecycle)).collect::<Vec<_>>();
    assert!(
        resident.iter().all(|(saved, lifecycle)| *saved && *lifecycle == CycleStatus::Stopped),
        "recovered resident cycles are sealed and stopped: {resident:?}"
    );
    let checkpoint = serde_json::from_slice(&std::fs::read(&marker).unwrap()).unwrap();
    let view = crate::service_support::native::open(&handle, &checkpoint).await;
    assert_eq!(view.execution().status, ExecutionStatus::Passed);
    let after = crate::service_support::native::samples(&handle, &checkpoint).await;
    let after_elapsed = elapsed_seconds(&after);
    for index in 0..PUBLISHED {
        assert!(after_elapsed.contains(&(index as f64).to_bits().to_string()), "completed sample {index} was lost across the process kill",);
    }
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

async fn run_telemetry_kill_child(published: usize) {
    let root = PathBuf::from(std::env::var_os("DOKIME_TELEMETRY_KILL_ROOT").expect("telemetry child root env"));
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_runtime, handle) = booted_with_fixture(&root, executor.clone()).await;

    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let test_cycle_id = start_cycle(&handle, suite, false).await;
    handle.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;

    let cycle = cycle_snapshot(&handle, &test_cycle_id).await;
    let execution =
        cycle.test_executions.iter().find(|execution| execution.status == ExecutionStatus::Running).expect("running execution exists");
    let case = cycle.cases().find(|case| case.test_case_id == execution.test_case_id).expect("execution case exists");
    let scope = RecordScope::default()
        .with_cycle(test_cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone())
        .with_template(case.test_case_template_id.clone());
    let samples = (0..published)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                Identifier::new("tracking").unwrap(),
                0,
                Timestamp::new(format!("2026-01-01T00:00:{index:02}Z")),
                "python.fixture",
            );
            sample.scope = Some(scope.clone());
            sample.values.insert("elapsed_s".to_owned(), Value::Number(index as f64));
            sample.values.insert("target_locked".to_owned(), Value::Bool(true));
            crate::service_support::contracts::typed_minimal_tracking_sample(sample)
        })
        .collect();
    handle
        .publish_telemetry_batch(TelemetryBatch::new(
            dokime_runtime::model::identity::new_id(),
            Identifier::new("tracking").unwrap(),
            samples,
        ))
        .await
        .unwrap();

    executor.release();
    wait_for_cycle_lifecycle(&handle, &test_cycle_id, CycleStatus::Completed).await;
    wait_for_record_type(&handle, "provider.execution.completed").await;
    let selection = TelemetrySelection {
        streams: vec![Identifier::new("tracking").unwrap().into()],
        scope: ScopeSelection { test_cycle_id: Some(test_cycle_id), ..ScopeSelection::default() },
        ..TelemetrySelection::default()
    };
    let before = wait_for_telemetry(&handle, &selection, published).await;
    assert!(before.len() >= published);
    // The checkpoint is published a moment after the telemetry settles.
    let checkpoint = crate::service_support::native::checkpoint(&handle, &execution.test_execution_id).await;
    std::fs::write(root.join("terminal-telemetry-ready"), serde_json::to_vec(&checkpoint).unwrap()).expect("write telemetry marker");
    crate::service_support::children::announce_ready();

    std::future::pending::<()>().await;
}
