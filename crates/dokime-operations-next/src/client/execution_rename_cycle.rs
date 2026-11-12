/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Rename Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.rename_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionRenameCycleRequest, v1::RenameReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionRenameCycleRequest) -> Result<v1::RenameReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut cycle = super::execution_state::cycle(context,&request.cycle_id)?;
        if request.label.trim().is_empty() { return Err(Error::Invalid("Rename Cycle: Supply a Nonempty Label".into())); }
        cycle.metadata.get_or_insert_default().label = request.label.trim().into();
        cycle.updated_at = Some(super::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(v1::RenameReply { label: request.label.trim().into() })
    }).await
}

/* === Private Functions === */

/* === Tests === */
