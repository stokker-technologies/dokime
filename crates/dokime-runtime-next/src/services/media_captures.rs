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
impl dokime_protocol::client::media::captures_server::Captures for Services {
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::media::CapturesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::media::CapturesCreateResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.media.Captures/Create: Operation Is Unavailable in This Host"))
    }
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::media::CapturesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::media::CapturesGetResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.media.Captures/Get: Operation Is Unavailable in This Host"))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::media::CapturesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::media::CapturesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.media.Captures/Watch: Operation Is Unavailable in This Host"))
    }
    async fn stop(
        &self, request: tonic::Request<dokime_protocol::client::media::CapturesStopRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::media::CapturesStopResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.media.Captures/Stop: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
