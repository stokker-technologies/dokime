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

pub use crate::Error;
pub use calls::{
    CHANNEL_HEADER, CREDIT_HEADER, DEADLINE_HEADER, FRAME_END, FRAME_ERROR, FRAME_MESSAGE, INPUT_HEADER, PATH_HEADER, STREAM_HEADER,
};

/* === Imports === */

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
use crate::connection::{ConnectionError, ConnectionSession, SessionLease, host};
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

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        rpc_error(status)
    }
}

impl From<crate::connection::host::Error> for Error {
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

    async fn execute<T, E: Into<Error>>(&self, future: impl Future<Output = Result<T, E>>) -> Result<T, Error> {
        let result: Result<T, Error> = tokio::select! {
            biased;
            _ = self.owner_cancel.cancelled() => Err(Error::Cancelled),
            result = self.lease.guard(future) => result.map_err(Error::from)?.map_err(Into::into),
        };
        let result = result.map_err(|error| match &error {
            Error::Grpc { message, details, .. } if message.len().saturating_add(details.len()) > protocol::FRAME_BYTES as usize => {
                Error::LimitExceeded
            }
            _ => error,
        });
        result
    }

    fn check(&self) -> Result<(), Error> {
        if self.owner_cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if self.lease.cancellation().is_cancelled() {
            return Err(ConnectionError::Superseded.into());
        }
        Ok(())
    }

    fn reply<M: Message>(&self, message: M) -> Result<tauri::ipc::Response, Error> {
        self.check()?;
        let mut payload = Vec::new();
        message
            .encode(&mut payload)
            .map_err(|error| rpc_error(tonic::Status::internal(format!("Encode Host Reply: {error}; reload the section"))))?;
        self.reply_bytes(&payload)
    }

