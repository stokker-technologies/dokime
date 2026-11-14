/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::BackendAppearanceRequest;
use dokime_protocol::host::BackendAppearanceResponse;
use dokime_protocol::host::BackendEnvironmentRequest;
use dokime_protocol::host::BackendEnvironmentResponse;
use dokime_protocol::host::backend_server::Backend;
use harmos_protocol::model::Callers;
use harmos_runtime::Failure;
use tonic::Request;
use tonic::Response;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Backend for Services {
    async fn appearance(&self, request: Request<BackendAppearanceRequest>) -> Result<Response<BackendAppearanceResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Backend/Appearance", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::backend::appearance::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn environment(&self, _request: Request<BackendEnvironmentRequest>) -> Result<Response<BackendEnvironmentResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.Backend/Environment")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
