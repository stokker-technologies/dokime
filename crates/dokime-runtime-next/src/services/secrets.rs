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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::secrets_server::Secrets for Services {
    async fn get(&self, request: tonic::Request<v1::SecretsGetRequest>) -> Result<tonic::Response<v1::SecretsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(&self, request: tonic::Request<v1::SecretsSetRequest>) -> Result<tonic::Response<v1::SecretsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::SecretsListRequest>,
    ) -> Result<tonic::Response<v1::SecretsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::SecretsWatchResponse>;
    async fn watch(&self, request: tonic::Request<v1::SecretsWatchRequest>) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<v1::SecretsDeleteRequest>,
    ) -> Result<tonic::Response<v1::SecretsDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn export(
        &self, request: tonic::Request<v1::SecretsExportRequest>,
    ) -> Result<tonic::Response<v1::SecretsExportResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_export::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn import(
        &self, request: tonic::Request<v1::SecretsImportRequest>,
    ) -> Result<tonic::Response<v1::SecretsImportResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_import::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
