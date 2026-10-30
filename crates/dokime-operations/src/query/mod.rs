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
use super::Endpoint;
use super::ORDINARY;
use dokime_domain::model::query::wire::*;

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(Aggregate);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for Aggregate {
    const PATH: &'static str = "/dokime.v1.Query/Aggregate";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
