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
use dokime_runtime::execution::HealthSeverity;
use dokime_runtime::records::{
    ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase,
    ProductConnectionDetails, ProductConnectionState, ProductRecordDraft,
};
use dokime_runtime::domains::evidence::RecordSelection;

/* === Constants === */
const WIFI: &str = "host.wifi.channel_occupancy";

/* === Macros === */

/* === Type Aliases === */
type JournalRecords = dokime_runtime::runtime::RecordSubscription;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn evidence_write_revision() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;
    let revision = || async {
        runtime
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .revision
    };
    let before = revision().await;

    let receipt = runtime
        .publish_product_record(product_calibration(ProductCalibrationPhase::Completed))
        .await
        .unwrap();
    let after_record = revision().await;
    runtime
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: Some("operator".to_owned()),
            message: "Observed setup.".to_owned(),
        })
        .await
        .unwrap();
    let after_note = revision().await;
    runtime
        .add_health_item(
            RecordScope::default(),
            HealthSeverity::Warning,
            "Runtime warning.".to_owned(),
            Some("fixture detail".to_owned()),
        )
        .await
        .unwrap();
    let after_health = revision().await;
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert_eq!(receipt.status, PublishStatus::Accepted);
    assert_eq!(receipt.accepted, 1);
    assert!(receipt.stream_cursors.is_empty());
    assert!(before < after_record);
    assert!(after_record < after_note);
    assert!(after_note < after_health);
    let calibrations = evidence
        .records
        .iter()
        .filter(|record| record.record_type.as_str() == "calibration.field.completed")
        .collect::<Vec<_>>();
    assert_eq!(calibrations.len(), 1);
    assert!(calibrations[0].sequence > 0);
    assert_eq!(calibrations[0].schema_version, 2);
    assert_eq!(evidence.notes.len(), 1);
    assert_eq!(evidence.notes[0].message, "Observed setup.");
    assert_eq!(evidence.health.len(), 1);
    assert_eq!(evidence.health[0].message, "Runtime warning.");
    assert_eq!(evidence.health[0].detail.as_deref(), Some("fixture detail"));
}

#[tokio::test]
async fn subscriptions_deliver_selection() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;
    let mut prefixed = runtime.subscribe_records();
    let mut open = runtime.subscribe_records();
    let mut pinned = runtime.subscribe_records();
    let prefix_selection = RecordSelection {
        record_type_prefixes: vec!["calibration.".to_owned()],
        ..RecordSelection::default()
    };
    let pinned_selection = RecordSelection {
        record_types: vec![Identifier::new("calibration.field.completed").unwrap()],
        ..RecordSelection::default()
    };

    runtime
        .publish_product_record(product_connection(ProductConnectionState::Connected))
        .await
        .unwrap();
    runtime
        .publish_product_record(product_calibration(ProductCalibrationPhase::Completed))
        .await
        .unwrap();
    runtime
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: Some("operator".to_owned()),
            message: "Observed setup.".to_owned(),
        })
        .await
        .unwrap();

    // The prefix skips the connection record committed ahead of the match.
    let (revision, record) = selected(&mut prefixed, &prefix_selection).await;
    assert_eq!(record.sequence, revision);
    assert!(revision > 0);
    assert_eq!(record.record_type.as_str(), "calibration.field.completed");

    // The open selection covers every runtime record path, each exactly once.
    let mut observed = Vec::new();
    for _ in 0..3 {
        let (_, record) = selected(&mut open, &RecordSelection::default()).await;
        observed.push(record.record_type.to_string());
    }
    assert_eq!(
        &observed[1..],
        ["calibration.field.completed", "operator.note_added"]
    );
    assert_ne!(observed[0], observed[1]);

    // A pinned type delivers only itself.
    let (_, record) = selected(&mut pinned, &pinned_selection).await;
    assert_eq!(record.record_type.as_str(), "calibration.field.completed");
}

#[tokio::test]
async fn telemetry_batches_sample() {
    for count in [2, 104, 600] {
        let root = tempdir().unwrap();
        let (_owner, runtime) = started(wifi_config(root.path())).await;
        let before = runtime
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .revision;

        let receipt = runtime
            .publish_telemetry_batch(samples(WIFI, count, None))
            .await
            .unwrap();
        let evidence = runtime.evidence_snapshot().await.unwrap();
        let retained = retained_wifi(&evidence);

        assert_eq!(receipt.status, PublishStatus::Accepted);
        assert_eq!(receipt.accepted, count as u64);
        assert_eq!(receipt.stream_cursors.len(), 1);
        assert_eq!(
            receipt.stream_cursors.values().copied().next(),
            Some(count as u64)
        );
        assert_eq!(retained.len(), count);
        assert_eq!(retained[0].sequence, 1);
        assert_eq!(retained.last().unwrap().sequence, count as u64);
        assert!(retained[0].values.contains_key("timestamp"));
        assert_eq!(evidence.counts().telemetry, count);
        // Telemetry is off the command path: Harmos's stream watch carries live
        // invalidation without manufacturing a product-state commit.
        assert_eq!(
            runtime
                .runtime_overview()
                .await
                .expect("runtime overview query succeeds")
                .revision,
            before
        );
        assert!(evidence.health.iter().all(|item| {
            item.detail.as_deref().is_none_or(|detail| {
                !detail.contains("producer evidence item(s) dropped under overload")
            })
        }));
    }

    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;

    assert!(
        runtime
            .publish_telemetry_batch(samples(WIFI, 5_000, None))
            .await
            .is_err()
    );
    assert!(
        runtime
            .evidence_snapshot()
            .await
            .unwrap()
            .telemetry
            .is_empty()
    );
    // Keep each publication below the byte limit while spanning multiple Harmos
    // read pages across the accepted stream.
    for _ in 0..20 {
        runtime
            .publish_telemetry_batch(samples(WIFI, 250, None))
            .await
            .unwrap();
    }
    let evidence = runtime.evidence_snapshot().await.unwrap();
    let retained = retained_wifi(&evidence);

    assert_eq!(retained.len(), 5_000);
    assert_eq!(retained[0].sequence, 1);
    assert_eq!(retained.last().unwrap().sequence, 5_000);
    assert_eq!(evidence.counts().telemetry, 5_000);
}

