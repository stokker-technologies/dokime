/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CasesCreateResponse;
use dokime_protocol::host::CasesDeleteRequest;
use dokime_protocol::host::CasesDeleteResponse;
use dokime_protocol::host::CasesGetRequest;
use dokime_protocol::host::CasesGetResponse;
use dokime_protocol::host::CasesListRequest;
use dokime_protocol::host::CasesListResponse;
use dokime_protocol::host::CasesParametersRequest;
use dokime_protocol::host::CasesParametersResponse;
use dokime_protocol::host::CasesWatchRequest;
use dokime_protocol::host::CasesWatchResponse;
use dokime_protocol::host::cases_server::Cases;
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
impl Cases for Services {
    async fn create(&self, request: Request<CasesCreateRequest>) -> Result<Response<CasesCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn get(&self, request: Request<CasesGetRequest>) -> Result<Response<CasesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<CasesListRequest>) -> Result<Response<CasesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<CasesWatchResponse>;
    async fn watch(&self, request: Request<CasesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<CasesDeleteRequest>) -> Result<Response<CasesDeleteResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Delete", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn parameters(&self, request: Request<CasesParametersRequest>) -> Result<Response<CasesParametersResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Parameters", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::cases::parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
