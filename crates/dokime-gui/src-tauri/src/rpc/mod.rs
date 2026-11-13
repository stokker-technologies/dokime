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
use prost::Message;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::connection::{ConnectionError, ConnectionSession, SessionLease};
use artifact_upload::ArtifactUploads;
use calls::Streams;

/* === Constants === */

const DESKTOP_CALLS: u64 = 8;
const DESKTOP_STREAMS: u64 = 24;

pub const FILE_DESCRIPTOR_SET: &[u8] = dokime_protocol::FILE_DESCRIPTOR_SET;

/// Fence Both the Product and Its Pinned Library Contracts.
pub fn descriptor_digest() -> [u8; 32] {
    crate::connection::host::DESCRIPTOR_SHA256
}
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

impl From<crate::connection::host::Error> for RpcError {
    fn from(error: crate::connection::host::Error) -> Self {
        match error {
            crate::connection::host::Error::Rpc(status) => rpc_error(status),
            error => ConnectionError::from(error).into(),
        }
    }
}

impl Context {
    fn client(&self) -> crate::connection::host::Session {
        self.lease.client()
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

    fn reply<M: Message>(&self, message: M) -> Result<tauri::ipc::Response, RpcError> {
        self.check()?;
        let mut payload = Vec::new();
        message.encode(&mut payload).map_err(|error| RpcError::Grpc {
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
        }
    }

    pub fn cancel_owner(&self, owner: &str) {
        if let Some(token) = self.owners.lock().expect("RPC owners poisoned").remove(owner) {
            token.cancel();
        }
        self.streams.cancel_owner(owner);
        self.uploads.cancel_owner(owner);
    }

    fn context(
        &self, session: &ConnectionSession, owner: &str, generation: u64, permit: Option<OwnedSemaphorePermit>,
    ) -> Result<Context, RpcError> {
        if owner != "main" {
            return Err(ConnectionError::PermissionDenied.into());
        }
        let lease = session.lease(&generation.to_string())?;
        let owner_cancel = self.owners.lock().expect("RPC owners poisoned").entry(owner.into()).or_default().clone();
        Ok(Context {
            generation,
            owner: owner.into(),
            owner_cancel,
            lease,
            _permit: permit,
        })
    }

    fn admit(&self, budget: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, RpcError> {
        Arc::clone(budget).try_acquire_owned().map_err(|_| RpcError::Busy)
    }

    fn prepare<M: Message + Default>(
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
    ]
}

/* === Private Functions === */

fn rpc_error(status: tonic::Status) -> RpcError {
    RpcError::Grpc {
        code: status.code() as i32,
        message: status.message().into(),
        details: status.details().to_vec(),
    }
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
    let compiled = descriptor_digest();
    if bytes[8..REQUEST_HEADER] != compiled[..] {
        return Err(RpcError::IncompatibleDescriptor);
    }
    Ok((read_u64(bytes)?, &bytes[REQUEST_HEADER..]))
}

fn read_u64(bytes: &[u8]) -> Result<u64, RpcError> {
    Ok(u64::from_le_bytes(bytes.get(..8).ok_or(RpcError::InvalidRequest)?.try_into().map_err(|_| RpcError::InvalidRequest)?))
}

fn decode<M: Message + Default>(bytes: &[u8], maximum: usize) -> Result<M, RpcError> {
    if bytes.len() > maximum {
        return Err(RpcError::LimitExceeded);
    }
    M::decode(bytes).map_err(|_| RpcError::InvalidRequest)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn size<Args, F: Future>(_: impl FnOnce(Args) -> F) -> usize {
        std::mem::size_of::<F>()
    }

    #[test]
    fn command_future_sizes() {
        macro_rules! measure {
            ($handler:path) => {
                let bytes = size(
                    |(session, bridge, window, request): (
                        tauri::State<'_, ConnectionSession>,
                        tauri::State<'_, Bridge>,
                        tauri::WebviewWindow,
                        tauri::ipc::Request<'_>,
                    )| $handler(session, bridge, window, request),
                );
                eprintln!("{}: {bytes} bytes", stringify!($handler));
                assert!(bytes <= 16 * 1024, "{} command future uses {bytes} bytes", stringify!($handler));
            };
        }
        measure!(calls::rpc);
        measure!(calls::rpc_stream_open);
        measure!(calls::rpc_stream_send);
        measure!(calls::rpc_stream_finish);
        measure!(calls::rpc_stream_cancel);
        measure!(calls::rpc_stream_ack);
        measure!(artifact_upload::artifact_upload_open);
        measure!(artifact_upload::artifact_upload_write);
        measure!(artifact_upload::artifact_upload_finish);
        measure!(artifact_upload::artifact_upload_cancel);
    }
}
