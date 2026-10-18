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
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

// Third Party Imports
use tokio::{
    sync::{Mutex, Notify, mpsc},
    task::JoinHandle,
};
use tokio_stream::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;

// Local Imports
use super::{Client, Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ResponseStream<T> {
    inner: Arc<Response<T>>,
}

struct Response<T> {
    receiver: Mutex<mpsc::Receiver<Result<T>>>,
    cancellation: CancellationToken,
    worker: Mutex<Option<JoinHandle<()>>>,
    demand: Option<Arc<Demand>>,
}

struct Demand {
    requested: AtomicBool,
    ready: Notify,
}

pub struct RequestStream<T, R> {
    inner: Arc<Upload<T, R>>,
}

struct Upload<T, R> {
    max_frame_bytes: usize,
    sender: Mutex<Option<mpsc::Sender<T>>>,
    cancellation: CancellationToken,
    worker: Mutex<Option<JoinHandle<Result<R>>>>,
}

pub(super) struct Input<S> {
    inner: Pin<Box<S>>,
    cancellation: Pin<Box<dyn Future<Output = ()> + Send>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<T: Send + 'static> ResponseStream<T> {
    pub(super) fn new<S>(client: &Client, stream: S) -> Result<Self>
    where
        S: Stream<Item = std::result::Result<T, tonic::Status>> + Send + 'static,
    {
        let demand = Arc::new(Demand { requested: AtomicBool::new(false), ready: Notify::new() });
        let credit = demand.clone();
        Self::start(client, Some(demand), move |sender| async move {
            let mut stream = std::pin::pin!(stream);
            loop {
                credit.ready.notified().await;
                let Some(row) = stream.next().await else {
                    break;
                };
                let terminal = row.is_err();
                if sender.send(row.map_err(Error::Rpc)).await.is_err() || terminal {
                    break;
                }
            }
        })
    }

    pub(super) fn spawn<F, Fut>(client: &Client, read: F) -> Result<Self>
    where
        F: FnOnce(mpsc::Sender<Result<T>>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::start(client, None, read)
    }

    fn start<F, Fut>(client: &Client, demand: Option<Arc<Demand>>, read: F) -> Result<Self>
    where
        F: FnOnce(mpsc::Sender<Result<T>>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // Serialize reader registration with close so its wait cannot miss a
        // reader created after the cancellation signal.
        let channel = client.inner.channel.lock().expect("client transport poisoned");
        if channel.is_none() || client.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }
        let cancellation = client.inner.cancellation.child_token();
        let stopped = cancellation.clone();
        let (sender, receiver) = mpsc::channel(1);
        let worker = client.inner.readers.spawn(async move {
            tokio::select! {
                biased;
                _ = stopped.cancelled() => {},
                _ = read(sender) => {},
            }
        });

        Ok(Self { inner: Arc::new(Response { receiver: Mutex::new(receiver), cancellation, worker: Mutex::new(Some(worker)), demand }) })
    }

    pub async fn message(&self) -> Result<Option<T>> {
        let mut receiver = tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => return Err(Error::Closed),
            receiver = self.inner.receiver.lock() => receiver,
        };
        if let Some(demand) = &self.inner.demand {
            // Retain Outstanding Credit Across Cancelled Message Waiters.
            if !demand.requested.swap(true, Ordering::AcqRel) {
                demand.ready.notify_one();
            }
        }
        let row = tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => return Err(Error::Closed),
            row = receiver.recv() => row,
        };
        if let Some(demand) = &self.inner.demand {
            demand.requested.store(false, Ordering::Release);
        }
        if self.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }

        row.transpose()
    }

    pub fn cancel(&self) {
        self.inner.cancellation.cancel();
    }

    pub async fn close(&self) {
        self.cancel();
        let mut worker = self.inner.worker.lock().await;
        if let Some(task) = worker.as_mut() {
            let _ = task.await;
        }
        worker.take();
        drop(worker);

        // A caller may retain an unpolled message future holding the receive
        // guard. Its lifetime must not prevent close from joining transport
        // cleanup. Cancellation fences that future before it can return data.
        if let Ok(mut receiver) = self.inner.receiver.try_lock() {
            receiver.close();
            while receiver.try_recv().is_ok() {}
        }
    }
}

