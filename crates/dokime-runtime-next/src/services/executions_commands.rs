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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::executions::commands_server::Commands for Services {
    async fn get(
        &self, request: tonic::Request<v1::executions::CommandsGetRequest>,
    ) -> Result<tonic::Response<v1::executions::CommandsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_commands_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::executions::CommandsCreateRequest>,
    ) -> Result<tonic::Response<v1::executions::CommandsCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::executions_commands_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
