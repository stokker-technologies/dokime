/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Resource Service

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
impl dokime_protocol::client::cycles_server::Cycles for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::CyclesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CyclesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::CyclesSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CyclesSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::CyclesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CyclesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::CyclesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::CyclesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::CyclesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CyclesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn seal(
        &self, request: tonic::Request<dokime_protocol::client::CyclesSealRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::CyclesSealResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_seal::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
