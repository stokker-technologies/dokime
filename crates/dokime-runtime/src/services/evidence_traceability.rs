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
impl dokime_protocol::host::evidence::traceability_server::Traceability for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::evidence::TraceabilityGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::evidence::TraceabilityGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Traceability/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::traceability::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::evidence::TraceabilityWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::evidence::TraceabilityWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Traceability/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::traceability::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
