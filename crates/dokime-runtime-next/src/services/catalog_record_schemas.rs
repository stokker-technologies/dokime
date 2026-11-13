/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Record Schemas Resource Service

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
impl v1::catalog::record_schemas_server::RecordSchemas for Services {
    async fn get(
        &self, request: tonic::Request<v1::catalog::RecordSchemasGetRequest>,
    ) -> Result<tonic::Response<v1::catalog::RecordSchemasGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_record_schemas_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::catalog::RecordSchemasListRequest>,
    ) -> Result<tonic::Response<v1::catalog::RecordSchemasListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_record_schemas_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
