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

pub(crate) async fn cycle_exposes_rows() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let run = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(run.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    publish_tracking_sample(&runtime, &running).await;
    let streamed = wait_for_streamed_tracking_sample(&runtime).await;
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;

    let streamed = streamed.expect("tracking sample was readable from the durable stream");
    assert_eq!(running.test_executions.len(), 1);
    assert_eq!(running.test_executions[0].status, ExecutionStatus::Running);
    // The Provider's Explicit Running Phase Is Recorded On Its Template Step.
    assert_eq!(
        running.test_executions[0].steps[0].status,
        StepStatus::Running
    );
    assert_eq!(streamed, 1);
    assert_eq!(running.test_executions[0].steps.len(), 1);
}

pub(crate) async fn provider_lane_run() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let mut records = runtime.subscribe_records();
    let tracking = |evidence: &dokime_app::domains::evidence::EvidenceSnapshot| {
        (
            evidence
                .telemetry
                .iter()
                .filter(|sample| sample.stream.as_str() == "tracking")
                .count(),
            evidence
                .records
                .iter()
                .filter(|record| record.record_type.as_str() == "tracking.fixture_warning")
                .count(),
        )
    };

    // A freshly created cycle carries a real wall-clock `created_at`, not a
    // sequence-derived stamp near the Unix epoch.
    let created = cycle_snapshot(&runtime, &test_cycle_id).await.created_at;
    assert_wall_clock(&created);

    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    wait_for_subscription_record(&mut records, "provider.execution.completed").await;
    for execution in &cycle.test_executions {
        crate::support::native::checkpoint(&runtime, &execution.test_execution_id).await;
    }
    let evidence = runtime.evidence_snapshot().await.unwrap();

    // Running it records a wall-clock `started_at` at or after `created_at`, so
    // the elapsed span is well-formed, and every record it emits is stamped the
    // same way.
    let started = cycle
        .started_at
        .clone()
        .expect("a running cycle records started_at");
    assert_wall_clock(&started);
    assert!(
        created.millis_until(&started).is_some(),
        "duration from created_at to started_at must be non-negative: {} -> {}",
        created.as_str(),
        started.as_str(),
    );
    assert!(!evidence.records.is_empty(), "the cycle emits events");
    for record in &evidence.records {
        assert_wall_clock(&record.timestamp);
    }

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert_eq!(cycle.lifecycle, CycleStatus::Completed);
    assert_eq!(cycle.test_executions.len(), 1);
    assert_eq!(cycle.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(cycle.result.as_ref().unwrap().status, ResultStatus::Passed);
    assert_eq!(cycle.test_executions[0].steps.len(), 1);
    assert_eq!(
        cycle.test_executions[0].steps[0]
            .metadata
            .identifier
            .as_str(),
        "fixture.phase"
    );
    assert_eq!(
        cycle.test_executions[0].steps[0].metadata.label,
        "Fixture Phase"
    );
    assert!(
        evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == "provider.execution.completed")
    );
    assert!(evidence.records.iter().any(|record| {
        record.record_type.as_str() == "tracking.fixture_warning"
            && record
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.clone())
                .is_some()
            && record.severity == RecordSeverity::Warning
    }));
    assert!(
        evidence
            .health
            .iter()
            .any(|item| item.message.contains("Provider execution"))
    );
    assert_eq!(tracking(&evidence), (2, 1));
    wait_for_execution_document_payload(&runtime, cycle.test_executions[0].scope.clone()).await;

    // The same template with an array parameter expands to two cases, and each
    // of them leaves its own evidence behind.
    let mut repeated = plt400_suite();
    repeated.test_case_templates[0].parameters[0].expansion =
        dokime::definition::ParameterExpansion::Cartesian;
    repeated.test_case_templates[0].parameters[0].expression = ParameterExpr::Array {
        values: vec![
            ParameterExpr::Literal {
                value: Value::from("passed"),
            },
            ParameterExpr::Literal {
                value: Value::from("alternate"),
            },
        ],
    };
    register_python_suite(&runtime, repeated.clone()).await;
    let repeated_id = start_cycle(&runtime, repeated, false).await;
    // Starting It Retires The First Sealed Cycle And Its Live Record Visibility.
    let baseline = tracking(&runtime.evidence_snapshot().await.unwrap());
    let queued = cycle_snapshot(&runtime, &repeated_id).await;
    assert_eq!(queued.cases().count(), 2);

    let receipt = runtime
        .run_cycle_cases(repeated_id.clone(), CommandContext::default())
        .await;
    let repeated_cycle =
        wait_for_cycle_lifecycle(&runtime, &repeated_id, CycleStatus::Completed).await;
    // Completion Precedes Durable Capture; Read After Those Writers Settle.
    for execution in &repeated_cycle.test_executions {
        crate::support::native::checkpoint(&runtime, &execution.test_execution_id).await;
    }
    let after = runtime.evidence_snapshot().await.unwrap();

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert_eq!(repeated_cycle.test_executions.len(), 2);
    assert_eq!(repeated_cycle.result.as_ref().unwrap().passed, 2);
    assert!(
        repeated_cycle
            .test_executions
            .iter()
            .all(|execution| execution.status == ExecutionStatus::Passed)
    );
    let (samples, warnings) = tracking(&after);
    assert_eq!(samples - baseline.0, 4);
    assert_eq!(warnings - baseline.1, 2);
}