    fn reply_bytes(&self, payload: &[u8]) -> Result<tauri::ipc::Response, Error> {
        self.check()?;
        if payload.len() > protocol::FRAME_BYTES as usize {
            return Err(Error::LimitExceeded);
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
    ) -> Result<Context, Error> {
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

    fn admit(&self, budget: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, Error> {
        Arc::clone(budget).try_acquire_owned().map_err(|_| Error::Busy)
    }

    fn prepare<M: Message + Default>(
        &self, session: &ConnectionSession, owner: &str, body: &tauri::ipc::InvokeBody,
    ) -> Result<(Context, M), Error> {
        let (generation, payload) = request(body, protocol::FRAME_BYTES as usize)?;
        let permit = self.admit(self.streams.budget())?;
        let context = self.context(session, owner, generation, Some(permit))?;
        let message = decode(payload, protocol::FRAME_BYTES as usize)?;
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

/// Sends Exactly One Generated Failure Protobuf in `error.detail.details`.
fn rpc_error(status: tonic::Status) -> Error {
    let failure = host::status_failure(&status);
    Error::Grpc {
        code: status.code() as i32,
        message: status.message().into(),
        details: failure.encode_to_vec(),
    }
}

fn request(body: &tauri::ipc::InvokeBody, maximum: usize) -> Result<(u64, &[u8]), Error> {
    let tauri::ipc::InvokeBody::Raw(bytes) = body else {
        return Err(Error::InvalidRequest);
    };
    if bytes.len() < REQUEST_HEADER {
        return Err(Error::InvalidRequest);
    }
    if bytes.len() > maximum.saturating_add(REQUEST_HEADER) {
        return Err(Error::LimitExceeded);
    }
    let compiled = descriptor_digest();
    if bytes[8..REQUEST_HEADER] != compiled[..] {
        return Err(Error::IncompatibleDescriptor);
    }
    Ok((read_u64(bytes)?, &bytes[REQUEST_HEADER..]))
}

fn read_u64(bytes: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(bytes.get(..8).ok_or(Error::InvalidRequest)?.try_into().map_err(|_| Error::InvalidRequest)?))
}

fn decode<M: Message + Default>(bytes: &[u8], maximum: usize) -> Result<M, Error> {
    if bytes.len() > maximum {
        return Err(Error::LimitExceeded);
    }
    M::decode(bytes).map_err(|_| Error::InvalidRequest)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn library_carrier_becomes_one_desktop_protobuf() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        use harmos_protocol::model::Failure;
        let captured = STANDARD
            .decode(concat!(
                "AQAAABRMaXN0IFBsdWdpbiBTZXR0aW5ncwEUTGlzdCBQbHVnaW4gU2V0dGluZ3MAAAG3AQgBEhJzZXR0aW5nc19ub3RfcmVhZHka",
                "FExpc3QgUGx1Z2luIFNldHRpbmdzIi1XYWl0IGZvciB0aGUgcGx1Z2luIHRvIHJlcG9ydCBpdHMgZGVzY3JpcHRpb24qFExpc3Qg",
                "UGx1Z2luIFNldHRpbmdzMgViZW5jaDo9EhNkZXNjcmlwdGlvbl9wZW5kaW5nGiZiZW5jaCBoYXMgbm90IHJlcG9ydGVkIGl0cyBk",
                "ZXNjcmlwdGlvbg==",
            ))
            .unwrap();
        let expected = STANDARD
            .decode(concat!(
                "CAESEnNldHRpbmdzX25vdF9yZWFkeRoUTGlzdCBQbHVnaW4gU2V0dGluZ3MiLVdhaXQgZm9yIHRoZSBwbHVnaW4gdG8gcmVwb3J0",
                "IGl0cyBkZXNjcmlwdGlvbioUTGlzdCBQbHVnaW4gU2V0dGluZ3MyBWJlbmNoOj0SE2Rlc2NyaXB0aW9uX3BlbmRpbmcaJmJlbmNo",
                "IGhhcyBub3QgcmVwb3J0ZWQgaXRzIGRlc2NyaXB0aW9u",
            ))
            .unwrap();
        let canonical = Failure::decode(expected.as_slice()).unwrap();
        assert_eq!(harmos_runtime::Failure::from(canonical.clone()).encode(), captured);
        let status = tonic::Status::with_details(tonic::Code::FailedPrecondition, "operation request failed", captured.into());
        let Error::Grpc { code, details, .. } = rpc_error(status) else { panic!("expected gRPC failure") };
        assert_eq!(code, tonic::Code::FailedPrecondition as i32);
        let decoded = Failure::decode(details.as_slice()).unwrap();
        assert_eq!(decoded, Failure::decode(expected.as_slice()).unwrap());
    }

    #[test]
    fn product_and_plain_status_use_the_same_desktop_shape() {
        use harmos_protocol::model::Failure;
        let expected = Failure {
            code: "bench_refused".into(),
            message: "bench_source".into(),
            action: "bench_action".into(),
            plugin: "bench".into(),
            ..Default::default()
        };
        let product = tonic::Status::with_details(tonic::Code::PermissionDenied, "bench_status", expected.encode_to_vec().into());
        assert_eq!(host::status_failure(&product), expected);
        let partial = Failure {
            message: String::new(),
            ..expected.clone()
        };
        let product = tonic::Status::with_details(tonic::Code::PermissionDenied, "bench_status", partial.encode_to_vec().into());
        assert_eq!(host::status_failure(&product), partial);
        let plain = tonic::Status::unavailable("bench_transport");
        let decoded = host::status_failure(&plain);
        assert_eq!(decoded.message, plain.message());
        assert_eq!(decoded.code, plain.code().to_string());
        let malformed = tonic::Status::with_details(tonic::Code::Internal, "bench_transport", vec![255].into());
        let decoded = host::status_failure(&malformed);
        assert!(decoded.message.contains(malformed.message()));
        assert!(decoded.message.contains(&(malformed.code() as i32).to_string()));
    }

    #[test]
    fn command_future_sizes() {
        fn measure<Args, F: Future>(handler: &str, _: impl FnOnce(Args) -> F) {
            let bytes = mem::size_of::<F>();
            eprintln!("{handler}: {bytes} bytes");
            assert!(bytes <= 16 * 1024, "{handler} command future uses {bytes} bytes");
        }
        macro_rules! measure {
            ($handler:path) => {
                measure(
                    stringify!($handler),
                    |(session, bridge, window, request): (
                        tauri::State<'_, ConnectionSession>,
                        tauri::State<'_, Bridge>,
                        tauri::WebviewWindow,
                        tauri::ipc::Request<'_>,
                    )| $handler(session, bridge, window, request),
                );
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
