/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod catalog;
mod cycle;
mod discovery;
mod download;
pub mod dynamic;
mod error;
mod execution;
mod follow;
pub mod operations;
pub mod pairing;
mod plugins;
pub mod profiles;
mod raw;
mod rpc;
pub mod secrets;
mod stream;
mod target;
mod transport;
mod upload;

/* === Re-Exports === */

pub use catalog::template_suite;
pub use cycle::TestCycleHandle;
pub use dokime_domain::operations as contracts;
pub use download::validate_destination as validate_download_destination;
pub use download::{Download, Error as DownloadError};
pub use error::Error;
pub use execution::{RunCallbacks, RunOptions, TestExecutionHandle};
pub use follow::FileResume;
pub use plugins::plugin_bundle_digest;
pub use raw::RawStreaming;
pub use stream::{RequestStream, ResponseStream};
pub use target::Target;
pub use tonic::{Code as RpcCode, Status as RpcStatus};
pub use upload::Error as UploadError;

/* === Imports === */

// Standard Library Imports
use std::{
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};

// Third Party Imports
use serde::Serialize;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};
use zeroize::Zeroizing;

// Local Imports

/* === Constants === */

const DISCOVERY_BYTES: u64 = 1_048_576;
const MAX_MESSAGE_BYTES: u64 = 16_777_216;

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
    timeout: Option<Duration>,
    frame_limit: usize,
}

struct Inner {
    settings: dokime_domain::model::settings::application::ClientPreferences,
    channel: Mutex<Option<Channel>>,
    authorization: Option<MetadataValue<Ascii>>,
    cancellation: CancellationToken,
    readers: TaskTracker,
    backend: Backend,
    expected_backend_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Backend {
    pub backend_id: String,
    pub epoch: String,
    pub display_name: String,
    pub product_version: String,
    pub target: String,
    pub api_version: String,
    pub descriptor_sha256: String,
    pub methods: Vec<String>,
    pub limits: Limits,
}

#[derive(Clone, Debug, Serialize)]
pub struct Limits {
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub max_content_bytes: Option<u64>,
    pub max_evidence_bytes: Option<u64>,
    pub max_chunk_bytes: Option<u32>,
    pub max_page_items: Option<u32>,
    pub max_stream_buffer_items: Option<u32>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionFailure {
    Configuration,
    Authentication,
    PermissionDenied,
    Unreachable,
    TimedOut,
    Incompatible,
    NotReady,
    Failed,
}

/* === Traits === */

/* === Implementations === */

impl Error {
    pub fn connection_failure(&self) -> ConnectionFailure {
        use ConnectionFailure as Failure;
        match self {
            Self::InvalidTarget { .. } => Failure::Configuration,
            Self::AuthenticationRequired => Failure::Authentication,
            Self::IncompatibleApi | Self::IdentityChanged => Failure::Incompatible,
            Self::BackendUnavailable => Failure::NotReady,
            Self::Transport(_) => Failure::Unreachable,
            Self::WaitTimedOut { .. } => Failure::TimedOut,
            Self::Rpc(status) => match status.code() {
                tonic::Code::Unauthenticated => Failure::Authentication,
                tonic::Code::PermissionDenied => Failure::PermissionDenied,
                tonic::Code::Unavailable => Failure::Unreachable,
                tonic::Code::DeadlineExceeded => Failure::TimedOut,
                _ => Failure::Failed,
            },
            _ => Failure::Failed,
        }
    }
}

impl Client {
    pub async fn connect(target: Target, token: Option<&str>, expected_id: Option<&str>) -> Result<Self> {
        let authorization = authorization(token)?;
        let channel = target.connect().await?;
        let settings = *target.settings();
        let negotiation = discovery::discover(channel.clone(), authorization.as_ref(), expected_id, DISCOVERY_BYTES as usize);
        let backend = tokio::time::timeout(Duration::from_millis(settings.request_timeout_ms), negotiation)
            .await
            .map_err(|_| Error::Rpc(tonic::Status::deadline_exceeded("backend negotiation deadline exceeded")))??;
        Ok(Self {
            timeout: Some(Duration::from_millis(settings.request_timeout_ms)),
            frame_limit: MAX_MESSAGE_BYTES as usize,
            inner: Arc::new(Inner {
                settings,
                channel: Mutex::new(Some(channel)),
                authorization,
                cancellation: CancellationToken::new(),
                readers: TaskTracker::new(),
                backend,
                expected_backend_id: expected_id.map(str::to_owned),
            }),
        })
    }

    pub fn require(&self, method: &str) -> Result<()> {
        if self.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }
        if !self.backend().methods.iter().any(|available| available == method) {
            return Err(Error::Rpc(tonic::Status::permission_denied("operation is not available to this caller")));
        }

        Ok(())
    }

    pub async fn verify(&self, live: bool) -> Result<()> {
        let identity = self.identity().await?;
        if identity.backend_id != self.backend().backend_id || (live && identity.epoch != self.backend().epoch) {
            return Err(Error::IdentityChanged);
        }

        Ok(())
    }

    pub fn backend(&self) -> &Backend {
        &self.inner.backend
    }

