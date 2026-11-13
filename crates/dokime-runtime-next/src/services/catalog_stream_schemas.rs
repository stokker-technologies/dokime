/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Stream Schemas Resource Service

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
impl v1::catalog::stream_schemas_server::StreamSchemas for Services {
    async fn get(
        &self, request: tonic::Request<v1::catalog::StreamSchemasGetRequest>,
    ) -> Result<tonic::Response<v1::catalog::StreamSchemasGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_stream_schemas_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::catalog::StreamSchemasListRequest>,
    ) -> Result<tonic::Response<v1::catalog::StreamSchemasListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_stream_schemas_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
