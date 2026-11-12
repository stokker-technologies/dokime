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
use harmos_protocol::v1::{self as h, plugin_client::PluginClient};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

// Local Imports
use crate::{Error, PlacementSettings, Result, SchedulerClock};
use dokime_operations_next::Clock;
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
    send: mpsc::Sender<h::PublishFrame>,
    output: Streaming<h::Samples>,
    interval_ns: i64,
    next_ns: i64,
    sequence: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionTime {
    pub async fn open(
        client: &PluginClient<Channel>, catalog: &h::Catalog, identifiers: &[String], execution: &str, store: Store,
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
            send.send(h::PublishFrame {
                frame: Some(h::publish_frame::Frame::Open(h::PublishRequest {
                    stream: identifier.clone(),
                    scope: execution.into(),
                })),
            })
            .await
            .map_err(|_| Error::Invalid("Reconnect the Plugin Stream".into()))?;
            let output = client.clone().publish(ReceiverStream::new(receive)).await?.into_inner();
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
            let at = prost_types::Timestamp {
                seconds: at_ns.div_euclid(1_000_000_000),
                nanos: at_ns.rem_euclid(1_000_000_000) as i32,
            };
            stream
                .send
                .send(h::PublishFrame {
                    frame: Some(h::publish_frame::Frame::Tick(h::Tick { at: Some(at) })),
                })
                .await
                .map_err(|_| Error::Invalid("Reconnect the Plugin Stream Before Sampling".into()))?;
            let samples =
                stream.output.message().await?.ok_or_else(|| Error::Invalid("Reconnect the Plugin After Its Stream Closed".into()))?;
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
                self.store.sample(
                    &self.execution,
                    Sample {
                        stream: stream.identifier.clone(),
                        position: 0,
                        source_ns,
                        receipt_ns: self.clock.now_ns(),
                        sequence: sample.sequence,
                        row: sample.row,
                    },
                )?;
            }
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
    async fn sleep(&self, duration_ns: i64) -> dokime_operations_next::Result<()> {
        self.advance(duration_ns).await.map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
