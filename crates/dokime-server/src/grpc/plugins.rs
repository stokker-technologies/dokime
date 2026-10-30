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
use sha2::{Digest, Sha256};
use tokio::sync::OwnedSemaphorePermit;
use tonic::Status;
use uuid::Uuid;

// Local Imports
use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::domain::model::plugins::wire as plugins_model;
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::{StreamInput, StreamItem};
use dokime::harmos::{Context, Response};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::plugins as contract;
use dokime::plugins::{PluginError, PluginStore};

/* === Constants === */

const SERVER_DIAGNOSTIC_TEXT_BYTES: u64 = 4_096;
const SERVER_PLUGIN_BUNDLE_BYTES: u64 = 285_212_672;
const SERVER_PLUGIN_UPLOAD_IDLE_MS: u64 = 30_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    pub runtime: LiveRuntime,
    pub admission: admission::Admission,
    pub max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Resources {
    async fn submit<T: Send + 'static, F: std::future::Future<Output = Result<T, PluginError>> + Send + 'static>(
        &self, permit: OwnedSemaphorePermit, operation: impl FnOnce(PluginStore) -> F + Send + 'static,
    ) -> Result<T, Status> {
        let store = dokime::app::live_runtime(&self.runtime).await?.plugin_store();
        tokio::spawn(async move {
            let _permit = permit;
            operation(store).await
        })
        .await
        .map_err(join)?
        .map_err(|error| failure(error, SERVER_DIAGNOSTIC_TEXT_BYTES as usize))
    }
}

impl Resources {
    fn produce<T: Portable + Send + Sync + 'static>(&self, build: impl FnOnce() -> Result<T, Status>) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.wire_len()? > self.max_message_bytes {
            return Err(Status::resource_exhausted("plugin response exceeds wire allowance"));
        }
        Ok(output.into())
    }
    async fn entry(&self, mut entry: dokime::plugins::PluginEntry) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        dokime::app::live_runtime(&self.runtime).await?.plugin_process_state(&mut entry);
        wire::normalize(&mut entry, self.max_message_bytes)?;
        self.produce(|| wire::entry(entry, self.max_message_bytes))
    }
}

impl Serve<v1::ListInstallations> for Resources {
    fn serve<'a>(
        &'a self, mut request: v1::ListInstallations, _: Context<'a, v1::ListInstallations>,
    ) -> Served<'a, plugins_model::InstallationPage> {
        Box::pin(async move {
            let size = dokime::domain::model::execution::wire_cursor::page_size(request.page_size, "")
                .map_err(super::value::canonical::cursor_failure)?;
            let after =
                request.after.take().map(|cursor| Ok::<_, Status>((id(&cursor.after_install_id)?, cursor.manifest_sha256))).transpose()?;
            let page = self
                .submit(self.admission.read()?, move |store| async move {
                    store.page(after.as_ref().map(|(id, hash)| (*id, hash.as_str())), size).await
                })
                .await?;
            let mut page = page;
            for entry in &mut page.entries {
                dokime::app::live_runtime(&self.runtime).await?.plugin_process_state(entry);
                wire::normalize(entry, self.max_message_bytes)?;
            }
            self.produce(|| {
                Ok(plugins_model::InstallationPage {
                    installations: page
                        .entries
                        .into_iter()
                        .map(|entry| wire::entry(entry, self.max_message_bytes))
                        .collect::<Result<_, Status>>()?,
                    next: page.next.map(|id| plugins_model::InstallationCursor {
                        manifest_sha256: page.manifest_sha256.clone(),
                        after_install_id: id.to_string(),
                    }),
                    total_count: page.total as u64,
                    manifest_sha256: page.manifest_sha256,
                })
            })
        })
    }
}

impl Serve<v1::GetInstallation> for Resources {
    fn serve<'a>(
        &'a self, request: v1::GetInstallation, _: Context<'a, v1::GetInstallation>,
    ) -> Served<'a, plugins_model::PluginInstallation> {
        Box::pin(async move {
            let id = id(&request.install_id)?;
            let entry = self.submit(self.admission.read()?, move |store| async move { store.get(id).await }).await?;
            self.entry(entry).await
        })
    }
}

impl Serve<v1::Install> for Resources {
    fn serve<'a>(&'a self, _: v1::Install, ctx: Context<'a, v1::Install>) -> Served<'a, plugins_model::PluginInstallation> {
        Box::pin(async move {
            let call_context = ctx.call();
            let input = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let operation = dokime::app::live_runtime(&self.runtime).await?.operation_progress().begin("Installing Plugin");
                operation.phase("Receiving Archive");
                let installed = self.install(input, &operation).await;
                if let Err(status) = &installed {
                    operation.fail(status.message());
                }
                installed
            })
            .await
        })
    }
}

impl Serve<v1::SetEnabled> for Resources {
    fn serve<'a>(&'a self, request: v1::SetEnabled, _: Context<'a, v1::SetEnabled>) -> Served<'a, plugins_model::PluginInstallation> {
        Box::pin(async move {
            let id = id(&request.install_id)?;
            let enabled = request.enabled;
            let entry = self
                .submit(self.admission.mutation()?, move |store| async move {
                    store.set_enabled(id, enabled).await?;
                    store.get(id).await
                })
                .await?;
            self.entry(entry).await
        })
    }
}

