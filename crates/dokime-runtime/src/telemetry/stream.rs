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

pub use dokime_domain::model::telemetry::TelemetryStream;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::telemetry::TelemetryRow;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;

/* === Constants === */

// The longest a buffered row waits before its sink is flushed.
#[cfg(feature = "native")]
const FLUSH_WINDOW: std::time::Duration = std::time::Duration::from_millis(10);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Streams {
    sink: Arc<dyn SampleSink>,

    #[cfg(feature = "native")]
    admission: Option<Arc<Admission>>,

    #[cfg(feature = "native")]
    failure: tokio::sync::watch::Sender<Option<crate::diagnostics::Diagnostic>>,

    #[cfg(feature = "native")]
    listening: tokio::sync::broadcast::Sender<crate::execution::PublishedRow>,

    #[cfg(feature = "native")]
    flushing: Arc<std::sync::atomic::AtomicBool>,
}

pub struct StreamSelection<'a, R> {
    streams: Streams,
    row: PhantomData<fn(R)>,
    borrowed: PhantomData<&'a ()>,
}

pub struct Pushing<'a, R> {
    stream: Identifier,
    streams: Streams,
    row: PhantomData<fn(R)>,
    borrowed: PhantomData<&'a ()>,
}

#[cfg(feature = "native")]
struct Admission {
    template: String,
    streams: std::collections::BTreeSet<String>,
}

/* === Enums === */

/* === Traits === */

pub trait SampleSink: Send + Sync {
    fn publish<'a>(
        &'a self, stream: &'a Identifier, values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>,
    ) -> crate::transport::TransportFuture<'a, Result<(), String>>;

    fn produced(&self, stream: &str) -> u64;

    // Publish every buffered row now, naming the first stream that failed.
    fn flush(&self) -> crate::transport::TransportFuture<'_, Result<(), (Identifier, String)>> {
        Box::pin(async { Ok(()) })
    }

    // Whether rows are waiting for a flush.
    fn buffering(&self) -> bool {
        false
    }
}

pub trait OwnedStream: TelemetryStream {}

/* === Implementations === */

impl Streams {
    pub fn new(sink: Arc<dyn SampleSink>) -> Self {
        Self {
            sink,
            #[cfg(feature = "native")]
            admission: None,
            #[cfg(feature = "native")]
            failure: tokio::sync::watch::channel(None).0,
            #[cfg(feature = "native")]
            listening: crate::execution::listening_rows(),
            #[cfg(feature = "native")]
            flushing: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    #[cfg(feature = "native")]
    pub(crate) fn with_listening(mut self, listening: tokio::sync::broadcast::Sender<crate::execution::PublishedRow>) -> Self {
        self.listening = listening;
        self
    }

    #[cfg(feature = "native")]
    pub(crate) fn restricted(mut self, template: &str, streams: impl IntoIterator<Item = String>) -> Self {
        self.admission = Some(Arc::new(Admission { template: template.to_owned(), streams: streams.into_iter().collect() }));
        self
    }

    #[cfg(feature = "native")]
    pub(crate) fn failure(&self) -> Option<crate::diagnostics::Diagnostic> {
        self.failure.borrow().clone()
    }

    #[cfg(feature = "native")]
    pub(crate) async fn refused(&self) -> crate::diagnostics::Diagnostic {
        let mut receiver = self.failure.subscribe();
        let value = receiver.wait_for(Option::is_some).await.expect("the grant retains its sender");
        value.as_ref().expect("a refusal is present").clone()
    }

    #[cfg(feature = "native")]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<crate::execution::PublishedRow> {
        self.listening.subscribe()
    }

    pub(crate) fn selecting<R: TelemetryRow>(&self) -> StreamSelection<'_, R> {
        StreamSelection { streams: self.clone(), row: PhantomData, borrowed: PhantomData }
    }

    pub(crate) fn produced(&self, stream: &str) -> u64 {
        self.sink.produced(stream)
    }

    // Publish every row the sink still buffers; a step ending, a stop and a
    // record that follows its rows all wait for this.
    pub(crate) async fn flush(&self) {
        if let Err((stream, error)) = self.sink.flush().await {
            self.failed(&stream, &error);
        }
    }

    #[cfg(feature = "native")]
    pub(crate) async fn published(&self, stream: &Identifier, values: BTreeMap<String, Value>, payload: Vec<u8>) {
        self.published_as(stream, values, Some(payload), stream.as_str()).await;
    }

