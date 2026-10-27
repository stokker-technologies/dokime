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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBinding {
    pub execution_id: String,
    pub case_id: String,
}

#[harmos::operation(resources = [crate::hosting::Host<AppendExecutions>], response = AppendCasesReply, id = "dokime.execution.append_executions",
    policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
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

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(AppendExecutions);
}