impl Serve<v1::Remove> for Resources {
    fn serve<'a>(&'a self, request: v1::Remove, _: Context<'a, v1::Remove>) -> Served<'a, plugins_model::RemovePluginReply> {
        Box::pin(async move {
            let id = id(&request.install_id)?;
            let entry = self
                .submit(self.admission.mutation()?, move |store| async move {
                    store.remove(id).await?;
                    match store.get(id).await {
                        Ok(entry) => Ok(Some(entry)),
                        Err(PluginError::Unknown(_)) => Ok(None),
                        Err(error) => Err(error),
                    }
                })
                .await?;
            let mut entry = entry;
            if let Some(entry) = &mut entry {
                dokime::app::live_runtime(&self.runtime).await?.plugin_process_state(entry);
                wire::normalize(entry, self.max_message_bytes)?;
            }
            self.produce(|| {
                Ok(plugins_model::RemovePluginReply {
                    installation: entry.map(|entry| wire::entry(entry, self.max_message_bytes)).transpose()?,
                })
            })
        })
    }
}

impl Serve<v1::ListAdmittedPlugins> for Resources {
    fn serve<'a>(
        &'a self, mut request: v1::ListAdmittedPlugins, ctx: Context<'a, v1::ListAdmittedPlugins>,
    ) -> Served<'a, plugins_model::AdmittedPluginPage> {
        Box::pin(async move {
            let context = ctx.call();
            let local = context.get::<super::access::Principal>().is_some_and(super::access::Principal::is_local);
            inventory::page(&dokime::app::live_runtime(&self.runtime).await?, &mut request, self.max_message_bytes, local)
        })
    }
}

impl Resources {
    async fn install(
        &self, input: &StreamInput<plugins_model::InstallPluginFrame>, operation: &dokime::runtime::progress::Operation,
    ) -> Result<Response<plugins_model::PluginInstallation>, Status> {
        let permit = self.admission.mutation().map_err(reject)?;
        let first = message(input, SERVER_PLUGIN_UPLOAD_IDLE_MS)
            .await?
            .ok_or_else(|| reject(Status::invalid_argument("install header is required")))?;
        let Some(plugins_model::InstallPluginFrameFrame::Header(header)) = &first.frame else {
            return Err(reject(Status::invalid_argument("first install frame must be a header")));
        };
        digest(&header.sha256).map_err(reject)?;
        if header.size_bytes == 0 || header.size_bytes > SERVER_PLUGIN_BUNDLE_BYTES {
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
        while let Some(frame) = message(input, SERVER_PLUGIN_UPLOAD_IDLE_MS).await? {
            let Some(plugins_model::InstallPluginFrameFrame::Chunk(chunk)) = &frame.frame else {
                return Err(reject(Status::invalid_argument("only chunks may follow install header")));
            };
            if chunk.offset != bytes.len() as u64 || chunk.data.is_empty() || chunk.data.len() > self.max_message_bytes {
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
        let runtime = dokime::app::live_runtime(&self.runtime).await?.clone();
        let inspect_started = Instant::now();
        let maximum = self.max_message_bytes;
        let entry = match tokio::spawn(async move {
            let _permit = permit;
            runtime
                .install_plugin(bytes, move |entry| {
                    let reply = wire::entry(entry.clone(), maximum).map_err(|_| PluginError::ResultLimit)?;
                    if reply.wire_len().map_err(|_| PluginError::ResultLimit)? > maximum {
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
            Ok(Err(error)) => {
                return Err(reject(failure(error, SERVER_DIAGNOSTIC_TEXT_BYTES as usize)));
            }
            Err(status) => return Err(reject(status)),
        };
        tracing::info!(elapsed_ms = inspect_started.elapsed().as_millis(), "plugin installation inspection completed",);
        self.entry(entry).await
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        admission: admission::Admission::new(),
        max_message_bytes: maximum,
    });

    setup.resource::<Host<v1::ListInstallations>>(Host::<v1::ListInstallations>::new(served.clone()));
    setup.resource::<Host<v1::GetInstallation>>(Host::<v1::GetInstallation>::new(served.clone()));
    setup.resource::<Host<v1::Install>>(Host::<v1::Install>::new(served.clone()));
    setup.resource::<Host<v1::SetEnabled>>(Host::<v1::SetEnabled>::new(served.clone()));
    setup.resource::<Host<v1::Remove>>(Host::<v1::Remove>::new(served.clone()));
    setup.resource::<Host<v1::ListAdmittedPlugins>>(Host::<v1::ListAdmittedPlugins>::new(served.clone()));

    Ok(())
}

/* === Private Functions === */

async fn message(
    input: &StreamInput<plugins_model::InstallPluginFrame>, idle_ms: u64,
) -> Result<Option<StreamItem<plugins_model::InstallPluginFrame>>, Status> {
    tokio::time::timeout(Duration::from_millis(idle_ms), input.recv())
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

fn join(_: tokio::task::JoinError) -> Status {
    Status::internal("plugin worker failed")
}

fn failure(error: PluginError, diagnostic_bytes: usize) -> Status {
    match error {
        PluginError::ResultLimit => Status::resource_exhausted("plugin installation result exceeds response budget"),
        PluginError::Limit => Status::resource_exhausted("plugin query IO budget exceeded; request a smaller page"),
        PluginError::Cursor => Status::failed_precondition("plugin manifest cursor changed; restart listing"),
        PluginError::Unknown(_) => Status::not_found("plugin installation not found"),
        PluginError::Invalid(message) => Status::invalid_argument(if message.len() <= diagnostic_bytes {
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
