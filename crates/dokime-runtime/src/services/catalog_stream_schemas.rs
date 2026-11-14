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

use dokime_protocol::host::StreamSchemasGetRequest;
use dokime_protocol::host::StreamSchemasGetResponse;
use dokime_protocol::host::StreamSchemasListRequest;
use dokime_protocol::host::StreamSchemasListResponse;
use dokime_protocol::host::stream_schemas_server::StreamSchemas;
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
impl StreamSchemas for Services {
    async fn get(&self, request: Request<StreamSchemasGetRequest>) -> Result<Response<StreamSchemasGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.StreamSchemas/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::stream_schemas::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<StreamSchemasListRequest>) -> Result<Response<StreamSchemasListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.StreamSchemas/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::stream_schemas::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
