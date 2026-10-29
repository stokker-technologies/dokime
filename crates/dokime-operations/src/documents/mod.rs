/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod capture_current_document;
mod update_plot_settings;

pub use capture_current_document::CaptureCurrentDocument;
pub use update_plot_settings::UpdatePlotSettings;

use super::Endpoint;
use super::ORDINARY;
use super::TRANSFER;

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;

use dokime_domain::model::documents::wire::*;
#[allow(unused_imports)]
use dokime_domain::model::plots::wire::*;

#[harmos::catalog]
pub enum Operations {
    CaptureCurrentDocument(CaptureCurrentDocument),
    UpdatePlotSettings(UpdatePlotSettings),
}

impl Endpoint for CaptureCurrentDocument {
    const PATH: &'static str = "/dokime.v1.Documents/CaptureCurrentDocument";
}

impl Endpoint for UpdatePlotSettings {
    const PATH: &'static str = "/dokime.v1.Documents/UpdatePlotSettings";
}
