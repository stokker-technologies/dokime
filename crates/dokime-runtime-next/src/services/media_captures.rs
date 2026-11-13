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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::media::captures_server::Captures for Services {
    async fn create(
        &self, request: tonic::Request<v1::media::CapturesCreateRequest>,
    ) -> Result<tonic::Response<v1::media::CapturesCreateResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.media.Captures/Create: Operation Is Unavailable in This Host"))
    }
    async fn get(
        &self, request: tonic::Request<v1::media::CapturesGetRequest>,
    ) -> Result<tonic::Response<v1::media::CapturesGetResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.media.Captures/Get: Operation Is Unavailable in This Host"))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::media::CapturesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<v1::media::CapturesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.media.Captures/Watch: Operation Is Unavailable in This Host"))
    }
    async fn stop(
        &self, request: tonic::Request<v1::media::CapturesStopRequest>,
    ) -> Result<tonic::Response<v1::media::CapturesStopResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.media.Captures/Stop: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
