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
    sync::{Arc, Mutex},
    task::{Context as PollContext, Poll},
    time::Duration,
};
// Third Party Imports
use tokio::sync::{Notify, mpsc};
use tokio_stream::Stream;
// Local Imports
use super::*;

/* === Constants === */
const MAX_CLIENT_STREAMS: usize = 8;
const CLIENT_STREAM_BUFFER: usize = 2;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

const FINISH_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/* === Macros === */

/* === Type Aliases === */
type FrameValidator = dyn Fn(&[u8]) -> Result<(), RpcError> + Send + Sync;

/* === Structs === */
pub(super) struct ClientWireStream<M> {
    receiver: Mutex<mpsc::Receiver<Vec<u8>>>,
    message: std::marker::PhantomData<fn() -> M>,
}

#[derive(Clone, Default)]
pub(super) struct ClientStreams(Arc<Mutex<Registry>>);

#[derive(Default)]
struct Registry {
    next_id: u64,
    reaper_started: bool,
    entries: HashMap<u64, Arc<Entry>>,
}

struct Entry {
    context: Arc<Owner>,
    cancel: CancellationToken,
    last_access: Mutex<tokio::time::Instant>,
    writer: tokio::sync::Mutex<Writer>,
    outcome: Arc<Outcome>,
    validate: Box<FrameValidator>,
}

struct Writer {
    sender: Option<mpsc::Sender<Vec<u8>>>,
    next_sequence: u64,
}

#[derive(Default)]
struct Outcome {
    result: Mutex<Option<Result<Vec<u8>, RpcError>>>,
    ready: Notify,
}

struct SettleOnDrop(Arc<Outcome>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<M> ClientWireStream<M> {
    fn new(receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            message: std::marker::PhantomData,
        }
    }
}

impl<M: Message + Default> Stream for ClientWireStream<M> {
    type Item = M;

    fn poll_next(self: Pin<&mut Self>, context: &mut PollContext<'_>) -> Poll<Option<Self::Item>> {
        let receiver = &mut *self
            .get_mut()
            .receiver
            .lock()
            .expect("client stream receiver poisoned");
        Pin::new(receiver).poll_recv(context).map(|row| {
            row.map(|bytes| {
                M::decode(bytes.as_slice()).expect("client stream frame validated before enqueue")
            })
        })
    }
}

impl Outcome {
    fn complete(&self, result: Result<Vec<u8>, RpcError>) {
        let mut guard = self
            .result
            .lock()
            .expect("RPC client stream outcome poisoned");
        if guard.is_some() {
            return;
        }
        *guard = Some(result);
        drop(guard);
        self.ready.notify_waiters();
    }

    async fn wait(&self) -> Result<Vec<u8>, RpcError> {
        loop {
            // Register the notification before inspecting the result. A
            // completion between those operations must wake this waiter.
            let notified = self.ready.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(result) = self
                .result
                .lock()
                .expect("RPC client stream outcome poisoned")
                .clone()
            {
                return result;
            }
            notified.await;
        }
    }

    async fn error(&self) -> RpcError {
        match self.wait().await {
            Ok(_) => RpcError::Cancelled,
            Err(error) => error,
        }
    }
}

impl Drop for SettleOnDrop {
    fn drop(&mut self) {
        self.0.complete(Err(RpcError::Grpc {
            code: tonic::Code::Internal as i32,
            message: "the plugin stream driver task ended without a result".into(),
            details: Vec::new(),
        }));
    }
}