impl<T> Clone for ResponseStream<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> Drop for Response<T> {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(worker) = self.worker.get_mut().take() {
            worker.abort();
        }
    }
}

impl<T: harmos::domain::Portable + Send + 'static, R: Send + 'static> RequestStream<T, R> {
    pub(super) fn new<F, Fut>(client: &Client, operation: F) -> Result<Self>
    where
        F: FnOnce(tokio_stream::wrappers::ReceiverStream<T>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<R>> + Send + 'static,
    {
        // Registration and client close share the same fence as response readers.
        let channel = client.inner.channel.lock().expect("client transport poisoned");
        if channel.is_none() || client.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }
        let cancellation = client.inner.cancellation.child_token();
        let stopped = cancellation.clone();
        let (sender, receiver) = mpsc::channel(1);
        let worker = client.inner.readers.spawn(async move {
            tokio::select! {
                biased;
                _ = stopped.cancelled() => Err(Error::Closed),
                result = operation(tokio_stream::wrappers::ReceiverStream::new(receiver)) => result,
            }
        });

        Ok(Self {
            inner: Arc::new(Upload {
                max_frame_bytes: client.request_limit(),
                sender: Mutex::new(Some(sender)),
                cancellation,
                worker: Mutex::new(Some(worker)),
            }),
        })
    }

    pub async fn send(&self, frame: T) -> Result<()> {
        let mut encoded = Vec::new();
        <harmos::grpc::Transcoded as harmos::grpc::Wire<T>>::encode(&frame, &mut encoded)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        if encoded.len() > self.inner.max_frame_bytes {
            return Err(Error::Rpc(tonic::Status::out_of_range("upload frame exceeds the negotiated request limit")));
        }

        let sender = tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => return Err(Error::Closed),
            sender = self.inner.sender.lock() => sender,
        };
        let sender = sender.as_ref().ok_or(Error::Closed)?;
        tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => Err(Error::Closed),
            sent = sender.send(frame) => sent.map_err(|_| Error::Closed),
        }
    }

    pub async fn end_input(&self) -> Result<()> {
        let mut sender = tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => return Err(Error::Closed),
            sender = self.inner.sender.lock() => sender,
        };
        sender.take();
        Ok(())
    }

    pub async fn finish(&self) -> Result<R> {
        let _cancel = self.inner.cancellation.clone().drop_guard();
        self.end_input().await?;
        self.response().await
    }

    pub async fn response(&self) -> Result<R> {
        let _cancel = self.inner.cancellation.clone().drop_guard();
        let mut worker = self.inner.worker.lock().await;
        let task = worker.as_mut().ok_or(Error::Closed)?;
        let result = task.await;
        worker.take();
        if self.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }

        result.map_err(Error::Task)?
    }

    pub fn cancel(&self) {
        self.inner.cancellation.cancel();
    }

    pub async fn close(&self) {
        self.cancel();
        let mut worker = self.inner.worker.lock().await;
        if let Some(task) = worker.as_mut() {
            let _ = task.await;
        }
        worker.take();
        drop(worker);

        if let Ok(mut sender) = self.inner.sender.try_lock() {
            sender.take();
        }
    }
}

impl<T, R> Clone for RequestStream<T, R> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T, R> Drop for Upload<T, R> {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(worker) = self.worker.get_mut().take() {
            worker.abort();
        }
    }
}

impl<S> Input<S> {
    pub(super) fn new(inner: S, cancellation: CancellationToken) -> Self {
        Self { inner: Box::pin(inner), cancellation: Box::pin(cancellation.cancelled_owned()) }
    }
}

impl<S: Stream> Stream for Input<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.cancellation.as_mut().poll(context).is_ready() {
            return Poll::Ready(None);
        }

        self.inner.as_mut().poll_next(context)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
