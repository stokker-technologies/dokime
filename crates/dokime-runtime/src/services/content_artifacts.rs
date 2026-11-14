/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse;
use dokime_protocol::host::content::ArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse;
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsListResponse;
use dokime_protocol::host::content::ArtifactsWatchRequest;
use dokime_protocol::host::content::ArtifactsWatchResponse;
use dokime_protocol::host::content::artifacts_server::Artifacts;
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
impl Artifacts for Services {
    async fn get(&self, request: Request<ArtifactsGetRequest>) -> Result<Response<ArtifactsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::artifacts::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, request: Request<ArtifactsCreateRequest>) -> Result<Response<ArtifactsCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Create", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::artifacts::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<ArtifactsListRequest>) -> Result<Response<ArtifactsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/List", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::artifacts::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<ArtifactsWatchResponse>;
    async fn watch(&self, request: Request<ArtifactsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::content::artifacts::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
