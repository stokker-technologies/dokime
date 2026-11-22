/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Capture Another Installed Plugin's Deterministic Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

// Third Party Imports
use harmos_protocol::plugin::{
    InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
    InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse, Sample, instance_invoke_request, instance_invoke_response,
    instance_samples_request,
    instance_server::{Instance, InstanceServer},
    response,
};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage};
use tokio::{
    net::TcpListener,
    sync::{Notify, mpsc, oneshot},
};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Response, Status, Streaming, transport::Server};

// Local Imports
use super::support;
use dokime::templates::{Step, Stream, Template};
use dokime::{Plugin, Suite, Termination, TestCase};
use dokime_protocol::{
    model::origin,
    plugin::{TestCasesMaterializeRequest, TestCasesMaterializeResponse},
};
use dokime_runtime::Host;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "peer_fixture")]
struct Reading {
    value: f64,
}

#[derive(Clone)]
struct Fixture {
    description: InstanceGetResponse,
    sampled: Arc<Notify>,
    entered: Arc<Notify>,
    proceed: Arc<Notify>,
    opened: Arc<AtomicUsize>,
    closed: Arc<AtomicUsize>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Instance for Fixture {
    async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        Ok(Response::new(self.description.clone()))
    }

    type InvokeStream = ReceiverStream<Result<InstanceInvokeResponse, Status>>;

    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let mut input = request.into_inner();
        let Some(instance_invoke_request::Frame::Request(request)) = input.message().await?.and_then(|frame| frame.frame) else {
            return Err(Status::invalid_argument("Invoke Fixture: Supply a Request"));
        };
        let (send, receive) = mpsc::channel(1);
        let fixture = self.clone();
        tokio::spawn(async move {
            let payload = if request.operation.ends_with(".materialize") {
                let parameters = TestCasesMaterializeRequest::decode(request.payload.as_slice()).unwrap().parameters;
                TestCasesMaterializeResponse {
                    test_case: Some(TestCase {
                        parameters,
                        streams: vec!["peer_fixture.system".into()],
                        steps: vec![dokime::Step {
                            metadata: Some(dokime::Metadata {
                                identifier: "capture.wait".into(),
                                ..Default::default()
                            }),
                            operation: "capture.check.wait".into(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                }
                .encode_to_vec()
            } else {
                if request.operation == "capture.check.wait" {
                    fixture.entered.notify_one();
                    tokio::select! {
                        _ = fixture.proceed.notified() => {},
                        _ = input.message() => {},
                    }
                }
                Vec::new()
            };
            let frame = InstanceInvokeResponse {
                frame: Some(instance_invoke_response::Frame::Response(harmos_protocol::plugin::Response {
                    result: Some(response::Result::Payload(payload)),
                })),
            };
            let _ = send.send(Ok(frame)).await;
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }

    type SamplesStream = ReceiverStream<Result<InstanceSamplesResponse, Status>>;

    async fn samples(&self, request: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        let mut input = request.into_inner();
        let Some(instance_samples_request::Frame::Open(selection)) = input.message().await?.and_then(|frame| frame.frame) else {
            return Err(Status::invalid_argument("Sample Fixture: Supply a Selection"));
        };
        let (send, receive) = mpsc::channel(1);
        let fixture = self.clone();
        tokio::spawn(async move {
            if selection.scope.is_empty() {
                while let Ok(Some(_)) = input.message().await {}
                drop(send);
                return;
            }
            assert_eq!(selection.stream, "peer_fixture.system");
            fixture.opened.fetch_add(1, Ordering::SeqCst);
            let frame = InstanceSamplesResponse {
                samples: vec![Sample {
                    row: Reading { value: 42.0 }.encode_to_vec(),
                    sequence: 1,
                    ..Default::default()
                }],
                ..Default::default()
            };
            send.send(Ok(frame)).await.unwrap();
            fixture.sampled.notify_one();
            while let Ok(Some(_)) = input.message().await {}
            fixture.closed.fetch_add(1, Ordering::SeqCst);
            drop(send);
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }

    async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn wait(_: &dokime::templates::Context) -> dokime::Result<()> {
    Ok(())
}

/* === Tests === */

#[test]
fn peer_stream_rows_decode_after_seal_and_bindings_release_after_cancellation() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // Declare Two Independent Fixture Plugins.
        let template =
            Template::of::<()>().identifier("check").label("Check").uses_stream("peer_fixture.system").step(Step::new("Wait", wait));
        let caller = Plugin::of::<()>("1.0.0")
            .identifier("capture_fixture")
            .suite(Suite::of::<()>().identifier("capture").label("Capture").template(template))
            .declarations()?;
        let peer = Plugin::of::<()>("1.0.0")
            .identifier("peer_fixture")
            .stream(Stream::<Reading>::new("peer_fixture.system", "System"))
            .stream(Stream::<Reading>::new("peer_fixture.unused", "Unused"))
            .declarations()?;
        let sampled = Arc::new(Notify::new());
        let entered = Arc::new(Notify::new());
        let proceed = Arc::new(Notify::new());
        let opened = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));
        let root = support::tempdir()?;
        let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
        let mut servers = Vec::new();
        let mut entries = Vec::new();
        for (name, catalog) in [("capture_fixture", caller), ("peer_fixture", peer)] {
            let fixture = Fixture {
                description: InstanceGetResponse {
                    id: name.into(),
                    version: "1.0.0".into(),
                    sdk: "rust".into(),
                    sdk_version: env!("CARGO_PKG_VERSION").into(),
                    message_bytes: support::settings().placement.message_bytes.get() as u64,
                    revision: 1,
                    declarations: Some(catalog),
                    started_at: Some(Default::default()),
                },
                sampled: sampled.clone(),
                entered: entered.clone(),
                proceed: proceed.clone(),
                opened: opened.clone(),
                closed: closed.clone(),
            };
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let endpoint = format!("http://{}", listener.local_addr()?);
            let (stop, stopped) = oneshot::channel();
            let task = tokio::spawn(Server::builder().add_service(InstanceServer::new(fixture)).serve_with_incoming_shutdown(
                TcpListenerStream::new(listener),
                async {
                    let _ = stopped.await;
                },
            ));
            entries.push(host.add_remote(name, &endpoint).await?);
            servers.push((stop, task));
        }

        // Complete and Cancel Runs; Every Peer Demand Must End Before the Run Seals.
        for cancelled in [false, true] {
            let origin = host.materialize(&entries[0], "capture.check", Default::default()).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            sampled.notified().await;
            entered.notified().await;
            if cancelled {
                host.cancel(&cycle.executions[0])?;
            } else {
                proceed.notify_one();
            }
            let result = cycle.wait().await?;
            let snapshot = &result.executions[0];
            assert_eq!(snapshot.execution.termination, if cancelled { Termination::Cancelled } else { Termination::Completed } as i32);
            assert_eq!(opened.load(Ordering::SeqCst), closed.load(Ordering::SeqCst));
            assert_eq!(snapshot.samples.len(), 1);
            let frozen = harmos_protocol::model::Declarations::decode(snapshot.execution.catalog.as_slice())?;
            let declaration = frozen.streams.iter().find(|stream| stream.id == "peer_fixture.system").unwrap();
            let pool = DescriptorPool::from_file_descriptor_set(frozen.types.unwrap())?;
            let row = DynamicMessage::decode(pool.get_message_by_name(&declaration.row).unwrap(), snapshot.samples[0].row.as_slice())?;
            assert_eq!(row.get_field_by_name("value").unwrap().as_f64(), Some(42.0));
            assert!(host.streams_closed(&snapshot.execution.identifier)?);
        }

        // Refuse an Unowned Stream Before Creating Execution Evidence.
        let mut origin = host.materialize(&entries[0], "capture.check", Default::default()).await?;
        let Some(origin::Kind::PluginRun(run)) = origin.kind.as_mut() else { unreachable!() };
        run.test_case.as_mut().unwrap().streams = vec!["missing_fixture.system".into()];
        let refusal = match host.start_cycle(vec![origin]).await {
            Ok(_) => panic!("Refuse an Undeclared Stream"),
            Err(error) => error,
        };
        assert!(refusal.to_string().contains("missing_fixture.system"));

        // Close Only the Fixture Servers This Test Started.
        host.shutdown().await?;
        for (stop, task) in servers {
            let _ = stop.send(());
            task.await??;
        }
        Ok(())
    }))
}
