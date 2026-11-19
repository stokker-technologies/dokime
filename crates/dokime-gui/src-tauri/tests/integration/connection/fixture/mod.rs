/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod delivery;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    convert::Infallible,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

// Third Party Imports
use tokio::{
    net::TcpListener,
    sync::{Notify, Semaphore},
    task::JoinHandle,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Request, Response, Status,
    body::Body,
    codegen::{self, BoxFuture, http},
    server,
    transport::{Error, Server},
};
use tonic_prost::ProstCodec;

// Local Imports
use dokime_sdk::DiscoveredContract;
use dokime_sdk::DiscoveredOperation;
use dokime_sdk::Grpc;

use dokime_client_internal::OverviewGetResponse;
use dokime_client_internal::RuntimeShellSnapshot;
use dokime_gui::connection::{ConnectRequest, ConnectionTarget};
use dokime_protocol::host::backend::OverviewGetRequest;
use harmos_protocol::host;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type HttpRequest = http::Request<Body>;
pub type HttpResponse = http::Response<Body>;

/* === Structs === */

pub struct Gate {
    pub entered: Notify,
    pub release: Semaphore,
}

#[derive(Clone)]
pub struct Service {
    pub id: String,
    pub download: Arc<Mutex<delivery::DownloadConfig>>,
}

pub struct Listener {
    pub endpoint: String,
    task: JoinHandle<Result<(), Error>>,
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
            entered: Notify::new(),
            release: Semaphore::new(0),
        }
    }
}

impl Service {
    pub fn new(id: &str) -> Self {
        Self { id: id.into(), download: Arc::default() }
    }
}

impl<const KIND: u8> server::NamedService for Rpc<KIND> {
    const NAME: &'static str = match KIND {
        0 => "dokime.host.v1.Status",
        1 => "dokime.host.v1.backend.Overview",
        _ => "dokime.host.v1.Operations",
    };
}

impl<const KIND: u8> codegen::Service<HttpRequest> for Rpc<KIND> {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = BoxFuture<HttpResponse, Infallible>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                "/dokime.host.v1.Status/Get" => {
                    let mut reply = host::StatusGetResponse::default();
                    let status = reply.status.get_or_insert_default();
                    status.available = true;
                    status.identity.get_or_insert_default().host_id = service.id;
                    let history = reply.resolved_at.get_or_insert_default().history.get_or_insert_default();
                    history.store = "desktop-fixture".into();
                    history.epoch = "status-history".into();
                    server::Grpc::new(ProstCodec::<_, host::StatusGetRequest>::default()).unary(Reply(reply), request).await
                }
                "/dokime.host.v1.backend.Overview/Get" => {
                    let reply = OverviewGetResponse {
                        overview: Some(RuntimeShellSnapshot {
                            session_id: "product-history".into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    server::Grpc::new(ProstCodec::<_, OverviewGetRequest>::default()).unary(Reply(reply), request).await
                }
                "/dokime.host.v1.Operations/List" => {
                    let reply = host::OperationsListResponse {
                        operations: vec![DiscoveredOperation {
                            grpc_available: true,
                            contract: Some(DiscoveredContract {
                                grpc: Some(Grpc {
                                    package: "dokime.host.v1".into(),
                                    service: "Status".into(),
                                    method: "Get".into(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                    };
                    server::Grpc::new(ProstCodec::<_, host::OperationsListRequest>::default()).unary(Reply(reply), request).await
                }
                _ => Status::unimplemented("Fixture Route Is Not Served").into_http(),
            };
            Ok(response)
        })
    }
}

impl<Q: Send + 'static, R: Clone + Send + 'static> server::UnaryService<Q> for Reply<R> {
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
        let listener = TcpListener::bind((ip, 0)).await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(
            Server::builder()
                .add_service(Rpc::<0>(service.clone()))
                .add_service(Rpc::<1>(service.clone()))
                .add_service(Rpc::<2>(service.clone()))
                .add_service(delivery::Content(service.clone()))
                .add_service(delivery::Storage(service))
                .serve_with_incoming(TcpListenerStream::new(listener)),
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
