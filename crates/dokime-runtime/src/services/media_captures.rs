/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Media Captures Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_operations::host::Output;
use dokime_protocol::host::media::CapturesCreateRequest;
use dokime_protocol::host::media::CapturesCreateResponse;
use dokime_protocol::host::media::CapturesGetRequest;
use dokime_protocol::host::media::CapturesGetResponse;
use dokime_protocol::host::media::CapturesStopRequest;
use dokime_protocol::host::media::CapturesStopResponse;
use dokime_protocol::host::media::CapturesWatchRequest;
use dokime_protocol::host::media::CapturesWatchResponse;
use dokime_protocol::host::media::captures_server::Captures;
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
impl Captures for Services {
    async fn create(&self, request: Request<CapturesCreateRequest>) -> Result<Response<CapturesCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let _ = request;
        Err(Status::unimplemented("dokime.host.v1.media.Captures/Create: Operation Is Unavailable in This Host"))
    }
    async fn get(&self, request: Request<CapturesGetRequest>) -> Result<Response<CapturesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let _ = request;
        Err(Status::unimplemented("dokime.host.v1.media.Captures/Get: Operation Is Unavailable in This Host"))
    }
    type WatchStream = Output<CapturesWatchResponse>;
    async fn watch(&self, request: Request<CapturesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let _ = request;
        Err(Status::unimplemented("dokime.host.v1.media.Captures/Watch: Operation Is Unavailable in This Host"))
    }
    async fn stop(&self, request: Request<CapturesStopRequest>) -> Result<Response<CapturesStopResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Stop", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let _ = request;
        Err(Status::unimplemented("dokime.host.v1.media.Captures/Stop: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
