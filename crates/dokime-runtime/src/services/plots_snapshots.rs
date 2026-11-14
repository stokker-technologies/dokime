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
use tonic::{Request, Response, Status};

// Local Imports
use super::Services;
use dokime_operations::host::{Output, plots_snapshots_list, plots_snapshots_watch, status};
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsListResponse;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::plots::SnapshotsWatchResponse;
use dokime_protocol::host::plots::snapshots_server::Snapshots;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Snapshots for Services {
    async fn list(&self, request: Request<SnapshotsListRequest>) -> Result<Response<SnapshotsListResponse>, Status> {
        let reply = plots_snapshots_list::handle(&self.context, request.into_inner()).await.map_err(status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<SnapshotsWatchResponse>;
    async fn watch(&self, request: Request<SnapshotsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        let reply = plots_snapshots_watch::handle(&self.context, request.into_inner()).await.map_err(status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
