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
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::preferences::Resources;
use dokime::app::LiveRuntime;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::settings as wire;

/* === Constants === */

const SERVER_SETTINGS_RESPONSE_BYTES: u64 = 4_194_304;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::ListStorageSources),
    Operation1(wire::ReorderStorageSources),
    Operation2(wire::SetStorageSourceEnabled),
}

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ListStorageSources> for Resources {
    fn serve<'a>(
        &'a self, _: wire::ListStorageSources, _: Context<'a, wire::ListStorageSources>,
    ) -> Served<'a, dokime::domain::model::settings::wire::StorageSources> {
        Box::pin(async move { sources(self).await })
    }
}

impl Serve<wire::ReorderStorageSources> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::ReorderStorageSources, _: Context<'a, wire::ReorderStorageSources>,
    ) -> Served<'a, dokime::domain::model::settings::wire::StorageSources> {
        Box::pin(async move {
            dokime::app::live_runtime(&self.runtime)
                .await?
                .reorder_storage_sources(std::mem::take(&mut request.identifiers))
                .await
                .map_err(super::super::execution::status)?;
            sources(self).await
        })
    }
}

impl Serve<wire::SetStorageSourceEnabled> for Resources {
    fn serve<'a>(
        &'a self, request: wire::SetStorageSourceEnabled, _: Context<'a, wire::SetStorageSourceEnabled>,
    ) -> Served<'a, dokime::domain::model::settings::wire::StorageSources> {
        Box::pin(async move {
            dokime::app::live_runtime(&self.runtime)
                .await?
                .set_storage_source_enabled(&request.identifier, request.enabled)
                .await
                .map_err(super::super::execution::status)?;
            sources(self).await
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum: maximum.min(usize::try_from(SERVER_SETTINGS_RESPONSE_BYTES).expect("validated settings response size")),
    });

    setup.resource::<Host<wire::ListStorageSources>>(Host::<wire::ListStorageSources>::new(served.clone()));
    setup.resource::<Host<wire::ReorderStorageSources>>(Host::<wire::ReorderStorageSources>::new(served.clone()));
    setup.resource::<Host<wire::SetStorageSourceEnabled>>(Host::<wire::SetStorageSourceEnabled>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

/* === Private Functions === */

async fn sources(resources: &Resources) -> Result<Response<dokime::domain::model::settings::wire::StorageSources>, Status> {
    let sources = dokime::app::live_runtime(&resources.runtime).await?.storage_sources().await.map_err(super::super::execution::status)?;
    resources.produce(|| {
        let mut output = Vec::with_capacity(sources.len());
        for source in sources {
            output.push(dokime::domain::model::settings::wire::StorageSource {
                identifier: source.identifier,
                adapter: source.adapter,
                priority: source.priority,
                enabled: source.enabled,
            });
        }
        dokime::domain::model::settings::wire::StorageSources { sources: output }
    })
}

/* === Tests === */
