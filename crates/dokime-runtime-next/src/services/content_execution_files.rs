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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::content::execution_files_server::ExecutionFiles for Services {
    async fn get(
        &self, request: tonic::Request<v1::content::ExecutionFilesGetRequest>,
    ) -> Result<tonic::Response<v1::content::ExecutionFilesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_execution_files_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::content::ExecutionFilesListRequest>,
    ) -> Result<tonic::Response<v1::content::ExecutionFilesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::content_execution_files_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::content::ExecutionFilesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<v1::content::ExecutionFilesWatchRequest>,
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
