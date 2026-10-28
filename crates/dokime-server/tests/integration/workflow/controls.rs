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

#[tokio::test]
async fn cycle_edits_state() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        service: _service,
        server,
        execution: mut client,
        cycle: id,
    } = Gated::open().await;
    let cases = client
        .unary::<contracts::execution::ListCases>(authenticated(contracts::execution::ListCases {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases;
    let case = contracts::execution::DeleteCase {
        cycle_id: id.clone(),
        case_id: cases[0].case_id.clone(),
        context: None,
    };
    assert_eq!(client.unary::<contracts::execution::DeleteCase>(case.clone()).await.unwrap_err().code(), Code::Unauthenticated);
    client
        .unary::<contracts::execution::UpdateCyclePolicy>(authenticated(contracts::execution::UpdateCyclePolicy {
            cycle_id: id.clone(),
            continue_automatically: false,
            context: None,
        }))
        .await
        .unwrap();
    assert!(
        !client
            .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.clone() }))
            .await
            .unwrap()
            .into_inner()
            .continue_automatically
    );
    client
        .start_case(authenticated(support::wire::CaseSelection {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
            context: None,
        }))
        .await
        .unwrap();
    gate.started.notified().await;
    let error = client.unary::<contracts::execution::SetCycleArchived>(authenticated(archive(&id, true))).await.unwrap_err();
    assert_eq!(error.code(), Code::FailedPrecondition);
    let rejection = tonic_types::Status::decode(error.details()).unwrap();
    assert_eq!(rejection.details[0].type_url, "type.googleapis.com/dokime.v1.Rejection");
    gate.release.notify_one();
    completed(&owner.handle(), &mut client, &id).await;
    let first = attempts(&mut client, &id).await;
    assert_eq!(first.len(), 1);
    assert_eq!(
        client.unary::<contracts::execution::DeleteCase>(authenticated(case.clone())).await.unwrap_err().code(),
        Code::FailedPrecondition
    );
    client
        .unary::<contracts::execution::AppendExecutions>(authenticated(contracts::execution::AppendExecutions {
            cycle_id: case.cycle_id.clone(),
            executions: vec![contracts::execution::ExecutionBinding {
                execution_id: uuid::Uuid::now_v7().to_string(),
                case_id: case.case_id.clone(),
            }],
        }))
        .await
        .unwrap();
    let planned = attempts(&mut client, &id).await;
    assert_eq!(planned.len(), 2);
    assert_eq!(planned[0], first[0]);
    assert_ne!(planned[1].execution_id, first[0].execution_id);
    assert_eq!(planned[1].status, model::execution::wire::ExecutionStatus::Skipped);
    client
        .start_case(authenticated(support::wire::CaseSelection {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
            context: None,
        }))
        .await
        .unwrap();
    gate.started.notified().await;
    gate.release.notify_one();
    completed(&owner.handle(), &mut client, &id).await;
    let second = attempts(&mut client, &id).await;
    assert_eq!(second.len(), 2);
    assert_eq!(second[0], first[0]);
    assert_ne!(second[0].execution_id, second[1].execution_id);

    client
        .unary::<contracts::execution::AppendExecutions>(authenticated(contracts::execution::AppendExecutions {
            cycle_id: case.cycle_id.clone(),
            executions: vec![contracts::execution::ExecutionBinding {
                execution_id: uuid::Uuid::now_v7().to_string(),
                case_id: case.case_id.clone(),
            }],
        }))
        .await
        .unwrap();
    let retained = attempts(&mut client, &id).await;
    assert_eq!(retained.len(), 3);
    assert_eq!(&retained[..2], &second[..]);
    assert_eq!(retained[2].case_id, case.case_id);
    assert_eq!(retained[2].status, model::execution::wire::ExecutionStatus::Skipped);
    assert!(second.iter().all(|previous| previous.execution_id != retained[2].execution_id));
    assert_eq!(client.unary::<contracts::execution::DeleteCase>(authenticated(case)).await.unwrap_err().code(), Code::FailedPrecondition);
    // A Re-Run Makes The Cycle Ready Again, Which Is Not Terminal
    assert_eq!(
        client.unary::<contracts::execution::SetCycleArchived>(authenticated(archive(&id, true))).await.unwrap_err().code(),
        Code::FailedPrecondition
    );
    client
        .start_case(authenticated(support::wire::CaseSelection {
            cycle_id: id.clone(),
            case_id: cases[0].case_id.clone(),
            context: None,
        }))
        .await
        .unwrap();
    gate.started.notified().await;
    gate.release.notify_one();
    completed(&owner.handle(), &mut client, &id).await;
    client
        .unary::<contracts::execution::SealCycle>(authenticated(contracts::execution::SealCycle { cycle_id: id.clone(), context: None }))
        .await
        .unwrap();
    // A Completed Cycle Archives: Hidden From The Default List, Shown When Asked
    client.unary::<contracts::execution::SetCycleArchived>(authenticated(archive(&id, true))).await.unwrap();
    let archived = client
        .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert!(archived.archived);
    assert_eq!(archived.status, model::execution::wire::CycleStatus::Completed);
    assert!(!listed(&mut client, &id, None).await);
    assert!(listed(&mut client, &id, Some(model::execution::wire::ArchiveScope::Archived)).await);
    client
        .unary::<contracts::execution::SetExecutionArchived>(authenticated(contracts::execution::SetExecutionArchived {
            execution_id: second[1].execution_id.clone(),
            archived: true,
            context: None,
        }))
        .await
        .unwrap();
    assert!(attempts(&mut client, &id).await[1].archived);
    client
        .unary::<contracts::execution::SetExecutionArchived>(authenticated(contracts::execution::SetExecutionArchived {
            execution_id: second[1].execution_id.clone(),
            archived: false,
            context: None,
        }))
        .await
        .unwrap();
    client.unary::<contracts::execution::SetCycleArchived>(authenticated(archive(&id, false))).await.unwrap();
    assert!(listed(&mut client, &id, None).await);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn labels_rename_state() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        service: _service,
        server,
        execution: mut client,
        cycle: id,
    } = Gated::open().await;
    let cases = client
        .unary::<contracts::execution::ListCases>(authenticated(contracts::execution::ListCases {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases;
    let case_id = cases[0].case_id.clone();
    let rename = contracts::execution::RenameCycle {
        cycle_id: id.clone(),
        label: "  Charger Burn-In ".into(),
        context: None,
    };
    assert_eq!(client.unary::<contracts::execution::RenameCycle>(rename.clone()).await.unwrap_err().code(), Code::Unauthenticated);

    // A Blank Or Oversized Label Is A Bad Argument
    for label in [String::from(" \t "), "x".repeat(4097)] {
        assert_eq!(
            client
                .unary::<contracts::execution::RenameCycle>(authenticated(contracts::execution::RenameCycle { label, ..rename.clone() }))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }

    // Before The Run Seals, The Cycle And Its Execution Take Their New Labels
    let reply = client.unary::<contracts::execution::RenameCycle>(authenticated(rename)).await.unwrap().into_inner();
    assert_eq!(reply.label, "Charger Burn-In");
    let execution_id = attempts(&mut client, &id).await[0].execution_id.clone();
    let execution = contracts::execution::RenameExecution {
        execution_id,
        label: "Charger Soak".into(),
        context: None,
    };
    let reply = client.unary::<contracts::execution::RenameExecution>(authenticated(execution.clone())).await.unwrap().into_inner();
    assert_eq!(reply.label, "Charger Soak");
    let cycle = client
        .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(cycle.metadata.unwrap().label, "Charger Burn-In");
    assert!(!cycle.sealed);
    let cases = client
        .unary::<contracts::execution::ListCases>(authenticated(contracts::execution::ListCases {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases;
    assert_eq!(cases[0].metadata.as_ref().unwrap().label, "Charger Soak");

    // Complete The Execution, Then Seal Its Cycle Explicitly.
    client
        .start_case(authenticated(support::wire::CaseSelection {
            cycle_id: id.clone(),
            case_id: case_id.clone(),
            context: None,
        }))
        .await
        .unwrap();
    gate.started.notified().await;
    gate.release.notify_one();
    completed(&owner.handle(), &mut client, &id).await;
    client
        .unary::<contracts::execution::SealCycle>(authenticated(contracts::execution::SealCycle { cycle_id: id.clone(), context: None }))
        .await
        .unwrap();
    let sealed = client
        .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert!(sealed.sealed);
    assert!(attempts(&mut client, &id).await[0].sealed);
    let late = client
        .unary::<contracts::execution::RenameCycle>(authenticated(contracts::execution::RenameCycle {
            cycle_id: id.clone(),
            label: "Too Late".into(),
            context: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(late.code(), Code::FailedPrecondition);
    assert!(late.message().contains("sealed"), "{}", late.message());
    assert_eq!(
        client.unary::<contracts::execution::RenameExecution>(authenticated(execution)).await.unwrap_err().code(),
        Code::FailedPrecondition
    );
    assert_eq!(
        client
            .unary::<contracts::execution::RenameExecution>(authenticated(contracts::execution::RenameExecution {
                execution_id: uuid::Uuid::now_v7().to_string(),
                label: "Charger Soak".into(),
                context: None,
            },))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

fn archive(id: &str, archived: bool) -> contracts::execution::SetCycleArchived {
    contracts::execution::SetCycleArchived {
        cycle_id: id.into(),
        archived,
        context: None,
    }
}

async fn listed(client: &mut WireClient, id: &str, archive: Option<model::execution::wire::ArchiveScope>) -> bool {
    client
        .unary::<contracts::execution::ListCycles>(authenticated(contracts::execution::ListCycles {
            filter: Some(model::execution::wire::CycleFilter { archive, ..Default::default() }),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cycles
        .iter()
        .any(|cycle| cycle.cycle_id == id)
}

async fn attempts(client: &mut WireClient, id: &str) -> Vec<model::execution::wire::ExecutionSummary> {
    client
        .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
            cycle_id: id.into(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions
}

async fn completed(handle: &dokime::RuntimeHandle, client: &mut WireClient, id: &str) {
    support::on_state_change(handle, async || {
        if support::cycle_settled(
            &client
                .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: id.into() }))
                .await
                .unwrap()
                .into_inner(),
            model::execution::wire::ResultStatus::Passed,
        ) {
            return Some(());
        }
        None
    })
    .await;
}

/* === Tests === */
