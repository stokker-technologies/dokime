/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;
use harmos_protocol::{
    plugin::{InstanceSamplesRequest, InstanceSamplesResponse, instance_client::InstanceClient, instance_samples_request::Frame},
    v1::{Catalog, SamplesSelection, Tick},
};
use prost_types::Timestamp;
use tokio::sync::{
    Mutex,
    mpsc::{self, Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

// Local Imports
use crate::{Error, PlacementSettings, Result, SchedulerClock};
use dokime_operations_next::{Clock, Result as OperationsResult};
use dokime_storage::{Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct ExecutionTime {
    clock: Arc<dyn SchedulerClock>,
    streams: Mutex<Vec<Publication>>,
    store: Store,
    execution: String,
}
struct Publication {
    identifier: String,
    send: Sender<InstanceSamplesRequest>,
    output: Streaming<InstanceSamplesResponse>,
    interval_ns: i64,
    next_ns: i64,
    sequence: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionTime {
    pub async fn open(
        client: &InstanceClient<Channel>, catalog: &Catalog, identifiers: &[String], execution: &str, store: Store,
        clock: Arc<dyn SchedulerClock>, settings: &PlacementSettings,
    ) -> Result<Arc<Self>> {
        let mut streams = Vec::new();
        for identifier in identifiers {
            let declaration = catalog
                .streams
                .iter()
                .find(|stream| &stream.id == identifier)
                .ok_or_else(|| Error::Invalid(format!("Declare the Execution Stream {identifier}")))?;
            let interval =
                declaration.interval.as_ref().ok_or_else(|| Error::Invalid("Configure an Observer Interval for This Stream".into()))?;
            let interval_ns = interval
                .seconds
                .checked_mul(1_000_000_000)
                .and_then(|value| value.checked_add(interval.nanos as i64))
                .filter(|value| *value > 0)
                .ok_or_else(|| Error::Invalid("Set a Positive Observer Interval".into()))?;
            store.open_stream(execution, identifier)?;
            let (send, receive) = mpsc::channel(settings.stream_window.get());
            send.send(InstanceSamplesRequest {
                frame: Some(Frame::Open(SamplesSelection {
                    stream: identifier.clone(),
                    scope: execution.into(),
                })),
            })
            .await
            .map_err(|_| Error::Invalid("Reconnect the Plugin Stream".into()))?;
            let output = client.clone().samples(ReceiverStream::new(receive)).await?.into_inner();
            streams.push(Publication {
                identifier: identifier.clone(),
                send,
                output,
                interval_ns,
                next_ns: clock.now_ns(),
                sequence: 0,
            });
        }
        Ok(Arc::new(Self {
            clock,
            streams: Mutex::new(streams),
            store,
            execution: execution.into(),
        }))
    }
    pub async fn close(&self) -> Result<()> {
        let publications = std::mem::take(&mut *self.streams.lock().await);
        let mut failure = None;
        for publication in publications {
            let Publication { send, mut output, .. } = publication;
            drop(send);
            loop {
                match output.message().await {
                    Ok(Some(_)) => continue,
                    Ok(None) => break,
                    Err(error) => {
                        failure.get_or_insert(Error::Status(error));
                        break;
                    }
                }
            }
        }
        self.store.close_streams(&self.execution)?;
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
    async fn advance(&self, duration_ns: i64) -> Result<()> {
        let end_ns = self
            .clock
            .now_ns()
            .checked_add(duration_ns)
            .ok_or_else(|| Error::Invalid("Choose a Sleep Duration Within the Evidence Clock Range".into()))?;
        let mut streams = self.streams.lock().await;
        while let Some((index, at_ns)) = streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (index, stream.next_ns))
            .filter(|(_, at)| *at <= end_ns)
            .min_by_key(|(_, at)| *at)
        {
            self.clock.wait_until(at_ns).await?;
            let stream = &mut streams[index];
            let at = Timestamp {
                seconds: at_ns.div_euclid(1_000_000_000),
                nanos: at_ns.rem_euclid(1_000_000_000) as i32,
            };
            stream
                .send
                .send(InstanceSamplesRequest {
                    frame: Some(Frame::Tick(Tick { at: Some(at) })),
                })
                .await
                .map_err(|_| Error::Invalid("Reconnect the Plugin Stream Before Sampling".into()))?;
            let samples =
                stream.output.message().await?.ok_or_else(|| Error::Invalid("Reconnect the Plugin After Its Stream Closed".into()))?;
            let mut batch = Vec::with_capacity(samples.samples.len());
            for sample in samples.samples {
                if sample.sequence <= stream.sequence {
                    return Err(Error::Invalid("Reconnect the Plugin After an Out-of-Order Sample".into()));
                }
                stream.sequence = sample.sequence;
                let source = sample.at.unwrap_or(at);
                let source_ns = source
                    .seconds
                    .checked_mul(1_000_000_000)
                    .and_then(|value| value.checked_add(source.nanos as i64))
                    .ok_or_else(|| Error::Invalid("Publish a Source Time Within the Evidence Clock Range".into()))?;
                batch.push(Sample {
                    stream: stream.identifier.clone(),
                    position: 0,
                    source_ns,
                    receipt_ns: self.clock.now_ns(),
                    sequence: sample.sequence,
                    row: sample.row,
                });
            }
            self.store.sample_batch(&self.execution, &batch)?;
            stream.next_ns = stream
                .next_ns
                .checked_add(stream.interval_ns)
                .ok_or_else(|| Error::Invalid("Restart Sampling Within the Clock Range".into()))?;
        }
        self.clock.wait_until(end_ns).await
    }
}
#[async_trait]
impl Clock for ExecutionTime {
    fn now_ns(&self) -> i64 {
        self.clock.now_ns()
    }
    async fn sleep(&self, duration_ns: i64) -> OperationsResult<()> {
        self.advance(duration_ns).await.map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::ExecutionTime;
    use crate::{ControlledClock, PlacementSettings};
    use dokime_operations_next::Clock;
    use dokime_protocol::{client::plots::PlotDomain, v1};
    use dokime_storage::{Execution, SampleWindow, Store};
    use harmos_protocol::{
        plugin::{
            InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
            InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse,
            instance_client::InstanceClient,
            instance_samples_request::Frame,
            instance_server::{Instance, InstanceServer},
        },
        v1::{Catalog, Sample, StreamDeclaration},
    };
    use prost::Message;
    use prost_types::{Duration as ProtoDuration, FileDescriptorSet, Timestamp};
    use std::{num::NonZeroUsize, sync::Arc, time::Duration, vec::IntoIter};
    use tokio::{net::TcpListener, sync::mpsc};
    use tokio_stream::{
        Iter,
        wrappers::{ReceiverStream, TcpListenerStream},
    };
    use tonic::{Request, Response, Status, Streaming, transport::Server};

    struct Producer;

    #[tonic::async_trait]
    impl Instance for Producer {
        async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
            Err(Status::unimplemented("Inspect Test Producer: Only Samples Are Served"))
        }
        type InvokeStream = Iter<IntoIter<Result<InstanceInvokeResponse, Status>>>;
        async fn invoke(&self, _: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
            Err(Status::unimplemented("Invoke Test Producer: Only Samples Are Served"))
        }
        type SamplesStream = ReceiverStream<Result<InstanceSamplesResponse, Status>>;
        async fn samples(&self, request: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
            let mut input = request.into_inner();
            let (sender, receiver) = mpsc::channel(8);
            tokio::spawn(async move {
                let mut sequence = 0;
                while let Some(frame) = input.message().await.unwrap() {
                    if !matches!(frame.frame, Some(Frame::Tick(_))) {
                        continue;
                    }
                    let samples = (0..2)
                        .map(|_| {
                            sequence += 1;
                            Sample {
                                sequence,
                                at: Some(Timestamp {
                                    seconds: 0,
                                    nanos: if sequence % 2 == 0 { 10 } else { 20 },
                                }),
                                row: PlotDomain {
                                    min: sequence as f64,
                                    max: sequence as f64 * 2.0,
                                }
                                .encode_to_vec(),
                            }
                        })
                        .collect();
                    if sender.send(Ok(InstanceSamplesResponse { samples })).await.is_err() {
                        break;
                    }
                }
            });
            Ok(Response::new(ReceiverStream::new(receiver)))
        }
        async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
            Ok(Response::new(InstanceStopResponse {}))
        }
    }

    #[tokio::test]
    async fn tick_batches_preserve_every_sample_and_late_source_time() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let catalog = Catalog {
            types: Some(FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET).unwrap()),
            streams: vec![StreamDeclaration {
                id: "readings".into(),
                row: "dokime.client.v1.plots.PlotDomain".into(),
                interval: Some(ProtoDuration { seconds: 0, nanos: 1_000_000 }),
                ..Default::default()
            }],
            ..Default::default()
        };
        store
            .create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
                }),
                catalog: catalog.encode_to_vec(),
                ..Default::default()
            })
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let (stopped, shutdown) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(Server::builder().add_service(InstanceServer::new(Producer)).serve_with_incoming_shutdown(
            TcpListenerStream::new(listener),
            async {
                let _ = shutdown.await;
            },
        ));
        let client = InstanceClient::connect(endpoint).await.unwrap();
        let settings = PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let time = ExecutionTime::open(
            &client,
            &catalog,
            &["readings".into()],
            "execution",
            store.clone(),
            Arc::new(ControlledClock::new(0)),
            &settings,
        )
        .await
        .unwrap();
        time.sleep(1_000_000).await.unwrap();
        time.close().await.unwrap();
        store.shutdown_samples().await.unwrap();
        let reopened = Store::open(root.path()).unwrap();
        let rows = reopened
            .sample_window(SampleWindow {
                execution: "execution",
                streams: &["readings".into()],
                after: &[],
                through: u64::MAX,
                start_ns: i64::MIN,
                end_ns: i64::MAX,
                latest: false,
                limit: NonZeroUsize::new(10).unwrap(),
            })
            .unwrap();
        assert_eq!(rows.iter().map(|row| row.sequence).collect::<Vec<_>>(), [1, 2, 3, 4]);
        assert_eq!(rows.iter().map(|row| row.source_ns).collect::<Vec<_>>(), [20, 10, 20, 10]);
        assert_eq!(rows.iter().map(|row| PlotDomain::decode(row.row.as_slice()).unwrap().min).collect::<Vec<_>>(), [1.0, 2.0, 3.0, 4.0]);
        stopped.send(()).unwrap();
        server.await.unwrap().unwrap();
    }
}
