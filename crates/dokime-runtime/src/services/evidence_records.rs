/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Records Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendResponse;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse;
use dokime_protocol::host::evidence::RecordsWatchRequest;
use dokime_protocol::host::evidence::RecordsWatchResponse;
use dokime_protocol::host::evidence::records_server::Records;
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
impl Records for Services {
    async fn list(&self, request: Request<RecordsListRequest>) -> Result<Response<RecordsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Records/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::records::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<RecordsWatchResponse>;
    async fn watch(&self, request: Request<RecordsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Records/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::records::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn append(&self, _request: Request<RecordsAppendRequest>) -> Result<Response<RecordsAppendResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.evidence.Records/Append")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
