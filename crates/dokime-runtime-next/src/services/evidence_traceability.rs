/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Traceability Client Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1::evidence;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl evidence::traceability_server::Traceability for Services {
    async fn get(
        &self, request: tonic::Request<evidence::TraceabilityGetRequest>,
    ) -> Result<tonic::Response<evidence::TraceabilityGetResponse>, tonic::Status> {
        let response = dokime_operations_next::client::evidence_traceability_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(response))
    }
    type WatchStream = dokime_operations_next::client::Output<evidence::TraceabilityWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<evidence::TraceabilityWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let response = dokime_operations_next::client::evidence_traceability_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(response))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
