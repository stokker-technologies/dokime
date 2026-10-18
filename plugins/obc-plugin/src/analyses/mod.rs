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
use dokime_runtime::analysis::{Operation, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[dokime_runtime::catalog]
pub enum Analyses {
    Ripple(Operation<ripple::Ripple>),
}

#[dokime_runtime::catalog]
pub enum Implemented {
    Ripple(Served<ripple::Ripple, crate::Resources>),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
