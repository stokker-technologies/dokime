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
use dokime_app::{
    core::{Identifier, Timestamp, Value},
    telemetry::{TelemetryBatch, TelemetrySample},
};
use dokime_protocol::v1::{self, telemetry_client::TelemetryClient};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn sequences(reply: &v1::TelemetrySamplesReply) -> Vec<u64> {
    reply.samples.iter().map(|sample| sample.sequence).collect()
}

pub(crate) async fn telemetry_pages_window() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    common::register_fixture_progress_execution(&handle, common::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    let samples = (0..5)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                id.clone(),
                index,
                Timestamp::new("2026-09-13T00:00:00Z"),
                "fixture",
            );
            sample
                .values
                .insert("progress".into(), Value::Number(index as f64));
            sample
        })
        .collect();
    handle
        .publish_telemetry_batch(TelemetryBatch::new(id.clone(), samples))
        .await
        .unwrap();

    let server = Server::start(handle.clone(), Config::local())
        .await
        .unwrap();
    let mut client = TelemetryClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let stream = dokime_app::capabilities::telemetry::telemetry_stream_id(None, &id);
    let base = v1::ListTelemetrySamplesRequest {
        streams: vec![id.to_string()],
        limit: 2,
        ..Default::default()
    };

    let first = client
        .list_samples(authenticated(base.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(sequences(&first), [1, 2]);
    assert_eq!(first.through_stream_cursors.get(&stream), Some(&2));

    let second = client
        .list_samples(authenticated(v1::ListTelemetrySamplesRequest {
            after_stream_cursors: first.through_stream_cursors.clone(),
            ..base.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(sequences(&second), [3, 4]);
    assert_eq!(second.through_stream_cursors.get(&stream), Some(&4));

    let tail = client
        .list_samples(authenticated(v1::ListTelemetrySamplesRequest {
            latest: true,
            ..base.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(sequences(&tail), [4, 5]);
    assert!(tail.through_stream_cursors.is_empty());

    let rejected = client
        .list_samples(authenticated(v1::ListTelemetrySamplesRequest {
            after_stream_cursors: second.through_stream_cursors,
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
