/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Execution Files Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::content::ExecutionFilesGetRequest;
use dokime_protocol::host::content::ExecutionFilesGetResponse;
use dokime_protocol::host::content::ExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesListResponse;
use dokime_protocol::host::content::ExecutionFilesWatchRequest;
use dokime_protocol::host::content::ExecutionFilesWatchResponse;
use dokime_protocol::host::content::execution_files_server::ExecutionFiles;
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
impl ExecutionFiles for Services {
    async fn get(&self, request: Request<ExecutionFilesGetRequest>) -> Result<Response<ExecutionFilesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/Get", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::execution_files::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<ExecutionFilesListRequest>) -> Result<Response<ExecutionFilesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/List", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::execution_files::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<ExecutionFilesWatchResponse>;
    async fn watch(&self, request: Request<ExecutionFilesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::execution_files::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
