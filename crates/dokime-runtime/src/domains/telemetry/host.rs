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
pub use dokime_host::HostTelemetryCollector;

/* === Imports === */
// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};
// Third Party Imports

// Local Imports
use super::{Sampler, SamplerSession, SettlementError};
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::{SamplingScope, TelemetryBatch, TelemetryCaptureLease, TelemetrySamplingRequest};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct HostTelemetrySource {
    collector: Arc<dyn HostTelemetryCollector>,
}

struct HostSampler {
    collector: Arc<dyn HostTelemetryCollector>,
}

struct HostSamplerSession {
    collector: Arc<dyn HostTelemetryCollector>,
    working: BTreeSet<String>,
    next_due: BTreeMap<String, tokio::time::Instant>,
    ready: Vec<(String, TelemetryBatch)>,
    refused: Option<String>,
    tasks: Vec<Collecting>,
    settlement_failure: Option<tokio::task::JoinError>,
}

struct Collecting {
    task: tokio::task::JoinHandle<std::result::Result<Option<TelemetryBatch>, String>>,
    stream: String,
    at: Timestamp,
    deadline: tokio::time::Instant,
    expired: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl HostTelemetrySource {
    pub fn new(collector: Arc<dyn HostTelemetryCollector>) -> Self {
        Self { collector }
    }

    fn sampler(&self) -> HostSampler {
        HostSampler { collector: self.collector.clone() }
    }
}

impl Sampler for HostSampler {
    fn open(&self) -> std::result::Result<Box<dyn SamplerSession>, String> {
        Ok(Box::new(HostSamplerSession {
            collector: self.collector.clone(),
            working: BTreeSet::new(),
            next_due: BTreeMap::new(),
            ready: Vec::new(),
            refused: None,
            tasks: Vec::new(),
            settlement_failure: None,
        }))
    }
}

impl HostSamplerSession {
    fn reap(&mut self) {
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        let now = tokio::time::Instant::now();
        let mut index = 0;
        while index < self.tasks.len() {
            let pending = &mut self.tasks[index];
            let result = std::future::Future::poll(std::pin::Pin::new(&mut pending.task), &mut context);
            let std::task::Poll::Ready(result) = result else {
                if !pending.expired && now >= pending.deadline {
                    pending.expired = true;
                    let stream = pending.stream.clone();
                    self.refuse(&stream, "host collection timed out".to_owned());
                }
                index += 1;
                continue;
            };
            let pending = self.tasks.remove(index);
            self.complete(pending.stream, pending.at, result);
        }
    }

    fn complete(
        &mut self, stream: String, at: Timestamp,
        result: std::result::Result<std::result::Result<Option<TelemetryBatch>, String>, tokio::task::JoinError>,
    ) {
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                let message = format!("host collection task failed: {error}");
                self.settlement_failure.get_or_insert(error);
                Err(message)
            }
        };
        match outcome {
            Ok(Some(mut batch)) => {
                for sample in &mut batch.samples {
                    sample.timestamp = at.clone();
                }
                self.ready.push((stream, batch));
            }
            Ok(None) => {
                self.working.remove(&stream);
            }
            Err(error) => {
                self.working.remove(&stream);
                self.refuse(&stream, error);
            }
        }
    }

    fn refuse(&mut self, stream: &str, error: String) {
        tracing::warn!(stream, %error, "host telemetry stream could not be collected on this tick");
        self.refused.get_or_insert(format!("{stream}: {error}"));
    }
}

impl SamplerSession for HostSamplerSession {
    fn tick(&mut self, at: Timestamp, streams: &[String]) {
        self.reap();
        let now = tokio::time::Instant::now();
        let due = due_streams(streams, &mut self.working, &mut self.next_due, now, |stream| self.collector.interval(stream));
        for stream in due {
            let collector = Arc::clone(&self.collector);
            let source = stream.clone();
            let task = tokio::task::spawn_blocking(move || collector.collect(&source));
            self.tasks.push(Collecting { task, deadline: now + collection_timeout(&stream), stream, at: at.clone(), expired: false });
        }
    }

