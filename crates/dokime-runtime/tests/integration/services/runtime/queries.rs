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
use futures_util::StreamExt;
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
    let requests = vec![
        serde_json::from_value(serde_json::json!({
            "stream": "fixture.telemetry",
            "y_keys": ["value"]
        }))
        .unwrap(),
    ];
    let mut telemetry_watch =
        runtime.watch_plot_telemetry(&requests).await.unwrap().pop().expect("the fixed telemetry domain is watchable before its first row");

    runtime.publish_telemetry_batch(published(1..=3)).await.unwrap();

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

    // The publish path writes to the live Harmos stream and the chunk store;
    // these reads observe the batch immediately.
    let summaries = runtime.telemetry_stream_summaries().await;
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].stream, stream);
    assert_eq!(summaries[0].total, 3);
    let selected = runtime.selected_telemetry(TelemetrySelection { streams: vec![stream.clone()], ..TelemetrySelection::default() }).await;
    assert_eq!(selected.len(), 3);
    assert_eq!(selected[0].stream, stream);
    assert_eq!(selected[0].sequence, 1);

    // The full window advances the cursor to the high-water (3), not to an
    // echoed None.
    let first = runtime.query_runtime_view(telemetry_query(RuntimeWindow::Full, None)).await.unwrap();
    assert_eq!(first.cursor, RuntimeCursor(3));
    let RuntimeViewPayload::Telemetry { frame } = first.payload else {
        panic!("expected a telemetry frame");
    };
    assert_eq!(frame.returned_samples, 3);

    // Plot consumers watch the durable Harmos stream itself; there is no
    // parallel wake stream or tail-filtering bus.
    let first_cursor = telemetry_watch.next().await.expect("telemetry stream remained open");

    runtime.publish_telemetry_batch(published(4..=5)).await.unwrap();

    // Re-pulling at-or-after the first cursor returns exactly the newer
    // samples: no gap, no dup.
    let second =
        runtime.query_runtime_view(telemetry_query(RuntimeWindow::Since { cursor: first.cursor }, Some(first.cursor))).await.unwrap();
    let RuntimeViewPayload::Telemetry { frame } = second.payload else {
        panic!("expected a telemetry frame");
    };
    let sequences = frame.streams.iter().flat_map(|stream| stream.samples.iter().map(|sample| sample.sequence)).collect::<Vec<_>>();
    assert_eq!(sequences, vec![4, 5]);
    assert_eq!(second.cursor, RuntimeCursor(5));
    let cursor = telemetry_watch.next().await.expect("telemetry stream watch stayed open");
    assert!(cursor > first_cursor);

    // A re-pull with nothing new returns no samples and never regresses.
    let third =
        runtime.query_runtime_view(telemetry_query(RuntimeWindow::Since { cursor: second.cursor }, Some(second.cursor))).await.unwrap();
    let RuntimeViewPayload::Telemetry { frame } = third.payload else {
        panic!("expected a telemetry frame");
    };
    assert_eq!(frame.returned_samples, 0);
    assert_eq!(third.cursor, RuntimeCursor(5));

    // Telemetry uses Harmos's stream watch and does not manufacture a product
    // state transaction merely for invalidation.
    assert_eq!(revision().await, before);
}

fn telemetry_config(root: impl Into<std::path::PathBuf>) -> RuntimeConfig {
    let artifact = dokime_runtime::InstalledPlugin {
        source: "fixture.telemetry".into(),
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

/* === Tests === */
