/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

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
impl v1::health_server::Health for Services {
    async fn list_health(
        &self, request: tonic::Request<v1::HealthListHealthRequest>,
    ) -> Result<tonic::Response<v1::ListHealthResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::health_list_health::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListHealthStream = dokime_operations_next::client::Output<v1::HealthListHealthRunOutput>;
    async fn run_list_health(
        &self, request: tonic::Request<tonic::Streaming<v1::HealthListHealthRunInput>>,
    ) -> Result<tonic::Response<Self::RunListHealthStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Health.RunListHealth: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
