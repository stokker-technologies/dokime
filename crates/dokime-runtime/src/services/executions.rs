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
impl dokime_protocol::host::executions_server::Executions for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::ExecutionsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn start(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsStartRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsStartResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Start", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::start::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn stop(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsStopRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsStopResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Stop", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::stop::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn cancel(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsCancelRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsCancelResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Cancel", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::cancel::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsDeleteRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsDeleteResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Delete", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn parameters(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsParametersRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsParametersResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Parameters", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn steps(
        &self, request: tonic::Request<dokime_protocol::host::ExecutionsStepsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::ExecutionsStepsResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Executions/Steps", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::steps::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