pub(crate) async fn failing_panicking_failed() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("failed"))).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let mut records = runtime.subscribe_records();

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Failed).await;
    wait_for_subscription_record(&mut records, "provider.execution.failed").await;
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert_eq!(cycle.result.as_ref().unwrap().status, ResultStatus::Failed);
    for report in [
        &cycle
            .saved
            .as_ref()
            .expect("faulted cycle has a final report")
            .report,
        &cycle.test_executions[0]
            .saved
            .as_ref()
            .expect("faulted execution has a final report")
            .report,
    ] {
        let payload = crate::support::reports::payload(runtime.storage(), report).await;
        assert_eq!(payload["test_executions"][0]["status"], "failed");
        assert!(
            payload["snapshot"]["records"]
                .as_array()
                .unwrap()
                .iter()
                .any(|record| { record["details"]["stacktrace"] == "fixture stacktrace" }),
            "fault reports retain available failure evidence"
        );
    }

    assert!(
        evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == "provider.execution.failed")
    );
    let failure_record = evidence
        .records
        .iter()
        .find(|record| record.record_type.as_str() == "test.step.failed")
        .expect("failed Python execution emits a step failure event");
    assert_eq!(
        failure_record.details.get("label"),
        Some(&Value::from("Collect Tracking"))
    );
    assert_eq!(
        failure_record.details.get("stacktrace"),
        Some(&Value::from("fixture stacktrace"))
    );
    assert!(
        evidence
            .health
            .iter()
            .any(|item| item.severity == dokime::execution::HealthSeverity::Error)
    );

    // A panicking provider cannot take the runtime with it: Harmos's job
    // supervision contains it and the cycle still reaches a terminal result.
    let panicking_root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(panicking_root.path(), Arc::new(PanickingExecutor)).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let panicking_id = start_cycle(&runtime, suite, false).await;

    let receipt = runtime
        .run_cycle_cases(panicking_id.clone(), CommandContext::default())
        .await;
    let panicking_cycle =
        wait_for_cycle_lifecycle(&runtime, &panicking_id, CycleStatus::Failed).await;
    let overview = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds");

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert_eq!(
        panicking_cycle.result.as_ref().unwrap().status,
        ResultStatus::Failed
    );
    for report in [
        &panicking_cycle
            .saved
            .as_ref()
            .expect("panic does not skip cycle report finalization")
            .report,
        &panicking_cycle.test_executions[0]
            .saved
            .as_ref()
            .expect("panic does not skip execution report finalization")
            .report,
    ] {
        let payload = crate::support::reports::payload(runtime.storage(), report).await;
        assert_eq!(payload["test_executions"][0]["status"], "failed");
    }
    assert!(overview.installed_plugin_errors.iter().any(|error| {
        error.test_cycle_id.as_ref() == Some(&panicking_id)
            && error.code.as_deref() == Some("provider_panic")
    }));
}

pub(crate) async fn stop_holds_unwind() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingErrorExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;

    let stop = runtime
        .stop_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    let stopping = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopping).await;
    let archive = runtime
        .set_cycle_archived(test_cycle_id.clone(), true, CommandContext::default())
        .await;
    executor.release();

    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopped).await;
    let cycle = cycle_snapshot(&runtime, &test_cycle_id).await;
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert!(stop.is_ok());
    assert_eq!(stopping.lifecycle, CycleStatus::Stopping);
    assert!(archive.is_err(), "a stopping cycle is not terminal yet");
    assert_eq!(cycle.test_executions[0].status, ExecutionStatus::Cancelled);
    assert_eq!(
        cycle.result.as_ref().unwrap().status,
        ResultStatus::Cancelled
    );
    assert!(
        evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == "test.execution.stopped")
    );
    // A stop is terminal: the cycle and its execution seal their final
    // reports exactly as completion does, labelled stopped.
    for report in [
        &cycle
            .saved
            .as_ref()
            .expect("a stopped cycle seals its final report")
            .report,
        &cycle.test_executions[0]
            .saved
            .as_ref()
            .expect("a stopped execution seals its final report")
            .report,
    ] {
        let payload = crate::support::reports::payload(runtime.storage(), report).await;
        assert_eq!(payload["test_executions"][0]["status"], "stopped");
    }
}

