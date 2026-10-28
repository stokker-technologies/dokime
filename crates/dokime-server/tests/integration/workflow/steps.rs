/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;
use dokime::definition::{CriterionDefinition, ExecutionPhaseDefinition};
use dokime::domain::model;
use dokime::{CriterionId, StepId};

#[tokio::test]
async fn step_pages_selections() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    suite.test_case_templates[0].phases = (0..103)
        .map(|index| ExecutionPhaseDefinition {
            repeatable: false,
            step_id: StepId::from_seed(format!("grpc-step-{index}")),
            metadata: support::metadata(&format!("step.{index}"), &format!("Step {index}")),
        })
        .collect();
    suite.test_case_templates[0].criteria = (0..103)
        .map(|index| CriterionDefinition {
            criterion_id: CriterionId::from_seed(format!("grpc-criterion-{index}")),
            metadata: support::metadata(&format!("criterion.{index}"), &format!("Criterion {index}")),
            expected: None,
        })
        .collect();
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
    client.start_first(authenticated(run(&id))).await.unwrap();
    gate.started.notified().await;
    let execution_id = client
        .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions[0]
        .execution_id
        .clone();
    let mut request = contracts::execution::ListExecutionSteps {
        execution_id: execution_id.clone(),
        limit: 31,
        cursor: None,
    };
    assert_eq!(client.unary::<contracts::execution::ListExecutionSteps>(request.clone()).await.unwrap_err().code(), Code::Unauthenticated);
    let first = client.unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone())).await.unwrap().into_inner();
    let initial_cursor = first.next_cursor.clone();
    let mut steps = first.steps;
    request.cursor = first.next_cursor;
    while request.cursor.is_some() {
        let page = client.unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone())).await.unwrap().into_inner();
        assert!(page.steps.len() <= 31);
        steps.extend(page.steps);
        request.cursor = page.next_cursor;
    }
    assert_eq!(steps.len(), 103);
    for (index, (step, definition)) in steps.iter().zip(&suite.test_case_templates[0].phases).enumerate() {
        assert_eq!(step.step_id, definition.step_id.to_string());
        assert_eq!(step.ordinal, index as u64);
    }
    let last = steps.last().unwrap();
    assert_eq!(last.criterion_count, 103);
    assert_eq!(last.criteria.len(), 103, "a step arrives with all of its criteria");
    for (index, criterion) in last.criteria.iter().enumerate() {
        assert_eq!(criterion.ordinal, index as u64);
        assert_eq!(criterion.status, model::execution::wire::CriterionStatus::Pending);
        assert!(criterion.observed.is_none());
    }
    assert!(steps.iter().all(|step| step.criteria.len() as u64 == step.criterion_count));
    let mut wrong_kind = request.clone();
    wrong_kind.cursor = initial_cursor.clone().map(|mut cursor| {
        cursor.kind = model::execution::wire::StepPageKind::Measurements;
        cursor
    });
    assert_eq!(
        client.unary::<contracts::execution::ListExecutionSteps>(authenticated(wrong_kind)).await.unwrap_err().code(),
        Code::InvalidArgument
    );
    request.limit = 501;
    assert_eq!(
        client.unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone())).await.unwrap_err().code(),
        Code::InvalidArgument
    );
    request.limit = 31;
    client.unary::<contracts::execution::CreateCycle>(authenticated(create())).await.unwrap();
    request.cursor = initial_cursor;
    assert_eq!(client.unary::<contracts::execution::ListExecutionSteps>(authenticated(request)).await.unwrap_err().code(), Code::Aborted);
    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        let cycle = client
            .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.clone() }))
            .await
            .unwrap()
            .into_inner();
        if support::cycle_settled(&cycle, model::execution::wire::ResultStatus::Passed) {
            return Some(());
        }
        None
    })
    .await;
    // The fixture reports no phase/criterion outcomes. Native finalization keeps
    // the admitted steps and distinguishes unreported evaluations as indeterminate.
    let first = client
        .unary::<contracts::execution::ListExecutionSteps>(authenticated(contracts::execution::ListExecutionSteps {
            execution_id: execution_id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    let with_criteria = first.steps.iter().find(|step| step.criterion_count > 0).unwrap();
    assert!(!with_criteria.criteria.is_empty());
    assert!(with_criteria.criteria.iter().all(|criterion| criterion.status == model::execution::wire::CriterionStatus::Indeterminate));
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
