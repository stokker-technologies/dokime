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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::aggregates_server::Aggregates for Services {
    async fn get(
        &self, request: tonic::Request<v1::AggregatesGetRequest>,
    ) -> Result<tonic::Response<v1::AggregatesGetResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.v1.Aggregates/Get: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
