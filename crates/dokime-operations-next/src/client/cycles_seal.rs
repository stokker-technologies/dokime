/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Seal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Cycles/Seal";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CyclesSealRequest, v1::CyclesSealResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::CyclesSealRequest) -> Result<v1::CyclesSealResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut cycle = super::execution_state::cycle(context, &request.name)?;
        for row in super::execution_state::snapshots(context, &request.name)? {
            if row.state != dokime_storage::ExecutionState::Sealed {
                return Err(Error::Precondition("Seal Cycle: Finish or Cancel Its Executions Before Sealing".into()));
            }
        }
        cycle.sealed = true;
        cycle.phase = v1::CyclePhase::Sealed as i32;
        cycle.updated_at = Some(super::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(v1::CyclesSealResponse {
            seal: super::paging::bytes(&cycle),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
