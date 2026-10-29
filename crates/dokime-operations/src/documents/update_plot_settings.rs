/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Update Mutable Plot Presentation Before Evidence Sealing.

use serde::Deserialize;
use serde::Serialize;

use super::*;
use dokime_domain::model::document::DownsampleTechnique;

#[harmos::operation(resources = [crate::hosting::Host<UpdatePlotSettings>], response = (), id = "dokime.documents.update_plot_settings",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Documents", method = "UpdatePlotSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePlotSettings {
    pub scope: Option<DocumentScope>,
    pub document_id: String,
    pub plot_id: String,
    pub display_points: Option<u32>,
    pub downsample: DownsampleTechnique,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdatePlotSettings);
}
