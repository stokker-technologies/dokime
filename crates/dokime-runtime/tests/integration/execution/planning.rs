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

// A cycle is created empty, refuses to run until cases are appended, then
// runs the appended cases to completion.
pub(crate) async fn empty_cycle_grows() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;

    // Created: Ready, Empty, And Visible
    let cycle_id = start_cases(&runtime, metadata("cycle.explicit", "Explicit Cycle"), Vec::new()).await;
    let created = cycle_snapshot(&runtime, &cycle_id).await;
    assert_eq!(created.lifecycle, CycleStatus::Ready);
    assert_eq!(created.cases().count(), 0);
    assert!(created.test_executions.is_empty());
    let query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Cycle,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::Cycles { frame } = query.payload else {
        panic!("expected cycle query frame");
    };
    assert_eq!(frame.count, 1);

    // Empty: Running Is Refused
    let refused = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(refused.status, CommandStatus::Rejected);
    assert_eq!(refused.rejection.as_ref().map(|rejection| rejection.code.as_str()), Some("execution_not_found"));
    assert_eq!(cycle_snapshot(&runtime, &cycle_id).await.lifecycle, CycleStatus::Ready);

    // Appended: Cases Join Unstarted
    let registration = runtime.test_suites().await.into_iter().find(|entry| entry.suite == suite).expect("fixture suite is installed");
    runtime
        .append_fixture_cases(cycle_id.clone(), materialize_cases(&registration.suite_id, &registration.suite), CommandContext::default())
        .await
        .unwrap();
    let appended = cycle_snapshot(&runtime, &cycle_id).await;
    assert_eq!(appended.lifecycle, CycleStatus::Ready);
    assert!(appended.started_at.is_none());
    assert_eq!(appended.cases().count(), 1);
    assert_eq!(appended.test_executions.len(), 1);
    assert_eq!(appended.test_executions[0].status, ExecutionStatus::Skipped);
    assert!(appended.test_executions[0].started_at.is_none());
    assert!(appended.test_executions[0].finished_at.is_none());

    // Run: The Appended Cases Complete
    let accepted = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(accepted.status, CommandStatus::Accepted, "{accepted:?}");
    let completed = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    assert_eq!(completed.test_executions.len(), 1);
    assert_eq!(completed.test_executions[0].status, ExecutionStatus::Passed);
}

// Sealing An Empty Cycle Persists An Empty Report.
pub(crate) async fn empty_cycle_seals() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let cycle_id = start_cases(&runtime, metadata("cycle.unused", "Unused Cycle"), Vec::new()).await;

    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let stopped = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Stopped).await;
    assert!(stopped.test_executions.is_empty());
    assert!(stopped.saved.is_some(), "stopping seals the empty cycle");

    let mut report = runtime.open_cycle_report(&cycle_id).await.unwrap();
    assert_eq!(report.origin, dokime_runtime::ReportOrigin::Sealed);
    let payload = crate::service_support::reports::parse(
        std::str::from_utf8(&crate::service_support::reports::opened_bytes(&mut report).await).unwrap(),
    );
    assert!(payload["test_executions"].as_array().is_none_or(|executions| executions.is_empty()));
}

// A Rerun Of A Sealed Cycle Plans Its Cases Again, Unrun, In Order, And Names Its Origin.
pub(crate) async fn sealed_cycle_reruns() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = rerun_suite(vec![swept_template("rerun.first", "First"), swept_template("rerun.second", "Second")]);
    register_python_suite(&runtime, suite.clone()).await;
    let source = sealed_source(&runtime, &suite).await;
    let planned = rerun_planned(&runtime, &source).await;
    assert_eq!(planned.len(), 4, "two swept values for each of two cases");

    let outcome = runtime.rerun_cycle(source.clone(), None, CommandContext::default()).await.unwrap();
    assert!(outcome.skipped.is_empty());
    assert!(outcome.dropped_parameters.is_empty());
    let rerun = cycle_snapshot(&runtime, &outcome.cycle_id).await;
    assert_eq!(rerun.metadata.label, "Rerun Source");
    assert!(rerun.continue_automatically);
    assert_eq!(rerun.phase, dokime_domain::model::execution::result::CyclePhase::Active);
    assert_eq!(rerun.metadata.attributes.get(dokime_runtime::RERUN_OF_ATTRIBUTE), Some(&Value::from(source.to_string())));
    assert_eq!(rerun.cases().map(|case| (case.test_case_template_id.clone(), case.parameters.clone())).collect::<Vec<_>>(), planned);
    assert_eq!(rerun.test_executions.len(), planned.len());
    assert!(rerun.test_executions.iter().all(|execution| execution.status == ExecutionStatus::Skipped && execution.started_at.is_none()));
    assert_eq!(rerun_planned(&runtime, &source).await, planned, "the source is untouched");
}

