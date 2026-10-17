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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::wire::Metadata;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
#[allow(unused_imports)]
use crate::model::plots::wire::*;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::values::wire::Value;
use super::Access;
use super::Endpoint;
use super::ORDINARY;
use super::TRANSFER;

use crate::model::documents::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */































































/* === Enums === */





























#[harmos::catalog]
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
