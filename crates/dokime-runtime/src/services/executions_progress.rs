/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Progress Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::host::executions::ProgressSetRequest;
use dokime_protocol::host::executions::ProgressSetResponse;
use dokime_protocol::host::executions::ProgressWatchRequest;
use dokime_protocol::host::executions::ProgressWatchResponse;
use dokime_protocol::host::executions::progress_server::Progress;
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
impl Progress for Services {
    async fn get(&self, request: Request<ProgressGetRequest>) -> Result<Response<ProgressGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.executions.Progress/Get", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::progress::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<ProgressWatchResponse>;
    async fn watch(&self, request: Request<ProgressWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.executions.Progress/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::executions::progress::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, _request: Request<ProgressSetRequest>) -> Result<Response<ProgressSetResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.executions.Progress/Set")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
