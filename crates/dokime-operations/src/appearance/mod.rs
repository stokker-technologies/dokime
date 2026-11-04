/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod get_appearance;

/* === Re-Exports === */

pub use get_appearance::GetAppearance;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::ORDINARY;
use dokime_domain::model::appearance::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    GetAppearance(GetAppearance),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
