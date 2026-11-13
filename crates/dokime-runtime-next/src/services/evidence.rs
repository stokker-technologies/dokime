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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::evidence_server::Evidence for Services {
    async fn statistics(
        &self, request: tonic::Request<v1::EvidenceStatisticsRequest>,
    ) -> Result<tonic::Response<v1::EvidenceStatisticsResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.Evidence/Statistics: Operation Is Unavailable in This Host"))
    }
    async fn archive(
        &self, request: tonic::Request<v1::EvidenceArchiveRequest>,
    ) -> Result<tonic::Response<v1::EvidenceArchiveResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_archive::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
