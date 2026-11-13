/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Resources, Result};

/* === Constants === */

pub const ID: &str = "harmos.context.v1.Time/Sleep";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<harmos_protocol::context::TimeSleepRequest, harmos_protocol::context::TimeSleepResponse>(
        ID,
        harmos_protocol::clock::catalog_types(),
    )
}
pub async fn handle(
    resources: &Resources, request: harmos_protocol::context::TimeSleepRequest,
) -> Result<harmos_protocol::context::TimeSleepResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    let duration = request.duration.ok_or_else(|| Error::Invalid("Supply the Duration to Sleep".into()))?;
    let duration_ns = duration
        .seconds
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(duration.nanos as i64))
        .filter(|value| *value >= 0)
        .ok_or_else(|| Error::Invalid("Supply a Nonnegative Sleep Duration Within Range".into()))?;
    resources.clock.sleep(duration_ns).await?;
    Ok(harmos_protocol::context::TimeSleepResponse {})
}

/* === Private Functions === */

/* === Tests === */
