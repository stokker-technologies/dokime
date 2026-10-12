/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod inventory;
mod wire;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;
// Local Imports
use super::value;
use dokime_app::{
    RuntimeHandle,
    plugins::{PluginError, PluginStore},
};
use dokime_protocol::v1::{self, plugins_server::Plugins};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub runtime: RuntimeHandle,
    pub admission: Arc<Semaphore>,
    pub stopping: watch::Receiver<bool>,
    pub max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Service {
    fn permit(&self) -> Result<OwnedSemaphorePermit, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("backend is draining"));
        }
        let permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::resource_exhausted("plugin management is busy"))?;
        if *self.stopping.borrow() {
            return Err(Status::unavailable("backend is draining"));
        }
        Ok(permit)
    }
    async fn blocking<T: Send + 'static>(
        &self,
        operation: impl FnOnce(PluginStore) -> Result<T, PluginError> + Send + 'static,
    ) -> Result<T, Status> {
        let permit = self.permit()?;
        let store = self.runtime.plugin_store();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            operation(store)
        })
        .await
        .map_err(join)?
        .map_err(failure)
    }
    fn entry(
        &self,
        mut entry: dokime_app::plugins::PluginEntry,
    ) -> Result<v1::PluginInstallation, Status> {
        self.runtime.plugin_process_state(&mut entry);
        wire::entry(entry, self.max_message_bytes)
    }
}

