/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod ripple;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::analysis::{Operation, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
/// The analyses an operator may select on any of this bench's plots.
#[dokime::catalog]
pub enum Analyses {
    Ripple(Operation<ripple::Ripple>),
}

/// The operations that answer [`Analyses`] inside this plugin.
#[dokime::catalog]
pub enum Implemented {
    Ripple(Served<ripple::Ripple, crate::Resources>),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
