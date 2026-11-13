/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Invocation-Scoped Authoritative Time

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Resources, Result};
use harmos_protocol::context::{TimeGetRequest, TimeGetResponse};

/* === Constants === */

pub const ID: &str = "harmos.context.v1.Time/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<harmos_protocol::v1::OperationContract> {
    crate::declaration::<TimeGetRequest, TimeGetResponse>(ID, harmos_protocol::clock::catalog_types())
}
pub async fn handle(resources: &Resources, _: TimeGetRequest) -> Result<TimeGetResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    let now_ns = resources.clock.now_ns();
    Ok(TimeGetResponse {
        now: Some(prost_types::Timestamp {
            seconds: now_ns.div_euclid(1_000_000_000),
            nanos: now_ns.rem_euclid(1_000_000_000) as i32,
        }),
    })
}

/* === Private Functions === */

/* === Tests === */
