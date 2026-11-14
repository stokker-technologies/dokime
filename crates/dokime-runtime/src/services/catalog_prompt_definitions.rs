/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Prompt Definitions Resource Service

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
impl dokime_protocol::host::prompt_definitions_server::PromptDefinitions for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::PromptDefinitionsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::PromptDefinitionsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.PromptDefinitions/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::prompt_definitions::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::PromptDefinitionsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::PromptDefinitionsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.PromptDefinitions/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::prompt_definitions::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
