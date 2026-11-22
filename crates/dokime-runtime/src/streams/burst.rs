/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Collection Burst Admission Through the Execution Sample Consumer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::error::Error;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// Third Party Imports
use harmos_protocol::model::{Declarations, StreamDeclaration};
use harmos_protocol::plugin::{
    CollectionBatch, CollectionDemandResponse, CollectionRows, InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest,
    InstanceInvokeResponse, InstanceSamplesRequest, InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse,
    instance_samples_request, instance_samples_response,
    instance_server::{Instance, InstanceServer},
};
use harmos_runtime::Runtime;
use harmos_runtime::clock::Clock;
use harmos_runtime::operations::{self, CallContext};
use harmos_runtime::plugin::grpc::{Placement, PlacementSettings};
use harmos_runtime::storage::Cursor;
use prost::Message;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Response, Status, Streaming, transport::Server};

// Local Imports
use super::{SampleScope, Samples};
use crate::services::ClientSettings;
use crate::services::composition::Application;
use dokime_operations::native::Binding;
use dokime_protocol::model::{Import, Origin, origin};
use dokime_storage::{Execution, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "fixture")]
struct Reading {
    value: u64,
}

#[derive(Clone)]
struct Fixture {
    description: InstanceGetResponse,
    burst: u64,
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

    async fn invoke(&self, _: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        Err(Status::unimplemented("Invoke Fixture: Use Its Collection"))
    }

    type SamplesStream = ReceiverStream<Result<InstanceSamplesResponse, Status>>;

