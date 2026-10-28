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
    timeout: std::pin::Pin<Box<tokio::time::Sleep>>,
    expired: bool,
    aborting: bool,
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
        let _ = self.poll_tasks(&mut context);
    }

    fn poll_tasks(&mut self, context: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        let mut completed = false;
        let mut index = 0;
        while index < self.tasks.len() {
            let pending = &mut self.tasks[index];
            let result = std::future::Future::poll(std::pin::Pin::new(&mut pending.task), context);
            let std::task::Poll::Ready(result) = result else {
                if !pending.expired && std::future::Future::poll(pending.timeout.as_mut(), context).is_ready() {
                    pending.expired = true;
                    let stream = pending.stream.clone();
                    self.refuse(&stream, "host collection timed out".to_owned());
                    completed = true;
                }
                index += 1;
                continue;
            };
            let pending = self.tasks.remove(index);
            self.complete(pending.stream, pending.at, pending.aborting, result);
            completed = true;
        }
        if completed { std::task::Poll::Ready(()) } else { std::task::Poll::Pending }
    }

    fn complete(
        &mut self, stream: String, at: Timestamp, aborting: bool,
        result: std::result::Result<std::result::Result<Option<TelemetryBatch>, String>, tokio::task::JoinError>,
    ) {
        if aborting && result.as_ref().is_err_and(|error| error.is_cancelled()) {
            self.working.remove(&stream);
            return;
        }
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
    fn poll_ready(&mut self, context: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        if !self.ready.is_empty() || self.refused.is_some() {
            return std::task::Poll::Ready(());
        }
        self.poll_tasks(context)
    }

    fn tick(&mut self, at: Timestamp, streams: &[String]) {
        self.reap();
        let now = tokio::time::Instant::now();
        let due = due_streams(streams, &mut self.working, &mut self.next_due, now, |stream| self.collector.interval(stream));
        for stream in due {
            let collector = Arc::clone(&self.collector);
            let source = stream.clone();
            let task = tokio::task::spawn_blocking(move || collector.collect(&source));
            self.tasks.push(Collecting {
                task,
                timeout: Box::pin(tokio::time::sleep_until(now + collection_timeout(&stream))),
                stream,
                at: at.clone(),
                expired: false,
                aborting: false,
            });
        }
    }

    fn settle(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>> {
        Box::pin(async move {
            loop {
                self.reap();
                // A Missed Sample Deadline Does Not Finish Its Blocking Collector.
                // Join The Owned Work Before Reporting Settlement To Shutdown.
                if self.tasks.is_empty() {
                    break;
                }
                std::future::poll_fn(|context| self.poll_tasks(context)).await;
            }
            match self.settlement_failure.take() {
                Some(error) => Err(Box::new(error) as SettlementError),
                None => Ok(()),
            }
        })
    }

    fn abandon(&mut self) {
        self.reap();
        for pending in &mut self.tasks {
            // A Running Blocking Collector Remains Owned Until Its Actual Join.
            pending.aborting = true;
            pending.task.abort();
        }
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
        self.bind_host_telemetry_source(dokime_host::host_telemetry_collector(dokime_host::HostTelemetryConfig {
            source: dokime_host::HostTelemetrySourceKind::Native,
            options: dokime_host::HostOptions::default(),
        }));
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
    let milliseconds = match stream {
        "host.wifi" => 10000,
        "host.wifi.latency" => 4000,
        "host.wifi.channel_occupancy" => 12000,
        "host.bluetooth" => 6000,
        _ => 2000,
    };
    Duration::from_millis(milliseconds)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    struct BlockingCollector {
        entered: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
        release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
    }

    impl HostTelemetryCollector for BlockingCollector {
        fn interval(&self, _: &str) -> Duration {
            Duration::from_secs(1)
        }

        fn collect(&self, _: &str) -> std::result::Result<Option<TelemetryBatch>, String> {
            self.entered.lock().unwrap().take().unwrap().send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            Ok(None)
        }
    }

    #[tokio::test(start_paused = true)]
    async fn expired_collection_joins_before_shutdown_settles() {
        let (entered, started) = tokio::sync::oneshot::channel();
        let (release, released) = std::sync::mpsc::channel();
        let collector = BlockingCollector {
            entered: std::sync::Mutex::new(Some(entered)),
            release: std::sync::Mutex::new(released),
        };
        let sampler = HostSampler { collector: Arc::new(collector) };
        let mut session = sampler.open().unwrap();
        session.tick(crate::model::time::now_utc(), &["host.system".into()]);
        started.await.unwrap();

        tokio::time::advance(collection_timeout("host.system")).await;
        std::future::poll_fn(|context| session.poll_ready(context)).await;
        assert!(session.collected().is_err(), "the sample deadline still reports failure");

        session.abandon();
        {
            let settled = session.settle();
            tokio::pin!(settled);
            tokio::select! {
                biased;
                result = &mut settled => panic!("collection is still physically running: {result:?}"),
                () = tokio::task::yield_now() => {}
            }
            release.send(()).unwrap();
            settled.await.unwrap();
        }
        assert!(session.collected().unwrap().is_empty());
    }
}
