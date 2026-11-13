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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::content::artifacts_server::Artifacts for Services {
    async fn get(
        &self, request: tonic::Request<v1::content::ArtifactsGetRequest>,
    ) -> Result<tonic::Response<v1::content::ArtifactsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::content::ArtifactsCreateRequest>,
    ) -> Result<tonic::Response<v1::content::ArtifactsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::content::ArtifactsListRequest>,
    ) -> Result<tonic::Response<v1::content::ArtifactsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_artifacts_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::content::ArtifactsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<v1::content::ArtifactsWatchRequest>,
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
