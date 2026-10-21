/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::domain::Portable;
use harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamOutput},
};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::wire::Timestamp;
use dokime::domain::operations::backend as wire;
use dokime::domain::operations::runtime as native;

/* === Constants === */

const DISCOVERY_PAGE_BYTES: usize = 4 * 1024 * 1024;
const DISCOVERY_WORK_BYTES: usize = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    identity: Response<dokime::domain::model::backend::wire::BackendIdentity>,
}

struct Identity {
    resources: Arc<Resources>,
}

struct Archive {
    resources: Arc<Resources>,
}

struct Logs {
    resources: Arc<Resources>,
}

struct Availability {
    resources: Arc<Resources>,
}

struct Health {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Identity {
    type Declaration = wire::GetIdentity;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetIdentity, _: &mut (),
    ) -> Result<Response<dokime::domain::model::backend::wire::BackendIdentity>, Status> {
        Ok(self.resources.identity.clone())
    }
}

impl Operation for Health {
    type Declaration = wire::GetHealth;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetHealth, _: &mut (),
    ) -> Result<Response<dokime::domain::model::backend::wire::BackendHealth>, Status> {
        let instance = self.resources.runtime.operations().submit(native::GetHealth {}).map_err(health_failure)?;
        let health = harmos::grpc::response(instance).await.map_err(health_failure)?;
        produce(self.resources.maximum, || dokime::domain::model::backend::wire::BackendHealth {
            backend_id: health.backend_id.clone(),
            epoch: health.epoch.clone(),
            readiness: match health.readiness {
                dokime::domain::model::runtime::wire::Readiness::Starting => dokime::domain::model::backend::wire::Readiness::Starting,
                dokime::domain::model::runtime::wire::Readiness::Ready => dokime::domain::model::backend::wire::Readiness::Ready,
                dokime::domain::model::runtime::wire::Readiness::Degraded => dokime::domain::model::backend::wire::Readiness::Degraded,
                dokime::domain::model::runtime::wire::Readiness::Stopping => dokime::domain::model::backend::wire::Readiness::Stopping,
            },
            observed_at: Some(Timestamp { seconds: health.observed_at_seconds, nanos: health.observed_at_nanoseconds as i32 }),
            message: health.message.clone(),
        })
    }
}

impl Operation for Availability {
    type Declaration = wire::GetAvailability;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, context: Self::Context<'a>, _: &mut wire::GetAvailability, _: &mut (),
    ) -> Result<Response<dokime::domain::model::backend::wire::BackendAvailability>, Status> {
        use harmos::operations::host::{Catalog, CatalogKind, DiscoveredOperation};
        let maximum = self.resources.maximum;
        let mut methods = Vec::new();
        let mut after = None;
        let mut read_bytes = 0usize;
        let mut required = std::mem::size_of::<dokime::domain::model::backend::wire::BackendAvailability>()
            .saturating_add(self.resources.identity.backend_id.len())
            .saturating_add(self.resources.identity.epoch.len());
        loop {
            let page = context
                .call(Catalog {
                    kind: CatalogKind::Operations,
                    schema: None,
                    after: after.clone(),
                    rows: Some(64),
                    bytes: Some(DISCOVERY_PAGE_BYTES),
                })
                .await
                .map_err(discovery_failure)?;
            read_bytes = read_bytes.saturating_add(page.entries.len());
            if read_bytes > DISCOVERY_WORK_BYTES {
                return Err(Status::resource_exhausted("availability discovery exceeds configured work budget"));
            }
            let entries: Vec<DiscoveredOperation> =
                serde_json::from_slice(&page.entries).map_err(|_| Status::data_loss("invalid shared operation discovery metadata"))?;
            if entries.len() > 64 {
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
                // Account For Conservative Vector Growth Before Retaining This Path.
                required = required.saturating_add(4 * std::mem::size_of::<String>()).saturating_add(path.capacity());
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
        let output = dokime::domain::model::backend::wire::BackendAvailability {
            backend_id: self.resources.identity.backend_id.clone(),
            epoch: self.resources.identity.epoch.clone(),
            methods,
            limits: Some(dokime::domain::model::backend::wire::BackendLimits {
                max_request_bytes: maximum as u64,
                max_response_bytes: maximum as u64,
                max_content_bytes: Some(super::super::content::MAX_CONTENT_BYTES),
                max_evidence_bytes: Some(dokime::domain::model::exports::EVIDENCE_EXPORT_MAX_BYTES),
                max_page_items: Some(dokime::domain::model::execution::wire_cursor::MAX_PAGE_ITEMS),
                max_stream_buffer_items: Some(1),
                max_chunk_bytes: Some(super::super::content::transfer_chunk_bytes(maximum) as u32),
            }),
        };
        if output.wire_len()? > maximum {
            return Err(Status::resource_exhausted("availability exceeds configured message budget"));
        }
        Ok(output.into())
    }
}

impl Operation for Logs {
    type Declaration = wire::WatchLogs;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::backend::wire::LogFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::WatchLogs, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let min_level = match request.min_level {
            dokime::domain::model::backend::wire::LogLevel::Unspecified => dokime::domain::model::backend::wire::LogLevel::Trace,
            level => level,
        };
        crate::grpc::logs::Watch {
            path: super::hub_log_path(&self.resources.runtime.application_root()),
            min_level,
            replay_lines: request.replay_lines,
            message_limit: self.resources.maximum,
        }
        .run(output)
        .await?;
        Ok(().into())
    }
}

impl Operation for Archive {
    type Declaration = wire::CollectDiagnostics;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::backend::wire::DiagnosticsBundleFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::CollectDiagnostics, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        super::archive::deliver(
            &self.resources.runtime,
            &self.resources.identity,
            self.resources.maximum,
            request.max_log_tail_bytes,
            output,
        )
        .await?;
        Ok(().into())
    }
}

/* === Public Functions === */

pub(in crate::grpc) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let instance = runtime.operations().submit(native::GetIdentity {})?;
    let identity = harmos::grpc::response(instance).await?;
    let identity = produce(maximum, || {
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
        }
    })
    .map_err(crate::grpc::Error::OperationService)?;
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum, identity });
    runtime.operations().register_wire_operation::<Identity>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Health>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Availability>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Logs>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Archive>("dokime", dokime::app::Lanes::Open, resources)?;
    super::diagnostics::register(runtime, maximum)?;
    Ok(())
}

pub(super) fn produce_result<T: MemorySize + Portable + Send + Sync + 'static>(
    maximum: usize, build: impl FnOnce() -> Result<T, Status>,
) -> Result<Response<T>, Status> {
    let output = build()?;
    if output.encoded_len() > maximum {
        return Err(Status::resource_exhausted("backend status exceeds configured message budget"));
    }
    Ok(output.into())
}

/* === Private Functions === */

fn produce<T: MemorySize + Portable + Send + Sync + 'static>(maximum: usize, build: impl FnOnce() -> T) -> Result<Response<T>, Status> {
    produce_result(maximum, || Ok(build()))
}

fn health_failure(source: harmos::Error) -> Status {
    crate::grpc::error::detailed(Status::unavailable("runtime health is unavailable"), &source.into())
}

fn discovery_failure(source: harmos::Error) -> Status {
    let status = match &source {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("operation discovery admission failed"),
        harmos::Error::PermissionDenied | harmos::Error::OperationUnauthorized { .. } => {
            Status::permission_denied("operation discovery is not authorized")
        }
        _ => Status::unavailable("operation discovery is unavailable"),
    };
    crate::grpc::error::detailed(status, &source.into())
}

/* === Tests === */
