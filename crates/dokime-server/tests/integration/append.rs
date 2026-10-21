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
use dokime::domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn selection(suite: &dokime::definition::TestSuite, index: usize) -> model::planning::wire::RegisteredSelection {
    model::planning::wire::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[index].test_case_template_id.to_string(),
        ..Default::default()
    }
}

fn plan(selection: model::planning::wire::RegisteredSelection) -> model::planning::wire::SelectionPlan {
    model::planning::wire::SelectionPlan { selections: vec![selection], ..Default::default() }
}

async fn cases(execution: &WireClient, cycle_id: &str) -> Vec<model::execution::wire::CaseSummary> {
    execution
        .unary::<contracts::execution::ListCases>(auth(
            contracts::execution::ListCases { cycle_id: cycle_id.into(), ..Default::default() },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cases
}

async fn client(server: &Server) -> WireClient {
    WireClient::connect(server.local_endpoint()).await.unwrap()
}

#[tokio::test]
async fn append_cases_preserve_ordinals() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = support::fixture_suite(&["passed", "passed"]);
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let execution = client(&server).await;

    // A cycle starts empty and Ready; creating it needs no plan.
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(auth(
            contracts::execution::CreateCycle {
                metadata: Some(model::identity::wire::Metadata { label: "Append cycle".into(), ..Default::default() }),
                ..Default::default()
            },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let created = execution
        .unary::<contracts::execution::GetCycle>(auth(contracts::execution::GetCycle { cycle_id: cycle_id.clone() }, TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.status, model::execution::wire::CycleStatus::Ready);
    assert_eq!(created.case_count, 0);
    // An empty identifier is derived by the server.
    assert!(created.metadata.unwrap().identifier.starts_with("cycle."));
    assert!(cases(&execution, &cycle_id).await.is_empty());

    // Unknown Attempts Fail Without Creating Or Starting Work.
    let empty = execution
        .unary::<contracts::execution::StartExecution>(auth(
            contracts::execution::StartExecution {
                execution_id: uuid::Uuid::now_v7().to_string(),
                mode: model::execution::wire::ExecutionMode::Automatic,
            },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(empty.outcome, Some(model::execution::wire::RunReplyOutcome::Rejected(ref rejection)) if rejection.code == "execution_not_found")
    );

    let appended = execution
        .append_selection(auth(
            support::wire::SelectionAppend { cycle_id: cycle_id.clone(), plan: Some(plan(selection(&suite, 0))), context: None },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    let before = cases(&execution, &cycle_id).await;
    assert_eq!(before.len(), 1);
    assert_eq!(appended.execution_ids, [before[0].latest_execution_id.clone().unwrap()]);

    let request = support::wire::SelectionAppend { cycle_id: cycle_id.clone(), plan: Some(plan(selection(&suite, 1))), context: None };
    let reply = execution.append_selection(auth(request, TOKEN)).await.unwrap().into_inner();
    assert!(reply.observed_cursor.is_some());

    let after = cases(&execution, &cycle_id).await;
    assert_eq!(after.len(), 2);
    assert_eq!(reply.execution_ids, [after[1].latest_execution_id.clone().unwrap()]);
    assert_ne!(reply.execution_ids, appended.execution_ids);
    assert_eq!(after[0].case_id, before[0].case_id);
    assert_eq!(after[0].template_id, before[0].template_id);
    assert_ne!(after[1].template_id, before[0].template_id);

    // Appending a template again continues its numbering within the cycle.
    execution
        .append_selection(auth(
            support::wire::SelectionAppend { cycle_id: cycle_id.clone(), plan: Some(plan(selection(&suite, 0))), context: None },
            TOKEN,
        ))
        .await
        .unwrap();
    let labels = cases(&execution, &cycle_id).await.into_iter().map(|case| case.metadata.unwrap().label).collect::<Vec<_>>();
    let first = &suite.test_case_templates[0].metadata.label;
    let second = &suite.test_case_templates[1].metadata.label;
    assert_eq!(labels, [format!("{first} #1"), format!("{second} #1"), format!("{first} #2")]);

    // Numbering continues from the stored ordinals: renaming "#2" never frees it.
    let renamed = cases(&execution, &cycle_id).await[2].latest_execution_id.clone().unwrap();
    execution
        .unary::<contracts::execution::RenameExecution>(auth(
            contracts::execution::RenameExecution { execution_id: renamed, label: "Bench Warm-Up".into(), context: None },
            TOKEN,
        ))
        .await
        .unwrap();
    execution
        .append_selection(auth(
            support::wire::SelectionAppend { cycle_id: cycle_id.clone(), plan: Some(plan(selection(&suite, 0))), context: None },
            TOKEN,
        ))
        .await
        .unwrap();
    let labels = cases(&execution, &cycle_id).await.into_iter().map(|case| case.metadata.unwrap().label).collect::<Vec<_>>();
    assert_eq!(labels[2..], ["Bench Warm-Up".to_owned(), format!("{first} #3")]);

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */

#[tokio::test]
async fn explicit_append_retries() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = support::fixture_suite(&["passed"]);
    let handle = owner.handle();
    support::register_fixture_execution(&handle, suite.clone()).await;
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let wire = client(&server).await;

    // Materialization Registers Immutable Cases And Never Creates Attempts.
    let count = handle.watch_state().latest().state().executions.len();
    let generated = wire
        .unary::<contracts::planning::MaterializeTemplate>(auth(
            contracts::planning::MaterializeTemplate { selection: Some(selection(&suite, 0)) },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(generated.cases.len(), 1);
    let case = &generated.cases[0];
    assert_eq!(handle.watch_state().latest().state().executions.len(), count);
    let registered =
        handle.watch_state().latest().state().suites.case(&dokime::CaseId::from_wire(case.case_id.clone()).unwrap()).cloned().unwrap();
    assert_eq!(registered.test_case_template_id.to_string(), case.template_id);
    assert_eq!(registered.metadata.label, case.metadata.as_ref().unwrap().label);

    let cycle = wire
        .unary::<contracts::execution::CreateCycle>(auth(
            contracts::execution::CreateCycle {
                metadata: Some(model::identity::wire::Metadata { label: "Explicit Attempts".into(), ..Default::default() }),
                ..Default::default()
            },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let ids = (0..2).map(|_| dokime::model::identity::new_id::<dokime::TestExecutionId>().to_string()).collect::<Vec<_>>();
    let request = contracts::execution::AppendExecutions {
        cycle_id: cycle.clone(),
        executions: ids
            .iter()
            .map(|id| contracts::execution::ExecutionBinding { execution_id: id.clone(), case_id: case.case_id.clone() })
            .collect(),
    };

    // Concurrent Retries Keep The Supplied Identities And Original Order.
    let first = wire.unary::<contracts::execution::AppendExecutions>(auth(request.clone(), TOKEN));
    let retry = wire.unary::<contracts::execution::AppendExecutions>(auth(request.clone(), TOKEN));
    let (first, retry) = tokio::join!(first, retry);
    assert_eq!(first.unwrap().into_inner().execution_ids, ids);
    assert_eq!(retry.unwrap().into_inner().execution_ids, ids);
    let members = wire
        .unary::<contracts::execution::ListExecutions>(auth(
            contracts::execution::ListExecutions { cycle_id: cycle.clone(), limit: 100, ..Default::default() },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .executions;
    assert_eq!(members.iter().map(|row| row.execution_id.clone()).collect::<Vec<_>>(), ids);
    assert!(handle.watch_state().latest().state().run_requests.is_empty());

    // Invalid Mixed Appends Are Atomic; Reordering Existing IDs Is A Conflict.
    let mut reordered = request.clone();
    reordered.executions.reverse();
    assert!(wire.unary::<contracts::execution::AppendExecutions>(auth(reordered, TOKEN)).await.is_err());
    let mut invalid = request.clone();
    invalid.executions[0].execution_id = dokime::model::identity::new_id::<dokime::TestExecutionId>().to_string();
    invalid.executions[0].case_id = dokime::model::identity::new_id::<dokime::CaseId>().to_string();
    assert!(wire.unary::<contracts::execution::AppendExecutions>(auth(invalid, TOKEN)).await.is_err());
    let oversized = contracts::execution::AppendExecutions {
        cycle_id: cycle.clone(),
        executions: (0..1001)
            .map(|_| contracts::execution::ExecutionBinding {
                execution_id: uuid::Uuid::now_v7().to_string(),
                case_id: case.case_id.clone(),
            })
            .collect(),
    };
    let error = wire.unary::<contracts::execution::AppendExecutions>(auth(oversized, TOKEN)).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::ResourceExhausted);
    assert_eq!(
        handle
            .watch_state()
            .latest()
            .state()
            .cycle_view(&dokime::TestCycleId::from_uuid(cycle.clone()).unwrap())
            .unwrap()
            .test_executions
            .len(),
        2
    );

    // A Retired Attempt Cannot Be Rebound To Another Active Container.
    wire.unary::<contracts::execution::SealCycle>(auth(contracts::execution::SealCycle { cycle_id: cycle, context: None }, TOKEN))
        .await
        .unwrap();
    let other = wire
        .unary::<contracts::execution::CreateCycle>(auth(
            contracts::execution::CreateCycle {
                metadata: Some(model::identity::wire::Metadata { label: "Other Container".into(), ..Default::default() }),
                ..Default::default()
            },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let mut reuse = request;
    reuse.cycle_id = other.clone();
    assert!(wire.unary::<contracts::execution::AppendExecutions>(auth(reuse, TOKEN)).await.is_err());
    assert!(cases(&wire, &other).await.is_empty());

    // Changing A Definition Cannot Silently Reinterpret An Earlier Materialized Case.
    let mut changed = suite.clone();
    changed.test_case_templates[0].metadata.description = Some("Changed After Materialization".into());
    handle.set_suites("provider:fixture", dokime::definition::SuiteSet::new([support::fixture_registration(changed)])).await.unwrap();
    let stale = contracts::execution::AppendExecutions {
        cycle_id: other.clone(),
        executions: vec![contracts::execution::ExecutionBinding {
            execution_id: dokime::model::identity::new_id::<dokime::TestExecutionId>().to_string(),
            case_id: case.case_id.clone(),
        }],
    };
    let error = wire.unary::<contracts::execution::AppendExecutions>(auth(stale, TOKEN)).await.unwrap_err();
    assert!(error.message().contains("definition or provider changed"), "{error}");
    assert!(cases(&wire, &other).await.is_empty());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
