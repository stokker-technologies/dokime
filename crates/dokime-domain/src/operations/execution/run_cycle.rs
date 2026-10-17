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
// RunCycle dispatches one provider run per execution target its runnable cases resolve to, sequentially and in case order, under one request.
#[harmos_domain::message]
#[harmos_domain::operation(response = RunReply, id = "dokime.execution.run_cycle", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RunCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RunCycle {
    pub cycle_id: String,
    pub context: Option<MutationContext>,
    pub idempotency_key: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
