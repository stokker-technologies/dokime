/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod admission;
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
use dokime::harmos::domain::{Message, Portable};
use dokime::harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamInput, StreamItem},
};
use sha2::{Digest, Sha256};
use tokio::sync::OwnedSemaphorePermit;
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
    pub admission: admission::Admission,
    pub max_message_bytes: usize,
}

struct ListInstallations {
    resources: Arc<Resources>,
}

struct GetInstallation {
    resources: Arc<Resources>,
}

struct Install {
    resources: Arc<Resources>,
}

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

impl Resources {
    async fn blocking<T: Send + 'static>(
        &self, permit: OwnedSemaphorePermit, operation: impl FnOnce(PluginStore) -> Result<T, PluginError> + Send + 'static,
    ) -> Result<T, Status> {
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
        let page = self
            .resources
            .blocking(self.resources.admission.read()?, move |store| {
                store.page(after.as_ref().map(|(id, hash)| (*id, hash.as_str())), size)
            })
            .await?;
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
        let entry = self.resources.blocking(self.resources.admission.read()?, move |store| store.get(id)).await?;
        self.resources.entry(entry)
    }
}

impl Operation for Install {
    type Declaration = v1::Install;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamInput<plugins_model::InstallPluginFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _request: &mut v1::Install, input: &mut Self::Endpoints,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let operation = self.resources.runtime.operation_progress().begin("Installing Plugin");
        operation.phase("Receiving Archive");
        let installed = self.install(input, &operation).await;
        if let Err(status) = &installed {
            operation.fail(status.message());
        }
        installed
    }
}

impl Install {
    async fn install(
        &mut self, input: &mut StreamInput<plugins_model::InstallPluginFrame>, operation: &dokime::runtime::progress::Operation,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let permit = self.resources.admission.mutation().map_err(reject)?;
        let first = message(input).await?.ok_or_else(|| reject(Status::invalid_argument("install header is required")))?;
        let Some(plugins_model::InstallPluginFrameFrame::Header(header)) = &first.frame else {
            return Err(reject(Status::invalid_argument("first install frame must be a header")));
        };
        digest(&header.sha256).map_err(reject)?;
        if header.size_bytes == 0 || header.size_bytes > plugins_model::INSTALL_ARCHIVE_MAX_BYTES {
            return Err(reject(Status::resource_exhausted("plugin archive size exceeds native limit")));
        }
        tracing::info!(declared_bytes = header.size_bytes, digest_prefix = &header.sha256[..8], "plugin installation header received",);
        let size_bytes = header.size_bytes;
        let expected_sha256 = header.sha256.clone();
        drop(first);
        let received_started = Instant::now();
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size_bytes as usize).map_err(|_| reject(Status::resource_exhausted("plugin upload memory unavailable")))?;
        let mut hash = Sha256::new();
        while let Some(frame) = message(input).await? {
            let Some(plugins_model::InstallPluginFrameFrame::Chunk(chunk)) = &frame.frame else {
                return Err(reject(Status::invalid_argument("only chunks may follow install header")));
            };
            if chunk.offset != bytes.len() as u64 || chunk.data.is_empty() || chunk.data.len() > self.resources.max_message_bytes {
                return Err(reject(Status::invalid_argument("invalid install chunk offset or size")));
            }
            if chunk.data.len() as u64 > size_bytes.saturating_sub(bytes.len() as u64) {
                return Err(reject(Status::invalid_argument("install exceeds declared size")));
            }
            hash.update(&chunk.data);
            bytes.extend_from_slice(&chunk.data);
            operation.progress(bytes.len() as u64, size_bytes);
        }
        if bytes.len() as u64 != size_bytes
            || hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>() != expected_sha256
        {
            return Err(reject(Status::invalid_argument("install size or digest mismatch")));
        }
        tracing::info!(
            received_bytes = bytes.len(),
            elapsed_ms = received_started.elapsed().as_millis(),
            "plugin installation upload completed",
        );
        operation.phase("Inspecting Archive");
        operation.indeterminate();
        let runtime = self.resources.runtime.clone();
        let inspect_started = Instant::now();
        let maximum = self.resources.max_message_bytes;
        let entry = match tokio::spawn(async move {
            let _permit = permit;
            runtime
                .install_plugin(bytes, move |entry| {
                    let reply = wire::entry(entry.clone(), maximum).map_err(|_| PluginError::ResultLimit)?;
                    if reply.encoded_len() > maximum {
                        return Err(PluginError::ResultLimit);
                    }
                    Ok(())
                })
                .await
        })
        .await
        .map_err(join)
        {
            Ok(Ok(entry)) => entry,
            Ok(Err(error)) => return Err(reject(failure(error))),
            Err(status) => return Err(reject(status)),
        };
        tracing::info!(elapsed_ms = inspect_started.elapsed().as_millis(), "plugin installation inspection completed",);
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
            .blocking(self.resources.admission.mutation()?, move |store| {
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
            .blocking(self.resources.admission.mutation()?, move |store| {
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
    let resources = Arc::new(Resources { runtime: runtime.clone(), admission: admission::Admission::new(), max_message_bytes: maximum });
    runtime.operations().register_wire_operation::<ListInstallations>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<GetInstallation>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Install>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<SetEnabled>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Remove>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<ListAdmittedPlugins>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    Ok(())
}

/* === Private Functions === */

async fn message(
    input: &mut StreamInput<plugins_model::InstallPluginFrame>,
) -> Result<Option<StreamItem<plugins_model::InstallPluginFrame>>, Status> {
    tokio::time::timeout(Duration::from_secs(30), input.recv())
        .await
        .map_err(|_| reject(Status::deadline_exceeded("plugin upload idle deadline exceeded")))
}

fn reject(status: Status) -> Status {
    tracing::warn!(code = ?status.code(), message = %status.message(), "plugin installation rejected");
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
        PluginError::ResultLimit => Status::resource_exhausted("plugin installation result exceeds response budget"),
        PluginError::Limit => Status::resource_exhausted("plugin query IO budget exceeded; request a smaller page"),
        PluginError::Cursor => Status::failed_precondition("plugin manifest cursor changed; restart listing"),
        PluginError::Unknown(_) => Status::not_found("plugin installation not found"),
        PluginError::Invalid(message) => Status::invalid_argument(if message.len() <= 4096 {
            format!("invalid plugin bundle: {message}")
        } else {
            "invalid plugin bundle (validation detail exceeds response limit)".into()
        }),
        PluginError::NotInstalled(_) | PluginError::Changed(_) | PluginError::Collision(_) => {
            Status::failed_precondition(error.to_string())
        }
        _ => Status::unavailable("plugin storage unavailable; reconcile installation state before retrying mutation"),
    }
}

/* === Tests === */
