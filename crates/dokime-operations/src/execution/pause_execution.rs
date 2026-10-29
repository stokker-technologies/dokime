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

// Controls require that execution's active execution-addressed request.
#[harmos::operation(resources = [crate::hosting::Host<PauseExecution>], response = ControlReply, id = "dokime.execution.pause_execution",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "PauseExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseExecution {
    pub execution_id: String,
    pub context: Option<MutationContext>,
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

    crate::hosted!(PauseExecution);
}
