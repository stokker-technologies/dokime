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

use dokime_operations::host::Output;
use dokime_protocol::host::DocumentsAppendPartsRequest;
use dokime_protocol::host::DocumentsAppendPartsResponse;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::DocumentsGetResponse;
use dokime_protocol::host::DocumentsWatchRequest;
use dokime_protocol::host::DocumentsWatchResponse;
use dokime_protocol::host::documents_server::Documents;
use harmos_protocol::model::Callers;
use harmos_runtime::Failure;
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
impl Documents for Services {
    async fn get(&self, request: Request<DocumentsGetRequest>) -> Result<Response<DocumentsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Documents/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::documents::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<DocumentsWatchResponse>;
    async fn watch(&self, request: Request<DocumentsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.Documents/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::documents::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn append_parts(&self, _request: Request<DocumentsAppendPartsRequest>) -> Result<Response<DocumentsAppendPartsResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.Documents/AppendParts")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
