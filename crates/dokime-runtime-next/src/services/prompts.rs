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
        let reply = dokime_operations_next::client::prompts_get_definition::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetDefinitionStream = dokime_operations_next::client::Output<v1::PromptsGetDefinitionRunOutput>;
    async fn run_get_definition(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsGetDefinitionRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetDefinitionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::PromptsGetDefinitionRunInput| match input.value? {
                v1::prompts_get_definition_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::prompts_get_definition_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::prompts_get_definition_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::PromptsGetDefinitionRunOutput {
                value: Some(v1::prompts_get_definition_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::prompts_get_definition::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_prompt(
        &self, request: tonic::Request<v1::PromptsGetPromptRequest>,
    ) -> Result<tonic::Response<v1::PromptView>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_get_prompt::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetPromptStream = dokime_operations_next::client::Output<v1::PromptsGetPromptRunOutput>;
    async fn run_get_prompt(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsGetPromptRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetPromptStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::PromptsGetPromptRunInput| match input.value? {
                v1::prompts_get_prompt_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::prompts_get_prompt_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::prompts_get_prompt_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::PromptsGetPromptRunOutput {
                value: Some(v1::prompts_get_prompt_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::prompts_get_prompt::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_definitions(
        &self, request: tonic::Request<v1::PromptsListDefinitionsRequest>,
    ) -> Result<tonic::Response<v1::ListPromptDefinitionsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_list_definitions::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListDefinitionsStream = dokime_operations_next::client::Output<v1::PromptsListDefinitionsRunOutput>;
    async fn run_list_definitions(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsListDefinitionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListDefinitionsStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::PromptsListDefinitionsRunInput| match input.value? {
                v1::prompts_list_definitions_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::prompts_list_definitions_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::prompts_list_definitions_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::PromptsListDefinitionsRunOutput {
                value: Some(v1::prompts_list_definitions_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::prompts_list_definitions::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_pending(
        &self, request: tonic::Request<v1::PromptsListPendingRequest>,
    ) -> Result<tonic::Response<v1::ListPendingPromptsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_list_pending::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListPendingStream = dokime_operations_next::client::Output<v1::PromptsListPendingRunOutput>;
    async fn run_list_pending(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsListPendingRunInput>>,
    ) -> Result<tonic::Response<Self::RunListPendingStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::PromptsListPendingRunInput| match input.value? {
                v1::prompts_list_pending_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::prompts_list_pending_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::prompts_list_pending_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::PromptsListPendingRunOutput {
                value: Some(v1::prompts_list_pending_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::prompts_list_pending::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn respond(
        &self, request: tonic::Request<v1::PromptsRespondRequest>,
    ) -> Result<tonic::Response<v1::PromptResponseReceipt>, tonic::Status> {
        let reply = dokime_operations_next::client::prompts_respond::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRespondStream = dokime_operations_next::client::Output<v1::PromptsRespondRunOutput>;
    async fn run_respond(
        &self, request: tonic::Request<tonic::Streaming<v1::PromptsRespondRunInput>>,
    ) -> Result<tonic::Response<Self::RunRespondStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::PromptsRespondRunInput| match input.value? {
                v1::prompts_respond_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::prompts_respond_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::prompts_respond_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::PromptsRespondRunOutput {
                value: Some(v1::prompts_respond_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::prompts_respond::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
