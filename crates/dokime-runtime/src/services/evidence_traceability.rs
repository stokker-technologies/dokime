/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Traceability Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::evidence::TraceabilityGetRequest;
use dokime_protocol::host::evidence::TraceabilityGetResponse;
use dokime_protocol::host::evidence::TraceabilitySetRequest;
use dokime_protocol::host::evidence::TraceabilitySetResponse;
use dokime_protocol::host::evidence::TraceabilityWatchRequest;
use dokime_protocol::host::evidence::TraceabilityWatchResponse;
use dokime_protocol::host::evidence::traceability_server::Traceability;
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
impl Traceability for Services {
    async fn get(&self, request: Request<TraceabilityGetRequest>) -> Result<Response<TraceabilityGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Traceability/Get", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::traceability::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<TraceabilityWatchResponse>;
    async fn watch(&self, request: Request<TraceabilityWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Traceability/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::traceability::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, _request: Request<TraceabilitySetRequest>) -> Result<Response<TraceabilitySetResponse>, Status> {
        Err(dokime_operations::caller_status(Failure::wrong_caller("dokime.host.v1.evidence.Traceability/Set")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
