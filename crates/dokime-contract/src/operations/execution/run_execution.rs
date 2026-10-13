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

// Local Imports
use super::{ORDINARY, RunExecutionRequest, RunReply};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Admit one existing, unstarted execution. Never reopens a finished attempt.
#[harmos::operation(request = RunExecutionRequest, response = RunReply, id = "dokime.execution.run_execution", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RunExecution"))]
pub struct RunExecution;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
