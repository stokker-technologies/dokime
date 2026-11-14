/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::ExecutionsCancelRequest;
use dokime_protocol::host::ExecutionsCancelResponse;
use dokime_protocol::host::ExecutionsCreateRequest;
use dokime_protocol::host::ExecutionsCreateResponse;
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsDeleteResponse;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsGetResponse;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsListResponse;
use dokime_protocol::host::ExecutionsParametersRequest;
use dokime_protocol::host::ExecutionsParametersResponse;
use dokime_protocol::host::ExecutionsSetRequest;
use dokime_protocol::host::ExecutionsSetResponse;
use dokime_protocol::host::ExecutionsStartRequest;
use dokime_protocol::host::ExecutionsStartResponse;
use dokime_protocol::host::ExecutionsStepsRequest;
use dokime_protocol::host::ExecutionsStepsResponse;
use dokime_protocol::host::ExecutionsStopRequest;
use dokime_protocol::host::ExecutionsStopResponse;
use dokime_protocol::host::ExecutionsWatchRequest;
use dokime_protocol::host::ExecutionsWatchResponse;
use dokime_protocol::host::executions_server::Executions;
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
impl Executions for Services {
    async fn get(&self, request: Request<ExecutionsGetRequest>) -> Result<Response<ExecutionsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, request: Request<ExecutionsSetRequest>) -> Result<Response<ExecutionsSetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Set", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<ExecutionsListRequest>) -> Result<Response<ExecutionsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<ExecutionsWatchResponse>;
    async fn watch(&self, request: Request<ExecutionsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, request: Request<ExecutionsCreateRequest>) -> Result<Response<ExecutionsCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn start(&self, request: Request<ExecutionsStartRequest>) -> Result<Response<ExecutionsStartResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Start", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::start::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<ExecutionsStopRequest>) -> Result<Response<ExecutionsStopResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Stop", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::stop::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn cancel(&self, request: Request<ExecutionsCancelRequest>) -> Result<Response<ExecutionsCancelResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Cancel", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::cancel::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<ExecutionsDeleteRequest>) -> Result<Response<ExecutionsDeleteResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Delete", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn parameters(&self, request: Request<ExecutionsParametersRequest>) -> Result<Response<ExecutionsParametersResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Parameters", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn steps(&self, request: Request<ExecutionsStepsRequest>) -> Result<Response<ExecutionsStepsResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Steps", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::steps::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
