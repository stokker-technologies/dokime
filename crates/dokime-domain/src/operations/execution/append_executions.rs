/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Register Ordered Explicit Execution Identities

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

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBinding {
    pub execution_id: String,
    pub case_id: String,
}

#[harmos_domain::message]
#[harmos_domain::operation(response = AppendCasesReply, id = "dokime.execution.append_executions", version = 1,
    policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Execution", method = "AppendExecutions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppendExecutions {
    pub cycle_id: String,
    pub executions: Vec<ExecutionBinding>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
