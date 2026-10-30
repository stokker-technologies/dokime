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

// Local Imports
use super::ORDINARY;
use dokime_domain::model::query::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
