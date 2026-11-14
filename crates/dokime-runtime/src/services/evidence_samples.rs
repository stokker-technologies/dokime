/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Samples Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::evidence::SamplesListResponse;
use dokime_protocol::host::evidence::SamplesWatchRequest;
use dokime_protocol::host::evidence::SamplesWatchResponse;
use dokime_protocol::host::evidence::samples_server::Samples;
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
impl Samples for Services {
    async fn list(&self, request: Request<SamplesListRequest>) -> Result<Response<SamplesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Samples/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::samples::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<SamplesWatchResponse>;
    async fn watch(&self, request: Request<SamplesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Samples/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::samples::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
