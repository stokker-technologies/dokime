/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Resource Service

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
impl dokime_protocol::client::cases_server::Cases for Services {
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::CasesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CasesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::CasesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CasesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::CasesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CasesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::CasesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::CasesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::client::CasesDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CasesDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<dokime_protocol::client::CasesParametersRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CasesParametersResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