impl ClientStreams {
    pub(super) fn open<M, O, F, Fut>(
        &self,
        context: Context,
        first: M,
        call: F,
    ) -> Result<tauri::ipc::Response, RpcError>
    where
        M: Message + Default + Send + Sync + 'static,
        O: Message + Default + Send + Sync + 'static,
        F: FnOnce(ClientWireStream<M>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<O, RpcError>> + Send + 'static,
    {
        self.open_owned(context.into(), first, call)
    }

    fn open_owned<M, O, F, Fut>(
        &self,
        context: Owner,
        first: M,
        call: F,
    ) -> Result<tauri::ipc::Response, RpcError>
    where
        M: Message + Default + Send + Sync + 'static,
        O: Message + Default + Send + Sync + 'static,
        F: FnOnce(ClientWireStream<M>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<O, RpcError>> + Send + 'static,
    {
        context.check()?;
        if first.encoded_len() > MAX_FRAME_BYTES {
            return Err(RpcError::LimitExceeded);
        }
        let (sender, receiver) = mpsc::channel(CLIENT_STREAM_BUFFER);
        sender
            .try_send(first.encode_to_vec())
            .map_err(|_| RpcError::Busy)?;
        let context = Arc::new(context);
        let cancel = CancellationToken::new();
        let outcome = Arc::new(Outcome::default());
        let entry = Arc::new(Entry {
            context: Arc::clone(&context),
            cancel: cancel.clone(),
            last_access: Mutex::new(tokio::time::Instant::now()),
            writer: tokio::sync::Mutex::new(Writer {
                sender: Some(sender),
                next_sequence: 1,
            }),
            outcome: Arc::clone(&outcome),
            validate: Box::new(|bytes| {
                M::decode(bytes)
                    .map(|_| ())
                    .map_err(|_| RpcError::InvalidRequest)
            }),
        });
        // Reserve a registry slot before starting the remote call. A
        // rejected open therefore cannot cause any server-side work.
        let response = self.insert(Arc::clone(&entry), first.encoded_len())?;
        let task_context = Arc::clone(&context);
        let task_cancel = cancel.clone();
        let task_outcome = Arc::clone(&outcome);
        tokio::spawn(async move {
            let _settle_on_drop = SettleOnDrop(Arc::clone(&task_outcome));
            let stream = ClientWireStream::new(receiver);
            let result = tokio::select! {
                biased;
                _ = task_cancel.cancelled() => Err(RpcError::Cancelled),
                result = task_context.execute(call(stream)) => {
                    result.map(|response| response.encode_to_vec())
                }
            };
            task_outcome.complete(result);
        });
        Ok(response)
    }

    fn insert(
        &self,
        entry: Arc<Entry>,
        first_frame_bytes: usize,
    ) -> Result<tauri::ipc::Response, RpcError> {
        let mut registry = self.0.lock().expect("RPC client streams poisoned");
        registry.prune();
        if registry.entries.len() >= MAX_CLIENT_STREAMS {
            return Err(RpcError::LimitExceeded);
        }
        registry.next_id = registry
            .next_id
            .checked_add(1)
            .ok_or(RpcError::LimitExceeded)?;
        let id = registry.next_id;
        // The generated openers (`rpc/generated.rs`) never pass a method
        // name down to `ClientStreams::open`, so the tonic method path is
        // not available here; the (owner, generation, stream_id) triple is
        // what later write/finish/cancel/prune lines correlate against.
        tracing::info!(
            owner = %entry.context.owner,
            generation = entry.context.generation,
            stream_id = id,
            first_frame_bytes,
            "client stream opened",
        );
        let mut response = entry.context.generation.to_le_bytes().to_vec();
        response.extend(id.to_le_bytes());
        registry.entries.insert(id, entry);
        if !registry.reaper_started {
            registry.reaper_started = true;
            let weak = Arc::downgrade(&self.0);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let Some(registry) = weak.upgrade() else {
                        break;
                    };
                    let mut registry = registry.lock().expect("RPC client streams poisoned");
                    registry.prune();
                    if registry.entries.is_empty() {
                        registry.reaper_started = false;
                        break;
                    }
                }
            });
        }
        Ok(tauri::ipc::Response::new(response))
    }

    fn get(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Entry>, RpcError> {
        let mut registry = self.0.lock().expect("RPC client streams poisoned");
        registry.prune();
        registry
            .entries
            .get(&id)
            .filter(|entry| entry.context.owner == owner && entry.context.generation == generation)
            .cloned()
            .ok_or(RpcError::UnknownStream)
    }

    async fn write(
        &self,
        owner: &str,
        generation: u64,
        id: u64,
        sequence: u64,
        bytes: Vec<u8>,
    ) -> Result<(), RpcError> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Err(RpcError::LimitExceeded);
        }
        let entry = self.get(owner, generation, id)?;
        let mut writer = entry.writer.lock().await;
        if sequence != writer.next_sequence {
            return Err(RpcError::InvalidAck);
        }
        entry.context.check()?;
        (entry.validate)(&bytes)?;
        let lease_cancellation = entry.context.session_cancel.clone();
        let Some(sender) = writer.sender.as_ref().cloned() else {
            drop(writer);
            return Err(entry.outcome.error().await);
        };
        *entry.last_access.lock().expect("RPC idle timer poisoned") = tokio::time::Instant::now();
        tracing::debug!(
            owner,
            generation,
            stream_id = id,
            sequence,
            bytes = bytes.len(),
            "client stream write",
        );
        let sent = tokio::select! {
            biased;
            _ = entry.cancel.cancelled() => Err(RpcError::Cancelled),
            _ = lease_cancellation.cancelled() => {
                Err(ConnectionError::Superseded.into())
            },
            // The remote call may already have failed (e.g. the server
            // rejected the first frame) while the body stream sits unpolled
            // and undropped. Without this arm `sender.send` would then wait
            // for buffer space that a dead call will never free, hanging
            // the write forever instead of surfacing the call's error.
            error = entry.outcome.error() => Err(error),
            result = sender.send(bytes) => result.map_err(|_| RpcError::Cancelled),
        };
        match sent {
            Ok(()) => {
                writer.next_sequence = writer
                    .next_sequence
                    .checked_add(1)
                    .ok_or(RpcError::LimitExceeded)?;
                *entry.last_access.lock().expect("RPC idle timer poisoned") =
                    tokio::time::Instant::now();
                Ok(())
            }
            Err(error) => {
                writer.sender = None;
                drop(writer);
                if matches!(error, RpcError::Cancelled) {
                    return Err(entry.outcome.error().await);
                }
                Err(error)
            }
        }
    }

    async fn finish(
        &self,
        owner: &str,
        generation: u64,
        id: u64,
    ) -> Result<tauri::ipc::Response, RpcError> {
        let entry = self.get(owner, generation, id)?;
        entry.context.check()?;
        tracing::info!(
            owner,
            generation,
            stream_id = id,
            "client stream finish requested"
        );
        *entry.last_access.lock().expect("RPC idle timer poisoned") = tokio::time::Instant::now();
        entry.writer.lock().await.sender = None;
        let result = wait_with_heartbeat(&entry).await;
        self.remove(owner, generation, id);
        match &result {
            Ok(bytes) => tracing::info!(
                owner,
                generation,
                stream_id = id,
                bytes = bytes.len(),
                "client stream finish completed",
            ),
            Err(error) => tracing::info!(
                owner,
                generation,
                stream_id = id,
                error = ?error,
                "client stream finish failed",
            ),
        }
        entry.context.check()?;
        entry.context.reply_bytes(&result?)
    }

    async fn cancel(&self, owner: &str, generation: u64, id: u64) -> Result<(), RpcError> {
        let entry = self.get(owner, generation, id)?;
        self.remove(owner, generation, id);
        tracing::info!(
            owner,
            generation,
            stream_id = id,
            reason = "requested",
            "client stream cancelled",
        );
        entry.cancel.cancel();
        entry.writer.lock().await.sender = None;
        Ok(())
    }

    fn remove(&self, owner: &str, generation: u64, id: u64) {
        let mut registry = self.0.lock().expect("RPC client streams poisoned");
        if registry.entries.get(&id).is_some_and(|entry| {
            entry.context.owner == owner && entry.context.generation == generation
        }) {
            registry.entries.remove(&id);
        }
    }

    pub(super) fn cancel_owner(&self, owner: &str) {
        let mut registry = self.0.lock().expect("RPC client streams poisoned");
        registry.entries.retain(|id, entry| {
            if entry.context.owner == owner {
                tracing::info!(
                    owner,
                    generation = entry.context.generation,
                    stream_id = *id,
                    reason = "owner_cancelled",
                    "client stream cancelled",
                );
                entry.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
}

impl Registry {
    fn prune(&mut self) {
        self.entries.retain(|id, entry| {
            let already_cancelled = entry.cancel.is_cancelled();
            let owner_cancelled = entry.context.owner_cancel.is_cancelled();
            let lease_cancelled = entry.context.session_cancel.clone().is_cancelled();
            let expired = entry
                .last_access
                .lock()
                .expect("RPC idle timer poisoned")
                .elapsed()
                >= IDLE_TIMEOUT;
            if already_cancelled || owner_cancelled || lease_cancelled || expired {
                // `cancel()` and `cancel_owner()` already log their own
                // reason before cancelling `entry.cancel` directly; only log
                // here when the reaper is the first to notice, so a stream
                // that was cancelled and then swept out on the next tick
                // does not get a second, redundant line.
                if !already_cancelled {
                    let reason = if owner_cancelled {
                        "owner_cancelled"
                    } else if lease_cancelled {
                        "lease_cancelled"
                    } else {
                        "idle"
                    };
                    tracing::info!(
                        owner = %entry.context.owner,
                        generation = entry.context.generation,
                        stream_id = *id,
                        reason,
                        "client stream reaped",
                    );
                }
                entry.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
}

/* === Public Functions === */
#[tauri::command]
pub async fn client_stream_write<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>,
    bridge: tauri::State<'_, Bridge>,
    webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = super::request(request.body())?;
    session.lease(&generation.to_string())?;
    if payload.len() < 16 {
        return Err(RpcError::InvalidRequest);
    }
    bridge
        .client_streams
        .write(
            webview.label(),
            generation,
            read_u64(payload)?,
            read_u64(&payload[8..])?,
            payload[16..].to_vec(),
        )
        .await
}

#[tauri::command]
pub async fn client_stream_finish<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>,
    bridge: tauri::State<'_, Bridge>,
    webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    let (generation, payload) = super::request(request.body())?;
    session.lease(&generation.to_string())?;
    if payload.len() != 8 {
        return Err(RpcError::InvalidRequest);
    }
    bridge
        .client_streams
        .finish(webview.label(), generation, read_u64(payload)?)
        .await
}

#[tauri::command]
pub async fn client_stream_cancel<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>,
    bridge: tauri::State<'_, Bridge>,
    webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = super::request(request.body())?;
    session.lease(&generation.to_string())?;
    if payload.len() != 8 {
        return Err(RpcError::InvalidRequest);
    }
    bridge
        .client_streams
        .cancel(webview.label(), generation, read_u64(payload)?)
        .await
}

/* === Private Functions === */
async fn wait_with_heartbeat(entry: &Entry) -> Result<Vec<u8>, RpcError> {
    let outcome = entry.outcome.wait();
    tokio::pin!(outcome);
    loop {
        tokio::select! {
            biased;
            result = &mut outcome => return result,
            _ = tokio::time::sleep(FINISH_HEARTBEAT_INTERVAL) => {
                *entry.last_access.lock().expect("RPC idle timer poisoned") =
                    tokio::time::Instant::now();
            }
        }
    }
}

/* === Tests === */
