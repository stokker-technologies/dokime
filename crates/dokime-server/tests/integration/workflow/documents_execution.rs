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
use dokime_client::contracts::documents::DocumentFrameFrame as Frame;

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
async fn running_execution_owners() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        service: _service,
        server,
        execution,
        cycle,
    } = Gated::open().await;
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle)))
        .await
        .unwrap();
    gate.started.notified().await;
    let attempts = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutions {
                cycle_id: cycle.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let id = attempts.executions[0].execution_id.clone();
    let documents = WireClient::connect(server.local_endpoint()).await.unwrap();
    let selected = contracts::documents::DocumentScope {
        cycle_id: cycle,
        execution_id: id.clone(),
    };
    let mut stream = documents
        .download::<contracts::documents::CaptureCurrentDocument>(authenticated(
            contracts::documents::CaptureCurrentDocument {
                scope: Some(selected),
                document_id: "default".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut kinds = Vec::new();
    let mut complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.frame.unwrap() {
            Frame::Header(value) => assert_eq!(
                value.scope.unwrap().execution_id.as_deref(),
                Some(id.as_str())
            ),
            Frame::Card(contracts::documents::DocumentCard {
                body: Some(contracts::documents::DocumentCardBody::Owner(reference)),
                ..
            }) => {
                kinds.push(reference.kind);
            }
            Frame::Completion(_) => complete = true,
            _ => {}
        }
    }
    assert!(complete);
    // One document, every card, one fixed order: the result and the operator's
    // notes open it and the evidence closes it. Nothing is selected and nothing is left behind.
    assert_eq!(
        kinds,
        [
            contracts::documents::DocumentCardKind::Steps,
            contracts::documents::DocumentCardKind::Criteria,
            contracts::documents::DocumentCardKind::Notes,
            contracts::documents::DocumentCardKind::Records,
            contracts::documents::DocumentCardKind::Artifacts,
            contracts::documents::DocumentCardKind::Media,
            contracts::documents::DocumentCardKind::Traceability,
        ]
    );
    gate.release.notify_one();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
