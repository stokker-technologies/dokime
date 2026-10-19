/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod capture_current_document;
mod update_plot_settings;

/* === Re-Exports === */
pub use capture_current_document::CaptureCurrentDocument;
pub use update_plot_settings::UpdatePlotSettings;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
use super::TRANSFER;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
#[allow(unused_imports)]
use crate::model::plots::wire::*;

use crate::model::documents::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    CaptureCurrentDocument(CaptureCurrentDocument),
    UpdatePlotSettings(UpdatePlotSettings),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for CaptureCurrentDocument {
    const PATH: &'static str = "/dokime.v1.Documents/CaptureCurrentDocument";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for UpdatePlotSettings {
    const PATH: &'static str = "/dokime.v1.Documents/UpdatePlotSettings";
    const ACCESS: Access = Access::Control;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
