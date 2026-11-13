/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Commands Resource Service

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
impl dokime_protocol::client::executions::commands_server::Commands for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::executions::CommandsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::executions::CommandsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_commands_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::executions::CommandsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::executions::CommandsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_commands_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
