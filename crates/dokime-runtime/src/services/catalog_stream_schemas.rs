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
impl dokime_protocol::host::stream_schemas_server::StreamSchemas for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::StreamSchemasGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::StreamSchemasGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.StreamSchemas/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::stream_schemas::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::StreamSchemasListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::StreamSchemasListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.StreamSchemas/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::stream_schemas::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
