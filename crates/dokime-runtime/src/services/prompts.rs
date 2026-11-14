/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::PromptsGetRequest;
use dokime_protocol::host::PromptsGetResponse;
use dokime_protocol::host::PromptsListRequest;
use dokime_protocol::host::PromptsListResponse;
use dokime_protocol::host::PromptsWatchRequest;
use dokime_protocol::host::PromptsWatchResponse;
use dokime_protocol::host::prompts_server::Prompts;
use harmos_protocol::model::Callers;
use harmos_runtime::Failure;
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
impl Prompts for Services {
    async fn get(&self, request: Request<PromptsGetRequest>) -> Result<Response<PromptsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompts::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<PromptsListRequest>) -> Result<Response<PromptsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompts::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<PromptsWatchResponse>;
    async fn watch(&self, request: Request<PromptsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompts::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, _request: Request<PromptsCreateRequest>) -> Result<Response<PromptsCreateResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.Prompts/Create")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
