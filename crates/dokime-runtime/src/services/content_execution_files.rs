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
impl dokime_protocol::host::content::execution_files_server::ExecutionFiles for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::content::ExecutionFilesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::content::ExecutionFilesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::execution_files::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::content::ExecutionFilesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::content::ExecutionFilesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::execution_files::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::content::ExecutionFilesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::content::ExecutionFilesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.content.ExecutionFiles/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::content::execution_files::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
