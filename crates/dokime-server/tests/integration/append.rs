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
use tonic::Code;
// Local Imports
use super::*;

/* === Constants === */
const CONTROL_TOKEN: &str = "append-control-token-012345678901234567890";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn selection(
    suite: &dokime_runtime::definition::TestSuite,
    index: usize,
) -> contracts::planning::RegisteredSelection {
    contracts::planning::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[index]
            .test_case_template_id
            .to_string(),
        ..Default::default()
    }
}

fn plan(selection: contracts::planning::RegisteredSelection) -> contracts::planning::SelectionPlan {
    contracts::planning::SelectionPlan {
        selections: vec![selection],
        ..Default::default()
    }
}

async fn cases(execution: &WireClient, cycle_id: &str) -> Vec<contracts::execution::CaseSummary> {
    execution
        .unary::<contracts::execution::ListCases>(auth(
            contracts::execution::ListCasesRequest {
                cycle_id: cycle_id.into(),
                ..Default::default()
            },
            CONTROL_TOKEN,
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
async fn append_cases_control() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = support::fixture_suite(&["passed", "passed"]);
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![
                grant(Role::Read),
                BearerGrant::new("append-control".into(), Role::Control, CONTROL_TOKEN).unwrap(),
            ])
            .unwrap(),
    )
    .await
    .unwrap();
    let execution = client(&server).await;

    // A cycle starts empty and Ready; creating it needs no plan.
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(auth(
            contracts::execution::CreateCycleRequest {
                metadata: Some(contracts::common::Metadata {
                    label: "Append cycle".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let created = execution
        .unary::<contracts::execution::GetCycle>(auth(
            contracts::execution::GetCycleRequest {
                cycle_id: cycle_id.clone(),
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.status, contracts::execution::CycleStatus::Ready);
    assert_eq!(created.case_count, 0);
    // An empty identifier is derived by the server.
    assert!(created.metadata.unwrap().identifier.starts_with("cycle."));
    assert!(cases(&execution, &cycle_id).await.is_empty());

    // Running an empty cycle is refused until cases are appended.
    let empty = execution
        .unary::<contracts::execution::RunCycle>(auth(
            contracts::execution::RunCycleRequest {
                cycle_id: cycle_id.clone(),
                context: None,
                idempotency_key: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        empty.outcome,
        Some(contracts::execution::RunReplyOutcome::Rejected(ref rejection))
            if rejection.code == "empty_cycle"
    ));

    execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(
            contracts::execution::AppendPlannedCasesRequest {
                cycle_id: cycle_id.clone(),
                plan: Some(plan(selection(&suite, 0))),
                context: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap();
    let before = cases(&execution, &cycle_id).await;
    assert_eq!(before.len(), 1);

    let request = contracts::execution::AppendPlannedCasesRequest {
        cycle_id: cycle_id.clone(),
        plan: Some(plan(selection(&suite, 1))),
        context: None,
    };
    assert_eq!(
        execution
            .unary::<contracts::execution::AppendPlannedCases>(auth(request.clone(), super::TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let reply = execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(request, CONTROL_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert!(reply.observed_cursor.is_some());

    let after = cases(&execution, &cycle_id).await;
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].case_id, before[0].case_id);
    assert_eq!(after[0].template_id, before[0].template_id);
    assert_ne!(after[1].template_id, before[0].template_id);

    // Appending a template again continues its numbering within the cycle.
    execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(
            contracts::execution::AppendPlannedCasesRequest {
                cycle_id: cycle_id.clone(),
                plan: Some(plan(selection(&suite, 0))),
                context: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap();
    let labels = cases(&execution, &cycle_id)
        .await
        .into_iter()
        .map(|case| case.metadata.unwrap().label)
        .collect::<Vec<_>>();
    let first = &suite.test_case_templates[0].metadata.label;
    let second = &suite.test_case_templates[1].metadata.label;
    assert_eq!(
        labels,
        [
            format!("{first} #1"),
            format!("{second} #1"),
            format!("{first} #2")
        ]
    );

    // Numbering continues from the stored ordinals: renaming "#2" never frees it.
    let renamed = cases(&execution, &cycle_id).await[2]
        .latest_execution_id
        .clone()
        .unwrap();
    execution
        .unary::<contracts::execution::RenameExecution>(auth(
            contracts::execution::RenameExecutionRequest {
                execution_id: renamed,
                label: "Bench Warm-Up".into(),
                context: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap();
    execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(
            contracts::execution::AppendPlannedCasesRequest {
                cycle_id: cycle_id.clone(),
                plan: Some(plan(selection(&suite, 0))),
                context: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap();
    let labels = cases(&execution, &cycle_id)
        .await
        .into_iter()
        .map(|case| case.metadata.unwrap().label)
        .collect::<Vec<_>>();
    assert_eq!(
        labels[2..],
        ["Bench Warm-Up".to_owned(), format!("{first} #3")]
    );

    let backend = WireClient::connect(server.local_endpoint()).await.unwrap();
    let read_methods = backend
        .unary::<contracts::backend::GetAvailability>(auth(
            contracts::backend::GetAvailabilityRequest::default(),
            super::TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(!read_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));
    let control_methods = backend
        .unary::<contracts::backend::GetAvailability>(auth(
            contracts::backend::GetAvailabilityRequest::default(),
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(control_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
