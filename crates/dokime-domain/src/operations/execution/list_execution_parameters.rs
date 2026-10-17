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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = ListMaterializedParametersReply, id = "dokime.execution.list_execution_parameters", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListExecutionParameters"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionParameters {
    pub execution_id: String,
    pub set: ExecutionParameterSet,
    pub limit: u32,
    pub after_name: String,
    pub cursor: Option<ParameterCursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
