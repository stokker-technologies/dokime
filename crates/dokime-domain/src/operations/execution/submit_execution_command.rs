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
#[harmos::message]
#[harmos::operation(response = ExecutionCommandReceipt, id = "dokime.execution.submit_execution_command", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SubmitExecutionCommand"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitExecutionCommand {
    pub request_id: String,
    pub command_id: String,
    pub execution_id: String,
    pub action: ExecutionCommandAction,
    pub step_id: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
