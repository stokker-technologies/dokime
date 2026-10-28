/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<GetSuite>], response = SuiteDefinition, id = "dokime.catalog.get_suite",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "GetSuite"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSuite {
    pub suite_id: String,
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

    crate::hosted!(GetSuite);
}
