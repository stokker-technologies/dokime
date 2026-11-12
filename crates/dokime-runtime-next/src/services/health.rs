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
impl v1::health_server::Health for Services {
    async fn list_health(&self, request: tonic::Request<v1::HealthListHealthRequest>) -> Result<tonic::Response<v1::ListHealthResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::health_list_health::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListHealthStream = dokime_operations_next::client::Output<v1::HealthListHealthRunOutput>;
    async fn run_list_health(&self, request: tonic::Request<tonic::Streaming<v1::HealthListHealthRunInput>>) -> Result<tonic::Response<Self::RunListHealthStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::health_list_health_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::health_list_health_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::health_list_health_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::HealthListHealthRunOutput { value: Some(v1::health_list_health_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::health_list_health::handle(&context, request).await },
        ).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
