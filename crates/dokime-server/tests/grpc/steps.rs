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
use dokime_app::core::{CriterionId, StepId};
use dokime_app::definition::{CriterionDefinition, ExecutionPhaseDefinition};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn step_pages_selections() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = common::fixture_suite(&["passed"]);
    suite.test_case_templates[0].phases = (0..103)
        .map(|index| ExecutionPhaseDefinition {
            repeatable: false,
            step_id: StepId::from_seed(format!("grpc-step-{index}")),
            metadata: common::metadata(&format!("step.{index}"), &format!("Step {index}")),
        })
        .collect();
    suite.test_case_templates[0].criteria = (0..103)
        .map(|index| CriterionDefinition {
            criterion_id: CriterionId::from_seed(format!("grpc-criterion-{index}")),
            metadata: common::metadata(
                &format!("criterion.{index}"),
                &format!("Criterion {index}"),
            ),
            expected: None,
        })
        .collect();
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = client
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    client
        .unary::<contracts::execution::RunCycle>(authenticated(run(&id)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), gate.started.notified())
        .await
        .unwrap();
    let execution_id = client
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .executions[0]
        .execution_id
        .clone();
    let mut request = contracts::execution::ListExecutionStepsRequest {
        execution_id: execution_id.clone(),
        limit: 31,
        cursor: None,
    };
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionSteps>(request.clone())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let first = client
        .unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone()))
        .await
        .unwrap()
        .into_inner();
    let initial_cursor = first.next_cursor.clone();
    let mut steps = first.steps;
    request.cursor = first.next_cursor;
    while request.cursor.is_some() {
        let page = client
            .unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone()))
            .await
            .unwrap()
            .into_inner();
        assert!(page.steps.len() <= 31);
        steps.extend(page.steps);
        request.cursor = page.next_cursor;
    }
    assert_eq!(steps.len(), 103);
    for (index, (step, definition)) in steps
        .iter()
        .zip(&suite.test_case_templates[0].phases)
        .enumerate()
    {
        assert_eq!(step.step_id, definition.step_id.to_string());
        assert_eq!(step.ordinal, index as u64);
    }
    let last = steps.last().unwrap();
    assert_eq!(last.criterion_count, 103);
    let mut evidence = contracts::execution::ListStepEvidenceRequest {
        execution_id: execution_id.clone(),
        step_id: last.step_id.clone(),
        limit: 37,
        cursor: None,
    };
    let first_criteria = client
        .unary::<contracts::execution::ListStepCriteria>(authenticated(evidence.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first_criteria.criteria.len(), 37);
    assert!(
        first_criteria
            .criteria
            .iter()
            .all(
                |criterion| criterion.status == contracts::execution::CriterionStatus::Pending
                    && criterion.observed.is_none()
            )
    );
    evidence.cursor = first_criteria.next_cursor;
    let mut wrong_kind = evidence.clone();
    wrong_kind.cursor.as_mut().unwrap().kind = contracts::execution::StepPageKind::Steps;
    assert_eq!(
        client
            .unary::<contracts::execution::ListStepCriteria>(authenticated(wrong_kind))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut wrong_step = evidence.clone();
    wrong_step.step_id = steps[0].step_id.clone();
    assert_eq!(
        client
            .unary::<contracts::execution::ListStepCriteria>(authenticated(wrong_step))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut criteria = first_criteria.criteria;
    while evidence.cursor.is_some() {
        let page = client
            .unary::<contracts::execution::ListStepCriteria>(authenticated(evidence.clone()))
            .await
            .unwrap()
            .into_inner();
        criteria.extend(page.criteria);
        evidence.cursor = page.next_cursor;
    }
    assert_eq!(criteria.len(), 103);
    for (index, criterion) in criteria.iter().enumerate() {
        assert_eq!(criterion.ordinal, index as u64);
    }
    evidence.step_id = "missing".into();
    assert_eq!(
        client
            .unary::<contracts::execution::ListStepCriteria>(authenticated(evidence))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    request.limit = 101;
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionSteps>(authenticated(request.clone()))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    request.limit = 31;
    client
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap();
    request.cursor = initial_cursor;
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionSteps>(authenticated(request))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    gate.release.notify_one();
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let cycle = client
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if cycle.status == contracts::execution::CycleStatus::Completed {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    // The fixture reports no phase/criterion outcomes. Native finalization keeps
    // the admitted steps and distinguishes unreported evaluations as indeterminate.
    let first = client
        .unary::<contracts::execution::ListExecutionSteps>(authenticated(
            contracts::execution::ListExecutionStepsRequest {
                execution_id: execution_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let with_criteria = first
        .steps
        .iter()
        .find(|step| step.criterion_count > 0)
        .unwrap();
    let terminal = client
        .unary::<contracts::execution::ListStepCriteria>(authenticated(
            contracts::execution::ListStepEvidenceRequest {
                execution_id,
                step_id: with_criteria.step_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(
        terminal
            .criteria
            .iter()
            .all(|criterion| criterion.status
                == contracts::execution::CriterionStatus::Indeterminate)
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
