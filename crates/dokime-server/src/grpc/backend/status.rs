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
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::{Context, Operation, Response, StreamOutput};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::wire::Timestamp;
use dokime::operations::backend as wire;
use dokime::operations::runtime as native;

/* === Constants === */

const DOCUMENT_EVIDENCE_EXPORT_BYTES: u64 = 8_589_934_592;
const PAGINATION_MAXIMUM_PAGE_ITEMS: u64 = 500;
const PAGINATION_PAGE_ITEMS: u64 = 50;
const SERVER_DISCOVERY_PAGE_BYTES: u64 = 4_194_304;
const SERVER_DISCOVERY_WORK_BYTES: u64 = 16_777_216;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    identity: Response<dokime::domain::model::backend::wire::BackendIdentity>,
    logs: Arc<crate::grpc::logs::Tail>,
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
        let health = dokime::harmos::grpc::response(instance).await.map_err(health_failure)?;
        produce(self.resources.maximum, || dokime::domain::model::backend::wire::BackendHealth {
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
        use dokime::harmos::builtins::host::{Catalog, CatalogKind, DiscoveredOperation};
        let maximum = self.resources.maximum;
        let page_items = usize::try_from(PAGINATION_PAGE_ITEMS).expect("validated discovery page count");
        let mut methods = Vec::new();
        let mut after = None;
        let mut read_bytes = 0usize;
        loop {
            let page = context
                .call(Catalog {
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
        let output = dokime::domain::model::backend::wire::BackendAvailability {
            backend_id: self.resources.identity.backend_id.clone(),
            epoch: self.resources.identity.epoch.clone(),
            methods,
            limits: Some(dokime::domain::model::backend::wire::BackendLimits {
                max_request_bytes: maximum as u64,
                max_response_bytes: maximum as u64,
                max_content_bytes: Some(i64::MAX as u64),
                max_evidence_bytes: Some(DOCUMENT_EVIDENCE_EXPORT_BYTES),
                max_page_items: Some(u32::try_from(PAGINATION_MAXIMUM_PAGE_ITEMS).expect("validated page size")),
                max_stream_buffer_items: Some(1),
                max_chunk_bytes: Some(super::super::content::transfer_chunk_bytes(&self.resources.runtime, maximum)? as u32),
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
        &mut self, call_context: Self::Context<'a>, request: &mut wire::WatchLogs, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            let min_level = match request.min_level {
                dokime::domain::model::backend::wire::LogLevel::Unspecified => dokime::domain::model::backend::wire::LogLevel::Trace,
                level => level,
            };
            let watch = crate::grpc::logs::Watch {
                min_level,
                replay_lines: request.replay_lines,
            };
            self.resources.logs.watch(watch, output).await?;
            Ok(().into())
        })
        .await
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
        &mut self, call_context: Self::Context<'a>, request: &mut wire::CollectDiagnostics, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            super::archive::deliver(
                &self.resources.runtime,
                &self.resources.identity,
                self.resources.maximum,
                request.max_log_tail_bytes,
                output,
            )
            .await?;
            Ok(().into())
        })
        .await
    }
}

/* === Public Functions === */

pub(in crate::grpc) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let instance = runtime.operations().submit(native::GetIdentity {})?;
    let identity = dokime::harmos::grpc::response(instance).await?;
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
            target: env!("DOKIME_BACKEND_TARGET").to_owned(),
        }
    })
    .map_err(crate::grpc::Error::OperationService)?;
    let poll_interval = runtime.storage().settings().filesystem.poll_interval;
    let logs = crate::grpc::logs::Tail::new(super::hub_log_path(&runtime.application_root()), poll_interval, maximum);
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
        identity,
        logs,
    });
    runtime.operations().register_wire_operation::<Identity>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Health>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Availability>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Logs>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Archive>("dokime", dokime::app::Lanes::Open, resources)?;
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

/* === Private Functions === */

fn produce<T: Portable + Send + Sync + 'static>(maximum: usize, build: impl FnOnce() -> T) -> Result<Response<T>, Status> {
    produce_result(maximum, || Ok(build()))
}

fn health_failure(source: dokime::harmos::Error) -> Status {
    crate::grpc::error::detailed(Status::unavailable("runtime health is unavailable"), &source.into())
}

fn discovery_failure(source: dokime::harmos::Error) -> Status {
    let status = match &source {
        dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("operation discovery admission failed"),
        dokime::harmos::Error::PermissionDenied | dokime::harmos::Error::OperationUnauthorized { .. } => {
            Status::permission_denied("operation discovery is not authorized")
        }
        _ => Status::unavailable("operation discovery is unavailable"),
    };
    crate::grpc::error::detailed(status, &source.into())
}

/* === Tests === */
