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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::content::artifacts_server::Artifacts for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::content::ArtifactsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::content::ArtifactsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::artifacts::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::content::ArtifactsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::content::ArtifactsCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::artifacts::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::content::ArtifactsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::content::ArtifactsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::artifacts::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::content::ArtifactsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::content::ArtifactsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.Artifacts/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::artifacts::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
