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
impl v1::runtime_server::Runtime for Services {
    async fn get_identity(&self, request: tonic::Request<v1::RuntimeGetIdentityRequest>) -> Result<tonic::Response<v1::BackendIdentity>, tonic::Status> {
        let reply = dokime_operations_next::client::runtime_get_identity::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetIdentityStream = dokime_operations_next::client::Output<v1::RuntimeGetIdentityRunOutput>;
    async fn run_get_identity(&self, request: tonic::Request<tonic::Streaming<v1::RuntimeGetIdentityRunInput>>) -> Result<tonic::Response<Self::RunGetIdentityStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::runtime_get_identity_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::runtime_get_identity_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::runtime_get_identity_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::RuntimeGetIdentityRunOutput { value: Some(v1::runtime_get_identity_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::runtime_get_identity::handle(&context, request).await },
        ).await
    }
    async fn get_health(&self, request: tonic::Request<v1::RuntimeGetHealthRequest>) -> Result<tonic::Response<v1::BackendHealth>, tonic::Status> {
        let reply = dokime_operations_next::client::runtime_get_health::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetHealthStream = dokime_operations_next::client::Output<v1::RuntimeGetHealthRunOutput>;
    async fn run_get_health(&self, request: tonic::Request<tonic::Streaming<v1::RuntimeGetHealthRunInput>>) -> Result<tonic::Response<Self::RunGetHealthStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::runtime_get_health_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::runtime_get_health_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::runtime_get_health_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::RuntimeGetHealthRunOutput { value: Some(v1::runtime_get_health_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::runtime_get_health::handle(&context, request).await },
        ).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
