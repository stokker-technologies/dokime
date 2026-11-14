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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::record_schemas_server::RecordSchemas for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::RecordSchemasGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::RecordSchemasGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.RecordSchemas/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::record_schemas::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::RecordSchemasListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::RecordSchemasListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.RecordSchemas/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::record_schemas::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