#[tonic::async_trait]
impl Plugins for Service {
    async fn list_installations(
        &self,
        request: Request<v1::ListInstallationsRequest>,
    ) -> Result<Response<v1::InstallationPage>, Status> {
        let request = request.into_inner();
        let size = page_size(request.page_size)?;
        let after = request
            .after
            .map(|cursor| Ok::<_, Status>((id(&cursor.after_install_id)?, cursor.manifest_sha256)))
            .transpose()?;
        let page = self
            .blocking(move |store| {
                store.page(after.as_ref().map(|(id, hash)| (*id, hash.as_str())), size)
            })
            .await?;
        let mut installations = Vec::new();
        let mut bytes = 256usize;
        for entry in page.entries {
            let entry = self.entry(entry)?;
            bytes = bytes
                .saturating_add(prost::Message::encoded_len(&entry))
                .saturating_add(8);
            if bytes > self.max_message_bytes {
                return Err(Status::resource_exhausted(
                    "plugin page exceeds transport budget",
                ));
            }
            installations.push(entry);
        }
        Ok(Response::new(v1::InstallationPage {
            installations,
            next: page.next.map(|id| v1::InstallationCursor {
                manifest_sha256: page.manifest_sha256.clone(),
                after_install_id: id.to_string(),
            }),
            total_count: page.total as u64,
            manifest_sha256: page.manifest_sha256,
        }))
    }
    async fn get_installation(
        &self,
        request: Request<v1::GetInstallationRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        let id = id(&request.into_inner().install_id)?;
        let entry = self.blocking(move |store| store.get(id)).await?;
        Ok(Response::new(self.entry(entry)?))
    }
    async fn stage(
        &self,
        request: Request<Streaming<v1::StagePluginFrame>>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        let permit = self.permit().map_err(reject)?;
        let mut input = request.into_inner();
        let mut stopping = self.stopping.clone();
        // `message` (and the `with_idle` deadline inside it) already warns
        // and tags its own rejections; only the two checks below construct a
        // new `Status` that still needs one.
        let first = message(&mut input, &mut stopping)
            .await?
            .ok_or_else(|| reject(Status::invalid_argument("stage header is required")))?;
        let Some(v1::stage_plugin_frame::Frame::Header(header)) = first.frame else {
            return Err(reject(Status::invalid_argument(
                "first stage frame must be a header",
            )));
        };
        digest(&header.sha256).map_err(reject)?;
        const LIMIT: u64 = 272 * 1024 * 1024;
        if header.size_bytes == 0 || header.size_bytes > LIMIT {
            return Err(reject(Status::resource_exhausted(
                "plugin archive size exceeds native limit",
            )));
        }
        tracing::info!(
            declared_bytes = header.size_bytes,
            digest_prefix = &header.sha256[..8],
            "plugin stage header received",
        );
        let received_started = Instant::now();
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(header.size_bytes as usize)
            .map_err(|_| {
                reject(Status::resource_exhausted(
                    "plugin upload memory unavailable",
                ))
            })?;
        let mut hash = Sha256::new();
        while let Some(frame) = message(&mut input, &mut stopping).await? {
            let Some(v1::stage_plugin_frame::Frame::Chunk(chunk)) = frame.frame else {
                return Err(reject(Status::invalid_argument(
                    "only chunks may follow stage header",
                )));
            };
            if chunk.offset != bytes.len() as u64
                || chunk.data.is_empty()
                || chunk.data.len() > 256 * 1024
            {
                return Err(reject(Status::invalid_argument(
                    "invalid stage chunk offset or size",
                )));
            }
            if chunk.data.len() as u64 > header.size_bytes.saturating_sub(bytes.len() as u64) {
                return Err(reject(Status::invalid_argument(
                    "stage exceeds declared size",
                )));
            }
            hash.update(&chunk.data);
            bytes.extend_from_slice(&chunk.data);
        }
        if bytes.len() as u64 != header.size_bytes
            || hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != header.sha256
        {
            return Err(reject(Status::invalid_argument(
                "stage size or digest mismatch",
            )));
        }
        if *stopping.borrow() {
            return Err(reject(Status::unavailable("backend is draining")));
        }
        tracing::info!(
            received_bytes = bytes.len(),
            elapsed_ms = received_started.elapsed().as_millis(),
            "plugin stage upload completed",
        );
        let store = self.runtime.plugin_store();
        let inspect_started = Instant::now();
        let entry = match tokio::task::spawn_blocking(move || {
            let _permit = permit;
            store.stage(&bytes)
        })
        .await
        .map_err(join)
        {
            Ok(Ok(entry)) => entry,
            Ok(Err(error)) => return Err(reject(failure(error))),
            Err(status) => return Err(reject(status)),
        };
        tracing::info!(
            elapsed_ms = inspect_started.elapsed().as_millis(),
            "plugin stage inspection completed",
        );
        Ok(Response::new(self.entry(entry)?))
    }
    async fn consent(
        &self,
        request: Request<v1::ConsentPluginRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        let request = request.into_inner();
        let id = id(&request.install_id)?;
        digest(&request.expected_sha256)?;
        let permit = self.permit()?;
        let runtime = self.runtime.clone();
        let max_message_bytes = self.max_message_bytes;
        // Admission retains its permit after transport cancellation. Consent can
        // already have published; callers reconcile by UUID, never blindly retry.
        let entry = tokio::spawn(async move {
            let _permit = permit;
            let store = runtime.plugin_store();
            let current = tokio::task::spawn_blocking(move || store.get(id))
                .await
                .map_err(join)?
                .map_err(failure)?;
            if current.installation.sha256 != request.expected_sha256 {
                return Err(Status::failed_precondition(
                    "consent digest does not match staged installation",
                ));
            }
            wire::disclosure(&current, max_message_bytes)?;
            runtime.consent_plugin(id).await.map_err(failure)
        })
        .await
        .map_err(join)??;
        Ok(Response::new(self.entry(entry)?))
    }
    async fn set_enabled(
        &self,
        request: Request<v1::SetPluginEnabledRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        let request = request.into_inner();
        let id = id(&request.install_id)?;
        let entry = self
            .blocking(move |store| {
                store.set_enabled(id, request.enabled)?;
                store.get(id)
            })
            .await?;
        Ok(Response::new(self.entry(entry)?))
    }
    async fn remove(
        &self,
        request: Request<v1::RemovePluginRequest>,
    ) -> Result<Response<v1::RemovePluginReply>, Status> {
        let id = id(&request.into_inner().install_id)?;
        let entry = self
            .blocking(move |store| {
                store.remove(id)?;
                match store.get(id) {
                    Ok(entry) => Ok(Some(entry)),
                    Err(PluginError::Unknown(_)) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .await?;
        Ok(Response::new(v1::RemovePluginReply {
            installation: entry.map(|entry| self.entry(entry)).transpose()?,
        }))
    }
    async fn list_admitted_plugins(
        &self,
        request: Request<v1::ListAdmittedPluginsRequest>,
    ) -> Result<Response<v1::AdmittedPluginPage>, Status> {
        inventory::page(&self.runtime, request.into_inner(), self.max_message_bytes)
            .map(Response::new)
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn message(
    input: &mut Streaming<v1::StagePluginFrame>,
    stopping: &mut watch::Receiver<bool>,
) -> Result<Option<v1::StagePluginFrame>, Status> {
    if *stopping.borrow() {
        return Err(reject(Status::unavailable("backend is draining")));
    }
    tokio::select! {
        biased;
        _ = stopping.changed() => Err(reject(Status::unavailable("backend is draining"))),
        result = with_idle(input.message(), Duration::from_secs(30)) => result,
    }
}

async fn with_idle<T>(
    future: impl std::future::Future<Output = Result<Option<T>, Status>>,
    duration: Duration,
) -> Result<Option<T>, Status> {
    tokio::time::timeout(duration, future).await.map_err(|_| {
        reject(Status::deadline_exceeded(
            "plugin upload idle deadline exceeded",
        ))
    })?
}

fn reject(status: Status) -> Status {
    tracing::warn!(code = ?status.code(), message = %status.message(), "plugin stage rejected");
    status
}

fn id(value: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument("invalid plugin installation UUID"))
}

fn digest(value: &str) -> Result<(), Status> {
    if value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        Ok(())
    } else {
        Err(Status::invalid_argument("expected lowercase SHA-256"))
    }
}

fn page_size(value: u32) -> Result<usize, Status> {
    match value {
        0 => Ok(25),
        1..=100 => Ok(value as usize),
        _ => Err(Status::invalid_argument("page size exceeds100")),
    }
}

fn join(_: tokio::task::JoinError) -> Status {
    Status::internal("plugin worker failed")
}

fn failure(error: PluginError) -> Status {
    match error {
        PluginError::Limit => {
            Status::resource_exhausted("plugin query IO budget exceeded; request a smaller page")
        }
        PluginError::Cursor => {
            Status::failed_precondition("plugin manifest cursor changed; restart listing")
        }
        PluginError::Unknown(_) => Status::not_found("plugin installation not found"),
        PluginError::Invalid(message) => Status::invalid_argument(if message.len() <= 4096 {
            format!("invalid plugin bundle: {message}")
        } else {
            "invalid plugin bundle (validation detail exceeds response limit)".into()
        }),
        PluginError::NotStaged(_)
        | PluginError::NotInstalled(_)
        | PluginError::Changed(_)
        | PluginError::Collision(_) => Status::failed_precondition(error.to_string()),
        _ => Status::unavailable(
            "plugin storage unavailable; reconcile installation state before retrying mutation",
        ),
    }
}

/* === Tests === */
