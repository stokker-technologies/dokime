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
impl dokime_protocol::client::aggregates_server::Aggregates for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::AggregatesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::AggregatesGetResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.Aggregates/Get: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
