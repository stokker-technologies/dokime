/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesCreateResponse;
use dokime_protocol::host::CyclesGetRequest;
use dokime_protocol::host::CyclesGetResponse;
use dokime_protocol::host::CyclesListRequest;
use dokime_protocol::host::CyclesListResponse;
use dokime_protocol::host::CyclesSealRequest;
use dokime_protocol::host::CyclesSealResponse;
use dokime_protocol::host::CyclesSetRequest;
use dokime_protocol::host::CyclesSetResponse;
use dokime_protocol::host::CyclesWatchRequest;
use dokime_protocol::host::CyclesWatchResponse;
use dokime_protocol::host::cycles_server::Cycles;
use harmos_protocol::model::Callers;
use tonic::Request;
use tonic::Response;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Cycles for Services {
    async fn get(&self, request: Request<CyclesGetRequest>) -> Result<Response<CyclesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, request: Request<CyclesSetRequest>) -> Result<Response<CyclesSetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Set", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<CyclesListRequest>) -> Result<Response<CyclesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<CyclesWatchResponse>;
    async fn watch(&self, request: Request<CyclesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, request: Request<CyclesCreateRequest>) -> Result<Response<CyclesCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn seal(&self, request: Request<CyclesSealRequest>) -> Result<Response<CyclesSealResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Seal", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cycles::seal::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
