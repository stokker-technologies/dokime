/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod artifact_upload;
mod client_streams;
mod generated;
mod streams;

/* === Re-Exports === */
pub use crate::Error as RpcError;
pub use artifact_upload::{
    RecordingHandle, artifact_upload_cancel, artifact_upload_finish, artifact_upload_open,
    artifact_upload_write, recording_cancel, recording_open, recording_write,
};
pub use client_streams::{client_stream_cancel, client_stream_finish, client_stream_write};
pub use generated::handler;

/* === Imports === */
// Standard Library Imports
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
// Local Imports
use crate::connection::{ConnectionError, ConnectionSession, SessionLease};
use artifact_upload::{ArtifactUploads, RecordingSinks};
use client_streams::ClientStreams;
use streams::Streams;

/* === Constants === */
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const MAX_CALLS: usize = 32;
const REQUEST_HEADER: usize = 40;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct Bridge {
    owners: Arc<Mutex<HashMap<String, CancellationToken>>>,
    calls: Arc<Semaphore>,
    streams: Streams,
    client_streams: ClientStreams,
    uploads: ArtifactUploads,
    recordings: RecordingSinks,
}

struct Owner {
    generation: u64,
    owner: String,
    owner_cancel: CancellationToken,
    session_cancel: CancellationToken,
    _permit: Option<OwnedSemaphorePermit>,
}

struct Context {
    generation: u64,
    owner: String,
    owner_cancel: CancellationToken,
    lease: SessionLease,
    permit: Option<OwnedSemaphorePermit>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Bridge {
    fn default() -> Self {
        Self {
            owners: Arc::default(),
            calls: Arc::new(Semaphore::new(MAX_CALLS)),
            streams: Streams::default(),
            client_streams: ClientStreams::default(),
            uploads: ArtifactUploads::default(),
            recordings: RecordingSinks::default(),
        }
    }
}

impl Owner {
    fn check(&self) -> Result<(), RpcError> {
        if self.owner_cancel.is_cancelled() {
            return Err(RpcError::Cancelled);
        }
        if self.session_cancel.is_cancelled() {
            return Err(ConnectionError::Superseded.into());
        }
        Ok(())
    }

    async fn execute<T, E: Into<RpcError>>(
        &self,
        future: impl Future<Output = Result<T, E>>,
    ) -> Result<T, RpcError> {
        let result = tokio::select! {
            biased;
            _ = self.owner_cancel.cancelled() => Err(RpcError::Cancelled),
            _ = self.session_cancel.cancelled() => Err(ConnectionError::Superseded.into()),
            result = future => {
                if self.session_cancel.is_cancelled() {
                    return Err(ConnectionError::Superseded.into());
                }
                result.map_err(Into::into)
            },
        };
        if let Err(error) = &result {
            tracing::debug!(owner = %self.owner, generation = self.generation, error = ?error, "rpc call failed");
        }
        result
    }
    fn reply_bytes(&self, payload: &[u8]) -> Result<tauri::ipc::Response, RpcError> {
        self.check()?;
        if payload.len() > MAX_FRAME_BYTES {
            return Err(RpcError::LimitExceeded);
        }
        let mut bytes = self.generation.to_le_bytes().to_vec();
        bytes.extend_from_slice(payload);
        Ok(tauri::ipc::Response::new(bytes))
    }
}

impl From<Context> for Owner {
    fn from(context: Context) -> Self {
        Self {
            generation: context.generation,
            owner: context.owner,
            owner_cancel: context.owner_cancel,
            session_cancel: context.lease.cancellation(),
            _permit: context.permit,
        }
    }
}

impl From<tonic::Status> for RpcError {
    fn from(status: tonic::Status) -> Self {
        rpc_error(status)
    }
}

impl From<dokime::client::Error> for RpcError {
    fn from(error: dokime::client::Error) -> Self {
        match error {
            dokime::client::Error::Rpc(status) => rpc_error(status),
            error => ConnectionError::from(error).into(),
        }
    }
}

impl Context {
    fn client(&self) -> dokime::client::Client {
        self.lease.client().with_frame_limit(
            std::num::NonZeroUsize::new(MAX_FRAME_BYTES).expect("positive desktop frame bound"),
        )
    }

