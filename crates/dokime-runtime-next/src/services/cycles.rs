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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::cycles_server::Cycles for Services {
    async fn get(&self, request: tonic::Request<v1::CyclesGetRequest>) -> Result<tonic::Response<v1::CyclesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(&self, request: tonic::Request<v1::CyclesSetRequest>) -> Result<tonic::Response<v1::CyclesSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(&self, request: tonic::Request<v1::CyclesListRequest>) -> Result<tonic::Response<v1::CyclesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::CyclesWatchResponse>;
    async fn watch(&self, request: tonic::Request<v1::CyclesWatchRequest>) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::CyclesCreateRequest>,
    ) -> Result<tonic::Response<v1::CyclesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn seal(&self, request: tonic::Request<v1::CyclesSealRequest>) -> Result<tonic::Response<v1::CyclesSealResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::cycles_seal::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
