/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::EvidenceArchiveRequest;
use dokime_protocol::host::EvidenceArchiveResponse;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_protocol::host::evidence_server::Evidence;
use harmos_protocol::model::Callers;
use tonic::Request;
use tonic::Response;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Evidence for Services {
    async fn statistics(&self, request: Request<EvidenceStatisticsRequest>) -> Result<Response<EvidenceStatisticsResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Evidence/Statistics", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::statistics::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn archive(&self, request: Request<EvidenceArchiveRequest>) -> Result<Response<EvidenceArchiveResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Evidence/Archive", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::archive::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
