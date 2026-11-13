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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::cases_server::Cases for Services {
    async fn create(
        &self, request: tonic::Request<v1::CasesCreateRequest>,
    ) -> Result<tonic::Response<v1::CasesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn get(&self, request: tonic::Request<v1::CasesGetRequest>) -> Result<tonic::Response<v1::CasesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(&self, request: tonic::Request<v1::CasesListRequest>) -> Result<tonic::Response<v1::CasesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::CasesWatchResponse>;
    async fn watch(&self, request: tonic::Request<v1::CasesWatchRequest>) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<v1::CasesDeleteRequest>,
    ) -> Result<tonic::Response<v1::CasesDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<v1::CasesParametersRequest>,
    ) -> Result<tonic::Response<v1::CasesParametersResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cases_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
