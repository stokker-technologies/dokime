/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Set Cycle Archived

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.set_cycle_archived";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionSetCycleArchivedRequest, v1::ArchiveReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionSetCycleArchivedRequest) -> Result<v1::ArchiveReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut cycle = super::execution_state::cycle(context, &request.cycle_id)?;
        cycle.archived = request.archived;
        cycle.updated_at = Some(super::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(v1::ArchiveReply { archived: request.archived })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