    pub fn settings(&self) -> &dokime_domain::model::settings::application::ClientPreferences {
        &self.inner.settings
    }

    pub fn channel(&self) -> Result<Channel> {
        self.inner.channel.lock().expect("client transport poisoned").clone().ok_or(Error::Closed)
    }

    pub fn with_timeout(&self, timeout: Duration) -> Self {
        Self {
            inner: self.inner.clone(),
            timeout: Some(timeout),
            frame_limit: self.frame_limit,
        }
    }

    pub fn with_frame_limit(&self, bytes: NonZeroUsize) -> Self {
        Self {
            inner: self.inner.clone(),
            timeout: self.timeout,
            frame_limit: self.frame_limit.min(bytes.get()),
        }
    }

    pub fn request<T>(&self, message: T) -> tonic::Request<T> {
        let mut request = request(message, self.inner.authorization.as_ref());
        if let Some(timeout) = self.timeout {
            request.set_timeout(timeout);
        }
        request
    }

    pub async fn guard<T>(&self, future: impl Future<Output = T>) -> Result<T> {
        let result = tokio::select! {
            biased;
            _ = self.inner.cancellation.cancelled() => return Err(Error::Closed),
            result = future => result,
        };
        if self.inner.cancellation.is_cancelled() {
            return Err(Error::Closed);
        }
        Ok(result)
    }

    pub fn execute<T>(&self, future: impl Future<Output = std::result::Result<T, tonic::Status>>) -> impl Future<Output = Result<T>> {
        // Keep Generated RPC State Out Of Every Enclosing Async Frame.
        let future = Box::pin(future);
        async move {
            let operation = async {
                // A Channel May Reconnect Transparently; Check Identity Before Application Calls.
                if let Some(expected) = &self.inner.expected_backend_id {
                    let identity = transport::unary::<dokime_domain::model::backend::wire::BackendIdentity, _>(
                        self.channel()?,
                        self.inner.authorization.as_ref(),
                        "/dokime.v1.Backend/GetIdentity",
                        contracts::backend::GetIdentity {},
                        DISCOVERY_BYTES as usize,
                        DISCOVERY_BYTES as usize,
                    )
                    .await?;
                    if transport::value(identity)?.backend_id != *expected {
                        return Err(Error::IdentityChanged);
                    }
                }
                future.await.map_err(normalize_rpc_error)
            };
            let operation = async {
                if let Some(timeout) = self.timeout {
                    tokio::time::timeout(timeout, operation)
                        .await
                        .map_err(|_| Error::Rpc(tonic::Status::deadline_exceeded("RPC deadline exceeded")))?
                } else {
                    operation.await
                }
            };
            self.guard(operation).await?
        }
    }

    fn request_limit(&self) -> usize {
        self.backend().limits.max_request_bytes.min(MAX_MESSAGE_BYTES).min(self.frame_limit as u64) as usize
    }

    fn response_limit(&self) -> usize {
        self.backend().limits.max_response_bytes.min(MAX_MESSAGE_BYTES).min(self.frame_limit as u64) as usize
    }

    pub async fn close_and_wait(&self) {
        self.close();
        self.inner.readers.wait().await;
    }

    pub fn close(&self) {
        self.inner.cancellation.cancel();
        let mut channel = self.inner.channel.lock().expect("client transport poisoned");
        channel.take();
        self.inner.readers.close();
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_observed_cursor(epoch: &str, cursor: Option<&dokime_domain::model::execution::wire_cursor::Cursor>) -> Result<()> {
    validate_observed_epoch(epoch, cursor.map(|cursor| cursor.epoch.as_str()))
}

fn validate_observed_epoch(epoch: &str, observed: Option<&str>) -> Result<()> {
    let observed =
        observed.filter(|value| !value.is_empty()).ok_or_else(|| Error::Rpc(tonic::Status::data_loss("reply has no observed epoch")))?;
    if observed != epoch {
        return Err(Error::IdentityChanged);
    }
    Ok(())
}

fn normalize_rpc_error(status: tonic::Status) -> Error {
    let mut source = std::error::Error::source(&status);
    while let Some(error) = source {
        if error.is::<tonic::TimeoutExpired>() {
            return Error::Rpc(tonic::Status::deadline_exceeded("RPC deadline exceeded"));
        }
        source = error.source();
    }
    Error::Rpc(status)
}

fn authorization(token: Option<&str>) -> Result<Option<MetadataValue<Ascii>>> {
    token
        .map(|token| {
            if token.is_empty() || token.chars().any(char::is_whitespace) {
                return Err(Error::AuthenticationRequired);
            }
            let text = Zeroizing::new(format!("Bearer {token}"));
            let mut value = MetadataValue::try_from(text.as_str()).map_err(|_| Error::AuthenticationRequired)?;
            value.set_sensitive(true);
            Ok(value)
        })
        .transpose()
}

fn request<T>(message: T, authorization: Option<&MetadataValue<Ascii>>) -> tonic::Request<T> {
    let mut request = tonic::Request::new(message);
    if let Some(value) = authorization {
        request.metadata_mut().insert("authorization", value.clone());
    }
    request
}

/* === Tests === */
