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
use contract as v1;
use dokime::harmos::domain::Portable;
use dokime::harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamInput, StreamItem},
};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tonic::Status;
use uuid::Uuid;

// Local Imports
use super::value;
use dokime::domain::model::plugins::wire as plugins_model;
use dokime::domain::operations::plugins as contract;
use dokime::{
    RuntimeHandle,
    plugins::{PluginError, PluginStore},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    pub runtime: RuntimeHandle,
    pub admission: Arc<Semaphore>,
    pub max_message_bytes: usize,
}

struct ListInstallations {
    resources: Arc<Resources>,
}

struct GetInstallation {
    resources: Arc<Resources>,
}

struct Stage {
    resources: Arc<Resources>,
}

struct Consent {
    resources: Arc<Resources>,
}

struct ConsentError(Status);

struct SetEnabled {
    resources: Arc<Resources>,
}

struct Remove {
    resources: Arc<Resources>,
}

struct ListAdmittedPlugins {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<PluginError> for ConsentError {
    fn from(error: PluginError) -> Self {
        Self(failure(error))
    }
}

impl From<std::io::Error> for ConsentError {
    fn from(error: std::io::Error) -> Self {
        Self(failure(PluginError::Io(error)))
    }
}

impl Resources {
    fn permit(&self) -> Result<OwnedSemaphorePermit, Status> {
        let permit = self.admission.clone().try_acquire_owned().map_err(|_| Status::resource_exhausted("plugin management is busy"))?;
        Ok(permit)
    }
    async fn blocking<T: Send + 'static>(
        &self, operation: impl FnOnce(PluginStore) -> Result<T, PluginError> + Send + 'static,
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
}

impl Resources {
    fn produce<T: MemorySize + Portable + Send + Sync + 'static>(
        &self, build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.encoded_len() > self.max_message_bytes {
            return Err(Status::resource_exhausted("plugin response exceeds wire allowance"));
        }
        Ok(output.into())
    }
    fn entry(&self, mut entry: dokime::plugins::PluginEntry) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        self.runtime.plugin_process_state(&mut entry);
        wire::normalize(&mut entry, self.max_message_bytes)?;
        self.produce(|| wire::entry(entry, self.max_message_bytes))
    }
}

impl Operation for ListInstallations {
    type Declaration = v1::ListInstallations;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::ListInstallations, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::InstallationPage>, Status> {
        let size = page_size(request.page_size)?;
        let after =
            request.after.take().map(|cursor| Ok::<_, Status>((id(&cursor.after_install_id)?, cursor.manifest_sha256))).transpose()?;
        let page = self.resources.blocking(move |store| store.page(after.as_ref().map(|(id, hash)| (*id, hash.as_str())), size)).await?;
        let mut page = page;
        let mut required = std::mem::size_of::<plugins_model::InstallationPage>() + 256;
        for entry in &mut page.entries {
            self.resources.runtime.plugin_process_state(entry);
            wire::normalize(entry, self.resources.max_message_bytes)?;
            required =
                required.saturating_add(wire::memory(entry)?).saturating_add(3 * std::mem::size_of::<plugins_model::PluginInstallation>());
        }
        self.resources.produce(|| {
            Ok(plugins_model::InstallationPage {
                installations: page
                    .entries
                    .into_iter()
                    .map(|entry| wire::entry(entry, self.resources.max_message_bytes))
                    .collect::<Result<_, Status>>()?,
                next: page.next.map(|id| plugins_model::InstallationCursor {
                    manifest_sha256: page.manifest_sha256.clone(),
                    after_install_id: id.to_string(),
                }),
                total_count: page.total as u64,
                manifest_sha256: page.manifest_sha256,
            })
        })
    }
}

impl Operation for GetInstallation {
    type Declaration = v1::GetInstallation;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::GetInstallation, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let id = id(&request.install_id)?;
        let entry = self.resources.blocking(move |store| store.get(id)).await?;
        self.resources.entry(entry)
    }
}

impl Operation for Stage {
    type Declaration = v1::Stage;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamInput<plugins_model::StagePluginFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _request: &mut v1::Stage, input: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let operation = self.resources.runtime.operation_progress().begin("Installing Plugin");
        operation.phase("Receiving Archive");
        let staged = self.stage(input, &operation).await;
        if let Err(status) = &staged {
            operation.fail(status.message());
        }
        staged
    }
}

impl Stage {
    async fn stage(
        &mut self, input: &mut StreamInput<plugins_model::StagePluginFrame>, operation: &dokime::runtime::progress::Operation,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let permit = self.resources.permit().map_err(reject)?;
        let first = message(input).await?.ok_or_else(|| reject(Status::invalid_argument("stage header is required")))?;
        let Some(plugins_model::StagePluginFrameFrame::Header(header)) = &first.frame else {
            return Err(reject(Status::invalid_argument("first stage frame must be a header")));
        };
        digest(&header.sha256).map_err(reject)?;
        if header.size_bytes == 0 || header.size_bytes > plugins_model::STAGE_ARCHIVE_MAX_BYTES {
            return Err(reject(Status::resource_exhausted("plugin archive size exceeds native limit")));
        }
        tracing::info!(declared_bytes = header.size_bytes, digest_prefix = &header.sha256[..8], "plugin stage header received",);
        let size_bytes = header.size_bytes;
        let expected_sha256 = header.sha256.clone();
        drop(first);
        let received_started = Instant::now();
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size_bytes as usize).map_err(|_| reject(Status::resource_exhausted("plugin upload memory unavailable")))?;
        let mut hash = Sha256::new();
        while let Some(frame) = message(input).await? {
            let Some(plugins_model::StagePluginFrameFrame::Chunk(chunk)) = &frame.frame else {
                return Err(reject(Status::invalid_argument("only chunks may follow stage header")));
            };
            if chunk.offset != bytes.len() as u64 || chunk.data.is_empty() || chunk.data.len() > self.resources.max_message_bytes {
                return Err(reject(Status::invalid_argument("invalid stage chunk offset or size")));
            }
            if chunk.data.len() as u64 > size_bytes.saturating_sub(bytes.len() as u64) {
                return Err(reject(Status::invalid_argument("stage exceeds declared size")));
            }
            hash.update(&chunk.data);
            bytes.extend_from_slice(&chunk.data);
            operation.progress(bytes.len() as u64, size_bytes);
        }
        if bytes.len() as u64 != size_bytes
            || hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>() != expected_sha256
        {
            return Err(reject(Status::invalid_argument("stage size or digest mismatch")));
        }
        tracing::info!(received_bytes = bytes.len(), elapsed_ms = received_started.elapsed().as_millis(), "plugin stage upload completed",);
        operation.phase("Inspecting Archive");
        operation.indeterminate();
        let store = self.resources.runtime.plugin_store();
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
        tracing::info!(elapsed_ms = inspect_started.elapsed().as_millis(), "plugin stage inspection completed",);
        self.resources.entry(entry)
    }
}

