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
use crate::rpc::protocol::UPLOAD_SLICE_BYTES as DESKTOP_UPLOAD_SLICE_BYTES;

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use tokio::sync::{Notify, mpsc};
use tokio_stream::wrappers::ReceiverStream;

// Local Imports
use super::*;

/* === Constants === */

const DESKTOP_ACTIVE_UPLOADS: u64 = 2;
const DESKTOP_INPUT_BUFFER_ITEMS: u64 = 2;
const DESKTOP_UPLOAD_HEARTBEAT_MS: u64 = 5_000;
const DESKTOP_UPLOAD_IDLE_MS: u64 = 30_000;
const DESKTOP_UPLOAD_REAPER_MS: u64 = 1_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub(super) struct ArtifactUploads(Arc<Mutex<Registry>>);

#[derive(Default)]
struct Registry {
    next_id: u64,
    reaper_started: bool,
    entries: HashMap<u64, Arc<Upload>>,
}

struct Upload {
    context: Arc<Context>,
    cancel: CancellationToken,
    last_access: Mutex<tokio::time::Instant>,
    writer: tokio::sync::Mutex<UploadWriter>,
    outcome: Arc<Outcome>,
}

struct UploadWriter {
    sender: Option<mpsc::Sender<dokime::domain::model::content::wire::UploadArtifactFrame>>,
    next_sequence: u64,
    offset: u64,
    digest: Sha256,
}

#[derive(Default)]
struct Outcome {
    result: Mutex<Option<Result<dokime::domain::model::content::wire::ArtifactMetadata, RpcError>>>,
    ready: Notify,
}

struct SettleOnDrop(Arc<Outcome>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Outcome {
    fn complete(&self, result: Result<dokime::domain::model::content::wire::ArtifactMetadata, RpcError>) {
        let mut guard = self.result.lock().expect("artifact upload outcome poisoned");
        if guard.is_some() {
            return;
        }
        *guard = Some(result);
        drop(guard);
        self.ready.notify_waiters();
    }

    async fn wait(&self) -> Result<dokime::domain::model::content::wire::ArtifactMetadata, RpcError> {
        loop {
            // Register the notification before inspecting the result. A
            // completion between those operations must wake this waiter.
            let notified = self.ready.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(result) = self.result.lock().expect("artifact upload outcome poisoned").clone() {
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
            message: "the artifact upload driver task ended without a result".into(),
            details: Vec::new(),
        }));
    }
}

