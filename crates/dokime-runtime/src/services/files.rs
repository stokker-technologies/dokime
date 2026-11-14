/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::Services;
use dokime_operations::host as operations;
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::FilesDeleteResponse;
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesGetResponse;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::FilesReadResponse;
use dokime_protocol::host::FilesWatchRequest;
use dokime_protocol::host::FilesWatchResponse;
use dokime_protocol::host::files_server::Files;
use tonic::{Request, Response, Status};

use dokime_protocol::host::FilesCaptureRequest;
use dokime_protocol::host::FilesCaptureResponse;
use harmos_protocol::model::Callers;
use harmos_runtime::Failure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Files for Services {
    async fn get(&self, request: Request<FilesGetRequest>) -> Result<Response<FilesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Files/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = operations::files::get::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn read(&self, request: Request<FilesReadRequest>) -> Result<Response<FilesReadResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Files/Read", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = operations::files::read::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<FilesDeleteRequest>) -> Result<Response<FilesDeleteResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Files/Delete", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = operations::files::delete::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = operations::Output<FilesWatchResponse>;
    async fn watch(&self, request: Request<FilesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Files/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = operations::files::watch::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn capture(&self, _request: Request<FilesCaptureRequest>) -> Result<Response<FilesCaptureResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.Files/Capture")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
