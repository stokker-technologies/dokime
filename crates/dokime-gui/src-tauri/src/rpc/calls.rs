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

use crate::rpc::protocol::FRAME_BYTES as DESKTOP_FRAME_BYTES;
use crate::rpc::protocol::STREAM_CREDIT as DESKTOP_STREAM_CREDIT;

// Standard Library Imports
use std::{str::FromStr, time::Duration};

// Third Party Imports
use tauri::ipc::{Channel, JavaScriptChannelId, Response};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Local Imports
use super::*;

/* === Constants === */

const DESKTOP_INPUT_BUFFER_ITEMS: u64 = 2;
const DESKTOP_STREAM_TIMEOUT_MS: u64 = 86_400_000;
const DESKTOP_UNARY_TIMEOUT_MS: u64 = 10_000;

pub const PATH_HEADER: &str = "dokime-rpc-path";
pub const DEADLINE_HEADER: &str = "dokime-rpc-deadline-ms";
pub const CHANNEL_HEADER: &str = "dokime-rpc-channel";
pub const INPUT_HEADER: &str = "dokime-rpc-input";
pub const STREAM_HEADER: &str = "dokime-rpc-stream";
pub const CREDIT_HEADER: &str = "dokime-rpc-credit";
pub const FRAME_MESSAGE: u8 = 0;
pub const FRAME_END: u8 = 1;
pub const FRAME_ERROR: u8 = 2;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Streams {
    budget: Arc<Semaphore>,
    registry: Arc<Mutex<Registry>>,
}

#[derive(Default)]
struct Registry {
    next_id: u64,
    entries: HashMap<u64, Arc<Entry>>,
}

struct Entry {
    owner: String,
    generation: u64,
    cancel: CancellationToken,
    input: tokio::sync::Mutex<Option<mpsc::Sender<Vec<u8>>>>,
    max_request_bytes: usize,
    credit: Arc<Semaphore>,
    delivery: Mutex<Delivery>,
}

#[derive(Default)]
struct Delivery {
    sent: u64,
    acknowledged: u64,
}