impl ArtifactUploads {
    fn open(
        &self, context: Context, header: dokime::domain::model::content::wire::UploadArtifactHeader,
    ) -> Result<tauri::ipc::Response, RpcError> {
        context.check()?;
        let mut encoded = Vec::new();
        <dokime::harmos::grpc::Transcoded as dokime::harmos::grpc::Wire<_>>::encode(&header, &mut encoded)
            .map_err(|_| RpcError::InvalidRequest)?;
        if encoded.len() > DESKTOP_FRAME_BYTES as usize {
            return Err(RpcError::LimitExceeded);
        }
        let (sender, receiver) = mpsc::channel(DESKTOP_INPUT_BUFFER_ITEMS as usize);
        sender
            .try_send(dokime::domain::model::content::wire::UploadArtifactFrame {
                payload: Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Header(header)),
            })
            .map_err(|_| RpcError::Busy)?;
        let context = Arc::new(context);
        let cancel = CancellationToken::new();
        let outcome = Arc::new(Outcome::default());
        let upload = Arc::new(Upload {
            context: Arc::clone(&context),
            cancel: cancel.clone(),
            last_access: Mutex::new(tokio::time::Instant::now()),
            writer: tokio::sync::Mutex::new(UploadWriter {
                sender: Some(sender),
                next_sequence: 1,
                offset: 0,
                digest: Sha256::new(),
            }),
            outcome: Arc::clone(&outcome),
        });
        // Reserve a registry slot before starting the remote call. A
        // rejected open therefore cannot cause any server-side work.
        let client = context.client();
        let response = self.insert(Arc::clone(&upload))?;
        let task_context = Arc::clone(&context);
        let task_cancel = cancel.clone();
        let task_outcome = Arc::clone(&outcome);
        tokio::spawn(async move {
            let _settle_on_drop = SettleOnDrop(Arc::clone(&task_outcome));
            let stream = ReceiverStream::new(receiver);
            let result = tokio::select! {
                biased;
                _ = task_cancel.cancelled() => Err(RpcError::Cancelled),
                result = task_context.execute(client.content_upload_artifact(stream)) => result,
            };
            task_outcome.complete(result);
        });
        Ok(response)
    }

    fn insert(&self, upload: Arc<Upload>) -> Result<tauri::ipc::Response, RpcError> {
        let mut registry = self.0.lock().expect("artifact upload registry poisoned");
        registry.prune();
        if registry.entries.len() >= DESKTOP_ACTIVE_UPLOADS as usize {
            return Err(RpcError::LimitExceeded);
        }
        registry.next_id = registry.next_id.checked_add(1).ok_or(RpcError::LimitExceeded)?;
        let id = registry.next_id;
        let mut response = upload.context.generation.to_le_bytes().to_vec();
        response.extend(id.to_le_bytes());
        registry.entries.insert(id, upload);
        if !registry.reaper_started {
            registry.reaper_started = true;
            let weak = Arc::downgrade(&self.0);
            let interval = DESKTOP_UPLOAD_REAPER_MS;
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                    let Some(registry) = weak.upgrade() else {
                        break;
                    };
                    let mut registry = registry.lock().expect("artifact upload registry poisoned");
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

    fn get(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Upload>, RpcError> {
        let mut registry = self.0.lock().expect("artifact upload registry poisoned");
        registry.prune();
        registry
            .entries
            .get(&id)
            .filter(|upload| upload.context.owner == owner && upload.context.generation == generation)
            .cloned()
            .ok_or(RpcError::UnknownStream)
    }

    async fn write(&self, owner: &str, generation: u64, id: u64, sequence: u64, bytes: Vec<u8>) -> Result<(), RpcError> {
        let upload = self.get(owner, generation, id)?;
        if bytes.len() > DESKTOP_UPLOAD_SLICE_BYTES as usize {
            return Err(RpcError::LimitExceeded);
        }
        let mut writer = upload.writer.lock().await;
        if sequence != writer.next_sequence {
            return Err(RpcError::InvalidAck);
        }
        upload.context.check()?;
        let lease_cancellation = upload.context.lease.cancellation();
        let Some(sender) = writer.sender.as_ref().cloned() else {
            drop(writer);
            return Err(upload.outcome.error().await);
        };
        *upload.last_access.lock().expect("artifact upload idle timer poisoned") = tokio::time::Instant::now();
        let offset = writer.offset;
        let length = bytes.len() as u64;
        writer.digest.update(&bytes);
        let frame = dokime::domain::model::content::wire::UploadArtifactFrame {
            payload: Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Chunk(
                dokime::domain::model::content::wire::ArtifactChunk { offset, data: bytes },
            )),
        };
        let sent = tokio::select! {
            biased;
            _ = upload.cancel.cancelled() => Err(RpcError::Cancelled),
            _ = lease_cancellation.cancelled() => {
                Err(ConnectionError::Superseded.into())
            },
            // A settled remote call (typically an error the server returned
            // before consuming every chunk) must fail this write immediately.
            // Without racing it here, `send` can sit on a full stream the
            // spawned task has already stopped draining, hanging instead of
            // surfacing the failure. Mirrors the same fix in
            // `client_streams::write` against `entry.outcome`.
            error = upload.outcome.error() => Err(error),
            result = sender.send(frame) => result.map_err(|_| RpcError::Cancelled),
        };
        match sent {
            Ok(()) => {
                writer.offset = offset.checked_add(length).ok_or(RpcError::LimitExceeded)?;
                writer.next_sequence = writer.next_sequence.checked_add(1).ok_or(RpcError::LimitExceeded)?;
                *upload.last_access.lock().expect("artifact upload idle timer poisoned") = tokio::time::Instant::now();
                Ok(())
            }
            Err(error) => {
                writer.sender = None;
                drop(writer);
                if matches!(error, RpcError::Cancelled) {
                    return Err(upload.outcome.error().await);
                }
                Err(error)
            }
        }
    }

    async fn finish(&self, owner: &str, generation: u64, id: u64) -> Result<tauri::ipc::Response, RpcError> {
        let upload = self.get(owner, generation, id)?;
        upload.context.check()?;
        *upload.last_access.lock().expect("artifact upload idle timer poisoned") = tokio::time::Instant::now();
        let mut writer = upload.writer.lock().await;
        let Some(sender) = writer.sender.take() else {
            drop(writer);
            return Err(upload.outcome.error().await);
        };
        let size_bytes = writer.offset;
        let digest = std::mem::replace(&mut writer.digest, Sha256::new());
        drop(writer);
        let completion = dokime::domain::model::content::wire::UploadArtifactFrame {
            payload: Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Complete(
                dokime::domain::model::content::wire::ArtifactCompletion {
                    size_bytes,
                    sha256: digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                },
            )),
        };
        // A closed receiver here just means the remote call already ended;
        // the outcome wait below surfaces that failure.
        let _ = sender.send(completion).await;
        drop(sender);
        let result = wait_with_heartbeat(&upload).await;
        self.remove(owner, generation, id);
        upload.context.check()?;
        upload.context.reply(result?)
    }

    async fn cancel(&self, owner: &str, generation: u64, id: u64) -> Result<(), RpcError> {
        let upload = self.get(owner, generation, id)?;
        self.remove(owner, generation, id);
        upload.cancel.cancel();
        upload.writer.lock().await.sender = None;
        Ok(())
    }

    fn remove(&self, owner: &str, generation: u64, id: u64) {
        let mut registry = self.0.lock().expect("artifact upload registry poisoned");
        if registry.entries.get(&id).is_some_and(|upload| upload.context.owner == owner && upload.context.generation == generation) {
            registry.entries.remove(&id);
        }
    }

    pub(super) fn cancel_owner(&self, owner: &str) {
        let mut registry = self.0.lock().expect("artifact upload registry poisoned");
        registry.entries.retain(|_, upload| {
            if upload.context.owner == owner {
                upload.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
}

impl Registry {
    fn prune(&mut self) {
        self.entries.retain(|_, upload| {
            let expired = upload.last_access.lock().expect("artifact upload idle timer poisoned").elapsed()
                >= Duration::from_millis(DESKTOP_UPLOAD_IDLE_MS);
            if upload.cancel.is_cancelled()
                || upload.context.owner_cancel.is_cancelled()
                || upload.context.lease.cancellation().is_cancelled()
                || expired
            {
                upload.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
}

/* === Public Functions === */

#[tauri::command]
pub async fn artifact_upload_open<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    let (context, header) =
        bridge.prepare::<dokime::domain::model::content::wire::UploadArtifactHeader>(&session, webview.label(), request.body())?;
    bridge.uploads.open(context, header)
}

#[tauri::command]
pub async fn artifact_upload_write<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    if payload.len() < 16 {
        return Err(RpcError::InvalidRequest);
    }
    bridge.uploads.write(webview.label(), generation, read_u64(payload)?, read_u64(&payload[8..])?, payload[16..].to_vec()).await
}

#[tauri::command]
pub async fn artifact_upload_finish<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    if payload.len() != 8 {
        return Err(RpcError::InvalidRequest);
    }
    bridge.uploads.finish(webview.label(), generation, read_u64(payload)?).await
}

#[tauri::command]
pub async fn artifact_upload_cancel<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
    session.lease(&generation.to_string())?;
    if payload.len() != 8 {
        return Err(RpcError::InvalidRequest);
    }
    bridge.uploads.cancel(webview.label(), generation, read_u64(payload)?).await
}

/* === Private Functions === */

async fn wait_with_heartbeat(upload: &Upload) -> Result<dokime::domain::model::content::wire::ArtifactMetadata, RpcError> {
    let outcome = upload.outcome.wait();
    tokio::pin!(outcome);
    loop {
        tokio::select! {
            biased;
            result = &mut outcome => return result,
            _ = tokio::time::sleep(Duration::from_millis(DESKTOP_UPLOAD_HEARTBEAT_MS)) => {
                *upload.last_access.lock().expect("artifact upload idle timer poisoned") =
                    tokio::time::Instant::now();
            }
        }
    }
}

/* === Tests === */
