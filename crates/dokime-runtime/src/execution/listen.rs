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

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;

// Third Party Imports
// Third-Party Imports
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::{Stream, StreamExt};

// Local Imports
use crate::records::DeclaredRecord;
use crate::records::Deliveries;
use crate::records::RecordSource;
use crate::telemetry::{Streams, TelemetryStream};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type PublishedRow = (Identifier, BTreeMap<String, Value>);
pub type Deadlined<R> = Pin<Box<dyn Stream<Item = crate::Result<R>> + Send>>;

/* === Structs === */

pub struct ListeningRows<C> {
    values: ListeningValues,
    stream_type: PhantomData<fn() -> C>,
}

pub struct ListeningRecords<R> {
    records: ListeningEvidence,
    record: PhantomData<fn() -> R>,
}

pub struct ListeningValues {
    stream: Identifier,
    rows: BroadcastStream<PublishedRow>,
    missed: Arc<AtomicU64>,
}

pub struct ListeningEvidence {
    record_type: String,
    source: Arc<dyn RecordSource>,
    cursor: usize,
    changed: Arc<tokio::sync::Notify>,
    waiting: Option<Pin<Box<tokio::sync::futures::OwnedNotified>>>,
}

/* === Enums === */

/* === Traits === */

pub trait Listenable: Sized {
    type Item;

    type Listening: Stream<Item = Self::Item> + Unpin + Send;

    fn subscription() -> crate::definition::Subscription;

    fn listening(streams: &Streams, deliveries: &Deliveries) -> Self::Listening;
}

/* === Implementations === */

impl<C: TelemetryStream> ListeningRows<C> {
    pub fn new(streams: &Streams) -> Self {
        Self {
            values: ListeningValues::new(streams, C::stream_id()),
            stream_type: PhantomData,
        }
    }

    pub fn missed(&self) -> u64 {
        self.values.missed()
    }

    pub fn timeout(self, within: Duration) -> Deadlined<C::Row>
    where
        C: Unpin + 'static,
        C::Row: Send + Unpin + 'static,
    {
        Box::pin(StreamExt::map(StreamExt::timeout(self, within), |item| item.map_err(Into::into)))
    }
}

impl<R: DeclaredRecord> ListeningRecords<R> {
    pub fn new(deliveries: &Deliveries) -> Self {
        Self {
            records: ListeningEvidence::new(deliveries, R::RECORD_TYPE.to_owned()),
            record: PhantomData,
        }
    }

    pub fn timeout(self, within: Duration) -> Deadlined<R>
    where
        R: Send + Unpin + 'static,
    {
        Box::pin(StreamExt::map(StreamExt::timeout(self, within), |item| item.map_err(Into::into)))
    }
}