// A Case The Installed Plugin No Longer Declares Is Skipped; Undeclared Parameters Are Dropped.
pub(crate) async fn rerun_skips_missing_cases() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let first = case_template_with_identifier("rerun.first", "First", "passed");
    let second = case_template_with_identifier("rerun.second", "Second", "passed");
    let suite = rerun_suite(vec![first.clone(), second]);
    register_python_suite(&runtime, suite.clone()).await;
    let suite_id = installed_suite_id(&runtime, &suite).await;
    let source = sealed_source(&runtime, &suite).await;

    let mut reduced = first;
    reduced.parameters.clear();
    register_python_suite(&runtime, rerun_suite(vec![reduced.clone()])).await;
    let outcome = runtime.rerun_cycle(source, Some("Rerun Again".to_owned()), CommandContext::default()).await.unwrap();
    assert_eq!(
        outcome.skipped.iter().map(|notice| (notice.case_id.as_str(), notice.suite_id.as_str())).collect::<Vec<_>>(),
        [("rerun.second", suite_id.as_str())]
    );
    assert!(outcome.skipped.iter().all(|notice| !notice.reason.is_empty()));
    assert_eq!(
        outcome.dropped_parameters.iter().map(|notice| (notice.case_id.as_str(), notice.suite_id.as_str())).collect::<Vec<_>>(),
        [("rerun.first", suite_id.as_str())]
    );
    assert!(outcome.dropped_parameters[0].reason.contains("fixture.result"));
    let rerun = cycle_snapshot(&runtime, &outcome.cycle_id).await;
    assert_eq!(rerun.metadata.label, "Rerun Again");
    let cases = rerun.cases().collect::<Vec<_>>();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].test_case_template_id, reduced.test_case_template_id);
    assert!(!cases[0].parameters.contains_key("fixture.result"));
}

// A Rerun With No Surviving Case Is Refused, Naming Its Plugin And Cases, And Creates Nothing.
pub(crate) async fn rerun_without_cases_refused() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = rerun_suite(vec![case_template_with_identifier("rerun.retired", "Retired", "passed")]);
    register_python_suite(&runtime, suite.clone()).await;
    let source = sealed_source(&runtime, &suite).await;

    register_python_suite(&runtime, rerun_suite(vec![case_template_with_identifier("rerun.current", "Current", "passed")])).await;
    let cycles = runtime.cycles().await.unwrap().len();
    let refused = runtime.rerun_cycle(source, None, CommandContext::default()).await.unwrap_err();
    assert_eq!(refused.code(), Some("rerun_no_cases"));
    let message = refused.to_string();
    assert!(message.contains("rerun.retired"), "{message}");
    assert!(message.contains("provider:testdb"), "{message}");
    assert_eq!(runtime.cycles().await.unwrap().len(), cycles, "a refused rerun creates no cycle");
}

/* === Private Functions === */

fn rerun_suite(templates: Vec<dokime_runtime::definition::TestCaseTemplate>) -> TestSuite {
    TestSuite {
        metadata: metadata("plt400.ract", "PLT400 RACT"),
        tags: vec!["plt400".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: templates,
    }
}

// A Template Whose Single Parameter Sweeps Two Values, Materializing Two Cases.
fn swept_template(identifier: &str, label: &str) -> dokime_runtime::definition::TestCaseTemplate {
    let mut template = case_template_with_identifier(identifier, label, "passed");
    template.parameters[0].expansion = dokime_runtime::ParameterExpansion::Cartesian;
    template.parameters[0].expression = ParameterExpr::Array {
        values: ["low", "high"].map(|value| ParameterExpr::Literal { value: Value::from(value) }).into(),
    };
    template
}

async fn installed_suite_id(runtime: &RuntimeHandle, suite: &TestSuite) -> String {
    runtime.test_suites().await.into_iter().find(|entry| &entry.suite == suite).expect("fixture suite is installed").suite_id
}

// A Sealed Cycle Named "Rerun Source" Holding Every Case The Suite Materializes.
async fn sealed_source(runtime: &RuntimeHandle, suite: &TestSuite) -> TestCycleId {
    let registration = runtime.test_suites().await.into_iter().find(|entry| &entry.suite == suite).expect("fixture suite is installed");
    let cases = materialize_cases(&registration.suite_id, &registration.suite);
    let source = start_cases(runtime, metadata("cycle.rerun.source", "Rerun Source"), cases).await;
    runtime.seal_cycle(source.clone(), CommandContext::default()).await.unwrap();
    until_state(runtime, |state| state.cycles.get(&source).is_some_and(|cycle| cycle.saved.is_some())).await;
    source
}

// The Source's Planned Cases, In Ordinal Order, As Template And Parameters.
async fn rerun_planned(
    runtime: &RuntimeHandle, source: &TestCycleId,
) -> Vec<(dokime_domain::model::identity::TemplateId, std::collections::BTreeMap<String, Value>)> {
    let inspected = runtime.inspect_cycle(source).await.unwrap();
    let cycle = inspected.state().cycle_view(source).expect("source cycle is readable");
    cycle.cases().map(|case| (case.test_case_template_id.clone(), case.parameters.clone())).collect()
}

/* === Tests === */
