/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Resume Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.resume_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionResumeCycleRequest, v1::ControlReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionResumeCycleRequest) -> Result<v1::ControlReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut cycle = super::execution_state::cycle(context, &request.cycle_id)?;
        if cycle.sealed {
            return Err(Error::Conflict("Control Cycle: Select an Unsealed Cycle".into()));
        }
        for row in super::execution_state::snapshots(context, &request.cycle_id)? {
            if context.host.controls(&row.execution.identifier)?.can_cancel {
                context.host.control_execution(&row.execution.identifier, super::RunAction::Resume)?;
            }
        }
        cycle.status = v1::CycleStatus::Ready as i32;
        if cycle.continue_automatically {
            for row in super::execution_state::snapshots(context, &request.cycle_id)? {
                if context.host.controls(&row.execution.identifier)?.can_start {
                    context.host.start_existing(&row.execution.identifier, v1::ExecutionMode::Automatic).await?;
                }
            }
        }
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
