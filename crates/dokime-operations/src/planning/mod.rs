/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod materialize_template;
mod preview_selection;
mod register_selection;

pub use materialize_template::MaterializeTemplate;
pub use preview_selection::PreviewSelection;
pub use register_selection::RegisterSelection;

use super::Endpoint;
use super::{MUTATION, ORDINARY};

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::planning::wire::*;

// Registration owns case identity independently of cycles. A nonempty key makes

// retries reproduce the same case IDs; conflicting content is rejected.

#[harmos::catalog]
pub enum Operations {
    MaterializeTemplate(MaterializeTemplate),
    RegisterSelection(RegisterSelection),
    PreviewSelection(PreviewSelection),
}

impl Endpoint for MaterializeTemplate {
    const PATH: &'static str = "/dokime.v1.Planning/MaterializeTemplate";
}

impl Endpoint for RegisterSelection {
    const PATH: &'static str = "/dokime.v1.Planning/RegisterSelection";
}

impl Endpoint for PreviewSelection {
    const PATH: &'static str = "/dokime.v1.Planning/PreviewSelection";
}
