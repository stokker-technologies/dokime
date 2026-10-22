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

pub(crate) use crate::transactions::RunProvider;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::commands::{CommandResult, rejected_receipt};
use crate::runtime_state::RuntimeState;
use crate::transactions::provider::request::RunSummary;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Plans The Admission Against The Current State; The Commit Applies The Same Plan.
pub(crate) fn plan(state: &RuntimeState, transaction: &RunProvider) -> CommandResult<(super::ProviderCycleAdmission, RunSummary)> {
    let plan = crate::transactions::provider::request::plan_queue(state, transaction).map_err(|rejection| {
        Box::new(rejected_receipt(transaction.command_id.clone(), &transaction.context, transaction.accepted_at.clone(), rejection))
    })?;
    let admission = super::ProviderCycleAdmission {
        job: super::ProviderCycleRequest {
            request_id: transaction.command_id.to_string(),
            test_cycle_id: transaction.test_cycle_id.to_string(),
            target: serde_json::to_value(plan.target).expect("provider target serializes"),
        },
    };
    Ok((admission, plan.summary))
}

/* === Private Functions === */

/* === Tests === */
