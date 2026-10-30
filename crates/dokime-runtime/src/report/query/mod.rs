/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod plots;
pub(crate) mod telemetry;

/* === Re-Exports === */

pub(super) use plots::{ReadCache, ReadPlan, materialize, read_plan};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::runtime::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Private Functions === */

/* === Tests === */
