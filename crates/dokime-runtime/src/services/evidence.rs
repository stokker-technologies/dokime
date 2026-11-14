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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::evidence_server::Evidence for Services {
    async fn statistics(
        &self, request: tonic::Request<dokime_protocol::host::EvidenceStatisticsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::EvidenceStatisticsResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Evidence/Statistics", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.Evidence/Statistics: Operation Is Unavailable in This Host"))
    }
    async fn archive(
        &self, request: tonic::Request<dokime_protocol::host::EvidenceArchiveRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::EvidenceArchiveResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Evidence/Archive", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::archive::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