impl Operation for Consent {
    type Declaration = v1::Consent;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::Consent, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let id = id(&request.install_id)?;
        digest(&request.expected_sha256)?;
        let permit = self.resources.permit()?;
        let expected_sha256 = std::mem::take(&mut request.expected_sha256);
        let runtime = self.resources.runtime.clone();
        let max_message_bytes = self.resources.max_message_bytes;
        // Admission retains its permit after transport cancellation. Consent can
        // already have published; callers reconcile by UUID, never blindly retry.
        let entry = tokio::spawn(async move {
            let _permit = permit;
            runtime
                .consent_plugin(id, move |current| {
                    if current.installation.sha256 != expected_sha256 {
                        return Err(ConsentError(Status::failed_precondition("consent digest does not match staged installation")));
                    }
                    wire::disclosure(current, max_message_bytes).map_err(ConsentError)
                })
                .await
                .map_err(|error| error.0)
        })
        .await
        .map_err(join)??;
        self.resources.entry(entry)
    }
}

impl Operation for SetEnabled {
    type Declaration = v1::SetEnabled;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::SetEnabled, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let id = id(&request.install_id)?;
        let enabled = request.enabled;
        let entry = self
            .resources
            .blocking(move |store| {
                store.set_enabled(id, enabled)?;
                store.get(id)
            })
            .await?;
        self.resources.entry(entry)
    }
}

impl Operation for Remove {
    type Declaration = v1::Remove;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::Remove, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::RemovePluginReply>, Status> {
        let id = id(&request.install_id)?;
        let entry = self
            .resources
            .blocking(move |store| {
                store.remove(id)?;
                match store.get(id) {
                    Ok(entry) => Ok(Some(entry)),
                    Err(PluginError::Unknown(_)) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .await?;
        let mut entry = entry;
        if let Some(entry) = &mut entry {
            self.resources.runtime.plugin_process_state(entry);
            wire::normalize(entry, self.resources.max_message_bytes)?;
        }
        self.resources.produce(|| {
            Ok(plugins_model::RemovePluginReply {
                installation: entry.map(|entry| wire::entry(entry, self.resources.max_message_bytes)).transpose()?,
            })
        })
    }
}

impl Operation for ListAdmittedPlugins {
    type Declaration = v1::ListAdmittedPlugins;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut v1::ListAdmittedPlugins, _: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::AdmittedPluginPage>, Status> {
        let local = context.context().get::<super::access::Principal>().is_some_and(super::access::Principal::is_local);
        inventory::page(&self.resources.runtime, request, self.resources.max_message_bytes, local)
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), admission: Arc::new(Semaphore::new(1)), max_message_bytes: maximum });
    runtime.operations().register_wire_operation::<ListInstallations>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<GetInstallation>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Stage>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Consent>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<SetEnabled>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Remove>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<ListAdmittedPlugins>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    Ok(())
}

/* === Private Functions === */

async fn message(
    input: &mut StreamInput<plugins_model::StagePluginFrame>,
) -> Result<Option<StreamItem<plugins_model::StagePluginFrame>>, Status> {
    tokio::time::timeout(Duration::from_secs(30), input.recv())
        .await
        .map_err(|_| reject(Status::deadline_exceeded("plugin upload idle deadline exceeded")))
}

fn reject(status: Status) -> Status {
    tracing::warn!(code = ?status.code(), message = %status.message(), "plugin stage rejected");
    status
}

fn id(value: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument("invalid plugin installation UUID"))
}

fn digest(value: &str) -> Result<(), Status> {
    if value.len() == 64 && value.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
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
        PluginError::Limit => Status::resource_exhausted("plugin query IO budget exceeded; request a smaller page"),
        PluginError::Cursor => Status::failed_precondition("plugin manifest cursor changed; restart listing"),
        PluginError::Unknown(_) => Status::not_found("plugin installation not found"),
        PluginError::Invalid(message) => Status::invalid_argument(if message.len() <= 4096 {
            format!("invalid plugin bundle: {message}")
        } else {
            "invalid plugin bundle (validation detail exceeds response limit)".into()
        }),
        PluginError::NotStaged(_) | PluginError::NotInstalled(_) | PluginError::Changed(_) | PluginError::Collision(_) => {
            Status::failed_precondition(error.to_string())
        }
        _ => Status::unavailable("plugin storage unavailable; reconcile installation state before retrying mutation"),
    }
}

/* === Tests === */
