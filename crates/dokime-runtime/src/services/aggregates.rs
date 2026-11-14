/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Aggregates Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::AggregatesGetRequest;
use dokime_protocol::host::AggregatesGetResponse;
use dokime_protocol::host::aggregates_server::Aggregates;
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
impl Aggregates for Services {
    async fn get(&self, request: Request<AggregatesGetRequest>) -> Result<Response<AggregatesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Aggregates/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let _ = request;
        Err(Status::unimplemented("dokime.host.v1.Aggregates/Get: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
