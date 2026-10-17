/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod preview_selection;
mod register_selection;

/* === Re-Exports === */
pub use preview_selection::PreviewSelection;
pub use register_selection::RegisterSelection;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::planning::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Registration owns case identity independently of cycles. A nonempty key makes
// retries reproduce the same case IDs; conflicting content is rejected.

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    RegisterSelection(RegisterSelection),
    PreviewSelection(PreviewSelection),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for RegisterSelection {
    const PATH: &'static str = "/dokime.v1.Planning/RegisterSelection";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for PreviewSelection {
    const PATH: &'static str = "/dokime.v1.Planning/PreviewSelection";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
