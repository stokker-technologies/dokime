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
mod calls;

/* === Re-Exports === */
pub use crate::Error as RpcError;
pub use artifact_upload::RecordingHandle;
pub use calls::{
    CHANNEL_HEADER, CREDIT_HEADER, DEADLINE_HEADER, FRAME_END, FRAME_ERROR, FRAME_MESSAGE, INPUT_HEADER, PATH_HEADER, STREAM_CREDIT,
    STREAM_HEADER,
};

/* === Imports === */
// Standard Library Imports
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
// Third Party Imports
use harmos::MemorySize;
use harmos::domain::Portable;
use harmos::grpc::{Transcoded, Wire};
use harmos_sdk::grpc::DecodeInput;
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
// Local Imports
use crate::connection::{ConnectionError, ConnectionSession, SessionLease};
use artifact_upload::{ArtifactUploads, RecordingSinks};
use calls::Streams;

/* === Constants === */
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/grpc/operations/descriptors.bin"));
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
/// Every Raw-Body Command This Module Answers; The Shell Routes Them Here.
pub const COMMANDS: &[&str] = &[
    "rpc",
    "rpc_stream_open",
    "rpc_stream_send",
    "rpc_stream_finish",
    "rpc_stream_cancel",
    "rpc_stream_ack",
    "artifact_upload_open",
    "artifact_upload_write",
    "artifact_upload_finish",
    "artifact_upload_cancel",
    "recording_open",
    "recording_write",
    "recording_cancel",
];
// Every call shares the session's single HTTP/2 connection, and a backend
// listener admits 32 concurrent calls on it. Long-lived calls, streams and
// uploads alike, own most of that budget and are refused beyond it rather than
// queued behind each other; unary calls queue for the rest, so long-lived calls
// never starve them.
const MAX_CALLS: usize = 8;
const MAX_STREAMS: usize = 24;
const REQUEST_HEADER: usize = 40;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Forwards Opaque Protobuf Calls From The WebView To The Session's Client.
#[derive(Clone)]
pub struct Bridge {
    owners: Arc<Mutex<HashMap<String, CancellationToken>>>,
    calls: Arc<Semaphore>,
    streams: Streams,
    uploads: ArtifactUploads,
    recordings: RecordingSinks,
}

struct Context {
    generation: u64,
    owner: String,
    owner_cancel: CancellationToken,
    lease: SessionLease,
    _permit: Option<OwnedSemaphorePermit>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Bridge {
    fn default() -> Self {
        Self {
            owners: Arc::default(),
            calls: Arc::new(Semaphore::new(MAX_CALLS)),
            streams: Streams::new(MAX_STREAMS),
            uploads: ArtifactUploads::default(),
            recordings: RecordingSinks::default(),
        }
    }
}

impl From<tonic::Status> for RpcError {
    fn from(status: tonic::Status) -> Self {
        rpc_error(status)
    }
}

impl From<dokime_client::Error> for RpcError {
    fn from(error: dokime_client::Error) -> Self {
        match error {
            dokime_client::Error::Rpc(status) => rpc_error(status),
            error => ConnectionError::from(error).into(),
        }
    }
}

impl Context {
    fn client(&self) -> dokime_client::Client {
        self.lease.client().with_frame_limit(std::num::NonZeroUsize::new(MAX_FRAME_BYTES).expect("positive desktop frame bound"))
    }

    async fn execute<T, E: Into<RpcError>>(&self, future: impl Future<Output = Result<T, E>>) -> Result<T, RpcError> {
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

    fn reply<M: Portable>(&self, message: M) -> Result<tauri::ipc::Response, RpcError> {
        self.check()?;
        let mut payload = Vec::new();
        <Transcoded as Wire<M>>::encode(&message, &mut payload).map_err(|error| RpcError::Grpc {
            code: tonic::Code::Internal as i32,
            message: format!("message encoding failed: {error}"),
            details: Vec::new(),
        })?;
        self.reply_bytes(&payload)
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
        if let Some(token) = self.owners.lock().expect("RPC owners poisoned").remove(owner) {
            token.cancel();
        }
        self.streams.cancel_owner(owner);
        self.uploads.cancel_owner(owner);
        self.recordings.cancel_owner(owner);
    }

    pub async fn take_recording(&self, owner: &str, generation: u64) -> Result<artifact_upload::RecordingHandle, RpcError> {
        self.recordings.take(owner, generation).await
    }

    /// Binds One Call To The Main Window's Current Generation.
    fn context(
        &self, session: &ConnectionSession, owner: &str, generation: u64, permit: Option<OwnedSemaphorePermit>,
    ) -> Result<Context, RpcError> {
        if owner != "main" {
            return Err(ConnectionError::PermissionDenied.into());
        }
        let lease = session.lease(&generation.to_string())?;
        let owner_cancel = self.owners.lock().expect("RPC owners poisoned").entry(owner.into()).or_default().clone();
        Ok(Context { generation, owner: owner.into(), owner_cancel, lease, _permit: permit })
    }

    /// Takes A Permit Without Waiting: An Exhausted Budget Answers `Busy`.
    fn admit(&self, budget: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, RpcError> {
        Arc::clone(budget).try_acquire_owned().map_err(|_| RpcError::Busy)
    }

    fn prepare<M: Portable + MemorySize>(
        &self, session: &ConnectionSession, owner: &str, body: &tauri::ipc::InvokeBody,
    ) -> Result<(Context, M), RpcError> {
        let (generation, payload) = request(body)?;
        let permit = self.admit(self.streams.budget())?;
        let context = self.context(session, owner, generation, Some(permit))?;
        let message = decode(payload)?;
        Ok((context, message))
    }
}

/* === Public Functions === */
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        calls::rpc,
        calls::rpc_stream_open,
        calls::rpc_stream_send,
        calls::rpc_stream_finish,
        calls::rpc_stream_cancel,
        calls::rpc_stream_ack,
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
    RpcError::Grpc { code: status.code() as i32, message: status.message().into(), details: status.details().to_vec() }
}

/// Splits A Fenced Request Into Its Connection Generation And Payload.
///
/// Every raw request starts with the generation the WebView believes is
/// current and the SHA-256 of the descriptor set its client was generated from.
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
    let compiled = Sha256::digest(FILE_DESCRIPTOR_SET);
    if bytes[8..REQUEST_HEADER] != compiled[..] {
        return Err(RpcError::IncompatibleDescriptor);
    }
    Ok((read_u64(bytes)?, &bytes[REQUEST_HEADER..]))
}

fn read_u64(bytes: &[u8]) -> Result<u64, RpcError> {
    Ok(u64::from_le_bytes(bytes.get(..8).ok_or(RpcError::InvalidRequest)?.try_into().map_err(|_| RpcError::InvalidRequest)?))
}

/// Native Transfer Input Is Bounded Before Any Message Allocation.
fn decode<M: Portable + MemorySize>(bytes: &[u8]) -> Result<M, RpcError> {
    if bytes.len() > MAX_FRAME_BYTES
        || <Transcoded as Wire<M>>::decoding_memory(DecodeInput::new(bytes)).map_err(|_| RpcError::InvalidRequest)? > MAX_FRAME_BYTES * 8
    {
        return Err(RpcError::LimitExceeded);
    }
    <Transcoded as Wire<M>>::decode(DecodeInput::new(bytes)).map_err(|_| RpcError::InvalidRequest)
}

/* === Tests === */
