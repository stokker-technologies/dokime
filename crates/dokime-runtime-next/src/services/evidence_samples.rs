/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Samples Resource Service

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
impl dokime_protocol::client::evidence::samples_server::Samples for Services {
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::evidence::SamplesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::SamplesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_samples_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::evidence::SamplesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::evidence::SamplesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_samples_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
