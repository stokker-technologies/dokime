/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::Context;
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::{InvocationError, Response};
use sha2::{Digest, Sha256};
use tonic::Status;

use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::wire::Timestamp;
use dokime::operations::backend as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::runtime as native;

const DOCUMENT_EVIDENCE_EXPORT_BYTES: u64 = 8_589_934_592;
const PAGINATION_MAXIMUM_PAGE_ITEMS: u64 = 500;
const PAGINATION_PAGE_ITEMS: u64 = 50;
const SERVER_DISCOVERY_PAGE_BYTES: u64 = 4_194_304;
const SERVER_DISCOVERY_WORK_BYTES: u64 = 16_777_216;

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
    logs: Arc<crate::grpc::logs::Tail>,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::GetIdentity),
    Operation1(wire::GetHealth),
    Operation2(wire::GetAvailability),
    Operation3(wire::WatchLogs),
    Operation4(wire::CollectDiagnostics),
}

impl Serve<wire::GetIdentity> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetIdentity, _: Context<'a, wire::GetIdentity>,
    ) -> Served<'a, dokime::domain::model::backend::wire::BackendIdentity> {
        Box::pin(async move { self.identity().await })
    }
}

impl Serve<wire::GetHealth> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetHealth, _: Context<'a, wire::GetHealth>,
    ) -> Served<'a, dokime::domain::model::backend::wire::BackendHealth> {
        Box::pin(async move {
            let instance = dokime::app::live_runtime(&self.runtime)
                .await?
                .operations()
                .submit::<native::GetHealth>(native::GetHealth {})
                .map_err(health_failure)?;
            let health = dokime::harmos::grpc::response(instance).await.map_err(health_failure)?;
            produce(self.maximum, || dokime::domain::model::backend::wire::BackendHealth {
                backend_id: health.backend_id.clone(),
                epoch: health.epoch.clone(),
                readiness: match health.readiness {
                    dokime::domain::model::runtime::wire::Readiness::Starting => dokime::domain::model::backend::wire::Readiness::Starting,
                    dokime::domain::model::runtime::wire::Readiness::Ready => dokime::domain::model::backend::wire::Readiness::Ready,
                    dokime::domain::model::runtime::wire::Readiness::Degraded => dokime::domain::model::backend::wire::Readiness::Degraded,
                    dokime::domain::model::runtime::wire::Readiness::Stopping => dokime::domain::model::backend::wire::Readiness::Stopping,
                },
                observed_at: Some(Timestamp {
                    seconds: health.observed_at_seconds,
                    nanos: health.observed_at_nanoseconds as i32,
                }),
                message: health.message.clone(),
            })
        })
    }
}

impl Serve<wire::GetAvailability> for Resources {
    fn serve<'a>(
        &'a self, _: wire::GetAvailability, ctx: Context<'a, wire::GetAvailability>,
    ) -> Served<'a, dokime::domain::model::backend::wire::BackendAvailability> {
        Box::pin(async move {
            use dokime::harmos::builtins::host::{Catalog, CatalogKind, DiscoveredOperation};
            let maximum = self.maximum;
            let page_items = usize::try_from(PAGINATION_PAGE_ITEMS).expect("validated discovery page count");
            let mut methods = Vec::new();
            let mut after = None;
            let mut read_bytes = 0usize;
            loop {
                let page = ctx
                    .operation(Catalog {
                        kind: CatalogKind::Operations,
                        schema: None,
                        after: after.clone(),
                        rows: Some(page_items),
                        bytes: Some(usize::try_from(SERVER_DISCOVERY_PAGE_BYTES).expect("validated discovery page size")),
                    })
                    .await
                    .map_err(discovery_failure)?;
                read_bytes = read_bytes.saturating_add(page.entries.len());
                if read_bytes > usize::try_from(SERVER_DISCOVERY_WORK_BYTES).expect("validated discovery work size") {
                    return Err(Status::resource_exhausted("availability discovery exceeds configured work budget"));
                }
                let entries: Vec<DiscoveredOperation> =
                    serde_json::from_slice(&page.entries).map_err(|_| Status::data_loss("invalid shared operation discovery metadata"))?;
                if entries.len() > page_items {
                    return Err(Status::data_loss("shared discovery exceeds requested page size"));
                }
                for entry in entries {
                    if !entry.grpc_available {
                        continue;
                    }
                    let Some(binding) = entry.contract.grpc() else {
                        return Err(Status::data_loss("shared discovery has no admitted gRPC binding"));
                    };
                    let path = binding.path();
                    methods.try_reserve(1).map_err(|_| Status::resource_exhausted("availability allocation refused"))?;
                    methods.push(path);
                }
                let Some(next) = &page.next else {
                    break;
                };
                if after.as_ref() == Some(next) {
                    return Err(Status::data_loss("shared discovery continuation did not advance"));
                }
                after = Some(next.clone());
            }
            methods.sort_unstable();
            if methods.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(Status::data_loss("shared discovery repeats a gRPC binding"));
            }
            let identity = self.identity().await?;
            let output = dokime::domain::model::backend::wire::BackendAvailability {
                backend_id: identity.backend_id.clone(),
                epoch: identity.epoch.clone(),
                methods,
                limits: Some(dokime::domain::model::backend::wire::BackendLimits {
                    max_request_bytes: maximum as u64,
                    max_response_bytes: maximum as u64,
                    max_content_bytes: Some(i64::MAX as u64),
                    max_evidence_bytes: Some(DOCUMENT_EVIDENCE_EXPORT_BYTES),
                    max_page_items: Some(u32::try_from(PAGINATION_MAXIMUM_PAGE_ITEMS).expect("validated page size")),
                    max_stream_buffer_items: Some(1),
                    max_chunk_bytes: Some(super::super::content::transfer_chunk_bytes(
                        &dokime::app::live_runtime(&self.runtime).await?,
                        maximum,
                    )? as u32),
                }),
            };
            if output.wire_len()? > maximum {
                return Err(Status::resource_exhausted("availability exceeds configured message budget"));
            }
            Ok(output.into())
        })
    }
}

