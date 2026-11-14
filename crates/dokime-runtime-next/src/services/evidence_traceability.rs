/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Traceability Resource Service

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
impl dokime_protocol::client::evidence::traceability_server::Traceability for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::evidence::TraceabilityGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::TraceabilityGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_traceability_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::evidence::TraceabilityWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::evidence::TraceabilityWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_traceability_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
