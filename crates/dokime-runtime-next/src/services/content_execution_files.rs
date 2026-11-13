/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Execution Files Resource Service

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
impl dokime_protocol::client::content::execution_files_server::ExecutionFiles for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::content::ExecutionFilesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::content::ExecutionFilesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_execution_files_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::content::ExecutionFilesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::content::ExecutionFilesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_execution_files_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::content::ExecutionFilesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::content::ExecutionFilesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::content_execution_files_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
