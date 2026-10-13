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
use super::{ORDINARY, RunCycleRequest, RunReply};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// RunCycle dispatches one provider run per execution target its runnable cases resolve to, sequentially and in case order, under one request.
#[harmos::operation(request = RunCycleRequest, response = RunReply, id = "dokime.execution.run_cycle", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RunCycle"))]
pub struct RunCycle;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
