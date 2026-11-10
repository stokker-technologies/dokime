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
use std::{net::Ipv4Addr, sync::Arc};
// Third Party Imports
use tonic::Status;
// Local Imports
use super::fixture::{Gate, Listener, Service, server_health};
use dokime::domain::model;
use dokime_gui::connection::{ConnectionError, ConnectionPhase, ConnectionSession, ConnectionTarget};

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn switching_cancels_work() {
    let session = ConnectionSession::default();
    let gate = Arc::new(Gate::new());
    let mut service = Service::new("first");
    service.gate = Some(gate.clone());
    let first = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let request = first.local();
    let pending = tokio::spawn({
        let session = session.clone();
        async move { session.connect(request).await }
    });
    gate.entered.notified().await;
    assert_eq!(session.status().phase, ConnectionPhase::Connecting);
    let second = Listener::start(Service::new("second"), false, Ipv4Addr::LOCALHOST).await;
    let connected = session.connect(second.local()).await.unwrap();
    assert_eq!(connected.backend.as_ref().unwrap().backend_id, "second");
    assert_eq!(pending.await.unwrap().unwrap_err(), ConnectionError::Superseded);
    gate.release.add_permits(1);
    assert_eq!(session.status().generation, connected.generation);
    assert!(matches!(session.lease("1"), Err(ConnectionError::Superseded)));
    let json = serde_json::to_value(&connected).unwrap();
    assert_eq!(json["backend"]["limits"]["maxContentBytes"], u64::MAX.to_string());
    assert_eq!(json["backend"]["limits"]["maxRequestBytes"], u64::MAX.to_string());
    let lease = session.lease(&connected.generation).unwrap();
    let pending_read = tokio::spawn({
        let lease = lease.clone();
        async move { lease.execute(std::future::pending::<Result<(), Status>>()).await }
    });
    let disconnected = session.disconnect().unwrap();
    assert_eq!(disconnected.phase, ConnectionPhase::Disconnected);
    assert_eq!(pending_read.await.unwrap(), Err(ConnectionError::Superseded));
    assert_eq!(lease.execute(async { Ok(()) }).await, Err(ConnectionError::Superseded));
    // The session never owns the server: it remains callable after disconnect.
    server_health(&second.endpoint).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn identity_is_consistent() {
    for fault in ["pin", "health", "availability", "api"] {
        let mut service = Service::new("backend");
        match fault {
            "health" => service.health_epoch = "restarted".into(),
            "availability" => service.availability_epoch = "restarted".into(),
            "api" => service.api_version = "2".into(),
            _ => {}
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let mut request = server.local();
        if fault == "pin" {
            request.expected_backend_id = Some("other".into());
        }
        let session = ConnectionSession::default();
        let error = session.connect(request).await.unwrap_err();
        assert_eq!(error, if fault == "api" { ConnectionError::IncompatibleApi } else { ConnectionError::IdentityChanged });
        assert_eq!(session.status().phase, ConnectionPhase::Failed);
        assert!(session.status().backend.is_none());
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn compatible_api_descriptor() {
    let mut service = Service::new("backend");
    service.descriptor = vec![7; 32];
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let connected = ConnectionSession::default().connect(server.local()).await.unwrap();
    assert_eq!(connected.backend.unwrap().descriptor_sha256, "07".repeat(32));
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn api_negotiation_availability() {
    let mut wrong_package = Service::new("backend");
    wrong_package.api_package = "other.v1".into();
    let server = Listener::start(wrong_package, false, Ipv4Addr::LOCALHOST).await;
    assert_eq!(ConnectionSession::default().connect(server.local()).await.unwrap_err(), ConnectionError::IncompatibleApi);

    for missing in [
        "/dokime.v1.Backend/GetIdentity",
        "/dokime.v1.Backend/GetHealth",
        "/dokime.v1.Backend/GetAvailability",
        "/dokime.v1.Runtime/GetIdentity",
        "/dokime.v1.Runtime/GetHealth",
    ] {
        let mut service = Service::new("backend");
        service.methods.retain(|method| method != missing);
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        assert_eq!(
            ConnectionSession::default().connect(server.local()).await.unwrap_err(),
            ConnectionError::IncompatibleApi,
            "missing required method {missing}"
        );
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn transport_and_secrets() {
    let key = "fixture-access-key-012345678901234567890123456789";
    for (tls, authenticated) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut service = Service::new("remote");
        service.token = authenticated.then(|| key.to_owned());
        let requests = service.requests.clone();
        let server = Listener::start(service.clone(), tls, Ipv4Addr::LOCALHOST).await;
        let session = ConnectionSession::default();

        if tls {
            let untrusted = server.remote(false, authenticated.then_some(key));
            assert_eq!(session.connect(untrusted).await.unwrap_err(), ConnectionError::Transport);
            let wrong_name = Listener::start(service, true, Ipv4Addr::new(127, 0, 0, 2)).await;
            let mismatch = wrong_name.remote(true, authenticated.then_some(key));
            assert_eq!(session.connect(mismatch).await.unwrap_err(), ConnectionError::Transport);
        }
        if authenticated {
            assert_eq!(session.connect(server.remote(tls, None)).await.unwrap_err(), ConnectionError::Unauthenticated);
            assert_eq!(
                session.connect(server.remote(tls, Some("wrong-key-012345678901234567890123456789"))).await.unwrap_err(),
                ConnectionError::Unauthenticated
            );
        }

        let connected = session.connect(server.remote(tls, authenticated.then_some(key))).await.unwrap();
        assert!(connected.remote, "plaintext or loopback spelling cannot convert a remote target into native local trust");
        assert_eq!(connected.backend.as_ref().unwrap().backend_id, "remote");
        assert!(!serde_json::to_string(&connected).unwrap().contains(key));
        let requests = requests.lock().unwrap();
        assert!(!requests.is_empty());
        if !authenticated {
            assert!(requests.iter().all(Option::is_none), "TLS alone must not add or require secrets");
        }
        drop(requests);
        session.disconnect().unwrap();
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn negotiation_matches_fixture() {
    let scenarios: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../../../../sdk/python/tests/integration/client/fixtures/negotiation.json")).unwrap();

    for case in scenarios {
        let mut service = Service::new("backend");
        service.api_version = case["version"].as_str().unwrap().into();
        service.readiness = match case["readiness"].as_i64().unwrap_or(2) {
            0 => model::backend::wire::Readiness::Unspecified,
            1 => model::backend::wire::Readiness::Starting,
            2 => model::backend::wire::Readiness::Ready,
            3 => model::backend::wire::Readiness::Degraded,
            4 => model::backend::wire::Readiness::Stopping,
            value => panic!("fixture has an unknown readiness: {value}"),
        };
        service.max_request_bytes = case["max_request_bytes"].as_u64().unwrap_or(u64::MAX);
        service.max_response_bytes = case["max_response_bytes"].as_u64().unwrap_or(u64::MAX);
        service.health_epoch = case["health_epoch"].as_str().unwrap_or("epoch").into();
        service.availability_epoch = case["availability_epoch"].as_str().unwrap_or("epoch").into();

        service.api_package = case["package"].as_str().unwrap_or("dokime.v1").into();
        service.descriptor = vec![case["descriptor"].as_u64().unwrap_or(0) as u8; 32];
        if let Some(missing) = case["missing"].as_str() {
            service.methods.retain(|method| method != missing);
        }

        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let session = ConnectionSession::default();
        let result = session.connect(server.local()).await;
        if case["accepted"].as_bool().unwrap() {
            assert!(result.is_ok(), "{}: {result:?}", case["name"]);
        } else {
            let expected = match case["error"].as_str() {
                Some("identity") => ConnectionError::IdentityChanged,
                Some("unavailable") => ConnectionError::BackendUnavailable,
                _ => ConnectionError::IncompatibleApi,
            };
            assert_eq!(result.unwrap_err(), expected, "{}", case["name"]);
        }
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn remote_candidates_require_identity_and_pinned_ca() {
    let wrong = Listener::start(Service::new("wrong"), true, Ipv4Addr::LOCALHOST).await;
    let good = Listener::start(Service::new("expected"), true, Ipv4Addr::LOCALHOST).await;
    let other = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()]).unwrap();
    let untrusted = Listener::start_with_identity(
        Service::new("expected"),
        Some((other.cert.pem(), other.signing_key.serialize_pem())),
        Ipv4Addr::LOCALHOST,
    )
    .await;
    let request = good.remote(true, None);
    if let ConnectionTarget::Remote { pairing_file } = &request.target {
        let path = std::path::Path::new(pairing_file);
        let mut pairing = dokime::client::pairing::read(path).unwrap();
        pairing.endpoints = vec![
            untrusted.endpoint.trim_end_matches('/').into(),
            wrong.endpoint.trim_end_matches('/').into(),
            good.endpoint.trim_end_matches('/').into(),
        ];
        std::fs::write(path, serde_json::to_vec(&pairing).unwrap()).unwrap();
    }
    let session = ConnectionSession::default();
    let connected = session.connect(request).await.unwrap();
    assert_eq!(connected.endpoint.as_deref(), Some(good.endpoint.as_str()));
    assert_eq!(connected.backend.unwrap().backend_id, "expected");
    let request = wrong.remote(true, None);
    if let ConnectionTarget::Remote { pairing_file } = &request.target {
        let path = std::path::Path::new(pairing_file);
        let mut pairing = dokime::client::pairing::read(path).unwrap();
        pairing.backend_identifier = "expected".into();
        std::fs::write(path, serde_json::to_vec(&pairing).unwrap()).unwrap();
    }
    assert_eq!(session.connect(request).await.unwrap_err(), ConnectionError::IdentityChanged);
    assert!(session.connect(untrusted.remote(true, None)).await.is_err());
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn remote_file_import_keeps_the_key_in_the_pairing() {
    // Import The Backend Pairing Into The Folder.
    let token = "synthetic-remote-import-key-01234567890123456789";
    let mut service = Service::new("imported-backend");
    service.token = Some(token.into());
    let server = Listener::start(service, true, Ipv4Addr::LOCALHOST).await;
    let request = server.remote(true, Some(token));
    let ConnectionTarget::Remote { pairing_file } = request.target else { unreachable!() };
    let pairing = dokime::client::pairing::read(std::path::Path::new(&pairing_file)).unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = dokime::client::profiles::Store::new(root.path().join("remotes"));
    let profile = store.import_pairing(&pairing, "imported").unwrap();

    // Connect Directly From The Saved Pairing And Keep Secrets Out Of Snapshots.
    let session = ConnectionSession::default();
    let connected = session
        .connect(dokime_gui::connection::ConnectRequest {
            target: ConnectionTarget::Remote { pairing_file: profile.pairing_file },
            expected_backend_id: None,
        })
        .await
        .unwrap();
    assert_eq!(connected.backend.as_ref().unwrap().backend_id, "imported-backend");
    assert!(!serde_json::to_string(&connected).unwrap().contains(token));
    session.disconnect().unwrap();
}
