/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

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
impl v1::prompts_server::Prompts for Services {
    async fn get_definition(
        &self, request: tonic::Request<v1::PromptsGetDefinitionRequest>,
    ) -> Result<tonic::Response<v1::Prompt>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.GetDefinition: Not Yet Served by the New Host"))
    }
    type RunGetDefinitionStream = dokime_operations_next::client::Output<v1::PromptsGetDefinitionRunOutput>;
    async fn run_get_definition(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsGetDefinitionRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetDefinitionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.RunGetDefinition: Not Yet Served by the New Host"))
    }
    async fn get_prompt(
        &self, request: tonic::Request<v1::PromptsGetPromptRequest>,
    ) -> Result<tonic::Response<v1::PromptView>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_get_prompt::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetPromptStream = dokime_operations_next::client::Output<v1::PromptsGetPromptRunOutput>;
    async fn run_get_prompt(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsGetPromptRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetPromptStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.RunGetPrompt: Not Yet Served by the New Host"))
    }
    async fn list_definitions(
        &self, request: tonic::Request<v1::PromptsListDefinitionsRequest>,
    ) -> Result<tonic::Response<v1::ListPromptDefinitionsReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.ListDefinitions: Not Yet Served by the New Host"))
    }
    type RunListDefinitionsStream = dokime_operations_next::client::Output<v1::PromptsListDefinitionsRunOutput>;
    async fn run_list_definitions(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsListDefinitionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListDefinitionsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.RunListDefinitions: Not Yet Served by the New Host"))
    }
    async fn list_pending(
        &self, request: tonic::Request<v1::PromptsListPendingRequest>,
    ) -> Result<tonic::Response<v1::ListPendingPromptsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_list_pending::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListPendingStream = dokime_operations_next::client::Output<v1::PromptsListPendingRunOutput>;
    async fn run_list_pending(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsListPendingRunInput>>,
    ) -> Result<tonic::Response<Self::RunListPendingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.RunListPending: Not Yet Served by the New Host"))
    }
    async fn respond(
        &self, request: tonic::Request<v1::PromptsRespondRequest>,
    ) -> Result<tonic::Response<v1::PromptResponseReceipt>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_respond::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRespondStream = dokime_operations_next::client::Output<v1::PromptsRespondRunOutput>;
    async fn run_respond(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsRespondRunInput>>,
    ) -> Result<tonic::Response<Self::RunRespondStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Prompts.RunRespond: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
