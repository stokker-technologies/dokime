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
use super::Endpoint;
use super::ORDINARY;
use super::TRANSFER;

use dokime_domain::model::documents::wire::*;

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(CaptureCurrentDocument, UpdatePlotSettings);

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
}

impl Endpoint for UpdatePlotSettings {
    const PATH: &'static str = "/dokime.v1.Documents/UpdatePlotSettings";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
