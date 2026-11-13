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

// Local Imports
use crate::support::server::{Fixture, attach};
use dokime_gui::connection::{ConnectionError, ConnectionPhase, ConnectionSession};
use dokime_gui::native_session::BackendSelection;

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
fn switching_cancels_work() {
    dokime_gui::test_support::run(async {
        // Connect Two Independent Native Sessions to One Host
        let fixture = Fixture::new().await;
        let session = ConnectionSession::default();
        let endpoint = fixture.session.status().endpoint.unwrap();
        let BackendSelection::Attach { request } = attach(endpoint.clone()) else { unreachable!() };
        let connected = session.connect(request).await.unwrap();
        let lease = session.lease(&connected.generation).unwrap();
        let pending = tokio::spawn(async move { lease.execute(std::future::pending::<Result<(), tonic::Status>>()).await });

        // Replacing the Generation Settles Pending Work Before the Next Lease
        let BackendSelection::Attach { request } = attach(endpoint) else { unreachable!() };
        let replacement = session.connect(request).await.unwrap();
        assert_ne!(connected.generation, replacement.generation);
        assert_eq!(pending.await.unwrap(), Err(ConnectionError::Superseded));
        assert!(matches!(session.lease(&connected.generation), Err(ConnectionError::Superseded)));
        assert_eq!(session.disconnect().unwrap().phase, ConnectionPhase::Disconnected);

        // An Attached Session Does Not Own Host Shutdown
        let client = fixture.client().await;
        let _: dokime_sdk::client::v1::StatusGetResponse =
            client.unary("/harmos.v1.Status/Get", dokime_sdk::client::v1::StatusGetRequest {}).await.unwrap();
        fixture.close().await;
    })
}

#[test]
fn identity_is_consistent() {
    dokime_gui::test_support::run(async {
        // Refuse an Explicit Identity Pin Without Publishing a Ready Connection
        let fixture = Fixture::new().await;
        let session = ConnectionSession::default();
        let BackendSelection::Attach { mut request } = attach(fixture.session.status().endpoint.unwrap()) else { unreachable!() };
        request.expected_backend_id = Some("another-host".into());
        assert_eq!(session.connect(request).await.unwrap_err(), ConnectionError::IdentityChanged);
        assert_eq!(session.status().phase, ConnectionPhase::Failed);
        assert!(session.status().backend.is_none());
        fixture.close().await;
    })
}

#[test]
fn disconnect_cancels_pending_readiness() {
    dokime_gui::test_support::run(async {
        // TCP Acceptance Proves Negotiation Entered the Transport
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let session = ConnectionSession::default();
        let BackendSelection::Attach { request } = attach(format!("http://{}", listener.local_addr().unwrap())) else { unreachable!() };
        let connecting = session.clone();
        let pending = tokio::spawn(async move { connecting.connect(request).await });
        let (_socket, _) = listener.accept().await.unwrap();
        assert_eq!(session.status().phase, ConnectionPhase::Connecting);
        session.disconnect().unwrap();
        assert_eq!(pending.await.unwrap().unwrap_err(), ConnectionError::Superseded);
    })
}
