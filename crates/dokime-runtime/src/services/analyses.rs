/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Analyses Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::AnalysesListRequest;
use dokime_protocol::host::AnalysesListResponse;
use dokime_protocol::host::analyses_server::Analyses;
use harmos_protocol::model::Callers;
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
impl Analyses for Services {
    async fn list(&self, request: Request<AnalysesListRequest>) -> Result<Response<AnalysesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Analyses/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::analyses::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
