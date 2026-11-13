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
impl dokime_protocol::client::reports_server::Reports for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::ReportsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ReportsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::ReportsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ReportsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn publish(
        &self, request: tonic::Request<dokime_protocol::client::ReportsPublishRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ReportsPublishResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_publish::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
