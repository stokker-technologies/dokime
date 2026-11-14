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

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CyclePhase;
use dokime_protocol::host::CyclesSealRequest;
use dokime_protocol::host::CyclesSealResponse;
use dokime_storage::ExecutionState;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Seal";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CyclesSealRequest, CyclesSealResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: CyclesSealRequest) -> Result<CyclesSealResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let mut cycle = crate::host::execution_state::cycle(context, &request.name)?;
        for row in crate::host::execution_state::snapshots(context, &request.name)? {
            if row.state != ExecutionState::Sealed {
                return Err(Error::Precondition("Seal Cycle: Finish or Cancel Its Executions Before Sealing".into()));
            }
        }
        cycle.sealed = true;
        cycle.phase = CyclePhase::Sealed as i32;
        cycle.updated_at = Some(crate::host::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(CyclesSealResponse {
            seal: crate::host::paging::bytes(&cycle),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