    fn settle(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>> {
        Box::pin(async move {
            while let Some(pending) = self.tasks.last_mut() {
                let result = (&mut pending.task).await;
                let pending = self.tasks.pop().expect("the original collector remains owned");
                self.complete(pending.stream, pending.at, result);
            }
            self.reap();
            match self.settlement_failure.take() {
                Some(error) => Err(Box::new(error) as SettlementError),
                None => Ok(()),
            }
        })
    }

    fn collected(&mut self) -> std::result::Result<Vec<TelemetryBatch>, String> {
        self.reap();
        let ready = std::mem::take(&mut self.ready)
            .into_iter()
            .map(|(stream, batch)| {
                self.working.remove(&stream);
                batch
            })
            .collect::<Vec<_>>();
        let refused = self.refused.take();
        // A stream that failed is that stream's problem while its neighbours
        // are still producing. A request whose every stream has failed is a
        // source that failed, and says so.
        match (ready.is_empty(), refused) {
            (true, Some(error)) => Err(error),
            _ => Ok(ready),
        }
    }
}

impl RuntimeHandle {
    pub fn bind_host_telemetry_source(&self, collector: Arc<dyn HostTelemetryCollector>) {
        *self.host_telemetry_source.write().expect("host telemetry source lock is healthy") =
            Some(Arc::new(HostTelemetrySource::new(collector)));
    }

    pub fn bind_native_host_telemetry(&self) {
        self.bind_host_telemetry_source(dokime_host::default_host_telemetry_collector());
    }

    pub async fn request_host_sampling(
        &self, scope: SamplingScope, requests: Vec<TelemetrySamplingRequest>,
    ) -> Result<Vec<TelemetryCaptureLease>> {
        self.request_host_sampling_bound(scope, requests, |_| Ok(())).await
    }

    pub(crate) async fn request_host_sampling_bound(
        &self, scope: SamplingScope, requests: Vec<TelemetrySamplingRequest>,
        bind: impl FnOnce(&[TelemetryCaptureLease]) -> Result<()> + Send,
    ) -> Result<Vec<TelemetryCaptureLease>> {
        let source = self
            .host_telemetry_source
            .read()
            .expect("host telemetry source lock is healthy")
            .clone()
            .ok_or_else(|| Error::Operation("host telemetry source is unavailable".to_owned()))?;
        let producer_key = host_producer_key();
        let sampler = Arc::new(source.sampler());
        self.request_grouped_sampling(scope, requests, sampler, producer_key, bind).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn due_streams<T: Copy + Ord + std::ops::Add<Duration, Output = T>>(
    streams: &[String], working: &mut BTreeSet<String>, next_due: &mut BTreeMap<String, T>, now: T, interval: impl Fn(&str) -> Duration,
) -> Vec<String> {
    let mut due = Vec::new();
    for stream in streams {
        if next_due.get(stream).is_some_and(|deadline| now < *deadline) {
            continue;
        }
        if working.contains(stream) {
            tracing::trace!(stream, "host telemetry stream is still collecting and skips this tick");
            continue;
        }
        working.insert(stream.clone());
        next_due.insert(stream.clone(), now + interval(stream));
        due.push(stream.clone());
    }
    due
}

fn host_producer_key() -> String {
    crate::telemetry_runtime::leases::HOST_PRODUCER_KEY.to_owned()
}

fn collection_timeout(stream: &str) -> Duration {
    match stream {
        "host.wifi" => Duration::from_secs(10),
        "host.wifi.latency" => Duration::from_secs(4),
        "host.wifi.channel_occupancy" => Duration::from_secs(12),
        "host.bluetooth" => Duration::from_secs(6),
        _ => Duration::from_secs(2),
    }
}

/* === Tests === */
