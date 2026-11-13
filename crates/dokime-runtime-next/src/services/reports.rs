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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::reports_server::Reports for Services {
    async fn get(&self, request: tonic::Request<v1::ReportsGetRequest>) -> Result<tonic::Response<v1::ReportsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::ReportsCreateRequest>,
    ) -> Result<tonic::Response<v1::ReportsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn publish(
        &self, request: tonic::Request<v1::ReportsPublishRequest>,
    ) -> Result<tonic::Response<v1::ReportsPublishResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_publish::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
