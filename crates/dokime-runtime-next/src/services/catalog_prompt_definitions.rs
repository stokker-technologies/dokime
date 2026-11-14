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
impl dokime_protocol::client::prompt_definitions_server::PromptDefinitions for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::PromptDefinitionsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::PromptDefinitionsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_prompt_definitions_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::PromptDefinitionsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::PromptDefinitionsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_prompt_definitions_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
