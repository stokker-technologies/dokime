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

pub(crate) async fn cycle_edits_state() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        execution: mut client,
        cycle: id,
    } = Gated::open().await;
    let cases = client
        .list_cases(authenticated(v1::ListCasesRequest {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases;
    let case = v1::CaseControlRequest {
        cycle_id: id.clone(),
        case_id: cases[0].case_id.clone(),
        context: None,
    };
    assert_eq!(
        client.delete_case(case.clone()).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    client
        .update_cycle_policy(authenticated(v1::UpdateCyclePolicyRequest {
            cycle_id: id.clone(),
            continue_on_failure: true,
            context: None,
        }))
        .await
        .unwrap();
    assert!(
        client
            .get_cycle(authenticated(v1::GetCycleRequest {
                cycle_id: id.clone()
            }))
            .await
            .unwrap()
            .into_inner()
            .continue_on_failure
    );
    client
        .run_case(authenticated(v1::RunCaseRequest {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
            context: None,
            idempotency_key: None,
        }))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let error = client
        .clear_cycle(authenticated(control(&id)))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::FailedPrecondition);
    let rejection = tonic_types::Status::decode(error.details()).unwrap();
    assert_eq!(
        rejection.details[0].type_url,
        "type.googleapis.com/dokime.v1.Rejection"
    );
    gate.release.notify_one();
    completed(&mut client, &id).await;
    let first = attempts(&mut client, &id).await;
    assert_eq!(first.len(), 1);
    assert_eq!(
        client
            .delete_case(authenticated(case.clone()))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    client
        .schedule_case_rerun(authenticated(case.clone()))
        .await
        .unwrap();
    let planned = attempts(&mut client, &id).await;
    assert_eq!(planned.len(), 2);
    assert_eq!(planned[0], first[0]);
    assert_ne!(planned[1].execution_id, first[0].execution_id);
    assert_eq!(planned[1].status(), v1::ExecutionStatus::Skipped);
    client
        .run_case(authenticated(v1::RunCaseRequest {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
            context: None,
            idempotency_key: None,
        }))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    gate.release.notify_one();
    completed(&mut client, &id).await;
    let second = attempts(&mut client, &id).await;
    assert_eq!(second.len(), 2);
    assert_eq!(second[0], first[0]);
    assert_ne!(second[0].execution_id, second[1].execution_id);
    client
        .schedule_case_rerun(authenticated(case.clone()))
        .await
        .unwrap();
    let retained = attempts(&mut client, &id).await;
    assert_eq!(retained.len(), 3);
    assert_eq!(&retained[..2], &second[..]);
    assert_eq!(retained[2].case_id, case.case_id);
    assert_eq!(retained[2].status(), v1::ExecutionStatus::Skipped);
    assert!(
        second
            .iter()
            .all(|previous| previous.execution_id != retained[2].execution_id)
    );
    assert_eq!(
        client
            .delete_case(authenticated(case))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    client
        .clear_cycle(authenticated(control(&id)))
        .await
        .unwrap();
    let cleared = client
        .get_cycle(authenticated(v1::GetCycleRequest {
            cycle_id: id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(cleared.status(), v1::CycleStatus::Cleared);
    assert_eq!(cleared.case_count, 0);
    assert_eq!(
        client
            .update_cycle_policy(authenticated(v1::UpdateCyclePolicyRequest {
                cycle_id: id,
                continue_on_failure: false,
                context: None
            }))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

async fn attempts(
    client: &mut ExecutionClient<tonic::transport::Channel>,
    id: &str,
) -> Vec<v1::ExecutionSummary> {
    client
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id: id.into(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions
}

async fn completed(client: &mut ExecutionClient<tonic::transport::Channel>, id: &str) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if client
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: id.into(),
                }))
                .await
                .unwrap()
                .into_inner()
                .status()
                == v1::CycleStatus::Completed
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

/* === Tests === */
