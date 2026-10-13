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
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response, StreamOutput},
};
use sha2::{Digest, Sha256};
use tonic::Status;
// Local Imports
use dokime_app::RuntimeHandle;
use dokime_contract::operations::{backend as wire, common::Timestamp, runtime as native};
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    identity: Response<wire::BackendIdentity>,
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
struct Capabilities {
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
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut wire::GetIdentityRequest,
        _: &mut (),
    ) -> Result<Response<wire::BackendIdentity>, Status> {
        Ok(self.resources.identity.clone())
    }
}
impl Operation for Health {
    type Declaration = wire::GetHealth;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut wire::GetHealthRequest,
        _: &mut (),
    ) -> Result<Response<wire::BackendHealth>, Status> {
        let instance = self
            .resources
            .runtime
            .operations()
            .submit::<native::GetHealth>(native::Request {})
            .map_err(health_failure)?;
        let health = harmos::grpc::response(instance)
            .await
            .map_err(health_failure)?;
        let required = std::mem::size_of::<wire::BackendHealth>()
            .saturating_add(health.backend_id.len())
            .saturating_add(health.epoch.len())
            .saturating_add(health.message.len());
        produce(
            &self.resources.runtime,
            self.resources.maximum,
            required,
            || wire::BackendHealth {
                backend_id: health.backend_id.clone(),
                epoch: health.epoch.clone(),
                readiness: match health.readiness {
                    native::Readiness::Starting => wire::Readiness::Starting,
                    native::Readiness::Ready => wire::Readiness::Ready,
                    native::Readiness::Degraded => wire::Readiness::Degraded,
                    native::Readiness::Stopping => wire::Readiness::Stopping,
                },
                observed_at: Some(Timestamp {
                    seconds: health.observed_at_seconds,
                    nanos: health.observed_at_nanoseconds as i32,
                }),
                message: health.message.clone(),
            },
        )
    }
}
impl Operation for Capabilities {
    type Declaration = wire::GetCapabilities;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        _: &mut wire::GetCapabilitiesRequest,
        _: &mut (),
    ) -> Result<Response<wire::BackendCapabilities>, Status> {
        use harmos::operations::host::{DiscoveredOperation, Operations, OperationsRequest};
        let maximum = self.resources.maximum;
        let bytes = NonZeroUsize::new(maximum)
            .ok_or_else(|| Status::resource_exhausted("invalid capability response allowance"))?;
        // Retain Admission While The Bounded Live Inventory Is Assembled.
        let preparation = self
            .resources
            .runtime
            .storage()
            .prepare_production(bytes)
            .map_err(memory_failure)?;
        let mut methods = Vec::new();
        let mut after = None;
        let mut read_bytes = 0usize;
        let mut required = std::mem::size_of::<wire::BackendCapabilities>()
            .saturating_add(self.resources.identity.backend_id.len())
            .saturating_add(self.resources.identity.epoch.len());
        let page_bytes = maximum.min(64 * 1024).min(u32::MAX as usize) as u32;
        if page_bytes < 2 {
            return Err(Status::resource_exhausted(
                "capability discovery budget is too small",
            ));
        }
        loop {
            let page = context
                .call::<Operations>(OperationsRequest {
                    after: after.clone(),
                    rows: Some(64),
                    bytes: Some(page_bytes),
                })
                .await
                .map_err(discovery_failure)?;
            read_bytes = read_bytes.saturating_add(page.catalog.len());
            if read_bytes > maximum {
                return Err(Status::resource_exhausted(
                    "capability discovery exceeds configured work budget",
                ));
            }
            let entries: Vec<DiscoveredOperation> = serde_json::from_slice(&page.catalog)
                .map_err(|_| Status::data_loss("invalid shared operation discovery metadata"))?;
            if entries.len() > 64 {
                return Err(Status::data_loss(
                    "shared discovery exceeds requested page size",
                ));
            }
            for entry in entries {
                if !entry.grpc_available {
                    continue;
                }
                let Some(binding) = entry.contract.grpc() else {
                    return Err(Status::data_loss(
                        "shared discovery has no admitted gRPC binding",
                    ));
                };
                let path = binding.path();
                // Account For Conservative Vector Growth Before Retaining This Path.
                required = required
                    .saturating_add(4 * std::mem::size_of::<String>())
                    .saturating_add(path.capacity());
                if required > maximum {
                    return Err(Status::resource_exhausted(
                        "capabilities exceed configured memory budget",
                    ));
                }
                methods.push(path);
            }
            let Some(next) = &page.next else {
                break;
            };
            if after.as_ref() == Some(next) {
                return Err(Status::data_loss(
                    "shared discovery continuation did not advance",
                ));
            }
            after = Some(next.clone());
        }
        methods.sort_unstable();
        if methods.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Status::data_loss("shared discovery repeats a gRPC binding"));
        }
        let output = wire::BackendCapabilities {
            backend_id: self.resources.identity.backend_id.clone(),
            epoch: self.resources.identity.epoch.clone(),
            methods,
            limits: Some(wire::BackendLimits {
                max_request_bytes: maximum as u64,
                max_response_bytes: maximum as u64,
                max_content_bytes: Some(super::super::content::MAX_CONTENT_BYTES),
                max_page_items: Some(dokime::operations::cursor::MAX_PAGE_ITEMS),
                max_stream_buffer_items: Some(1),
                max_chunk_bytes: Some(super::super::content::transfer_chunk_bytes(maximum) as u32),
            }),
        };
        if output.encoded_len() > maximum {
            return Err(Status::resource_exhausted(
                "capabilities exceed configured message budget",
            ));
        }
        preparation
            .finish(output)
            .map_err(memory_failure)?
            .try_into_value()
            .map_err(memory_failure)?
            .try_into_response()
            .map_err(memory_failure)
    }
}
impl Operation for Logs {
    type Declaration = wire::WatchLogs;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::LogFrame>;
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::WatchLogsRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let min_level = match request.min_level {
            wire::LogLevel::Unspecified => wire::LogLevel::Trace,
            level => level,
        };
        crate::grpc::logs::Watch {
            runtime: self.resources.runtime.clone(),
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
    type Endpoints = StreamOutput<wire::DiagnosticsBundleFrame>;
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::CollectDiagnosticsRequest,
        output: &mut Self::Endpoints,
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
pub(in crate::grpc) async fn register(
    runtime: &RuntimeHandle,
    maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let instance = runtime
        .operations()
        .submit::<native::GetIdentity>(native::Request {})?;
    let identity = harmos::grpc::response(instance).await?;
    let required = std::mem::size_of::<wire::BackendIdentity>()
        .saturating_add(identity.backend_id.len())
        .saturating_add(identity.epoch.len())
        .saturating_add(identity.product_version.len())
        .saturating_add(identity.display_name.len())
        .saturating_add(wire::API_PACKAGE.len())
        .saturating_add(wire::API_VERSION.len())
        .saturating_add(32);
    let identity = produce(runtime, maximum, required, || wire::BackendIdentity {
        backend_id: identity.backend_id.clone(),
        epoch: identity.epoch.clone(),
        api: Some(wire::ApiIdentity {
            package: wire::API_PACKAGE.into(),
            version: wire::API_VERSION.into(),
            descriptor_sha256: Sha256::digest(super::super::FILE_DESCRIPTOR_SET).to_vec(),
        }),
        product_version: identity.product_version.clone(),
        source_revision: None,
        sdk_revision: None,
        // Listener Start Time Is Not The Original Runtime's Boot Time.
        started_at: None,
        display_name: identity.display_name.clone(),
    })
    .map_err(crate::grpc::Error::OperationService)?;
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
        identity,
    });
    runtime.operations().register_wire_operation::<Identity>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Health>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime
        .operations()
        .register_wire_operation::<Capabilities>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime.operations().register_wire_operation::<Logs>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Archive>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources,
    )?;
    super::diagnostics::register(runtime, maximum)?;
    Ok(())
}
/* === Private Functions === */
fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
    runtime: &RuntimeHandle,
    maximum: usize,
    required: usize,
    build: impl FnOnce() -> T,
) -> Result<Response<T>, Status> {
    produce_result(runtime, maximum, required, || Ok(build()))
}
pub(super) fn produce_result<T: MemorySize + Protobuf + Send + Sync + 'static>(
    runtime: &RuntimeHandle,
    maximum: usize,
    required: usize,
    build: impl FnOnce() -> Result<T, Status>,
) -> Result<Response<T>, Status> {
    if required > maximum {
        return Err(Status::resource_exhausted(
            "backend status exceeds configured memory budget",
        ));
    }
    let bytes = NonZeroUsize::new(required)
        .ok_or_else(|| Status::resource_exhausted("invalid backend status allowance"))?;
    let preparation = runtime
        .storage()
        .prepare_production(bytes)
        .map_err(memory_failure)?;
    let output = build()?;
    if output.encoded_len() > maximum {
        return Err(Status::resource_exhausted(
            "backend status exceeds configured message budget",
        ));
    }
    preparation
        .finish(output)
        .map_err(memory_failure)?
        .try_into_value()
        .map_err(memory_failure)?
        .try_into_response()
        .map_err(memory_failure)
}
fn memory_failure(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("backend status admission failed")
    } else {
        Status::internal("backend status admission failed")
    };
    crate::grpc::error::detailed(status, &source.into())
}
fn health_failure(source: harmos::Error) -> Status {
    crate::grpc::error::detailed(
        Status::unavailable("runtime health is unavailable"),
        &source.into(),
    )
}
fn discovery_failure(source: harmos::Error) -> Status {
    let status = match &source {
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("operation discovery admission failed")
        }
        harmos::Error::PermissionDenied | harmos::Error::OperationUnauthorized { .. } => {
            Status::permission_denied("operation discovery is not authorized")
        }
        _ => Status::unavailable("operation discovery is unavailable"),
    };
    crate::grpc::error::detailed(status, &source.into())
}
/* === Tests === */
