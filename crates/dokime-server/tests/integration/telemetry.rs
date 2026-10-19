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
use dokime_domain::model;
use {
    dokime_runtime::Identifier, dokime_runtime::Timestamp, dokime_runtime::Value, dokime_runtime::telemetry::TelemetryBatch,
    dokime_runtime::telemetry::TelemetrySample,
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
fn sequences(reply: &model::telemetry::wire::TelemetrySamplesReply) -> Vec<u64> {
    reply.samples.iter().map(|sample| sample.sequence).collect()
}

fn cursor(reply: &model::telemetry::wire::TelemetrySamplesReply, stream: &str) -> Option<u64> {
    reply.through_stream_cursors.iter().find(|entry| entry.key == stream).map(|entry| entry.value)
}

fn continuation(
    reply: &model::telemetry::wire::TelemetrySamplesReply,
) -> Vec<model::telemetry::wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry> {
    reply
        .through_stream_cursors
        .iter()
        .map(|entry| model::telemetry::wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry {
            key: entry.key.clone(),
            value: entry.value,
        })
        .collect()
}

#[tokio::test]
async fn telemetry_pages_window() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    let samples = (0..5)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                id.clone(),
                index,
                Timestamp::new("2026-09-13T00:00:00Z"),
                "fixture",
            );
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.with_storage_payload(harmos::Message::encode_to_vec(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), id.clone(), samples)).await.unwrap();

    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let stream = dokime_runtime::domains::telemetry::telemetry_stream_id(None, &id);
    let base = contracts::telemetry::ListSamples {
        streams: vec!["fixture.*".into(), "fixture.progres?".into(), "missing.*".into()],
        limit: 2,
        ..Default::default()
    };

    let excluded = client
        .unary::<contracts::telemetry::ListSamples>(authenticated(contracts::telemetry::ListSamples {
            streams: vec!["missing.*".into()],
            ..base.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(excluded.samples.is_empty());

    let malformed = client
        .unary::<contracts::telemetry::ListSamples>(authenticated(contracts::telemetry::ListSamples {
            streams: vec!["fixture.[ab]".into()],
            ..base.clone()
        }))
        .await
        .unwrap_err();
    assert_eq!(malformed.code(), Code::InvalidArgument);

    let first = client.unary::<contracts::telemetry::ListSamples>(authenticated(base.clone())).await.unwrap().into_inner();
    assert_eq!(sequences(&first), [1, 2]);
    assert_eq!(cursor(&first, &stream), Some(2));

    let second = client
        .unary::<contracts::telemetry::ListSamples>(authenticated(contracts::telemetry::ListSamples {
            after_stream_cursors: continuation(&first),
            ..base.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(sequences(&second), [3, 4]);
    assert_eq!(cursor(&second, &stream), Some(4));

    let tail = client
        .unary::<contracts::telemetry::ListSamples>(authenticated(contracts::telemetry::ListSamples { latest: true, ..base.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(sequences(&tail), [4, 5]);
    assert!(tail.through_stream_cursors.is_empty());

    let rejected = client
        .unary::<contracts::telemetry::ListSamples>(authenticated(contracts::telemetry::ListSamples {
            after_stream_cursors: continuation(&second),
            latest: true,
            ..base
        }))
        .await
        .unwrap_err();
    assert_eq!(rejected.code(), Code::InvalidArgument);

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