    pub(crate) async fn published_as(
        &self, stream: &Identifier, mut values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>, row_type: &str,
    ) {
        // The one place every row this execution produces is stamped: a
        // observer-measured row and a step-pushed row both land here before
        // the subscription clone and before the sink, so whichever one it is,
        // this is the production instant. A row that already stamped its own
        // "timestamp" — an observer reading a device's own clock, say — keeps
        // it; `or_insert_with` never overwrites.
        values.entry("timestamp".to_owned()).or_insert_with(|| Value::from(crate::model::time::now_utc()));

        #[cfg(not(feature = "native"))]
        let _ = row_type;
        #[cfg(feature = "native")]
        if let Some(admission) = &self.admission
            && !admission.streams.contains(stream.as_str())
        {
            self.failure.send_if_modified(|failure| {
                if failure.is_some() { return false; }
                *failure = Some(crate::diagnostics::Diagnostic::of(format!(
                    "template {} cannot publish row {row_type} on undeclared stream {stream}; add the stream type to the template's `streams` catalog or declare an observer for it",
                    admission.template
                )));
                true
            });
            return;
        }
        // Every row an execution produces goes through here, whether an observer
        // measured it or a step pushed it, so a step waiting on its own observer
        // and a step waiting on what it pushed itself are one mechanism.
        //
        // The row is copied for the subscriptions only when there are any. A
        // observer at five milliseconds is the hot path of a run, and most runs
        // listen to nothing at all — paying for a map clone on every sample so
        // that a subscription nobody opened could have read it would be the
        // whole mechanism costing something on the runs that do not use it.
        #[cfg(feature = "native")]
        if self.listening.receiver_count() > 0 {
            // A send that races the last subscriber away is not a failure: the
            // row is already this run's evidence either way.
            let _ = self.listening.send((stream.clone(), values.clone()));
        }
        match self.sink.publish(stream, values, storage_payload).await {
            Ok(()) => self.flush_later(),
            Err(error) => self.failed(stream, &error),
        }
    }

    fn failed(&self, stream: &Identifier, error: &str) {
        #[cfg(feature = "native")]
        self.failure.send_if_modified(|failure| {
            if failure.is_some() {
                return false;
            }
            *failure = Some(crate::diagnostics::Diagnostic::of(format!("stream {stream} publication failed: {error}")));
            true
        });
        #[cfg(not(feature = "native"))]
        let _ = (stream, error);
    }

    // A buffered row waits at most one window: the first row after a flush
    // arms one timer, and the timer flushes whatever is buffered by then.
    fn flush_later(&self) {
        #[cfg(feature = "native")]
        if self.sink.buffering()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
            && !self.flushing.swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            let streams = self.clone();
            runtime.spawn(async move {
                tokio::time::sleep(FLUSH_WINDOW).await;
                streams.flushing.store(false, std::sync::atomic::Ordering::Release);
                streams.flush().await;
            });
        }
    }
}

impl<'a, R: TelemetryRow> StreamSelection<'a, R> {
    pub fn open(self) -> Pushing<'a, R>
    where
        R: OwnedStream<Row = R>,
    {
        let stream = <R as TelemetryStream>::stream_id();

        self.pushing(stream)
    }

    // `Into` takes no type parameter, so there is nothing for this to be
    // confused with; the name is the one an author reads at the call.
    #[allow(clippy::should_implement_trait)]
    pub fn into<C>(self) -> Pushing<'a, R>
    where
        C: OwnedStream<Row = R>,
    {
        let stream = <C as TelemetryStream>::stream_id();

        self.pushing(stream)
    }

    fn pushing(self, stream: Identifier) -> Pushing<'a, R> {
        Pushing { stream, streams: self.streams, row: PhantomData, borrowed: PhantomData }
    }
}

impl<R: TelemetryStream<Row = R> + TelemetryRow> StreamSelection<'_, R> {
    pub fn produced(&self) -> u64 {
        self.streams.produced(R::ID)
    }
}

impl<R: TelemetryRow + harmos::Message> Pushing<'_, R> {
    pub async fn push(&self, row: R) {
        let payload = row.encode_to_vec();
        self.streams.published_as(&self.stream, row.into_row(), Some(payload), std::any::type_name::<R>()).await;
    }

    pub fn produced(&self) -> u64 {
        self.streams.produced(self.stream.as_str())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
