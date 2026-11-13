/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Resource Service

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
impl dokime_protocol::client::executions_server::Executions for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::ExecutionsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn start(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsStartRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsStartResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_start::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn stop(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsStopRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsStopResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_stop::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn cancel(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsCancelRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsCancelResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_cancel::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsParametersRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsParametersResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn steps(
        &self, request: tonic::Request<dokime_protocol::client::ExecutionsStepsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::ExecutionsStepsResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_steps::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
