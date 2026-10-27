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
pub mod protocol;

/* === Re-Exports === */

pub use crate::Error as RpcError;
pub use artifact_upload::RecordingHandle;
pub use calls::{
    CHANNEL_HEADER, CREDIT_HEADER, DEADLINE_HEADER, FRAME_END, FRAME_ERROR, FRAME_MESSAGE, INPUT_HEADER, PATH_HEADER, STREAM_HEADER,
};

/* === Imports === */

use crate::rpc::protocol::FRAME_BYTES as DESKTOP_FRAME_BYTES;

// Standard Library Imports
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

// Third Party Imports
use dokime::harmos::domain::Portable;
use dokime::harmos::grpc::{Transcoded, Wire};
use harmos_sdk::grpc::DecodeInput;
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::connection::{ConnectionError, ConnectionSession, SessionLease};
use artifact_upload::{ArtifactUploads, RecordingSinks};
use calls::Streams;

/* === Constants === */

const DESKTOP_CALLS: u64 = 8;
const DESKTOP_STREAMS: u64 = 24;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/grpc/operations/descriptors.bin"));
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

// Calls Share The Session's HTTP/2 Connection And Configured Admission Budget.
// Long-Lived Streams And Uploads Have Separate Admission From Unary Calls.
const REQUEST_HEADER: usize = 40;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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
        Self::new()
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
        self.lease
            .client()
            .with_frame_limit(std::num::NonZeroUsize::new(DESKTOP_FRAME_BYTES as usize).expect("positive desktop frame bound"))
    }

    async fn execute<T, E: Into<RpcError>>(&self, future: impl Future<Output = Result<T, E>>) -> Result<T, RpcError> {
        let result: Result<T, RpcError> = tokio::select! {
            biased;
            _ = self.owner_cancel.cancelled() => Err(RpcError::Cancelled),
            result = self.lease.guard(future) => result.map_err(RpcError::from)?.map_err(Into::into),
        };
        let result = result.map_err(|error| match &error {
            RpcError::Grpc { message, details, .. } if message.len().saturating_add(details.len()) > DESKTOP_FRAME_BYTES as usize => {
                RpcError::LimitExceeded
            }
            _ => error,
        });
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
        if payload.len() > DESKTOP_FRAME_BYTES as usize {
            return Err(RpcError::LimitExceeded);
        }
        let mut bytes = self.generation.to_le_bytes().to_vec();
        bytes.extend_from_slice(payload);
        Ok(tauri::ipc::Response::new(bytes))
    }
}

impl Bridge {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Semaphore::new(DESKTOP_CALLS as usize)),
            streams: Streams::new(DESKTOP_STREAMS as usize),
            owners: Arc::default(),
            uploads: ArtifactUploads::default(),
            recordings: RecordingSinks::default(),
        }
    }

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

    fn admit(&self, budget: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, RpcError> {
        Arc::clone(budget).try_acquire_owned().map_err(|_| RpcError::Busy)
    }

    fn prepare<M: Portable>(
        &self, session: &ConnectionSession, owner: &str, body: &tauri::ipc::InvokeBody,
    ) -> Result<(Context, M), RpcError> {
        let (generation, payload) = request(body, DESKTOP_FRAME_BYTES as usize)?;
        let permit = self.admit(self.streams.budget())?;
        let context = self.context(session, owner, generation, Some(permit))?;
        let message = decode(payload, DESKTOP_FRAME_BYTES as usize)?;
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
    RpcError::Grpc { code: status.code() as i32, message: status.message().into(), details: status.details().to_vec() }
}

fn request(body: &tauri::ipc::InvokeBody, maximum: usize) -> Result<(u64, &[u8]), RpcError> {
    let tauri::ipc::InvokeBody::Raw(bytes) = body else {
        return Err(RpcError::InvalidRequest);
    };
    if bytes.len() < REQUEST_HEADER {
        return Err(RpcError::InvalidRequest);
    }
    if bytes.len() > maximum.saturating_add(REQUEST_HEADER) {
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

fn decode<M: Portable>(bytes: &[u8], maximum: usize) -> Result<M, RpcError> {
    if bytes.len() > maximum {
        return Err(RpcError::LimitExceeded);
    }
    harmos_sdk::grpc::decode::<M>(DecodeInput::new(bytes)).map_err(|_| RpcError::InvalidRequest)
}

/* === Tests === */
