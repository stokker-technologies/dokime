/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::ReportsPublishResponse;
use dokime_protocol::host::reports_server::Reports;
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
impl Reports for Services {
    async fn get(&self, request: Request<ReportsGetRequest>) -> Result<Response<ReportsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::reports::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, request: Request<ReportsCreateRequest>) -> Result<Response<ReportsCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::reports::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn publish(&self, request: Request<ReportsPublishRequest>) -> Result<Response<ReportsPublishResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Reports/Publish", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::reports::publish::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
