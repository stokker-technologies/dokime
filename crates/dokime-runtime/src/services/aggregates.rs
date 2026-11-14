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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::host::aggregates_server::Aggregates for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::AggregatesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::AggregatesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Aggregates/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.host.v1.Aggregates/Get: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
