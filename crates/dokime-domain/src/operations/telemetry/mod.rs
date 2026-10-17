/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod list_samples;

/* === Re-Exports === */
pub use list_samples::ListSamples;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::values::wire::Value;
use super::Access;
use super::Endpoint;
use super::ORDINARY;

use crate::model::telemetry::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */









/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListSamples(ListSamples),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/ListSamples";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
