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
use super::Endpoint;
use super::ORDINARY;
use crate::model::appearance::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    GetAppearance(GetAppearance),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for GetAppearance {
    const PATH: &'static str = "/dokime.v1.Appearance/GetAppearance";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
