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
use std::{future::Future, net::Ipv4Addr, sync::Arc};
// Third Party Imports
use tokio::sync::Notify;
// Local Imports
use super::fixture::{Gate, Listener, Service, UploadInput, server_health};
use dokime::client::contracts::{backend, catalog};
use dokime::client::{Client, Error, Target};
use dokime::domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn assert_pending<F: Future + Unpin>(future: &mut F) {
    tokio::task::yield_now().await;
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    assert!(std::pin::Pin::new(future).poll(&mut context).is_pending(), "the future completed before its release");
}

/* === Tests === */

#[tokio::test]
async fn client_close_cancels() {
    let server = Listener::start(Service::new("shared"), false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, Some("shared")).await.unwrap();
    assert_eq!(client.backend().backend_id, "shared");
    assert_eq!(client.backend().limits.max_request_bytes, u64::MAX);
    let retained = client.clone();
    let (started, running) = tokio::sync::oneshot::channel();
    let pending = tokio::spawn(async move {
        retained
            .guard(async move {
                started.send(()).unwrap();
                std::future::pending::<()>().await
            })
            .await
    });
    running.await.unwrap();

    client.close();
    client.close();
    assert!(matches!(pending.await.unwrap(), Err(Error::Closed)));
    assert!(matches!(client.channel(), Err(Error::Closed)));
    assert!(matches!(client.execute(async { Ok(()) }).await, Err(Error::Closed)));

    server_health(&server.endpoint).await;
}

#[tokio::test]
async fn client_verifies_identity() {
    let service = Service::new("shared");
    let epoch = service.identity_epoch.clone();
    let requests = service.requests.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();

    client.verify(true).await.unwrap();
    let health = client.backend_get_health(backend::GetHealth {}).await.unwrap();
    assert_eq!(health.backend_id, "shared");
    let calls = requests.lock().unwrap().len();
    assert!(matches!(
        client.catalog_get_suite(catalog::GetSuite { suite_id: "absent".into() }).await,
        Err(Error::Rpc(status)) if status.code() == tonic::Code::PermissionDenied
    ));

    assert_eq!(requests.lock().unwrap().len(), calls);

    *epoch.lock().unwrap() = "restarted".into();
    assert!(matches!(client.verify(true).await, Err(Error::IdentityChanged)));
    client.verify(false).await.unwrap();

    client.close();
    assert!(matches!(client.verify(false).await, Err(Error::Closed)));
}

#[tokio::test]
async fn client_response_limit() {
    let mut service = Service::new("shared");
    service.max_response_bytes = 1;
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();

    assert!(matches!(
        client.backend_get_health(backend::GetHealth {}).await,
        Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange
    ));
}

#[tokio::test]
async fn stream_close_releases() {
    for action in ["client", "stream", "drop"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Backend/WatchLogs".into());
        let closed = service.stream_closed.clone();
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();
        let stream = client
            .backend_watch_logs(backend::WatchLogs {
                min_level: model::backend::wire::LogLevel::Unspecified,
                replay_lines: 0,
            })
            .await
            .unwrap();
        assert!(stream.message().await.unwrap().is_some());

        match action {
            "client" => {
                client.close_and_wait().await;
                assert!(matches!(stream.message().await, Err(Error::Closed)));
            }
            "stream" => {
                stream.close().await;
                client.backend_get_health(backend::GetHealth {}).await.unwrap();
            }
            "drop" => drop(stream),
            _ => unreachable!(),
        }

        closed.notified().await;
        server_health(&server.endpoint).await;
    }
}

#[tokio::test]
async fn stream_clones_close() {
    let mut service = Service::new("shared");
    service.methods.push("/dokime.v1.Backend/WatchLogs".into());
    let closed = service.stream_closed.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();
    let stream = client
        .backend_watch_logs(backend::WatchLogs {
            min_level: model::backend::wire::LogLevel::Unspecified,
            replay_lines: 0,
        })
        .await
        .unwrap();
    let other = stream.clone();
    assert!(stream.message().await.unwrap().is_some());
    drop(stream);

    let first = other.message();
    let second = other.message();
    tokio::pin!(first, second);
    assert_pending(&mut first).await;
    assert_pending(&mut second).await;
    other.close().await;
    assert!(matches!(first.await, Err(Error::Closed)));
    assert!(matches!(second.await, Err(Error::Closed)));
    other.close().await;
    closed.notified().await;
    client.backend_get_health(backend::GetHealth {}).await.unwrap();
}

#[tokio::test]
async fn upload_stop_releases() {
    for action in ["reply", "caller", "client"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Plugins/Install".into());
        let gate = Arc::new(Gate::new());
        if action != "reply" {
            service.upload_gate = Some(gate.clone());
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();
        let dropped = Arc::new(Notify::new());
        let input = UploadInput { first: true, dropped: dropped.clone() };
        let retained = client.clone();
        let task = tokio::spawn(async move { retained.plugins_install(input).await });

        match action {
            "reply" => {
                task.await.unwrap().unwrap();
            }
            "caller" => {
                gate.entered.notified().await;
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
            }
            "client" => {
                gate.entered.notified().await;
                client.close_and_wait().await;
                assert!(matches!(task.await.unwrap(), Err(Error::Closed)));
            }
            _ => unreachable!(),
        }

        dropped.notified().await;
        gate.release.add_permits(1);
        server_health(&server.endpoint).await;
    }
}

#[tokio::test]
async fn upload_operation_closes() {
    for action in ["finish", "cancel", "client", "waiter"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Plugins/Install".into());
        service.upload_drain = true;
        // An Empty Canonical Input Envelope Requires Two Framing Bytes.
        service.max_request_bytes = 2;
        let gate = Arc::new(Gate::new());
        if action != "finish" {
            service.upload_gate = Some(gate.clone());
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None).await.unwrap();
        let upload = client.plugins_install_open().unwrap();
        let oversized = model::plugins::wire::InstallPluginFrame {
            frame: Some(model::plugins::wire::InstallPluginFrameFrame::Header(model::plugins::wire::InstallPluginHeader {
                size_bytes: 1,
                sha256: String::new(),
            })),
        };
        assert!(matches!(upload.send(oversized).await,
            Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange));
        let retained = upload.clone();
        upload.send(model::plugins::wire::InstallPluginFrame::default()).await.unwrap();
        drop(upload);

        if action == "finish" {
            retained.send(model::plugins::wire::InstallPluginFrame::default()).await.unwrap();
            assert_eq!(retained.finish().await.unwrap().install_id, "2");
            assert!(matches!(retained.finish().await, Err(Error::Closed)));
        } else {
            gate.entered.notified().await;
            match action {
                "cancel" => retained.close().await,
                "client" => client.close_and_wait().await,
                "waiter" => {
                    let waiting = retained.clone();
                    let (started, running) = tokio::sync::oneshot::channel();
                    let task = tokio::spawn(async move {
                        let result = waiting.finish();
                        tokio::pin!(result);
                        assert_pending(&mut result).await;
                        started.send(()).unwrap();
                        result.await
                    });
                    running.await.unwrap();
                    task.abort();
                    assert!(task.await.unwrap_err().is_cancelled());
                    retained.close().await;
                }
                _ => unreachable!(),
            }
            gate.release.add_permits(1);
        }

        assert!(matches!(retained.send(model::plugins::wire::InstallPluginFrame::default()).await, Err(Error::Closed)));
        retained.close().await;
        retained.close().await;
        server_health(&server.endpoint).await;
    }
}