    async fn execute<T, E: Into<RpcError>>(
        &self,
        future: impl Future<Output = Result<T, E>>,
    ) -> Result<T, RpcError> {
        let result: Result<T, RpcError> = tokio::select! {
            biased;
            _ = self.owner_cancel.cancelled() => Err(RpcError::Cancelled),
            result = self.lease.guard(future) => result.map_err(RpcError::from)?.map_err(Into::into),
        };
        if let Err(error) = &result {
            tracing::debug!(
                owner = %self.owner,
                generation = self.generation,
                error = ?error,
                "rpc call failed",
            );
        }
        result
    }
    fn check(&self) -> Result<(), RpcError> {
        if self.owner_cancel.is_cancelled() {
            return Err(RpcError::Cancelled);
        }
        if self.lease.cancellation().is_cancelled() {
            return Err(ConnectionError::Superseded.into());
        }
        Ok(())
    }
    fn reply(&self, message: impl Message) -> Result<tauri::ipc::Response, RpcError> {
        self.check()?;
        if message.encoded_len() > MAX_FRAME_BYTES {
            return Err(RpcError::LimitExceeded);
        }
        let mut bytes = self.generation.to_le_bytes().to_vec();
        message
            .encode(&mut bytes)
            .map_err(|_| RpcError::InvalidRequest)?;
        Ok(tauri::ipc::Response::new(bytes))
    }
    fn reply_bytes(&self, payload: &[u8]) -> Result<tauri::ipc::Response, RpcError> {
        self.check()?;
        if payload.len() > MAX_FRAME_BYTES {
            return Err(RpcError::LimitExceeded);
        }
        let mut bytes = self.generation.to_le_bytes().to_vec();
        bytes.extend_from_slice(payload);
        Ok(tauri::ipc::Response::new(bytes))
    }
}

impl Bridge {
    pub fn cancel_owner(&self, owner: &str) {
        if let Some(token) = self
            .owners
            .lock()
            .expect("RPC owners poisoned")
            .remove(owner)
        {
            token.cancel();
        }
        self.streams.cancel_owner(owner);
        self.client_streams.cancel_owner(owner);
        self.uploads.cancel_owner(owner);
        self.recordings.cancel_owner(owner);
    }
    pub async fn take_recording(
        &self,
        owner: &str,
        generation: u64,
    ) -> Result<artifact_upload::RecordingHandle, RpcError> {
        self.recordings.take(owner, generation).await
    }
    fn context(
        &self,
        session: &ConnectionSession,
        owner: &str,
        generation: u64,
    ) -> Result<Context, RpcError> {
        let lease = session.lease(&generation.to_string())?;
        let permit = self
            .calls
            .clone()
            .try_acquire_owned()
            .map_err(|_| RpcError::Busy)?;
        let owner_cancel = self
            .owners
            .lock()
            .expect("RPC owners poisoned")
            .entry(owner.into())
            .or_default()
            .clone();
        Ok(Context {
            generation,
            owner: owner.into(),
            owner_cancel,
            lease,
            permit: Some(permit),
        })
    }
    fn prepare<M: Message + Default>(
        &self,
        session: &ConnectionSession,
        owner: &str,
        body: &tauri::ipc::InvokeBody,
    ) -> Result<(Context, M), RpcError> {
        let (generation, payload) = request(body)?;
        let context = self.context(session, owner, generation)?;
        let message = M::decode(payload).map_err(|_| RpcError::InvalidRequest)?;
        Ok((context, message))
    }
    fn open<M: Message + Send + 'static>(
        &self,
        mut context: Context,
        stream: dokime::client::ResponseStream<M>,
    ) -> Result<tauri::ipc::Response, RpcError> {
        context.check()?;
        context.permit.take();
        self.streams.open(context, stream)
    }

    fn open_client<M, O, F, Fut>(
        &self,
        context: Context,
        first: M,
        call: F,
    ) -> Result<tauri::ipc::Response, RpcError>
    where
        M: Message + Default + Send + Sync + 'static,
        O: Message + Default + Send + Sync + 'static,
        F: FnOnce(client_streams::ClientWireStream<M>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<O, RpcError>> + Send + 'static,
    {
        self.client_streams.open(context, first, call)
    }
}

/* === Public Functions === */
pub fn artifact_upload_handler<R: tauri::Runtime>()
-> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        artifact_upload::artifact_upload_open,
        artifact_upload::artifact_upload_write,
        artifact_upload::artifact_upload_finish,
        artifact_upload::artifact_upload_cancel,
        artifact_upload::recording_open,
        artifact_upload::recording_write,
        artifact_upload::recording_cancel,
    ]
}

/* === Private Functions === */
fn rpc_error(status: tonic::Status) -> RpcError {
    if status.message().len() + status.details().len() > MAX_FRAME_BYTES {
        return RpcError::LimitExceeded;
    }
    RpcError::Grpc {
        code: status.code() as i32,
        message: status.message().into(),
        details: status.details().to_vec(),
    }
}

fn request(body: &tauri::ipc::InvokeBody) -> Result<(u64, &[u8]), RpcError> {
    let tauri::ipc::InvokeBody::Raw(bytes) = body else {
        return Err(RpcError::InvalidRequest);
    };
    if bytes.len() < REQUEST_HEADER {
        return Err(RpcError::InvalidRequest);
    }
    if bytes.len() > MAX_FRAME_BYTES + REQUEST_HEADER {
        return Err(RpcError::LimitExceeded);
    }
    let compiled = Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET);
    if bytes[8..REQUEST_HEADER] != compiled[..]
        || generated::DESCRIPTOR_SHA256
            != compiled
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
    {
        return Err(RpcError::IncompatibleDescriptor);
    }
    Ok((read_u64(bytes)?, &bytes[REQUEST_HEADER..]))
}

fn read_u64(bytes: &[u8]) -> Result<u64, RpcError> {
    Ok(u64::from_le_bytes(
        bytes
            .get(..8)
            .ok_or(RpcError::InvalidRequest)?
            .try_into()
            .map_err(|_| RpcError::InvalidRequest)?,
    ))
}

#[tauri::command]
async fn stream_next<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>,
    bridge: tauri::State<'_, Bridge>,
    webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    let (generation, payload) = self::request(request.body())?;
    session.lease(&generation.to_string())?;
    if payload.len() != 16 {
        return Err(RpcError::InvalidRequest);
    }
    bridge
        .streams
        .next(
            webview.label(),
            generation,
            read_u64(payload)?,
            read_u64(&payload[8..])?,
        )
        .await
}

#[tauri::command]
fn stream_cancel<R: tauri::Runtime>(
    session: tauri::State<'_, ConnectionSession>,
    bridge: tauri::State<'_, Bridge>,
    webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    let (generation, payload) = self::request(request.body())?;
    session.lease(&generation.to_string())?;
    if payload.len() != 8 {
        return Err(RpcError::InvalidRequest);
    }
    bridge
        .streams
        .cancel(webview.label(), generation, read_u64(payload)?)
}

/* === Tests === */
