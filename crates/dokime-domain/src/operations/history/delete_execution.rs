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

#[harmos_domain::operation(response = DeleteExecutionResponse, id = "dokime.history.delete_execution",
        policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "DeleteExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExecution {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub request_identifier: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
