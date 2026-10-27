/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Materialize And Register Canonical Template Cases

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::operation(response = PreviewSelectionReply, id = "dokime.planning.materialize_template",
    policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Planning", method = "MaterializeTemplate"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializeTemplate {
    pub selection: Option<RegisteredSelection>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
