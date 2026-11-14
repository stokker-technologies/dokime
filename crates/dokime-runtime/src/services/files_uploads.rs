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
use dokime_operations::host as operations;
use dokime_protocol::host::files::UploadsAppendRequest;
use dokime_protocol::host::files::UploadsAppendResponse;
use dokime_protocol::host::files::UploadsCreateRequest;
use dokime_protocol::host::files::UploadsCreateResponse;
use dokime_protocol::host::files::UploadsDeleteRequest;
use dokime_protocol::host::files::UploadsDeleteResponse;
use dokime_protocol::host::files::UploadsGetRequest;
use dokime_protocol::host::files::UploadsGetResponse;
use dokime_protocol::host::files::UploadsPublishRequest;
use dokime_protocol::host::files::UploadsPublishResponse;
use dokime_protocol::host::files::UploadsResumeRequest;
use dokime_protocol::host::files::UploadsResumeResponse;
use dokime_protocol::host::files::uploads_server::Uploads;
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
