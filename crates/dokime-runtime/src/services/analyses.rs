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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::analyses_server::Analyses for Services {
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::AnalysesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::AnalysesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Analyses/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::analyses::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
