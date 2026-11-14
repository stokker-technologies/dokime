/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Snapshots Resource Service

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
impl dokime_protocol::client::plots::snapshots_server::Snapshots for Services {
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::plots::SnapshotsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::plots::SnapshotsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::plots_snapshots_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::plots::SnapshotsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::plots::SnapshotsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::plots_snapshots_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
