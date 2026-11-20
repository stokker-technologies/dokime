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

use dokime_client_internal::StatusGetOptions;

use crate::support::server::Fixture;
use dokime_client_internal::OverviewWatchResponse;
use dokime_protocol::host::StatusGetResponse;
use harmos_protocol::host;

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
        // Read the Same Product Status Through Typed and Binary Adapter Calls
        let fixture = Fixture::new().await;
        let client = fixture.client().await;
        let typed =
            client.execute(client.client().status.get_with(StatusGetOptions { context: client.call_options().unwrap() })).await.unwrap();
        let bytes = client.call_raw("/dokime.host.v1.Status/Get", &[]).await.unwrap();
        let binary = StatusGetResponse::decode(bytes.as_slice()).unwrap();
        assert_eq!(typed.status.unwrap().identity, binary.status.unwrap().identity);
        assert_eq!(typed.build, binary.build);

        // Publish the Answering Host's Build in the Native Connection Details
        let build = typed.build.unwrap();
        let backend = fixture.session.status().backend.unwrap();
        assert_eq!(backend.source_revision.as_deref(), Some(build.source_revision.as_str()));
        assert_eq!(backend.channel.as_deref().unwrap_or_default(), build.channel);
        fixture.close().await;
    })
}

#[test]
fn disconnect_joins_the_idle_watch() {
    dokime_gui::test_support::run(async {
        // Open a Real Host Watch Before Disconnecting Its Connection Generation
        let fixture = Fixture::new().await;
        let client = fixture.client().await;
        let mut watch = client.watch_raw("/dokime.host.v1.backend.Overview/Watch", &[]).await.unwrap();
        let opening = watch.next().await.unwrap().unwrap();
        assert!(OverviewWatchResponse::decode(opening.as_slice()).unwrap().opening);
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
        let response: StatusGetResponse = client.unary("/dokime.host.v1.Status/Get", host::StatusGetRequest {}).await.unwrap();
        assert!(response.status.unwrap().available);
        fixture.close().await;
    })
}
