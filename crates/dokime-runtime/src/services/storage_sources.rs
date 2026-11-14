/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Sources Resource Service

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
impl dokime_protocol::host::storage::sources_server::Sources for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::storage::SourcesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::storage::SourcesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.storage.Sources/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::storage::sources::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::storage::SourcesSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::storage::SourcesSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.storage.Sources/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::storage::sources::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
