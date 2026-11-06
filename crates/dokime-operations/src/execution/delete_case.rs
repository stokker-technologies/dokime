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

// DeleteCase Only Removes An Unexecuted Case. Reruns Bind A New Explicit Execution Identity.
#[harmos::operation(resources = [crate::hosting::Host<DeleteCase>], response = ControlReply, id = "dokime.execution.delete_case",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "DeleteCase"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCase {
    pub cycle_id: String,
    pub case_id: String,
    pub context: Option<MutationContext>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
