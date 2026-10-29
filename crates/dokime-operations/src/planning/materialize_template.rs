/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Materialize And Register Canonical Template Cases

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<MaterializeTemplate>], response = PreviewSelectionReply, id = "dokime.planning.materialize_template",
    policy = MUTATION, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Planning", method = "MaterializeTemplate"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializeTemplate {
    pub selection: Option<RegisteredSelection>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(MaterializeTemplate);
}
