/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Health Events Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::evidence::HealthEventsListRequest;
use dokime_protocol::host::evidence::HealthEventsListResponse;
use dokime_protocol::host::evidence::HealthEventsWatchRequest;
use dokime_protocol::host::evidence::HealthEventsWatchResponse;
use dokime_protocol::host::evidence::health_events_server::HealthEvents;
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
impl HealthEvents for Services {
    async fn list(&self, request: Request<HealthEventsListRequest>) -> Result<Response<HealthEventsListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.HealthEvents/List", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::health_events::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<HealthEventsWatchResponse>;
    async fn watch(&self, request: Request<HealthEventsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.HealthEvents/Watch", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::health_events::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
