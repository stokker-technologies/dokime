/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

#[tokio::test]
async fn orchestrator_frames_exposed() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;

    let frame = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Overview,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Summary,
            cursor: None,
        })
        .await
        .unwrap();

    assert_eq!(frame.cursor, RuntimeCursor(runtime.runtime_overview().await.unwrap().revision));
    let RuntimeViewPayload::Overview { frame } = frame.payload else {
        panic!("expected overview frame");
    };
    assert_eq!(frame.installed_plugins, 0);
    assert_eq!(frame.telemetry_streams, 0);
    assert_eq!(frame.telemetry_samples, 0);
    assert_eq!(frame.resources, 0);
    assert_eq!(frame.health, 0);
    assert_eq!(frame.traceability, 0);
    assert_eq!(frame.notes, 0);
    assert_eq!(frame.artifacts, 0);
    assert!(frame.record_definitions > 0);
    assert_eq!(frame.telemetry_definitions, 0);
}

#[tokio::test]
async fn live_revisions_coalesce() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let mut live = runtime.watch_state();
    let initial_revision = live.latest().revision();

    record_health(&runtime, "live one").await.unwrap();
    record_health(&runtime, "live two").await.unwrap();
    live.next().await.unwrap();
    let latest = live.latest().revision();

    assert!(latest >= initial_revision + 2);
}

#[tokio::test]
async fn runtime_overview_payload() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;

    let overview = runtime.runtime_overview().await.expect("runtime overview query succeeds");
    let json = serde_json::to_value(&overview).unwrap();

    assert!(json.get("test_suites").is_none());
    assert_eq!(json.get("test_suite_count").and_then(serde_json::Value::as_u64), Some(0));
}

#[tokio::test]
async fn telemetry_reads_stream() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(telemetry_config(root.path())).await;
    let stream = Identifier::new("fixture.telemetry").unwrap();
    let revision = || async { runtime.runtime_overview().await.expect("runtime overview query succeeds").revision };
    let telemetry_query = |window: RuntimeWindow, cursor: Option<RuntimeCursor>| RuntimeViewQuery {
        scope: None,
        topic: RuntimeViewTopic::Telemetry { stream: Some(stream.clone()) },
        window,
        resolution: RuntimeResolution::Raw,
        cursor,
    };
    let published = |values: std::ops::RangeInclusive<i64>| {
        TelemetryBatch::new(
            dokime_runtime::model::identity::new_id(),
            stream.clone(),
            values.map(|value| telemetry_sample(&stream, value)).collect(),
        )
    };
    let before = revision().await;
    let before_saved = runtime.storage().cursor_saved();

    runtime.publish_telemetry_batch(published(1..=3)).await.unwrap();
    runtime.storage().flush().await.unwrap();

    // The overview counter reflects durable telemetry at either resolution.
    for resolution in [RuntimeResolution::Summary, RuntimeResolution::Raw] {
        let frame = runtime
            .query_runtime_view(RuntimeViewQuery {
                scope: None,
                topic: RuntimeViewTopic::Overview,
                window: RuntimeWindow::Full,
                resolution,
                cursor: None,
            })
            .await
            .unwrap();
        let RuntimeViewPayload::Overview { frame } = frame.payload else {
            panic!("expected overview frame");
        };
        assert_eq!(frame.telemetry_samples, 3);
    }

    // These Reads Observe The Flushed Native Storage Batch.
    let summaries = runtime.telemetry_stream_summaries().await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].stream, stream);
    assert_eq!(summaries[0].total, 3);
    let selected = runtime
        .selected_telemetry(TelemetrySelection {
            streams: vec![stream.clone().into()],
            ..TelemetrySelection::default()
        })
        .await
        .unwrap();
    assert_eq!(selected.len(), 3);
    assert_eq!(selected[0].stream, stream);
    assert!(selected[0].sequence > before_saved.sequence);

    // The full window advances the cursor to the high-water (3), not to an
    // echoed None.
    let first = runtime.query_runtime_view(telemetry_query(RuntimeWindow::Full, None)).await.unwrap();
    assert_eq!(first.cursor, RuntimeCursor(selected.last().unwrap().sequence));
    let RuntimeViewPayload::Telemetry { frame } = first.payload else {
        panic!("expected a telemetry frame");
    };
    assert_eq!(frame.returned_samples, 3);

    // Plot consumers watch the durable Harmos stream itself; there is no
    // parallel wake stream or tail-filtering bus.
    let first_cursor = runtime.storage().wait_saved_after(&before_saved).await.unwrap();

    runtime.publish_telemetry_batch(published(4..=5)).await.unwrap();
    runtime.storage().flush().await.unwrap();

    // Re-pulling at-or-after the first cursor returns exactly the newer
    // samples: no gap, no dup.
    let second =
        runtime.query_runtime_view(telemetry_query(RuntimeWindow::Since { cursor: first.cursor }, Some(first.cursor))).await.unwrap();
    let RuntimeViewPayload::Telemetry { frame } = second.payload else {
        panic!("expected a telemetry frame");
    };
    let sequences = frame.streams.iter().flat_map(|stream| stream.samples.iter().map(|sample| sample.sequence)).collect::<Vec<_>>();
    assert_eq!(sequences.len(), 2);
    assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(sequences[0] > first.cursor.0);
    assert_eq!(second.cursor, RuntimeCursor(*sequences.last().unwrap()));
    let cursor = runtime.storage().wait_saved_after(&first_cursor).await.unwrap();
    assert!(cursor.sequence > first_cursor.sequence);

    // A re-pull with nothing new returns no samples and never regresses.
    let third =
        runtime.query_runtime_view(telemetry_query(RuntimeWindow::Since { cursor: second.cursor }, Some(second.cursor))).await.unwrap();
    let RuntimeViewPayload::Telemetry { frame } = third.payload else {
        panic!("expected a telemetry frame");
    };
    assert_eq!(frame.returned_samples, 0);
    assert_eq!(third.cursor, second.cursor);

    // Telemetry uses Harmos's stream watch and does not manufacture a product
    // state transaction merely for invalidation.
    assert_eq!(revision().await, before);
}

