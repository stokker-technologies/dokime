/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Resource Service

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
impl dokime_protocol::host::reports_server::Reports for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::ReportsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ReportsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::reports::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::ReportsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ReportsCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::reports::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn publish(
        &self, request: tonic::Request<dokime_protocol::host::ReportsPublishRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ReportsPublishResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Publish", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::reports::publish::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
