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
use super::ORDINARY;
use super::TRANSFER;

use dokime_domain::model::documents::wire::*;

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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