#[tokio::test]
async fn record_history_ranges() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;

    runtime
        .publish_product_record(product_calibration(ProductCalibrationPhase::Started))
        .await
        .unwrap();
    for index in 0..520 {
        runtime
            .add_health_item(
                RecordScope::default(),
                HealthSeverity::Info,
                format!("transaction-only fixture {index}"),
                None,
            )
            .await
            .expect("health transaction is accepted");
    }
    runtime
        .publish_product_record(product_calibration(ProductCalibrationPhase::Completed))
        .await
        .unwrap();

    let records = runtime
        .selected_records(RecordSelection::default())
        .await
        .unwrap();
    assert!(records.iter().any(|record| {
        record.record_type.as_str() == "calibration.field.completed" && record.sequence > 512
    }));
}

#[tokio::test]
async fn publish_record_warning() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;
    let result = runtime
        .publish_record(invalid_product_record(
            "calibration.field.completed",
            "done",
        ))
        .await;
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert!(result.is_err());
    assert!(evidence.records.is_empty());
    assert_eq!(evidence.health.len(), 1);
    assert_eq!(
        evidence.health[0].severity,
        dokime_runtime::execution::HealthSeverity::Warning
    );
    assert!(
        evidence.health[0]
            .detail
            .as_ref()
            .unwrap()
            .contains("Telemetry")
            || evidence.health[0]
                .detail
                .as_ref()
                .unwrap()
                .contains("Record")
    );
    assert!(evidence.health[0].detail.is_some());
}

// telemetry rejects unknown execution without partial publication.

#[tokio::test]
async fn telemetry_refuses_partial() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(wifi_config(root.path())).await;
    let mut batch = samples(WIFI, 2, None);
    batch.samples[1].scope =
        Some(RecordScope::default().with_test_execution(TestExecutionId::new()));

    let error = runtime.publish_telemetry_batch(batch).await.unwrap_err();
    assert!(error.to_string().contains("acquisition"), "{error}");
    assert!(
        runtime
            .evidence_snapshot()
            .await
            .unwrap()
            .telemetry
            .is_empty()
    );
}

fn product_connection(connection_state: ProductConnectionState) -> ProductRecordDraft {
    ProductConnectionDetails::new("plt400-1", connection_state).into_draft("python.product")
}

fn product_calibration(calibration_phase: ProductCalibrationPhase) -> ProductRecordDraft {
    ProductCalibrationDetails::new("plt400-1", ProductCalibrationKind::Field, calibration_phase)
        .into_draft("python.product")
}

fn invalid_product_record(record_type: &str, state: &str) -> RecordPublishRequest {
    let mut request =
        RecordPublishRequest::new(Identifier::new(record_type).unwrap(), RecordSeverity::Info);
    request.message = Some("Product state changed.".to_owned());
    request.details = if record_type.starts_with("calibration.") {
        BTreeMap::from([
            ("resource_id".to_owned(), Value::from("plt400-1")),
            ("resource_kind".to_owned(), Value::from("tool")),
            ("calibration_kind".to_owned(), Value::from("field")),
            ("calibration_phase".to_owned(), Value::from(state)),
        ])
    } else {
        BTreeMap::from([
            ("resource_id".to_owned(), Value::from("plt400-1")),
            ("resource_kind".to_owned(), Value::from("tool")),
            ("connection_state".to_owned(), Value::from(state)),
        ])
    };

    request
}

fn retained_wifi(
    evidence: &dokime_runtime::domains::evidence::EvidenceSnapshot,
) -> Vec<&dokime_runtime::telemetry::TelemetrySample> {
    evidence
        .telemetry
        .iter()
        .filter(|sample| sample.stream.as_str() == WIFI)
        .collect()
}

async fn selected(
    subscription: &mut JournalRecords,
    selection: &RecordSelection,
) -> (u64, dokime_runtime::runtime::CapturedRecord) {
    loop {
        let record = subscription.next().await.unwrap().unwrap();
        let revision = record.sequence;
        if selection.matches(&record) {
            return (revision, record);
        }
    }
}

/* === Tests === */
