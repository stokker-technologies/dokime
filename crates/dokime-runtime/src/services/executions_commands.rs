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
impl dokime_protocol::host::executions::commands_server::Commands for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::executions::CommandsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::executions::CommandsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.executions.Commands/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::commands::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::executions::CommandsCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::executions::CommandsCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.executions.Commands/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::executions::commands::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
