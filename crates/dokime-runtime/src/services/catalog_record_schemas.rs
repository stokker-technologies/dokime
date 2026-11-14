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

use dokime_protocol::host::RecordSchemasGetRequest;
use dokime_protocol::host::RecordSchemasGetResponse;
use dokime_protocol::host::RecordSchemasListRequest;
use dokime_protocol::host::RecordSchemasListResponse;
use dokime_protocol::host::record_schemas_server::RecordSchemas;
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
impl RecordSchemas for Services {
    async fn get(&self, request: Request<RecordSchemasGetRequest>) -> Result<Response<RecordSchemasGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.RecordSchemas/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::record_schemas::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<RecordSchemasListRequest>) -> Result<Response<RecordSchemasListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.RecordSchemas/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::record_schemas::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
