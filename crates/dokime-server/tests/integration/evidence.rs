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
use crate::support::wire::WireMessage;
use crate::{WireClient, contracts};
use dokime::domain::model;
use dokime_server::grpc::{Config, Server};
use {
    dokime::Identifier, dokime::Metadata, dokime::Runtime, dokime::RuntimeConfig, dokime::RuntimeHandle,
    model::records::product::ProductCalibrationDetails, model::records::product::ProductCalibrationKind,
    model::records::product::ProductCalibrationPhase,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn start(root: &std::path::Path) -> (Runtime, Server, WireClient) {
    start_bounded(root, 1024 * 1024).await
}

async fn start_bounded(root: &std::path::Path, maximum: usize) -> (Runtime, Server, WireClient) {
    let config = crate::support::budgets::runtime_config(
        root,
        Metadata::new(Identifier::new("evidence.fixture").unwrap(), "Evidence fixture", None),
    );
    start_config(config, maximum).await
}

async fn start_config(config: RuntimeConfig, maximum: usize) -> (Runtime, Server, WireClient) {
    let owner = Runtime::start(config).await.unwrap();
    let server = Server::start(owner.handle(), Config::default().with_max_message_bytes(maximum)).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    (owner, server, client)
}

async fn publish(handle: &RuntimeHandle) {
    handle
        .publish_product_record(
            ProductCalibrationDetails::new("fixture-device", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture.product"),
        )
        .await
        .unwrap();
}

fn selection() -> model::evidence::wire::CapturedRecordSelection {
    model::evidence::wire::CapturedRecordSelection {
        record_types: vec!["calibration.field.complet?d".into(), "calibration.field.*".into(), "missing.*".into()],
        ..Default::default()
    }
}

#[tokio::test]
async fn fixed_pages_gap() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, server, client) = start(root.path()).await;
    let handle = owner.handle();
    for _ in 0..1005 {
        publish(&handle).await;
    }
    let first = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.records.len(), 1000);
    assert!(!first.replay_complete);
    publish(&handle).await;
    let second = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            after: first.scanned,
            through: first.through.clone(),
            latest: 0,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.records.len(), 5);
    assert!(second.replay_complete);
    assert_eq!(second.through, first.through);
    assert!(
        first
            .records
            .iter()
            .chain(&second.records)
            .map(|record| record.sequence)
            .collect::<Vec<_>>()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    let mut follow = client
        .download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords {
            selection: Some(selection()),
            after: second.scanned,
        })
        .await
        .unwrap()
        .into_inner();
    let initial = follow.message().await.unwrap().unwrap();
    assert_eq!(initial.records.len(), 1);
    let record = &initial.records[0];
    assert!(!record.record_id.is_empty());
    assert_eq!(record.schema_version, 2);
    assert_eq!(record.severity, model::identity::wire::Severity::Info);
    assert!(record.timestamp.is_some());
    let schema = client
        .unary::<contracts::evidence::GetCapturedRecordSchema>(contracts::evidence::GetCapturedRecordSchema {
            record_type: record.record_type.clone(),
            schema_version: record.schema_version,
            epoch: initial.scanned.as_ref().unwrap().epoch.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(schema.schema_version, record.schema_version);
    publish(&handle).await;
    let next = loop {
        let frame = follow.message().await.unwrap().unwrap();
        if !frame.records.is_empty() {
            break frame;
        }
    };
    assert_eq!(next.records.len(), 1);
    assert!(next.records[0].sequence > record.sequence);
    drop(follow);
    publish(&handle).await;
    drop(client);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn tail_then_follow() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, server, client) = start(root.path()).await;
    let handle = owner.handle();
    for _ in 0..7 {
        publish(&handle).await;
    }
    let full = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(full.records.len(), 7);
    let tail = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            latest: 3,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    let ids = |page: &[model::evidence::wire::CapturedRecord]| page.iter().map(|record| record.record_id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&tail.records), ids(&full.records[4..]), "the tail holds the newest records, oldest first");
    assert!(tail.replay_complete);
    assert_eq!(tail.scanned, tail.through);
    let frontier = tail.through.clone().unwrap();
    assert_eq!(frontier.position, handle.captured_records().frontier());
    assert_eq!(Some(&frontier), full.through.as_ref());

    for (after, through) in [(tail.scanned.clone(), tail.through.clone()), (tail.scanned.clone(), None)] {
        let error = client
            .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
                selection: Some(selection()),
                after,
                through,
                latest: 3,
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
    }

    let mut follow = client
        .download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords {
            selection: Some(selection()),
            after: Some(frontier),
        })
        .await
        .unwrap()
        .into_inner();
    publish(&handle).await;
    let next = loop {
        let frame = follow.message().await.unwrap().unwrap();
        if !frame.records.is_empty() {
            break frame;
        }
    };
    assert_eq!(next.records.len(), 1);
    assert!(next.records[0].sequence > tail.records[2].sequence);
    assert!(!ids(&full.records).contains(&next.records[0].record_id));
    drop(follow);
    drop(client);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn filtered_scans_resync() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, server, client) = start(root.path()).await;
    let handle = owner.handle();
    for _ in 0..1005 {
        publish(&handle).await;
    }
    let selection = model::evidence::wire::CapturedRecordSelection {
        sources: vec!["absent".into()],
        ..Default::default()
    };
    let first = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection.clone()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(first.records.is_empty());
    assert!(first.scanned.as_ref().unwrap().position > 0);
    assert!(!first.replay_complete);
    let second = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection.clone()),
            after: first.scanned.clone(),
            through: first.through.clone(),
            latest: 0,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(second.records.is_empty());
    assert!(second.replay_complete);
    assert!(second.scanned.as_ref().unwrap().position > first.scanned.as_ref().unwrap().position);
    for change_epoch in [true, false] {
        let mut cursor = second.scanned.clone().unwrap();
        if change_epoch {
            cursor.epoch = uuid::Uuid::now_v7().to_string();
        } else {
            cursor.stream = "checkpoint.records".into();
        }
        let error = client
            .download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords {
                selection: Some(selection.clone()),
                after: Some(cursor),
            })
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::FailedPrecondition);
    }
    let epoch = second.scanned.unwrap().epoch;
    assert_eq!(
        client
            .unary::<contracts::evidence::GetCapturedRecordSchema>(contracts::evidence::GetCapturedRecordSchema {
                record_type: "calibration.field.completed".into(),
                schema_version: u32::MAX,
                epoch: epoch.clone()
            })
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .unary::<contracts::evidence::GetCapturedRecordSchema>(contracts::evidence::GetCapturedRecordSchema {
                record_type: "calibration.field.completed".into(),
                schema_version: 2,
                epoch: "old-process".into()
            })
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    drop(client);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn capture_cursors_restarts() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::support::budgets::runtime_config(
        root.path(),
        Metadata::new(Identifier::new("evidence.restart").unwrap(), "Evidence Restart", None),
    );
    let session_id = config.session_id.to_string();
    let (mut owner, server, client) = start_config(config.clone(), 1024 * 1024).await;
    let identity = WireClient::connect(server.local_endpoint())
        .await
        .unwrap()
        .unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {})
        .await
        .unwrap()
        .into_inner();
    let shell = WireClient::connect(server.local_endpoint())
        .await
        .unwrap()
        .unary::<contracts::observation::GetRuntimeShell>(contracts::observation::GetRuntimeShell::default())
        .await
        .unwrap()
        .into_inner();
    assert_eq!(shell.session_id, session_id);
    assert_eq!(shell.observed_cursor.as_ref().unwrap().epoch, identity.epoch);
    assert_eq!(owner.handle().process_epoch().to_string(), identity.epoch);
    publish(&owner.handle()).await;
    let old = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(old.records.len(), 1);
    let cursor = old.scanned.unwrap();
    assert_eq!(cursor.epoch, identity.epoch);
    let schema_request = contracts::evidence::GetCapturedRecordSchema {
        record_type: "calibration.field.completed".into(),
        schema_version: old.records[0].schema_version,
        epoch: cursor.epoch.clone(),
    };
    client.unary::<contracts::evidence::GetCapturedRecordSchema>(schema_request.clone()).await.unwrap();
    drop(client);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    // The identical configuration intentionally reuses the same SessionId.
    let (mut owner, server, client) = start_config(config, 1024 * 1024).await;
    let fresh_identity = WireClient::connect(server.local_endpoint())
        .await
        .unwrap()
        .unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fresh_identity.backend_id, identity.backend_id);
    assert_ne!(fresh_identity.epoch, identity.epoch);
    assert_eq!(owner.handle().session_context().await.0.to_string(), session_id);
    let error = WireClient::connect(server.local_endpoint())
        .await
        .unwrap()
        .unary::<contracts::observation::GetRuntimeShell>(contracts::observation::GetRuntimeShell {
            cursor: shell.observed_cursor,
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::FailedPrecondition);
    let fresh = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(fresh.records.is_empty());
    let mut fresh_cursor = fresh.scanned.unwrap();
    assert_eq!(fresh_cursor.epoch, fresh_identity.epoch);
    fresh_cursor.position = 0;
    for (after, through) in [(Some(cursor.clone()), Some(cursor.clone())), (Some(fresh_cursor), Some(cursor.clone()))] {
        let error = client
            .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
                selection: Some(selection()),
                after,
                through,
                latest: 0,
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::FailedPrecondition);
    }
    assert_eq!(
        client
            .download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords {
                selection: Some(selection()),
                after: Some(cursor.clone()),
            })
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition,
    );
    assert_eq!(
        client.unary::<contracts::evidence::GetCapturedRecordSchema>(schema_request.clone()).await.unwrap_err().code(),
        Code::FailedPrecondition,
    );

    publish(&owner.handle()).await;
    let current = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(current.records.len(), 1, "a fresh process reader excludes prior-boot rows");
    assert_ne!(current.records[0].record_id, old.records[0].record_id);
    let current_epoch = current.scanned.unwrap().epoch;
    assert_ne!(current_epoch, cursor.epoch);
    client
        .unary::<contracts::evidence::GetCapturedRecordSchema>(contracts::evidence::GetCapturedRecordSchema {
            epoch: current_epoch,
            ..schema_request
        })
        .await
        .unwrap();
    drop(client);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn oversized_delivery_evidence() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::support::budgets::runtime_config(root.path(), super::support::metadata("evidence.fixture", "Evidence fixture"));
    let (mut owner, bounded, limited) = start_config(config.clone(), 1024).await;
    let handle = owner.handle();
    let before = handle.captured_records().frontier();
    let mut draft = ProductCalibrationDetails::new("fixture-device", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
        .into_draft("fixture.product");
    draft.message = Some("x".repeat(2000));
    handle.publish_product_record(draft.clone()).await.unwrap();
    assert_eq!(
        limited
            .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
                selection: Some(selection()),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::ResourceExhausted
    );
    // A Rejected Delivery Must Leave The Original Captured Record Readable
    {
        let captured = handle.captured_records();
        let retained = captured.next(before, captured.frontier()).await.unwrap().unwrap();
        assert_eq!(retained.message.as_deref(), draft.message.as_deref());
    }
    drop(handle);
    drop(limited);
    bounded.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
    let (mut owner, normal, client) = start_config(config.clone(), 1024 * 1024).await;
    let handle = owner.handle();
    handle.publish_product_record(draft.clone()).await.unwrap();
    publish(&handle).await;
    let page = client
        .unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords {
            selection: Some(selection()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.records.len(), 2);
    assert_eq!(page.records[0].message.as_ref().unwrap().len(), 2000);
    assert!(page.records[1].sequence > page.records[0].sequence);
    let mut schema_request = contracts::evidence::GetCapturedRecordSchema {
        record_type: page.records[0].record_type.clone(),
        schema_version: page.records[0].schema_version,
        epoch: page.scanned.as_ref().unwrap().epoch.clone(),
    };
    let schema = client.unary::<contracts::evidence::GetCapturedRecordSchema>(schema_request.clone()).await.unwrap().into_inner();
    let schema_limit = schema.wire_bytes().len() - 1;
    drop(client);
    normal.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
    let (mut owner, schema_server, schema_client) = start_config(config, schema_limit).await;
    owner.handle().publish_product_record(draft).await.unwrap();
    schema_request.epoch = owner.handle().process_epoch().to_string();
    assert_eq!(
        schema_client.unary::<contracts::evidence::GetCapturedRecordSchema>(schema_request).await.unwrap_err().code(),
        Code::ResourceExhausted
    );
    drop(schema_client);
    schema_server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
