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
use dokime_protocol::v1::{document_frame::Frame, documents_client::DocumentsClient};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn running_execution_owners() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        mut execution,
        cycle,
    } = Gated::open().await;
    execution
        .run_cycle(authenticated(run(&cycle)))
        .await
        .unwrap();
    gate.started.notified().await;
    let attempts = execution
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id: cycle.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    let id = attempts.executions[0].execution_id.clone();
    let mut documents = DocumentsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let selected = v1::DocumentScope {
        cycle_id: cycle,
        execution_id: id.clone(),
    };
    let mut stream = documents
        .capture_current_document(authenticated(v1::CaptureCurrentDocumentRequest {
            scope: Some(selected),
            document_id: "default".into(),
        }))
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
            Frame::Card(v1::DocumentCard {
                body: Some(v1::document_card::Body::Owner(reference)),
                ..
            }) => {
                kinds.push(reference.kind);
            }
            Frame::Completion(_) => complete = true,
            _ => {}
        }
    }
    assert!(complete);
    // One document, every card, one fixed order: the result opens it and the
    // evidence closes it. Nothing is selected and nothing is left behind.
    assert_eq!(
        kinds,
        [
            v1::DocumentCardKind::Steps as i32,
            v1::DocumentCardKind::Criteria as i32,
            v1::DocumentCardKind::Records as i32,
            v1::DocumentCardKind::Artifacts as i32,
            v1::DocumentCardKind::Media as i32,
            v1::DocumentCardKind::Traceability as i32,
            v1::DocumentCardKind::Notes as i32,
        ]
    );
    gate.release.notify_one();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
