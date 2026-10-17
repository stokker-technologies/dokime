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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn transition(state: &mut RuntimeState, transaction: &RunProvider) -> CommandResult<super::ProviderCycleAdmission> {
    let target = crate::transactions::provider::apply_run_provider(state, transaction).map_err(|rejection| {
        Box::new(rejected_receipt(transaction.command_id.clone(), &transaction.context, transaction.accepted_at.clone(), rejection))
    })?;
    Ok(super::ProviderCycleAdmission {
        job: super::ProviderCycleRequest {
            request_id: transaction.command_id.to_string(),
            test_cycle_id: transaction.test_cycle_id.to_string(),
            target: serde_json::to_value(target).expect("provider target serializes"),
        },
    })
}

/* === Private Functions === */

/* === Tests === */
