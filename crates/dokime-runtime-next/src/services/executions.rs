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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::executions_server::Executions for Services {
    async fn get(
        &self, request: tonic::Request<v1::ExecutionsGetRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<v1::ExecutionsSetRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::ExecutionsListRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::ExecutionsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<v1::ExecutionsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::ExecutionsCreateRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn start(
        &self, request: tonic::Request<v1::ExecutionsStartRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsStartResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_start::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn stop(
        &self, request: tonic::Request<v1::ExecutionsStopRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsStopResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_stop::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn cancel(
        &self, request: tonic::Request<v1::ExecutionsCancelRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsCancelResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_cancel::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<v1::ExecutionsDeleteRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsDeleteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<v1::ExecutionsParametersRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsParametersResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn steps(
        &self, request: tonic::Request<v1::ExecutionsStepsRequest>,
    ) -> Result<tonic::Response<v1::ExecutionsStepsResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_steps::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