struct Opening {
    path: String,
    deadline: Duration,
    payload: Vec<u8>,
    keep_input: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Streams {
    pub(super) fn new(capacity: usize) -> Self {
        Self { budget: Arc::new(Semaphore::new(capacity)), registry: Arc::default() }
    }

    pub(super) fn cancel_owner(&self, owner: &str) {
        let mut registry = self.registry.lock().expect("RPC stream registry poisoned");
        registry.entries.retain(|_, entry| {
            if entry.owner != owner {
                return true;
            }
            entry.cancel.cancel();
            false
        });
    }

    fn open(&self, context: Context, channel: Channel<Response>, opening: Opening) -> Result<Response, RpcError> {
        context.check()?;
        let client = context.client();
        let max_request_bytes = client.max_request_bytes();
        if opening.payload.len() > max_request_bytes {
            return Err(RpcError::LimitExceeded);
        }
        let (sender, receiver) = mpsc::channel(DESKTOP_INPUT_BUFFER_ITEMS as usize);
        sender.try_send(opening.payload).map_err(|_| RpcError::Busy)?;
        let entry = Arc::new(Entry {
            owner: context.owner.clone(),
            generation: context.generation,
            cancel: CancellationToken::new(),
            input: tokio::sync::Mutex::new(opening.keep_input.then_some(sender)),
            max_request_bytes,
            credit: Arc::new(Semaphore::new(DESKTOP_STREAM_CREDIT as usize)),
            delivery: Mutex::default(),
        });
        let id = {
            let mut registry = self.registry.lock().expect("RPC stream registry poisoned");
            registry.next_id = registry.next_id.checked_add(1).ok_or(RpcError::LimitExceeded)?;
            let id = registry.next_id;
            registry.entries.insert(id, Arc::clone(&entry));
            id
        };
        let response = context.reply_bytes(&id.to_le_bytes())?;
        let registry = Arc::clone(&self.registry);
        tokio::spawn(async move {
            let generation = context.generation;
            let result = tokio::select! {
                biased;
                _ = entry.cancel.cancelled() => Err(RpcError::Cancelled),
                result = context.execute(forward(
                    &entry,
                    &context,
                    &client,
                    &channel,
                    &opening.path,
                    receiver,
                    opening.deadline,
                )) => result,
            };
            // Cancelling deregisters under this lock, so the outcome is settled
            // here: a stream a caller removed first ends cancelled, whatever
            // its call reached meanwhile; one still registered ends with its
            // own result and refuses any later cancel as unknown. The lock is
            // held until the terminal frame is out, so a send that finds the
            // stream gone can always learn why from the channel. A WebView that
            // already cancelled ignores it; one that is gone drops it.
            let mut registry = registry.lock().expect("RPC stream registry poisoned");
            let result = match registry.entries.remove(&id) {
                Some(_) => result,
                None => Err(RpcError::Cancelled),
            };
            let terminal = match result {
                Ok(()) => frame(generation, FRAME_END, &[]),
                Err(error) => frame(generation, FRAME_ERROR, &serde_json::to_vec(&error).expect("RPC errors serialize")),
            };
            let _ = channel.send(Response::new(terminal));
        });
        Ok(response)
    }

    fn get(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Entry>, RpcError> {
        self.registry
            .lock()
            .expect("RPC stream registry poisoned")
            .entries
            .get(&id)
            .filter(|entry| entry.owner == owner && entry.generation == generation)
            .cloned()
            .ok_or(RpcError::UnknownStream)
    }

    fn remove(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Entry>, RpcError> {
        let mut registry = self.registry.lock().expect("RPC stream registry poisoned");
        match registry.entries.get(&id) {
            Some(entry) if entry.owner == owner && entry.generation == generation => {
                Ok(registry.entries.remove(&id).expect("the stream is registered"))
            }
            _ => Err(RpcError::UnknownStream),
        }
    }

    pub(super) fn budget(&self) -> &Arc<Semaphore> {
        &self.budget
    }
}

impl Entry {
    async fn send(&self, payload: Vec<u8>) -> Result<(), RpcError> {
        if payload.len() > self.max_request_bytes {
            return Err(RpcError::LimitExceeded);
        }
        let sender = self.input.lock().await.clone().ok_or(RpcError::UnknownStream)?;
        tokio::select! {
            biased;
            _ = self.cancel.cancelled() => Err(RpcError::Cancelled),
            sent = sender.send(payload) => sent.map_err(|_| RpcError::UnknownStream),
        }
    }

    fn acknowledge(&self, count: u64) -> Result<(), RpcError> {
        let mut delivery = self.delivery.lock().expect("RPC stream credit poisoned");
        let acknowledged =
            delivery.acknowledged.checked_add(count).filter(|acknowledged| *acknowledged <= delivery.sent).ok_or(RpcError::InvalidAck)?;
        delivery.acknowledged = acknowledged;
        drop(delivery);
        self.credit.add_permits(count as usize);
        Ok(())
    }
}

/* === Public Functions === */

#[tauri::command]
pub async fn rpc<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<Response, RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    let path = path(request.headers())?;
    let deadline =
        deadline(request.headers(), Duration::from_millis(DESKTOP_UNARY_TIMEOUT_MS), Duration::from_millis(DESKTOP_STREAM_TIMEOUT_MS))?;
    // Unary calls wait for their budget: they are short, and a burst of
    // queries must not fail because a few others are still in flight.
    let permit = Arc::clone(&bridge.calls).acquire_owned().await.map_err(|_| RpcError::Cancelled)?;
    let context = bridge.context(&session, webview.label(), generation, Some(permit))?;
    let client = context.client();
    let response = context.execute(client.call_raw(path, payload.to_vec(), deadline)).await?;
    context.reply_bytes(&response)
}

#[tauri::command]
pub async fn rpc_stream_open<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<Response, RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    let headers = request.headers();
    let opening = Opening {
        path: path(headers)?.to_owned(),
        deadline: deadline(headers, Duration::from_millis(DESKTOP_STREAM_TIMEOUT_MS), Duration::from_millis(DESKTOP_STREAM_TIMEOUT_MS))?,
        payload: payload.to_vec(),
        keep_input: match header(headers, INPUT_HEADER)? {
            None | Some("closed") => false,
            Some("open") => true,
            Some(_) => return Err(RpcError::InvalidRequest),
        },
    };
    let channel = JavaScriptChannelId::from_str(header(headers, CHANNEL_HEADER)?.ok_or(RpcError::InvalidRequest)?)
        .map_err(|_| RpcError::InvalidRequest)?
        .channel_on(webview.as_ref().clone());
    let permit = bridge.admit(bridge.streams.budget())?;
    let context = bridge.context(&session, webview.label(), generation, Some(permit))?;
    bridge.streams.open(context, channel, opening)
}

#[tauri::command]
pub async fn rpc_stream_send<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    let entry = bridge.streams.get(webview.label(), generation, stream(request.headers())?)?;
    entry.send(payload.to_vec()).await
}

#[tauri::command]
pub async fn rpc_stream_finish<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, _) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    let entry = bridge.streams.get(webview.label(), generation, stream(request.headers())?)?;
    entry.input.lock().await.take().map(drop).ok_or(RpcError::UnknownStream)
}

#[tauri::command]
pub async fn rpc_stream_cancel<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, _) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    bridge.streams.remove(webview.label(), generation, stream(request.headers())?)?.cancel.cancel();
    Ok(())
}

#[tauri::command]
pub async fn rpc_stream_ack<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, _) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    let headers = request.headers();
    let count = header(headers, CREDIT_HEADER)?
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|count| (1..=DESKTOP_STREAM_CREDIT).contains(count))
        .ok_or(RpcError::InvalidAck)?;
    bridge.streams.get(webview.label(), generation, stream(headers)?)?.acknowledge(count)
}

/* === Private Functions === */

