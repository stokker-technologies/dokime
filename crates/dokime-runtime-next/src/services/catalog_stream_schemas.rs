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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::client::stream_schemas_server::StreamSchemas for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::StreamSchemasGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::StreamSchemasGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_stream_schemas_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::StreamSchemasListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::StreamSchemasListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_stream_schemas_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
