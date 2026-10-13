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
use harmos::operations::{Context, Operation, Response};
use tonic::Status;
// Local Imports
use super::preferences::Resources;
use dokime_app::RuntimeHandle;
use dokime_contract::operations::settings as wire;

/* === Constants === */
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct ListStorageSources {
    resources: Arc<Resources>,
}
struct ReorderStorageSources {
    resources: Arc<Resources>,
}
struct SetStorageSourceEnabled {
    resources: Arc<Resources>,
}
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl Operation for ListStorageSources {
    type Declaration = wire::ListStorageSources;
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
        _: &mut wire::ListStorageSourcesRequest,
        _: &mut (),
    ) -> Result<Response<wire::StorageSources>, Status> {
        sources(&self.resources).await
    }
}
impl Operation for ReorderStorageSources {
    type Declaration = wire::ReorderStorageSources;
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
        request: &mut wire::ReorderStorageSourcesRequest,
        _: &mut (),
    ) -> Result<Response<wire::StorageSources>, Status> {
        self.resources
            .runtime
            .reorder_storage_sources(std::mem::take(&mut request.identifiers))
            .await
            .map_err(super::super::execution::status)?;
        sources(&self.resources).await
    }
}
impl Operation for SetStorageSourceEnabled {
    type Declaration = wire::SetStorageSourceEnabled;
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
        request: &mut wire::SetStorageSourceEnabledRequest,
        _: &mut (),
    ) -> Result<Response<wire::StorageSources>, Status> {
        self.resources
            .runtime
            .set_storage_source_enabled(&request.identifier, request.enabled)
            .await
            .map_err(super::super::execution::status)?;
        sources(&self.resources).await
    }
}
/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum: maximum.min(MAX_RESPONSE_BYTES),
    });
    runtime
        .operations()
        .register_wire_operation::<ListStorageSources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<ReorderStorageSources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<SetStorageSourceEnabled>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    Ok(())
}
/* === Private Functions === */
async fn sources(resources: &Resources) -> Result<Response<wire::StorageSources>, Status> {
    let sources = resources
        .runtime
        .storage_sources()
        .await
        .map_err(super::super::execution::status)?;
    let required = sources.iter().fold(
        std::mem::size_of::<wire::StorageSources>().saturating_add(
            sources
                .len()
                .saturating_mul(std::mem::size_of::<wire::StorageSource>()),
        ),
        |bytes, source| {
            bytes
                .saturating_add(source.identifier.capacity())
                .saturating_add(source.adapter.capacity())
        },
    );
    resources.produce(required, || {
        let mut output = Vec::with_capacity(sources.len());
        for source in sources {
            output.push(wire::StorageSource {
                identifier: source.identifier,
                adapter: source.adapter,
                priority: source.priority,
                enabled: source.enabled,
            });
        }
        wire::StorageSources { sources: output }
    })
}
/* === Tests === */
