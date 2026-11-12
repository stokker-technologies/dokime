/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Seal Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.seal_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionSealCycleRequest, v1::ControlReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionSealCycleRequest) -> Result<v1::ControlReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut cycle = super::execution_state::cycle(context, &request.cycle_id)?;
        for row in super::execution_state::snapshots(context, &request.cycle_id)? {
            if row.state != dokime_storage::ExecutionState::Sealed {
                return Err(Error::Conflict("Seal Cycle: Finish or Cancel Its Executions Before Sealing".into()));
            }
        }
        cycle.sealed = true;
        cycle.phase = v1::CyclePhase::Sealed as i32;
        cycle.updated_at = Some(super::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(v1::ControlReply {
            observed_cursor: Some(super::execution_state::observed(context)),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
