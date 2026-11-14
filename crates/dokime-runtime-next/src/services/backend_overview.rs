/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Overview Resource Service

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
impl dokime_protocol::client::backend::overview_server::Overview for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::backend::OverviewGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::backend::OverviewGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_overview_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::backend::OverviewWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::backend::OverviewWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_overview_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