impl Serve<wire::WatchLogs> for Resources {
    fn serve<'a>(&'a self, request: wire::WatchLogs, ctx: Context<'a, wire::WatchLogs>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let min_level = match request.min_level {
                    dokime::domain::model::backend::wire::LogLevel::Unspecified => dokime::domain::model::backend::wire::LogLevel::Trace,
                    level => level,
                };
                let watch = crate::grpc::logs::Watch {
                    min_level,
                    replay_lines: request.replay_lines,
                };
                self.logs.watch(watch, ctx.streams()).await?;
                Ok(().into())
            })
            .await
        })
    }
}

impl Serve<wire::CollectDiagnostics> for Resources {
    fn serve<'a>(&'a self, request: wire::CollectDiagnostics, ctx: Context<'a, wire::CollectDiagnostics>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                super::archive::deliver(
                    &dokime::app::live_runtime(&self.runtime).await?,
                    &*self.identity().await?,
                    self.maximum,
                    request.max_log_tail_bytes,
                    ctx.streams(),
                )
                .await?;
                Ok(().into())
            })
            .await
        })
    }
}

pub(in crate::grpc) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize, config: &dokime::RuntimeConfig,
) -> Result<(), crate::grpc::Error> {
    let poll_interval = config.storage.filesystem.poll_interval;
    let logs = crate::grpc::logs::Tail::new(super::hub_log_path(&config.application), poll_interval, maximum);
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum, logs });

    setup.resource::<Host<wire::GetIdentity>>(Host::<wire::GetIdentity>::new(resources.clone()));
    setup.resource::<Host<wire::GetHealth>>(Host::<wire::GetHealth>::new(resources.clone()));
    setup.resource::<Host<wire::GetAvailability>>(Host::<wire::GetAvailability>::new(resources.clone()));
    setup.resource::<Host<wire::WatchLogs>>(Host::<wire::WatchLogs>::new(resources.clone()));
    setup.resource::<Host<wire::CollectDiagnostics>>(Host::<wire::CollectDiagnostics>::new(resources));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

pub(super) fn produce_result<T: Portable + Send + Sync + 'static>(
    maximum: usize, build: impl FnOnce() -> Result<T, Status>,
) -> Result<Response<T>, Status> {
    let output = build()?;
    if output.wire_len()? > maximum {
        return Err(Status::resource_exhausted("backend status exceeds configured message budget"));
    }
    Ok(output.into())
}

fn produce<T: Portable + Send + Sync + 'static>(maximum: usize, build: impl FnOnce() -> T) -> Result<Response<T>, Status> {
    produce_result(maximum, || Ok(build()))
}

fn health_failure(source: dokime::harmos::Error) -> Status {
    crate::grpc::error::detailed(Status::unavailable("runtime health is unavailable"), &source.into())
}

fn discovery_failure(source: InvocationError) -> Status {
    let cause = match &source {
        InvocationError::Failed { source, .. } => source.downcast_ref::<dokime::harmos::Error>(),
        _ => None,
    };
    let status = match cause {
        Some(dokime::harmos::Error::ResourceExhausted { .. }) => Status::resource_exhausted("operation discovery admission failed"),
        Some(dokime::harmos::Error::PermissionDenied | dokime::harmos::Error::OperationUnauthorized { .. }) => {
            Status::permission_denied("operation discovery is not authorized")
        }
        _ => Status::unavailable("operation discovery is unavailable"),
    };
    crate::grpc::error::detailed_error(status, &source)
}

impl Resources {
    async fn identity(&self) -> Result<Response<dokime::domain::model::backend::wire::BackendIdentity>, Status> {
        let instance = dokime::app::live_runtime(&self.runtime)
            .await?
            .operations()
            .submit::<native::GetIdentity>(native::GetIdentity {})
            .map_err(health_failure)?;
        let identity = dokime::harmos::grpc::response(instance).await.map_err(health_failure)?;
        let identity = produce(self.maximum, || {
            dokime::domain::model::backend::wire::BackendIdentity {
                backend_id: identity.backend_id.clone(),
                epoch: identity.epoch.clone(),
                api: Some(dokime::domain::model::backend::wire::ApiIdentity {
                    package: dokime::domain::model::backend::wire::API_PACKAGE.into(),
                    version: dokime::domain::model::backend::wire::API_VERSION.into(),
                    descriptor_sha256: Sha256::digest(super::super::FILE_DESCRIPTOR_SET).to_vec(),
                }),
                product_version: identity.product_version.clone(),
                source_revision: None,
                sdk_revision: None,
                // Listener Start Time Is Not The Original Runtime's Boot Time.
                started_at: None,
                display_name: identity.display_name.clone(),
                target: env!("DOKIME_BACKEND_TARGET").to_owned(),
            }
        })?;

        Ok(identity)
    }
}
