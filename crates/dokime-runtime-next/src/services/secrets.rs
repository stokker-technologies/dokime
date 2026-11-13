/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Resource Service

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
impl dokime_protocol::client::secrets_server::Secrets for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::SecretsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::SecretsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::SecretsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::SecretsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::SecretsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::client::SecretsDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn export(
        &self, request: tonic::Request<dokime_protocol::client::SecretsExportRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsExportResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_export::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn import(
        &self, request: tonic::Request<dokime_protocol::client::SecretsImportRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SecretsImportResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_import::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
