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

// Admit one existing, unstarted execution. Never reopens a finished attempt.
#[harmos::operation(resources = [crate::hosting::Host<StartExecution>], response = RunReply, id = "dokime.execution.start_execution",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "StartExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StartExecution {
    pub execution_id: String,
    pub mode: ExecutionMode,
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

    crate::hosted!(StartExecution);
}
