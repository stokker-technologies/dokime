/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Suites Resource Service

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
impl dokime_protocol::host::suites_server::Suites for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::SuitesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SuitesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Suites/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::suites::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::SuitesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SuitesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Suites/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::suites::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
