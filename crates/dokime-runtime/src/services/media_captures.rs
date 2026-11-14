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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::media::captures_server::Captures for Services {
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::media::CapturesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::media::CapturesCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.media.Captures/Create: Operation Is Unavailable in This Host"))
    }
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::media::CapturesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::media::CapturesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.media.Captures/Get: Operation Is Unavailable in This Host"))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::media::CapturesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::media::CapturesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.media.Captures/Watch: Operation Is Unavailable in This Host"))
    }
    async fn stop(
        &self, request: tonic::Request<dokime_protocol::host::media::CapturesStopRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::media::CapturesStopResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.media.Captures/Stop", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.media.Captures/Stop: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
