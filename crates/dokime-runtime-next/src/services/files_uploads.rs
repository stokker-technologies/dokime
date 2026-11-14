/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Uploads Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::Services;
use dokime_operations_next::client as operations;
use dokime_protocol::client::files::uploads_server::Uploads;
use dokime_protocol::client::files::{
    UploadsAppendRequest, UploadsAppendResponse, UploadsCreateRequest, UploadsCreateResponse, UploadsDeleteRequest, UploadsDeleteResponse,
    UploadsGetRequest, UploadsGetResponse, UploadsPublishRequest, UploadsPublishResponse, UploadsResumeRequest, UploadsResumeResponse,
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
impl Uploads for Services {
    async fn create(&self, request: Request<UploadsCreateRequest>) -> Result<Response<UploadsCreateResponse>, Status> {
        let reply = operations::files_uploads_create::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn get(&self, request: Request<UploadsGetRequest>) -> Result<Response<UploadsGetResponse>, Status> {
        let reply = operations::files_uploads_get::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn resume(&self, request: Request<UploadsResumeRequest>) -> Result<Response<UploadsResumeResponse>, Status> {
        let reply = operations::files_uploads_resume::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn append(&self, request: Request<UploadsAppendRequest>) -> Result<Response<UploadsAppendResponse>, Status> {
        let reply = operations::files_uploads_append::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn publish(&self, request: Request<UploadsPublishRequest>) -> Result<Response<UploadsPublishResponse>, Status> {
        let reply = operations::files_uploads_publish::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<UploadsDeleteRequest>) -> Result<Response<UploadsDeleteResponse>, Status> {
        let reply = operations::files_uploads_delete::handle(&self.context, request.into_inner()).await.map_err(operations::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
