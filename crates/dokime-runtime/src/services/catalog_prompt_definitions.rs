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

use dokime_protocol::host::PromptDefinitionsGetRequest;
use dokime_protocol::host::PromptDefinitionsGetResponse;
use dokime_protocol::host::PromptDefinitionsListRequest;
use dokime_protocol::host::PromptDefinitionsListResponse;
use dokime_protocol::host::prompt_definitions_server::PromptDefinitions;
use harmos_protocol::model::Callers;
use tonic::Request;
use tonic::Response;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl PromptDefinitions for Services {
    async fn get(&self, request: Request<PromptDefinitionsGetRequest>) -> Result<Response<PromptDefinitionsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.PromptDefinitions/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompt_definitions::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<PromptDefinitionsListRequest>) -> Result<Response<PromptDefinitionsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.PromptDefinitions/List", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompt_definitions::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
