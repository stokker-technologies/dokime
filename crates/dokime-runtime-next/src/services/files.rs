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
use dokime_operations_next::client as operations;
use dokime_protocol::client::files_server::Files;
use dokime_protocol::client::{
    FilesDeleteRequest, FilesDeleteResponse, FilesGetRequest, FilesGetResponse, FilesReadRequest, FilesReadResponse, FilesWatchRequest,
    FilesWatchResponse,
};
use tonic::{Request, Response, Status};

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
        let reply = operations::files_get::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn read(&self, request: Request<FilesReadRequest>) -> Result<Response<FilesReadResponse>, Status> {
        let reply = operations::files_read::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<FilesDeleteRequest>) -> Result<Response<FilesDeleteResponse>, Status> {
        let reply = operations::files_delete::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = operations::Output<FilesWatchResponse>;
    async fn watch(&self, request: Request<FilesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        let reply = operations::files_watch::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