impl ListeningValues {
    pub fn new(streams: &Streams, stream: Identifier) -> Self {
        Self {
            stream,
            rows: BroadcastStream::new(streams.subscribe()),
            missed: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn missed(&self) -> u64 {
        self.missed.load(Ordering::Acquire)
    }
}

impl ListeningEvidence {
    pub fn new(deliveries: &Deliveries, record_type: String) -> Self {
        let source = deliveries.source();
        let changed = source.changed();
        Self {
            record_type,
            source,
            cursor: 0,
            changed,
            waiting: None,
        }
    }
}

impl Stream for ListeningValues {
    type Item = BTreeMap<String, Value>;

    fn poll_next(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let listening = self.get_mut();
        loop {
            match Pin::new(&mut listening.rows).poll_next(context) {
                Poll::Ready(Some(Ok((published, values)))) => {
                    if published == listening.stream {
                        return Poll::Ready(Some(values));
                    }
                }
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(lost)))) => {
                    let total = listening.missed.fetch_add(lost, Ordering::AcqRel) + lost;
                    tracing::warn!(stream = %listening.stream,
                        "a listener fell behind and lost {lost} rows on this stream; {total} lost in all");
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl Stream for ListeningEvidence {
    type Item = Record;

    fn poll_next(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let listening = self.get_mut();
        loop {
            let waiting = listening.waiting.get_or_insert_with(|| Box::pin(Arc::clone(&listening.changed).notified_owned()));
            waiting.as_mut().enable();
            let record = listening.source.next_record(&mut listening.cursor);
            let closed = record.is_none() && listening.source.closed();
            // Closure Follows Final Publication; Recheck After Observing It.
            let record = if closed { listening.source.next_record(&mut listening.cursor) } else { record };
            if let Some(record) = record {
                listening.waiting = None;
                if record.record_type.as_str() == listening.record_type {
                    return Poll::Ready(Some(record));
                }
                continue;
            }
            if closed {
                return Poll::Ready(None);
            }
            let waiting = listening.waiting.as_mut().expect("the inbox change waiter is registered");
            match waiting.as_mut().poll(context) {
                Poll::Ready(()) => listening.waiting = None,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<C: TelemetryStream + Unpin> Stream for ListeningRows<C> {
    type Item = C::Row;

    fn poll_next(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let listening = self.get_mut();
        loop {
            let values = match Pin::new(&mut listening.values).poll_next(context) {
                Poll::Ready(Some(values)) => values,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };
            match <C::Row as crate::telemetry::TelemetryRow>::from_row(&values) {
                Ok(row) => return Poll::Ready(Some(row)),
                Err(refusal) => tracing::warn!(stream = %listening.values.stream,
                    "a published row does not match the shape that declared this stream: {refusal}"),
            }
        }
    }
}

impl<R: DeclaredRecord + Unpin> Stream for ListeningRecords<R> {
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let listening = self.get_mut();
        loop {
            let record = match Pin::new(&mut listening.records).poll_next(context) {
                Poll::Ready(Some(record)) => record,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };
            match R::from_record(&record) {
                Ok(decoded) => return Poll::Ready(Some(decoded)),
                Err(refusal) => tracing::warn!(record = %record.record_type,
                    "a filed record does not carry what its type declared: {refusal}"),
            }
        }
    }
}

/* === Public Functions === */

pub(crate) fn rows(capacity: usize) -> broadcast::Sender<PublishedRow> {
    broadcast::channel(capacity).0
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    struct ClosingSource {
        published: AtomicBool,
        changed: Arc<tokio::sync::Notify>,
        record: Record,
    }

    impl RecordSource for ClosingSource {
        fn delivered(&self, index: usize) -> Option<Record> {
            (index == 0 && self.published.load(Ordering::Acquire)).then(|| self.record.clone())
        }

        fn changed(&self) -> Arc<tokio::sync::Notify> {
            Arc::clone(&self.changed)
        }

        fn closed(&self) -> bool {
            self.published.store(true, Ordering::Release);
            true
        }
    }

    #[test]
    fn closure_drains_records() {
        let request = dokime_domain::model::records::publication::RecordPublishRequest::new(
            Identifier::new("fixture.final").unwrap(),
            dokime_domain::model::records::RecordSeverity::Info,
        );
        let record = request.into_record(
            dokime_domain::model::identity::RecordId::from_seed("final"),
            1,
            dokime_domain::model::time::Timestamp::new("2000-01-01T00:00:00Z"),
            "fixture",
        );
        let source = Arc::new(ClosingSource {
            published: AtomicBool::new(false),
            changed: Arc::default(),
            record: record.clone(),
        });
        let deliveries = Deliveries::new(source);
        let mut listening = ListeningEvidence::new(&deliveries, "fixture.final".into());
        let mut context = TaskContext::from_waker(std::task::Waker::noop());

        assert_eq!(Pin::new(&mut listening).poll_next(&mut context), Poll::Ready(Some(record)));
        assert_eq!(Pin::new(&mut listening).poll_next(&mut context), Poll::Ready(None));
    }
}