    async fn samples(&self, request: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        let mut input = request.into_inner();
        let (send, receive) = mpsc::channel(1);
        let burst = self.burst;
        tokio::spawn(async move {
            while let Some(request) = input.message().await.unwrap() {
                let frame = match request.frame.unwrap() {
                    instance_samples_request::Frame::Open(_) => continue,
                    instance_samples_request::Frame::Demand(_) => instance_samples_response::Frame::Demand(CollectionDemandResponse {}),
                    instance_samples_request::Frame::Window(window) => {
                        assert_eq!(window.count, 1);
                        instance_samples_response::Frame::Collection(CollectionBatch {
                            offset: 0,
                            last: true,
                            streams: vec![CollectionRows {
                                stream: "readings".into(),
                                rows: (0..burst)
                                    .map(|index| {
                                        Reading {
                                            value: (window.sequence - 1) * burst + index,
                                        }
                                        .encode_to_vec()
                                    })
                                    .collect(),
                            }],
                        })
                    }
                    _ => panic!("Expected Collection Demand or Window"),
                };
                if send.send(Ok(InstanceSamplesResponse { samples: vec![], frame: Some(frame) })).await.is_err() {
                    return;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }

    async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn capture(burst: u64, window: usize) -> Result<(Vec<u64>, Option<String>), Box<dyn Error>> {
    // Admit the Same Row and Collection Shape as a Scan, Without Calling a Platform Collector.
    let mut catalog = Declarations {
        types: Some(<Reading as dokime::encoding::contract::Message>::types()),
        streams: vec![StreamDeclaration {
            id: "readings".into(),
            row: <Reading as dokime::encoding::contract::Message>::name(),
            collection: "readings".into(),
            interval: Some(prost_types::Duration { seconds: 60, nanos: 0 }),
            ..Default::default()
        }],
        ..Default::default()
    };
    catalog.digest = harmos_protocol::schema::declarations_digest(&catalog);
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&Execution {
        identifier: "execution".into(),
        cycle: "cycle".into(),
        origin: Some(Origin {
            kind: Some(origin::Kind::Import(Import::default())),
        }),
        catalog: catalog.encode_to_vec(),
        ..Default::default()
    })?;
    store.open_stream("execution", "readings")?;
    let failures = Arc::new(Mutex::new(HashMap::new()));
    let scopes = Arc::new(Mutex::new(HashMap::from([("execution".into(), SampleScope::new(&catalog, &["readings".into()])?)])));

    // Attach the Real Execution Consumer and Freeze Time Between Collection Ticks.
    let (clock, controller) = Clock::virtual_at(SystemTime::UNIX_EPOCH);
    let settings = ClientSettings {
        read_timeout: crate::PLUGINS_TIMEOUT,
        mutation_timeout: crate::PLUGINS_TIMEOUT,
        watch_timeout: harmos_protocol::policy::UNBOUNDED,
        max_cases: 1000,
        page_items: 500,
        message_bytes: 4 * 1024 * 1024,
        stream_buffer: window,
    };
    let runtime = Runtime::<Application>::start_with((), Some(settings), |setup| {
        setup.resource::<Binding>(Arc::new(Binding::default()));
        setup.settings().clock(clock.clone()).operation_settings(operations::Settings {
            preparation_bytes: NonZeroUsize::new(4 * 1024 * 1024),
            ..Default::default()
        });
        setup.settings().attach_streams(Samples {
            store: store.clone(),
            cursor: Cursor::ORIGIN,
            failures: failures.clone(),
            scopes,
        });
    })
    .await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let fixture = Fixture {
        description: InstanceGetResponse {
            id: "fixture".into(),
            version: "1.0.0".into(),
            sdk: "rust".into(),
            sdk_version: env!("CARGO_PKG_VERSION").into(),
            message_bytes: harmos_protocol::transfer::default_message_bytes().get() as u64,
            revision: 1,
            started_at: Some(Default::default()),
            declarations: Some(catalog),
        },
        burst,
    };
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(Server::builder().add_service(InstanceServer::new(fixture)).serve_with_incoming_shutdown(
        TcpListenerStream::new(listener),
        async {
            let _ = stopped.await;
        },
    ));
    let attached = runtime
        .attach_grpc(
            Placement::Remote(endpoint),
            PlacementSettings {
                reconnect_delay: Duration::from_secs(1),
                stop_grace: crate::STOP_GRACE,
                stream_window: NonZeroUsize::new(window).unwrap(),
                interval_window: Duration::from_secs(1),
                interval_lag: Default::default(),
            },
        )
        .await?;
    let binding = runtime.bind_grpc(&attached.activation().unwrap(), "execution", clock, &CallContext::default()).await?;

    // A Single-Thread Executor Cannot Drain Until the Publisher Awaits; Two Ticks Reveal a Terminated Demand.
    binding.start_samples(["readings"]).await?;
    controller.advance_by(Duration::from_secs(120)).await;
    let outcome = binding.stop_samples().await.err().map(|error| error.to_string());
    binding.close().await?;
    attached.stop("Fixture Complete").await?;
    runtime.stop().await?;
    let _ = stop.send(());
    server.await??;

    // Verify the Consumer Preserved Every Accepted Row, Independently of the Publisher's Outcome.
    assert!(failures.lock().unwrap().is_empty());
    store.flush_samples()?;
    let through = store.cut("execution")?.frontier;
    let saved = store.sample_page("execution", "readings", 0, through, NonZeroUsize::new(100).unwrap())?;
    let values = saved.into_iter().map(|sample| Reading::decode(sample.row.as_slice()).map(|row| row.value)).collect::<Result<_, _>>()?;
    Ok((values, outcome))
}

/* === Tests === */

#[tokio::test]
async fn collection_burst_larger_than_the_window_exposes_library_backlog() -> Result<(), Box<dyn Error>> {
    let (rows, failure) = capture(3, 2).await?;
    assert_eq!(rows, [0, 1]);
    assert!(failure.unwrap().contains("the stream window is full of rows its ingest has not taken in"));
    Ok(())
}

#[tokio::test]
async fn execution_ingest_drains_every_row_of_successive_collection_bursts() -> Result<(), Box<dyn Error>> {
    let (rows, failure) = capture(3, 3).await?;
    assert_eq!(rows, [0, 1, 2, 3, 4, 5]);
    assert!(failure.is_none(), "{failure:?}");
    Ok(())
}
