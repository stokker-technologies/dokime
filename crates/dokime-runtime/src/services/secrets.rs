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

use dokime_operations::host::Output;
use dokime_protocol::host::SecretsDeleteRequest;
use dokime_protocol::host::SecretsDeleteResponse;
use dokime_protocol::host::SecretsExportRequest;
use dokime_protocol::host::SecretsExportResponse;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;
use dokime_protocol::host::SecretsImportRequest;
use dokime_protocol::host::SecretsImportResponse;
use dokime_protocol::host::SecretsListRequest;
use dokime_protocol::host::SecretsListResponse;
use dokime_protocol::host::SecretsSetRequest;
use dokime_protocol::host::SecretsSetResponse;
use dokime_protocol::host::SecretsWatchRequest;
use dokime_protocol::host::SecretsWatchResponse;
use dokime_protocol::host::secrets_server::Secrets;
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
impl Secrets for Services {
    async fn get(&self, request: Request<SecretsGetRequest>) -> Result<Response<SecretsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, request: Request<SecretsSetRequest>) -> Result<Response<SecretsSetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Set", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<SecretsListRequest>) -> Result<Response<SecretsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<SecretsWatchResponse>;
    async fn watch(&self, request: Request<SecretsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn delete(&self, request: Request<SecretsDeleteRequest>) -> Result<Response<SecretsDeleteResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Delete", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn export(&self, request: Request<SecretsExportRequest>) -> Result<Response<SecretsExportResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Export", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::export::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn import(&self, request: Request<SecretsImportRequest>) -> Result<Response<SecretsImportResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Secrets/Import", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::secrets::import::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
