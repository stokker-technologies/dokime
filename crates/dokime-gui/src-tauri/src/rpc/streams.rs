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
    pin::Pin,
    task::{Context as PollContext, Poll},
    time::Duration,
};
// Third Party Imports
use tokio::time::Instant;
use tokio_stream::{Stream, StreamExt};
// Local Imports
use super::*;

/* === Constants === */
const MAX_STREAMS: usize = 16;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */
type WireStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, RpcError>> + Send>>;
type Pending<M> = Pin<Box<dyn Future<Output = dokime::client::Result<Option<M>>> + Send>>;

/* === Structs === */
struct Native<M: Send + 'static> {
    reader: dokime::client::ResponseStream<M>,
    pending: Option<Pending<M>>,
}

#[derive(Clone, Default)]
pub(super) struct Streams(Arc<Mutex<Registry>>);

#[derive(Default)]
struct Registry {
    next_id: u64,
    reaper_started: bool,
    entries: HashMap<u64, Arc<Entry>>,
}

struct Entry {
    context: Owner,
    cancel: CancellationToken,
    last_access: Mutex<Instant>,
    cursor: tokio::sync::Mutex<Cursor>,
}

struct Cursor {
    stream: WireStream,
    sequence: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<M: Message + Send + 'static> Stream for Native<M> {
    type Item = Result<Vec<u8>, RpcError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut PollContext<'_>) -> Poll<Option<Self::Item>> {
        let reader = self.get_mut();
        if reader.pending.is_none() {
            let inner = reader.reader.clone();
            reader.pending = Some(Box::pin(async move { inner.message().await }));
        }
        let result = match reader
            .pending
            .as_mut()
            .expect("pending native read")
            .as_mut()
            .poll(context)
        {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(result) => result,
        };
        reader.pending = None;
        Poll::Ready(match result {
            Ok(Some(message)) if message.encoded_len() > MAX_FRAME_BYTES => Some(Err(
                tonic::Status::resource_exhausted("RPC frame exceeds desktop limit").into(),
            )),
            Ok(Some(message)) => Some(Ok(message.encode_to_vec())),
            Ok(None) => None,
            Err(error) => Some(Err(error.into())),
        })
    }
}

impl<M: Send + 'static> Drop for Native<M> {
    fn drop(&mut self) {
        self.reader.cancel();
    }
}

impl Streams {
    pub(super) fn open<M: Message + Send + 'static>(
        &self,
        context: Context,
        reader: dokime::client::ResponseStream<M>,
    ) -> Result<tauri::ipc::Response, RpcError> {
        self.insert(
            context.into(),
            Box::pin(Native {
                reader,
                pending: None,
            }),
        )
    }
    fn insert(&self, context: Owner, stream: WireStream) -> Result<tauri::ipc::Response, RpcError> {
        let mut registry = self.0.lock().expect("RPC streams poisoned");
        registry.prune();
        if registry.entries.len() >= MAX_STREAMS {
            return Err(RpcError::LimitExceeded);
        }
        registry.next_id = registry
            .next_id
            .checked_add(1)
            .ok_or(RpcError::LimitExceeded)?;
        let id = registry.next_id;
        let mut response = context.generation.to_le_bytes().to_vec();
        response.extend(id.to_le_bytes());
        let entry = Arc::new(Entry {
            context,
            cancel: CancellationToken::new(),
            last_access: Mutex::new(Instant::now()),
            cursor: tokio::sync::Mutex::new(Cursor {
                stream,
                sequence: 0,
            }),
        });
        registry.entries.insert(id, entry.clone());
        if !registry.reaper_started {
            registry.reaper_started = true;
            let weak = Arc::downgrade(&self.0);
            // One timer per registry, independent of stream churn. Never reads data.
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let Some(registry) = weak.upgrade() else {
                        break;
                    };
                    let mut registry = registry.lock().expect("RPC streams poisoned");
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
        let mut registry = self.0.lock().expect("RPC streams poisoned");
        registry.prune();
        registry
            .entries
            .get(&id)
            .filter(|entry| entry.context.owner == owner && entry.context.generation == generation)
            .cloned()
            .ok_or(RpcError::UnknownStream)
    }

    pub(super) fn cancel(&self, owner: &str, generation: u64, id: u64) -> Result<(), RpcError> {
        let entry = self.get(owner, generation, id)?;
        entry.cancel.cancel();
        self.0
            .lock()
            .expect("RPC streams poisoned")
            .entries
            .remove(&id);
        Ok(())
    }
    pub(super) fn cancel_owner(&self, owner: &str) {
        self.0
            .lock()
            .expect("RPC streams poisoned")
            .entries
            .retain(|_, entry| {
                if entry.context.owner == owner {
                    entry.cancel.cancel();
                    false
                } else {
                    true
                }
            });
    }
    pub(super) async fn next(
        &self,
        owner: &str,
        generation: u64,
        id: u64,
        ack: u64,
    ) -> Result<tauri::ipc::Response, RpcError> {
        let entry = self.get(owner, generation, id)?;
        let mut cursor = entry.cursor.try_lock().map_err(|_| RpcError::Busy)?;
        if ack != cursor.sequence {
            return Err(RpcError::InvalidAck);
        }
        entry.context.check()?;
        *entry.last_access.lock().expect("RPC idle timer poisoned") = Instant::now();
        let result = tokio::select! {
            biased;
            _ = entry.cancel.cancelled() => Err(RpcError::Cancelled),
            result = entry.context.execute(async { cursor.stream.next().await.transpose() }) => result,
            _ = tokio::time::sleep(Duration::from_secs(25)) => {
                let mut bytes = generation.to_le_bytes().to_vec();
                bytes.extend(id.to_le_bytes());
                bytes.extend(cursor.sequence.to_le_bytes());
                bytes.push(2);
                *entry.last_access.lock().expect("RPC idle timer poisoned") = Instant::now();
                return Ok(tauri::ipc::Response::new(bytes));
            },
        };
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                let _ = self.cancel(owner, generation, id);
                return Err(error);
            }
        };
        entry.context.check()?;
        cursor.sequence = cursor
            .sequence
            .checked_add(1)
            .ok_or(RpcError::LimitExceeded)?;
        let mut bytes = generation.to_le_bytes().to_vec();
        bytes.extend(id.to_le_bytes());
        bytes.extend(cursor.sequence.to_le_bytes());
        bytes.push(u8::from(message.is_none()));
        match message {
            Some(message) => bytes.extend(message),
            None => {
                let _ = self.cancel(owner, generation, id);
            }
        }
        *entry.last_access.lock().expect("RPC idle timer poisoned") = Instant::now();
        Ok(tauri::ipc::Response::new(bytes))
    }
}

impl Registry {
    fn prune(&mut self) {
        self.entries.retain(|_, entry| {
            let expired = entry.cursor.try_lock().is_ok()
                && entry
                    .last_access
                    .lock()
                    .expect("RPC idle timer poisoned")
                    .elapsed()
                    >= IDLE_TIMEOUT;
            if entry.cancel.is_cancelled()
                || entry.context.owner_cancel.is_cancelled()
                || entry.context.session_cancel.is_cancelled()
                || expired
            {
                entry.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
