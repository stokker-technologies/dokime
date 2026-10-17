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
#[harmos::message]
#[harmos::operation(response = RunReply, id = "dokime.execution.run_execution", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RunExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RunExecution {
    pub execution_id: String,
    pub context: Option<MutationContext>,
    pub idempotency_key: Option<String>,
    pub mode: ExecutionMode,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
