/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Documents Resource Service

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
impl dokime_protocol::host::documents_server::Documents for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::DocumentsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::DocumentsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Documents/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::documents::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::DocumentsWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::DocumentsWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Documents/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::documents::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
