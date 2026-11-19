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
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

// Third Party Imports
use futures_util::Stream;
use harmos_protocol::host::OperationsListRequest;
use harmos_protocol::model::{Grpc, History, Position, Protobuf, ReadCut, Receipt, Visibility};
use prost::Message;
use prost_reflect::ReflectMessage;
use prost_types::FileDescriptorSet;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Request, Response, Status, body,
    codegen::{Body, BoxFuture, Service, StdError, http},
    server::{self, NamedService, ServerStreamingService, UnaryService},
    transport::{self, Channel, Endpoint, Server},
};
use tonic_prost::ProstCodec;

// Local Imports
use super::TestResult;
use dokime::client::CycleSummary;
use dokime::{DiscoveredContract, DiscoveredOperation, Metadata, OperationsListResponse};
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesCreateResponse;
use dokime_protocol::host::ExecutionDetail;
use dokime_protocol::host::ExecutionSummary;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsListResponse;
use dokime_protocol::host::backend::OverviewGetRequest;
use dokime_protocol::host::backend::OverviewGetResponse;
use dokime_protocol::host::backend::OverviewWatchRequest;
use dokime_protocol::host::backend::OverviewWatchResponse;
use dokime_protocol::host::backend::RuntimeShellSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FakeServer {
    pub channel: Channel,
    stop: oneshot::Sender<()>,
    task: JoinHandle<Result<(), transport::Error>>,
}

#[derive(Clone)]
struct Rpc<const RESOURCE: u8>(Arc<Mutex<Option<CreatedCycle>>>);

struct CreatedCycle {
    name: String,
    executions: Vec<ExecutionDetail>,
    at: ReadCut,
}

struct CreateCycle(Arc<Mutex<Option<CreatedCycle>>>);

struct ListExecutions(Arc<Mutex<Option<CreatedCycle>>>);

struct Discover;

struct Overview;

struct Watch;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FakeServer {
    pub async fn start() -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let (stop, stopped) = oneshot::channel();
        let cycle = Arc::new(Mutex::new(None));
        let task = tokio::spawn(
            Server::builder()
                .add_service(Rpc::<0>(cycle.clone()))
                .add_service(Rpc::<1>(cycle.clone()))
                .add_service(Rpc::<2>(cycle.clone()))
                .add_service(Rpc::<3>(cycle))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                }),
        );
        let channel = Endpoint::from_shared(format!("http://{address}"))?.connect().await?;
        Ok(Self { channel, stop, task })
    }
    pub async fn close(self) -> TestResult<()> {
        let _ = self.stop.send(());
        self.task.await??;
        Ok(())
    }
}

impl<const R: u8> NamedService for Rpc<R> {
    const NAME: &'static str = match R {
        0 => "harmos.host.v1.Operations",
        1 => "dokime.host.v1.backend.Overview",
        2 => "dokime.host.v1.Cycles",
        _ => "dokime.host.v1.Executions",
    };
}

impl<const R: u8, B: Body + Send + 'static> Service<http::Request<B>> for Rpc<R>
where
    B::Error: Into<StdError> + Send,
{
    type Response = http::Response<body::Body>;
    type Error = Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: http::Request<B>) -> Self::Future {
        let path = request.uri().path().to_owned();
        let cycle = self.0.clone();
        Box::pin(async move {
            if R == 2 {
                return Ok(server::Grpc::new(ProstCodec::default()).unary(CreateCycle(cycle), request).await);
            }
            if R == 3 {
                return Ok(server::Grpc::new(ProstCodec::default()).unary(ListExecutions(cycle), request).await);
            }
            if R == 0 {
                return Ok(server::Grpc::new(ProstCodec::default()).unary(Discover, request).await);
            }
            if path == "/dokime.host.v1.backend.Overview/Watch" {
                return Ok(server::Grpc::new(ProstCodec::default()).server_streaming(Watch, request).await);
            }
            Ok(server::Grpc::new(ProstCodec::default()).unary(Overview, request).await)
        })
    }
}

impl UnaryService<OperationsListRequest> for Discover {
    type Response = OperationsListResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<OperationsListRequest>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            let operations = [contract()]
                .into_iter()
                .map(|contract| DiscoveredOperation {
                    owner: "host".into(),
                    contract: Some(contract),
                    grpc_available: true,
                    ..Default::default()
                })
                .collect();
            Ok(Response::new(OperationsListResponse { operations }))
        })
    }
}

impl UnaryService<OverviewGetRequest> for Overview {
    type Response = OverviewGetResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<OverviewGetRequest>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            Ok(Response::new(overview()))
        })
    }
}

