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
    async fn materialize_template(
        &self, request: tonic::Request<v1::PlanningMaterializeTemplateRequest>,
    ) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let reply = dokime_operations_next::client::planning_materialize_template::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunMaterializeTemplateStream = dokime_operations_next::client::Output<v1::PlanningMaterializeTemplateRunOutput>;
    async fn run_materialize_template(
        &self, request: tonic::Request<tonic::Streaming<v1::PlanningMaterializeTemplateRunInput>>,
    ) -> Result<tonic::Response<Self::RunMaterializeTemplateStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Planning.RunMaterializeTemplate: Not Yet Served by the New Host"))
    }
    async fn preview_selection(
        &self, request: tonic::Request<v1::PlanningPreviewSelectionRequest>,
    ) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Planning.PreviewSelection: Not Yet Served by the New Host"))
    }
    type RunPreviewSelectionStream = dokime_operations_next::client::Output<v1::PlanningPreviewSelectionRunOutput>;
    async fn run_preview_selection(
        &self, request: tonic::Request<tonic::Streaming<v1::PlanningPreviewSelectionRunInput>>,
    ) -> Result<tonic::Response<Self::RunPreviewSelectionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Planning.RunPreviewSelection: Not Yet Served by the New Host"))
    }
    async fn register_selection(
        &self, request: tonic::Request<v1::PlanningRegisterSelectionRequest>,
    ) -> Result<tonic::Response<v1::PreviewSelectionReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Planning.RegisterSelection: Not Yet Served by the New Host"))
    }
    type RunRegisterSelectionStream = dokime_operations_next::client::Output<v1::PlanningRegisterSelectionRunOutput>;
    async fn run_register_selection(
        &self, request: tonic::Request<tonic::Streaming<v1::PlanningRegisterSelectionRunInput>>,
    ) -> Result<tonic::Response<Self::RunRegisterSelectionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Planning.RunRegisterSelection: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
