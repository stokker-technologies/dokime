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
impl dokime_protocol::client::content::artifacts_server::Artifacts for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::content::ArtifactsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::content::ArtifactsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::content::ArtifactsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::content::ArtifactsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::content::ArtifactsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::content::ArtifactsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::content::ArtifactsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::content::ArtifactsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
