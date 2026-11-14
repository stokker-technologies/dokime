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
impl dokime_protocol::host::cycles_server::Cycles for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::CyclesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CyclesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::CyclesSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CyclesSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::CyclesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CyclesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::CyclesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::CyclesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::CyclesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CyclesCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn seal(
        &self, request: tonic::Request<dokime_protocol::host::CyclesSealRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::CyclesSealResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Cycles/Seal", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::cycles::seal::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
