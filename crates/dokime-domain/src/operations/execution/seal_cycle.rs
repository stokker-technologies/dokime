/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Seal A Cycle After Its Executions Become Durable

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;
use crate::operations::TRANSFER;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::operation(response = ControlReply, id = "dokime.execution.seal_cycle",
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Execution", method = "SealCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SealCycle {
    pub cycle_id: String,
    pub context: Option<MutationContext>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
