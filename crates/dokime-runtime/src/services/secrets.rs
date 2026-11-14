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
impl dokime_protocol::host::secrets_server::Secrets for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::SecretsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::SecretsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::SecretsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::SecretsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::SecretsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::host::SecretsDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsDeleteResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Delete", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn export(
        &self, request: tonic::Request<dokime_protocol::host::SecretsExportRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsExportResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Export", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::export::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn import(
        &self, request: tonic::Request<dokime_protocol::host::SecretsImportRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SecretsImportResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Import", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::secrets::import::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
