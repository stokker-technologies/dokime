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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::catalog::suites_server::Suites for Services {
    async fn get(
        &self, request: tonic::Request<v1::catalog::SuitesGetRequest>,
    ) -> Result<tonic::Response<v1::catalog::SuitesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_suites_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::catalog::SuitesListRequest>,
    ) -> Result<tonic::Response<v1::catalog::SuitesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_suites_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
