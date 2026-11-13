/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-Process Generated-Message Client Server

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

// Third Party Imports
use harmos_protocol::v1 as host;
use prost::Message;
use prost_reflect::ReflectMessage;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{
    Request, Response, Status,
    codegen::{Body, BoxFuture, Service, StdError, http},
    server::NamedService,
};

// Local Imports
use super::TestResult;
use dokime::{
    client::{CatalogEntriesListResponse, DiscoveredContract, DiscoveredOperation},
    v1,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FakeServer {
    pub channel: tonic::transport::Channel,
    stop: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

#[derive(Clone)]
struct Rpc<const DISCOVERY: bool>;

struct Discover;

struct Health;

struct Watch;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FakeServer {
    pub async fn start() -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let (stop, stopped) = oneshot::channel();
        let task = tokio::spawn(
            tonic::transport::Server::builder().add_service(Rpc::<true>).add_service(Rpc::<false>).serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async {
                    let _ = stopped.await;
                },
            ),
        );
        let channel = tonic::transport::Endpoint::from_shared(format!("http://{address}"))?.connect().await?;
        Ok(Self { channel, stop, task })
    }
    pub async fn close(self) -> TestResult<()> {
        let _ = self.stop.send(());
        self.task.await??;
        Ok(())
    }
}

impl<const D: bool> NamedService for Rpc<D> {
    const NAME: &'static str = if D { "harmos.v1.CatalogEntries" } else { "dokime.v1.Backend" };
}

impl<const D: bool, B: Body + Send + 'static> Service<http::Request<B>> for Rpc<D>
where
    B::Error: Into<StdError> + Send,
{
    type Response = http::Response<tonic::body::Body>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let path = request.uri().path().to_owned();
        Box::pin(async move {
            if D {
                return Ok(tonic::server::Grpc::new(tonic_prost::ProstCodec::default()).unary(Discover, request).await);
            }
            if path == "/dokime.v1.Backend/RunWatchLogs" {
                return Ok(tonic::server::Grpc::new(tonic_prost::ProstCodec::default()).streaming(Watch, request).await);
            }
            Ok(tonic::server::Grpc::new(tonic_prost::ProstCodec::default()).unary(Health, request).await)
        })
    }
}

impl tonic::server::UnaryService<host::CatalogEntriesListRequest> for Discover {
    type Response = CatalogEntriesListResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<host::CatalogEntriesListRequest>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            let operations = [contract(false), contract(true)]
                .into_iter()
                .map(|contract| DiscoveredOperation {
                    owner: "host".into(),
                    contract: Some(contract),
                    grpc_available: true,
                    ..Default::default()
                })
                .collect();
            Ok(Response::new(CatalogEntriesListResponse { operations, ..Default::default() }))
        })
    }
}

impl tonic::server::UnaryService<v1::GetBackendHealth> for Health {
    type Response = v1::BackendHealth;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<v1::GetBackendHealth>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            Ok(Response::new(health()))
        })
    }
}

impl tonic::server::StreamingService<v1::BackendWatchLogsRunInput> for Watch {
    type Response = v1::BackendWatchLogsRunOutput;
    type ResponseStream = Pin<Box<dyn futures_util::Stream<Item = Result<Self::Response, Status>> + Send>>;
    type Future = BoxFuture<Response<Self::ResponseStream>, Status>;
    fn call(&mut self, request: Request<tonic::Streaming<v1::BackendWatchLogsRunInput>>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            let first = request.into_inner().message().await?.ok_or_else(|| Status::invalid_argument("Missing Opening"))?;
            if !matches!(first.value, Some(v1::backend_watch_logs_run_input::Value::Request(_))) {
                return Err(Status::invalid_argument("Missing Request"));
            }
            let frame = v1::LogFrame {
                time: Some(prost_types::Timestamp { seconds: 100, nanos: 0 }),
                level: v1::LogLevel::Info as i32,
                target: "sdk.story".into(),
                ..Default::default()
            };
            let output: Self::ResponseStream = Box::pin(tokio_stream::iter([
                Ok(v1::BackendWatchLogsRunOutput {
                    value: Some(v1::backend_watch_logs_run_output::Value::Item(frame)),
                }),
                Ok(v1::BackendWatchLogsRunOutput {
                    value: Some(v1::backend_watch_logs_run_output::Value::Response(v1::BackendWatchLogsCompletion {})),
                }),
            ]));
            Ok(Response::new(output))
        })
    }
}

/* === Public Functions === */

pub fn contract(stream: bool) -> DiscoveredContract {
    let descriptor = v1::GetBackendHealth::default().descriptor();
    let descriptors = prost_types::FileDescriptorSet {
        file: descriptor.parent_pool().files().map(|file| file.file_descriptor_proto().clone()).collect(),
    }
    .encode_to_vec();
    let protobuf = host::Protobuf {
        descriptors,
        request: if stream { "dokime.v1.WatchBackendLogs" } else { "dokime.v1.GetBackendHealth" }.into(),
        response: if stream { "dokime.v1.BackendWatchLogsCompletion" } else { "dokime.v1.BackendHealth" }.into(),
        input: None,
        output: stream.then(|| "dokime.v1.LogFrame".into()),
    };
    let id = if stream { "dokime.backend.watch_logs" } else { "dokime.backend.get_health" };
    DiscoveredContract {
        id: id.into(),
        visibility: host::Visibility::Public as i32,
        fingerprint: harmos_protocol::schema::host_fingerprint(id, &protobuf).unwrap(),
        protobuf: Some(protobuf),
        grpc: Some(host::Grpc {
            managed: stream,
            package: "dokime.v1".into(),
            service: "Backend".into(),
            method: if stream { "RunWatchLogs" } else { "GetHealth" }.into(),
        }),
        ..Default::default()
    }
}

/* === Private Functions === */

fn authorize<T>(request: &Request<T>) -> Result<(), Status> {
    if request.metadata().get("authorization").and_then(|value| value.to_str().ok()) != Some("Bearer story") {
        return Err(Status::unauthenticated("Supply the Story Token"));
    }
    Ok(())
}

fn health() -> v1::BackendHealth {
    v1::BackendHealth {
        backend_id: "bench".into(),
        epoch: "boot".into(),
        readiness: v1::BackendReadiness::Ready as i32,
        observed_at: Some(prost_types::Timestamp { seconds: 100, nanos: 0 }),
        message: "x".repeat(4096),
    }
}

/* === Tests === */
