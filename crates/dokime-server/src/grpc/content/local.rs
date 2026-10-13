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
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;
// Local Imports
use crate::grpc::value;
use dokime_app::{RuntimeHandle, core::TestCycleId};
use dokime_contract::operations::content as wire;

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    local_only: bool,
}
struct Bundle;
struct Folder {
    resources: Arc<Resources>,
}

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl Operation for Bundle {
    type Declaration = wire::DownloadCycleBundle;
    type Resources = ();
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::CycleBundleFrame>;
    fn new(_: Arc<()>) -> Result<Self, Status> {
        Ok(Self)
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::DownloadCycleBundleRequest,
        _: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let cycle = cycle_id(&request.cycle_id)?;
        Err(Status::unimplemented(format!(
            "cycle bundle export is not supported for {cycle}"
        )))
    }
}
impl Operation for Folder {
    type Declaration = wire::GetCycleFolderPath;
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
        request: &mut wire::GetCycleFolderPathRequest,
        _: &mut (),
    ) -> Result<Response<wire::LocalPathResponse>, Status> {
        if !self.resources.local_only {
            return Err(Status::permission_denied(
                "cycle folder paths are only available on local listeners",
            ));
        }
        let cycle = cycle_id(&request.cycle_id)?;
        let state = self.resources.runtime.watch_state().latest();
        if !state.state().cycles.contains_key(&cycle) {
            return Err(Status::not_found("cycle not found"));
        }
        Err(Status::unimplemented(
            "native storage does not expose cycle directories",
        ))
    }
}

/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    local_only: bool,
) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        local_only,
    });
    runtime.operations().register_wire_operation::<Bundle>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(()),
    )?;
    runtime.operations().register_wire_operation::<Folder>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources,
    )?;
    Ok(())
}

/* === Private Functions === */
fn cycle_id(id: &str) -> Result<TestCycleId, Status> {
    value::text(id)?;
    TestCycleId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))
}

/* === Tests === */
