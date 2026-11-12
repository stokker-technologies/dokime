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
impl v1::planning_server::Planning for Services {
    async fn materialize_template(&self, request: tonic::Request<v1::PlanningMaterializeTemplateRequest>) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let reply = dokime_operations_next::client::planning_materialize_template::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunMaterializeTemplateStream = dokime_operations_next::client::Output<v1::PlanningMaterializeTemplateRunOutput>;
    async fn run_materialize_template(&self, request: tonic::Request<tonic::Streaming<v1::PlanningMaterializeTemplateRunInput>>) -> Result<tonic::Response<Self::RunMaterializeTemplateStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(), request.into_inner(), |input: v1::PlanningMaterializeTemplateRunInput| match input.value? {
            v1::planning_materialize_template_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::planning_materialize_template_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::planning_materialize_template_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::PlanningMaterializeTemplateRunOutput { value: Some(v1::planning_materialize_template_run_output::Value::Response(response)) }, |context, request| async move { dokime_operations_next::client::planning_materialize_template::handle(&context, request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn preview_selection(&self, request: tonic::Request<v1::PlanningPreviewSelectionRequest>) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let reply = dokime_operations_next::client::planning_preview_selection::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPreviewSelectionStream = dokime_operations_next::client::Output<v1::PlanningPreviewSelectionRunOutput>;
    async fn run_preview_selection(&self, request: tonic::Request<tonic::Streaming<v1::PlanningPreviewSelectionRunInput>>) -> Result<tonic::Response<Self::RunPreviewSelectionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(), request.into_inner(), |input: v1::PlanningPreviewSelectionRunInput| match input.value? {
            v1::planning_preview_selection_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::planning_preview_selection_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::planning_preview_selection_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::PlanningPreviewSelectionRunOutput { value: Some(v1::planning_preview_selection_run_output::Value::Response(response)) }, |context, request| async move { dokime_operations_next::client::planning_preview_selection::handle(&context, request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn register_selection(&self, request: tonic::Request<v1::PlanningRegisterSelectionRequest>) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let reply = dokime_operations_next::client::planning_register_selection::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRegisterSelectionStream = dokime_operations_next::client::Output<v1::PlanningRegisterSelectionRunOutput>;
    async fn run_register_selection(&self, request: tonic::Request<tonic::Streaming<v1::PlanningRegisterSelectionRunInput>>) -> Result<tonic::Response<Self::RunRegisterSelectionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(), request.into_inner(), |input: v1::PlanningRegisterSelectionRunInput| match input.value? {
            v1::planning_register_selection_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::planning_register_selection_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::planning_register_selection_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::PlanningRegisterSelectionRunOutput { value: Some(v1::planning_register_selection_run_output::Value::Response(response)) }, |context, request| async move { dokime_operations_next::client::planning_register_selection::handle(&context, request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