async fn forward(
    entry: &Entry, context: &Context, client: &dokime::client::Client, channel: &Channel<Response>, path: &str,
    receiver: mpsc::Receiver<Vec<u8>>, deadline: Duration,
) -> Result<(), RpcError> {
    let mut incoming = client.stream_raw(path, ReceiverStream::new(receiver), deadline).await?;
    loop {
        entry.credit.acquire().await.map_err(|_| RpcError::Cancelled)?.forget();
        let Some(message) = incoming.message().await? else {
            return Ok(());
        };
        context.check()?;
        entry.delivery.lock().expect("RPC stream credit poisoned").sent += 1;
        channel.send(Response::new(frame(context.generation, FRAME_MESSAGE, &message))).map_err(|_| RpcError::Cancelled)?;
    }
}

fn frame(generation: u64, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(9 + payload.len());
    bytes.extend_from_slice(&generation.to_le_bytes());
    bytes.push(kind);
    bytes.extend_from_slice(payload);
    bytes
}

fn header<'a>(headers: &'a tauri::http::HeaderMap, name: &str) -> Result<Option<&'a str>, RpcError> {
    headers.get(name).map(|value| value.to_str().map_err(|_| RpcError::InvalidRequest)).transpose()
}

fn path(headers: &tauri::http::HeaderMap) -> Result<&str, RpcError> {
    let path = header(headers, PATH_HEADER)?.ok_or(RpcError::InvalidRequest)?;
    let (service, method) = path.strip_prefix('/').and_then(|path| path.split_once('/')).ok_or(RpcError::InvalidRequest)?;
    let valid = !service.is_empty()
        && !method.is_empty()
        && service.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
        && method.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    valid.then_some(path).ok_or(RpcError::InvalidRequest)
}

fn deadline(headers: &tauri::http::HeaderMap, default: Duration, maximum: Duration) -> Result<Duration, RpcError> {
    let Some(value) = header(headers, DEADLINE_HEADER)? else {
        return Ok(default);
    };
    value
        .parse::<u64>()
        .ok()
        .filter(|milliseconds| *milliseconds > 0)
        .map(|milliseconds| Duration::from_millis(milliseconds).min(maximum))
        .ok_or(RpcError::InvalidRequest)
}

fn stream(headers: &tauri::http::HeaderMap) -> Result<u64, RpcError> {
    header(headers, STREAM_HEADER)?.and_then(|value| value.parse().ok()).ok_or(RpcError::UnknownStream)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> tauri::http::HeaderMap {
        pairs
            .iter()
            .map(|(name, value)| (tauri::http::HeaderName::from_static(name), tauri::http::HeaderValue::from_static(value)))
            .collect()
    }

    #[test]
    fn paths_name_one_method() {
        for valid in ["/dokime.v1.Plots/Watch", "/a/b"] {
            assert_eq!(path(&headers(&[(PATH_HEADER, valid)])).unwrap(), valid);
        }
        for invalid in [
            "",
            "/",
            "dokime.v1.Plots/Watch",
            "/dokime.v1.Plots/",
            "//Watch",
            "/dokime.v1.Plots/Watch/extra",
            "/dokime.v1.Plots/Wa.tch",
            "/dokime v1/Watch",
        ] {
            assert!(path(&headers(&[(PATH_HEADER, invalid)])).is_err(), "{invalid}");
        }
        assert!(path(&headers(&[])).is_err());
    }

    #[test]
    fn deadlines_are_positive_and_bounded() {
        let unary = Duration::from_millis(DESKTOP_UNARY_TIMEOUT_MS);
        let maximum = Duration::from_millis(DESKTOP_STREAM_TIMEOUT_MS);
        assert_eq!(deadline(&headers(&[]), unary, maximum).unwrap(), unary);
        assert_eq!(deadline(&headers(&[(DEADLINE_HEADER, "1500")]), unary, maximum).unwrap(), Duration::from_millis(1500));
        assert_eq!(deadline(&headers(&[(DEADLINE_HEADER, "999999999999")]), unary, maximum).unwrap(), maximum);
        for invalid in ["0", "-1", "soon"] {
            assert!(deadline(&headers(&[(DEADLINE_HEADER, invalid)]), unary, maximum).is_err());
        }
    }

    #[test]
    fn acknowledgements_never_exceed_delivery() {
        let entry = Entry {
            owner: "main".into(),
            generation: 1,
            cancel: CancellationToken::new(),
            input: tokio::sync::Mutex::new(None),
            max_request_bytes: 1,
            credit: Arc::new(Semaphore::new(0)),
            delivery: Mutex::new(Delivery { sent: 3, acknowledged: 0 }),
        };
        entry.acknowledge(2).unwrap();
        assert_eq!(entry.credit.available_permits(), 2);
        assert!(matches!(entry.acknowledge(2), Err(RpcError::InvalidAck)));
        entry.acknowledge(1).unwrap();
        assert_eq!(entry.credit.available_permits(), 3);
    }
}
