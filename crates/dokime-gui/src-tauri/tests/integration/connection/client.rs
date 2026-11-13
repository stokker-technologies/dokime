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

// Third Party Imports
use prost::Message;
use tokio_stream::StreamExt;

// Local Imports
use crate::support::server::Fixture;
use dokime_sdk::{client::v1 as library, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn sdk_and_binary_calls_share_host_identity() {
    dokime_gui::test_support::run(async {
        // Read the Same Library Status Through Typed and Binary Adapter Calls
        let fixture = Fixture::new().await;
        let client = fixture.client().await;
        let typed =
            client.execute(client.client().status.get(client.request(library::StatusGetRequest {}).unwrap())).await.unwrap().into_inner();
        let bytes = client.call_raw("/harmos.v1.Status/Get", &[]).await.unwrap();
        let binary = library::StatusGetResponse::decode(bytes.as_slice()).unwrap();
        assert_eq!(typed.status.unwrap().identity, binary.status.unwrap().identity);
        fixture.close().await;
    })
}

#[test]
fn disconnect_joins_the_idle_watch() {
    dokime_gui::test_support::run(async {
        // Open a Real Host Watch Before Disconnecting Its Connection Generation
        let fixture = Fixture::new().await;
        let client = fixture.client().await;
        let mut watch = client.watch_raw("/dokime.v1.Observation/WatchRuntime", &[]).await.unwrap();
        let opening = watch.next().await.unwrap().unwrap();
        assert!(matches!(
            v1::ObservationWatchRuntimeOutput::decode(opening.as_slice()).unwrap().value,
            Some(v1::observation_watch_runtime_output::Value::Item(_))
        ));
        fixture.close().await;
        assert!(client.cancellation().is_cancelled());
        while let Some(frame) = watch.next().await {
            frame.unwrap();
        }
    })
}

#[test]
fn sdk_status_remains_available_after_native_detach() {
    dokime_gui::test_support::run(async {
        // Native Detach Cancels Its Leases Without Stopping the Host
        let fixture = Fixture::new().await;
        fixture.session.disconnect().unwrap();
        let client = fixture.client().await;
        let response: library::StatusGetResponse = client.unary("/harmos.v1.Status/Get", library::StatusGetRequest {}).await.unwrap();
        assert!(response.status.unwrap().available);
        fixture.close().await;
    })
}
