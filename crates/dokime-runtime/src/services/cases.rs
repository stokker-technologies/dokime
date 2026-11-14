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
impl dokime_protocol::host::cases_server::Cases for Services {
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::CasesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CasesCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::CasesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CasesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::CasesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CasesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::CasesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::CasesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::host::CasesDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CasesDeleteResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Delete", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<dokime_protocol::host::CasesParametersRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CasesParametersResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cases/Parameters", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cases::parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
