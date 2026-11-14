/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Health Events Resource Service

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
impl dokime_protocol::client::evidence::health_events_server::HealthEvents for Services {
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::evidence::HealthEventsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::HealthEventsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_health_events_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::evidence::HealthEventsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::evidence::HealthEventsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_health_events_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