pub(crate) async fn execution_stop_spares_sibling() {
    // === Start A Two-Case Cycle === //

    let root = tempdir().unwrap();
    let executor = Arc::new(MemberStopExecutor::default());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let mut suite = plt400_suite();
    suite.test_case_templates[0].parameters[0].expansion =
        dokime::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Array {
        values: vec![
            ParameterExpr::Literal {
                value: Value::from("passed"),
            },
            ParameterExpr::Literal {
                value: Value::from("alternate"),
            },
        ],
    };
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let stopped_id = running
        .test_executions
        .iter()
        .find(|execution| execution.status == ExecutionStatus::Running)
        .expect("first execution is running")
        .test_execution_id
        .clone();

    // === Stop One Execution === //

    let stop = runtime
        .stop_execution(stopped_id.clone(), CommandContext::default())
        .await;
    let during = cycle_snapshot(&runtime, &test_cycle_id).await;
    let repeated = runtime
        .stop_execution(stopped_id.clone(), CommandContext::default())
        .await;

    // === Settle And Read The Sealed Reports === //

    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopped).await;
    let stopped = cycle
        .test_executions
        .iter()
        .find(|execution| execution.test_execution_id == stopped_id)
        .unwrap();
    let sibling = cycle
        .test_executions
        .iter()
        .find(|execution| execution.test_execution_id != stopped_id)
        .unwrap();
    let cycle_report = crate::support::reports::payload(
        runtime.storage(),
        &cycle
            .saved
            .as_ref()
            .expect("a cycle with a stopped execution seals its final report")
            .report,
    )
    .await;
    let execution_report = crate::support::reports::payload(
        runtime.storage(),
        &stopped
            .saved
            .as_ref()
            .expect("a stopped execution seals its final report")
            .report,
    )
    .await;

    assert!(stop.is_ok(), "{stop:?}");
    assert!(
        repeated.is_err(),
        "a second stop of the same execution is refused"
    );
    assert_eq!(during.lifecycle, CycleStatus::Running);
    assert_eq!(cycle.test_executions.len(), 2);
    assert_eq!(stopped.status, ExecutionStatus::Cancelled);
    assert_eq!(sibling.status, ExecutionStatus::Passed);
    assert_eq!(execution_report["test_executions"][0]["status"], "stopped");
    let statuses = cycle_report["test_executions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|execution| execution["status"].as_str().unwrap().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(statuses, ["passed".to_owned(), "stopped".to_owned()].into());
}

pub(crate) async fn cycle_spanning_group() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let first = plt400_suite();
    let second = plt400_diagnostics_suite();
    register_python_suite_pair(&runtime, first.clone(), second.clone()).await;
    let test_cycle_id = start_cycle(&runtime, first, false).await;
    runtime
        .append_cycle_cases(
            test_cycle_id.clone(),
            materialize_cases("provider:testdb:plt400.diagnostics", &second),
            CommandContext::default(),
        )
        .await
        .unwrap();

    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(
        receipt.status,
        CommandStatus::Accepted,
        "{:?}",
        receipt.rejection
    );

    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    assert_eq!(cycle.cases().count(), 2);
    assert_eq!(cycle.test_executions.len(), 2);
    let suites = cycle
        .test_executions
        .iter()
        .map(|execution| {
            assert_eq!(execution.status, ExecutionStatus::Passed);
            assert!(execution.started_at.is_some());
            execution
                .suite
                .as_ref()
                .expect("attempt carries its suite")
                .identifier
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(suites, vec!["plt400.ract", "plt400.diagnostics"]);

    // The terminal checkpoint seals every group, not only the last one that
    // reported: an earlier group's attempt is published history too.
    for execution in &cycle.test_executions {
        crate::support::native::checkpoint(&runtime, &execution.test_execution_id).await;
    }
}

pub(crate) async fn fail_fast_failure() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let first = plt400_suite_with_result("failed");
    let second = plt400_diagnostics_suite();
    register_python_suite_pair(&runtime, first.clone(), second.clone()).await;
    let test_cycle_id = start_cycle(&runtime, first, false).await;
    runtime
        .append_cycle_cases(
            test_cycle_id.clone(),
            materialize_cases("provider:testdb:plt400.diagnostics", &second),
            CommandContext::default(),
        )
        .await
        .unwrap();

    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(
        receipt.status,
        CommandStatus::Accepted,
        "{:?}",
        receipt.rejection
    );

    let cycle = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Failed).await;
    assert_eq!(cycle.test_executions.len(), 2);
    assert_eq!(cycle.test_executions[0].status, ExecutionStatus::Failed);
    assert_eq!(cycle.test_executions[1].status, ExecutionStatus::Skipped);
    assert!(cycle.test_executions[1].started_at.is_none());
}

/* === Tests === */
