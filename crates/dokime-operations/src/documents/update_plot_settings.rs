/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Update Mutable Plot Presentation Before Evidence Sealing.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;
use dokime_domain::model::document::DownsampleTechnique;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<UpdatePlotSettings>], response = (), id = "dokime.documents.update_plot_settings",
    policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Documents", method = "UpdatePlotSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePlotSettings {
    pub scope: Option<DocumentScope>,
    pub document_id: String,
    pub plot_id: String,
    pub display_points: Option<u32>,
    pub downsample: DownsampleTechnique,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdatePlotSettings);
}
