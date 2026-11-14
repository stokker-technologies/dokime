/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Resource Service

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
impl dokime_protocol::host::prompts_server::Prompts for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::PromptsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::PromptsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::prompts::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::PromptsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::PromptsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::prompts::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::PromptsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::PromptsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Prompts/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::prompts::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