#[tokio::test]
async fn telemetry_batch_receipts_and_order() {
    const ROWS: usize = 4097;
    let root = tempdir().unwrap();
    let (mut owner, runtime) = started(telemetry_config(root.path())).await;
    let stream = Identifier::new("fixture.telemetry").unwrap();
    let samples = (0..ROWS).map(|index| telemetry_sample(&stream, i64::try_from(index).unwrap())).collect();
    let receipt = runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream.clone(), samples))
        .await
        .unwrap();
    assert_eq!(receipt.status, dokime_runtime::domains::evidence::PublishStatus::Accepted);
    assert_eq!(receipt.accepted, ROWS as u64);
    assert_eq!(receipt.dropped, 0);
    assert_eq!(receipt.stream_cursors.values().copied().collect::<Vec<_>>(), [ROWS as u64]);

    let tail = telemetry_sample(&stream, ROWS as i64);
    let receipt = runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream.clone(), vec![tail]))
        .await
        .unwrap();
    assert_eq!(receipt.accepted, 1);
    assert_eq!(receipt.dropped, 0);
    assert_eq!(receipt.stream_cursors.values().copied().collect::<Vec<_>>(), [(ROWS + 1) as u64]);
    runtime.storage().flush().await.unwrap();
    let rows = runtime
        .collect_telemetry(&TelemetrySelection {
            streams: vec![stream.into()],
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), ROWS + 1);
    assert!(rows.windows(2).all(|rows| rows[0].sequence < rows[1].sequence));
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.values.get("value"), Some(&Value::Number(index as f64)));
    }
    drop(rows);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

fn telemetry_config(root: impl Into<std::path::PathBuf>) -> RuntimeConfig {
    let artifact = dokime_runtime::InstalledPlugin {
        source: "fixture_telemetry".into(),
        version: "test".into(),
        description: "Telemetry query fixture".into(),
        enabled: true,
        telemetry_contracts: vec![fixture_telemetry_contract()],
        ..Default::default()
    };

    config(root).with_installed_plugins([artifact])
}

fn telemetry_sample(stream: &Identifier, value: i64) -> TelemetrySample {
    let mut sample = TelemetrySample::new(
        dokime_runtime::model::identity::new_id(),
        stream.clone(),
        0,
        dokime_runtime::model::time::now_utc(),
        "runtime.test",
    );
    sample.values.insert("value".to_owned(), Value::from(value));
    crate::service_support::contracts::typed_fixture_sample(sample)
}
