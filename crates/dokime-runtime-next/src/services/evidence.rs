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
impl dokime_protocol::client::evidence_server::Evidence for Services {
    async fn statistics(
        &self, request: tonic::Request<dokime_protocol::client::EvidenceStatisticsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::EvidenceStatisticsResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.Evidence/Statistics: Operation Is Unavailable in This Host"))
    }
    async fn archive(
        &self, request: tonic::Request<dokime_protocol::client::EvidenceArchiveRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::EvidenceArchiveResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_archive::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