impl ServerStreamingService<OverviewWatchRequest> for Watch {
    type Response = OverviewWatchResponse;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<Self::Response, Status>> + Send>>;
    type Future = BoxFuture<Response<Self::ResponseStream>, Status>;
    fn call(&mut self, request: Request<OverviewWatchRequest>) -> Self::Future {
        Box::pin(async move {
            authorize(&request)?;
            let output: Self::ResponseStream = Box::pin(tokio_stream::iter([Ok(OverviewWatchResponse {
                overview: overview().overview,
                opening: true,
                ..Default::default()
            })]));
            Ok(Response::new(output))
        })
    }
}

impl UnaryService<CyclesCreateRequest> for CreateCycle {
    type Response = CyclesCreateResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<CyclesCreateRequest>) -> Self::Future {
        let state = self.0.clone();
        Box::pin(async move {
            let request = request.into_inner();
            if request.request_id.is_empty() || !request.continue_automatically {
                return Err(Status::invalid_argument("Create Cycle: Supply Its Identity and Automatic Admission"));
            }
            let name = request.request_id.clone();
            let position = Position {
                history: Some(History { store: "sdk".into(), epoch: name.clone() }),
                sequence: 1,
            };
            let executions = request
                .origins
                .into_iter()
                .enumerate()
                .map(|(index, origin)| ExecutionDetail {
                    origin: Some(origin),
                    cycle_id: name.clone(),
                    summary: Some(ExecutionSummary {
                        execution_id: format!("{name}/{index}"),
                        case_id: format!("case/{index}"),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            let cycle = CycleSummary {
                cycle_id: name.clone(),
                execution_count: executions.len() as u64,
                ..Default::default()
            };
            *state.lock().unwrap() = Some(CreatedCycle {
                name,
                executions,
                at: ReadCut {
                    positions: vec![position.clone()],
                    revisions: Vec::new(),
                },
            });
            Ok(Response::new(CyclesCreateResponse {
                cycle: Some(cycle),
                receipt: Some(Receipt {
                    request_id: request.request_id,
                    durable_at: Some(position),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        })
    }
}

impl UnaryService<ExecutionsListRequest> for ListExecutions {
    type Response = ExecutionsListResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<ExecutionsListRequest>) -> Self::Future {
        let state = self.0.clone();
        Box::pin(async move {
            let request = request.into_inner();
            let state = state.lock().unwrap();
            let cycle = state.as_ref().ok_or_else(|| Status::not_found("List Executions: Create a Cycle First"))?;
            if request.parent != cycle.name || request.at.as_ref().is_some_and(|at| at != &cycle.at) {
                return Err(Status::invalid_argument("List Executions: Retain the Created Cycle and Its Durable Cut"));
            }
            let offset = request
                .page
                .as_ref()
                .map(|page| page.page_token.as_str())
                .filter(|token| !token.is_empty())
                .map(str::parse::<usize>)
                .transpose()
                .map_err(|_| Status::invalid_argument("List Executions: Invalid Continuation"))?
                .unwrap_or(0);
            if offset != 0 && request.at.as_ref() != Some(&cycle.at) {
                return Err(Status::invalid_argument("List Executions: Retain the First Page's Durable Cut"));
            }
            let executions = cycle.executions.iter().skip(offset).take(1).cloned().collect::<Vec<_>>();
            let next_page_token =
                if offset + executions.len() < cycle.executions.len() { (offset + executions.len()).to_string() } else { String::new() };
            Ok(Response::new(ExecutionsListResponse {
                executions,
                resolved_at: Some(cycle.at.clone()),
                next_page_token,
                ..Default::default()
            }))
        })
    }
}

/* === Public Functions === */

pub fn contract() -> DiscoveredContract {
    let descriptor = OverviewGetRequest::default().descriptor();
    let descriptors = FileDescriptorSet {
        file: descriptor.parent_pool().files().map(|file| file.file_descriptor_proto().clone()).collect(),
    }
    .encode_to_vec();
    let protobuf = Protobuf {
        descriptors,
        request: "dokime.host.v1.backend.OverviewGetRequest".into(),
        response: "dokime.host.v1.backend.OverviewGetResponse".into(),
        input: None,
        output: None,
    };
    let id = "dokime.host.v1.backend.Overview/Get";
    DiscoveredContract {
        id: id.into(),
        visibility: Visibility::Public as i32,
        fingerprint: harmos_protocol::schema::host_fingerprint(id, &protobuf).unwrap(),
        protobuf: Some(protobuf),
        grpc: Some(Grpc {
            managed: false,
            package: "dokime.host.v1.backend".into(),
            service: "Overview".into(),
            method: "Get".into(),
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

fn overview() -> OverviewGetResponse {
    OverviewGetResponse {
        overview: Some(RuntimeShellSnapshot {
            session_id: "bench".into(),
            hub_metadata: Some(Metadata {
                description: "x".repeat(4096),
                ..Default::default()
            }),
            ..Default::default()
        }),
        resolved_at: None,
    }
}

/* === Tests === */
