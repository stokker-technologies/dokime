/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Progress Client Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1::executions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl executions::progress_server::Progress for Services {
    async fn get(
        &self, request: tonic::Request<executions::ProgressGetRequest>,
    ) -> Result<tonic::Response<executions::ProgressGetResponse>, tonic::Status> {
        let response = dokime_operations_next::client::executions_progress_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(response))
    }
    type WatchStream = dokime_operations_next::client::Output<executions::ProgressWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<executions::ProgressWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let response = dokime_operations_next::client::executions_progress_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(response))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
