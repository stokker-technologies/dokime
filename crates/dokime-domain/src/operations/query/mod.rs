/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod aggregate;

/* === Re-Exports === */
pub use aggregate::Aggregate;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
use crate::model::values::wire::Value;

use crate::model::query::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A zero limit means the server ceiling of 10,000 rows. Larger values are
// clamped to it rather than rejected. `group_by` may be empty, which
// aggregates the whole filtered selection into a single row. `aggregates` must
// name at least one output.

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for Aggregate {
    const PATH: &'static str = "/dokime.v1.Query/Aggregate";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
