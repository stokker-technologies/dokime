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

// Reruns a cycle as a new planned, unrun cycle resolved against the installed
// plugin. Missing cases are skipped and dropped parameters reported; the source
// stays untouched. A label replaces the source label when given.
#[harmos::operation(resources = [crate::hosting::Host<RerunCycle>], response = RerunCycleReply, id = "dokime.execution.rerun_cycle",
        policy = MUTATION, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RerunCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RerunCycle {
    pub source_cycle_id: String,
    pub label: Option<String>,
    pub context: Option<MutationContext>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
