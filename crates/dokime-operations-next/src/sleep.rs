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

pub const ID: &str = "harmos.clock.sleep";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<h::Sleep, h::SleepResponse>(ID, harmos_protocol::clock::catalog_types())
}
pub async fn handle(resources: &Resources, request: h::Sleep) -> Result<h::SleepResponse> {
    let duration = request.duration.ok_or_else(|| Error::Invalid("Supply the Duration to Sleep".into()))?;
    let duration_ns = duration
        .seconds
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(duration.nanos as i64))
        .filter(|value| *value >= 0)
        .ok_or_else(|| Error::Invalid("Supply a Nonnegative Sleep Duration Within Range".into()))?;
    resources.clock.sleep(duration_ns).await?;
    Ok(h::SleepResponse {})
}

/* === Private Functions === */

/* === Tests === */
