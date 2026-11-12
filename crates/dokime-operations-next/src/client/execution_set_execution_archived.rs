/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Set Execution Archived

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.set_execution_archived";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionSetExecutionArchivedRequest, v1::ArchiveReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionSetExecutionArchivedRequest) -> Result<v1::ArchiveReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::execution_state::read(context,&request.execution_id)?;
        context.host.store().set_execution_presentation(&request.execution_id,None,Some(request.archived))?;
        context.host.signal_change();
        Ok(v1::ArchiveReply { archived: request.archived })
    }).await
}

/* === Private Functions === */

/* === Tests === */
