/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "testing")]
#[path = "delivery/transport.rs"]
pub mod delivery;

/* === Re-Exports === */

/* === Imports === */

use dokime_protocol::client;

// Standard Library Imports
use std::{
    convert::Infallible,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

// Third Party Imports
use tonic::{Request, Response, Status, codegen::BoxFuture};

// Local Imports
use dokime_gui::connection::{ConnectRequest, ConnectionTarget};
use dokime_sdk::client::v1 as library;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type HttpRequest = tonic::codegen::http::Request<tonic::body::Body>;
pub type HttpResponse = tonic::codegen::http::Response<tonic::body::Body>;

/* === Structs === */

pub struct Gate {
    pub entered: tokio::sync::Notify,
    pub release: tokio::sync::Semaphore,
}

#[derive(Clone)]
pub struct Service {
    pub id: String,
    pub download: Arc<Mutex<delivery::DownloadConfig>>,
}

pub struct Listener {
    pub endpoint: String,
    task: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

#[derive(Clone)]
struct Rpc<const KIND: u8>(Service);

struct Reply<T>(T);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Gate {
    pub fn new() -> Self {
        Self {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Semaphore::new(0),
        }
    }
}

impl Service {
    pub fn new(id: &str) -> Self {
        Self { id: id.into(), download: Arc::default() }
    }
}

impl<const KIND: u8> tonic::server::NamedService for Rpc<KIND> {
    const NAME: &'static str = match KIND {
        0 => "harmos.v1.Status",
        1 => "dokime.client.v1.backend.Overview",
        _ => "harmos.v1.CatalogEntries",
    };
}

impl<const KIND: u8> tonic::codegen::Service<HttpRequest> for Rpc<KIND> {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = BoxFuture<HttpResponse, Infallible>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                "/harmos.v1.Status/Get" => {
                    let mut reply = library::StatusGetResponse::default();
                    let status = reply.status.get_or_insert_default();
                    status.available = true;
                    status.identity.get_or_insert_default().host_id = service.id;
                    let history = reply.resolved_at.get_or_insert_default().history.get_or_insert_default();
                    history.store = "desktop-fixture".into();
                    history.epoch = "status-history".into();
                    tonic::server::Grpc::new(tonic_prost::ProstCodec::<_, library::StatusGetRequest>::default())
                        .unary(Reply(reply), request)
                        .await
                }
                "/dokime.client.v1.backend.Overview/Get" => {
                    let reply = client::backend::OverviewGetResponse {
                        overview: Some(client::backend::RuntimeShellSnapshot {
                            session_id: "product-history".into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    tonic::server::Grpc::new(tonic_prost::ProstCodec::<_, client::backend::OverviewGetRequest>::default())
                        .unary(Reply(reply), request)
                        .await
                }
                "/harmos.v1.CatalogEntries/List" => {
                    let reply = library::CatalogEntriesListResponse {
                        operations: vec![library::DiscoveredOperation {
                            grpc_available: true,
                            contract: Some(library::DiscoveredContract {
                                grpc: Some(library::Grpc {
                                    package: "harmos.v1".into(),
                                    service: "Storage".into(),
                                    method: "GetFileBytes".into(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    };
                    tonic::server::Grpc::new(tonic_prost::ProstCodec::<_, library::CatalogEntriesListRequest>::default())
                        .unary(Reply(reply), request)
                        .await
                }
                _ => Status::unimplemented("Fixture Route Is Not Served").into_http(),
            };
            Ok(response)
        })
    }
}

impl<Q: Send + 'static, R: Clone + Send + 'static> tonic::server::UnaryService<Q> for Reply<R> {
    type Response = R;
    type Future = BoxFuture<Response<R>, Status>;

    fn call(&mut self, _: Request<Q>) -> Self::Future {
        let reply = self.0.clone();
        Box::pin(async move { Ok(Response::new(reply)) })
    }
}

impl Listener {
    pub async fn start(service: Service, tls: bool, ip: Ipv4Addr) -> Self {
        assert!(!tls, "Pinned TLS Uses the Real Host Connection Stories");
        let listener = tokio::net::TcpListener::bind((ip, 0)).await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(Rpc::<0>(service.clone()))
                .add_service(Rpc::<1>(service.clone()))
                .add_service(Rpc::<2>(service.clone()))
                .add_service(delivery::Content(service.clone()))
                .add_service(delivery::Storage(service))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
        );
        Self { endpoint, task }
    }

    pub fn local(&self) -> ConnectRequest {
        ConnectRequest {
            target: ConnectionTarget::Local { endpoint: self.endpoint.clone() },
            expected_backend_id: None,
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
