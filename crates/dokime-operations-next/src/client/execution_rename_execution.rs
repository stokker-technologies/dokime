/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Rename Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.rename_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionRenameExecutionRequest, v1::RenameReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionRenameExecutionRequest) -> Result<v1::RenameReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.label.trim().is_empty() { return Err(Error::Invalid("Rename Execution: Supply a Nonempty Label".into())); }
        super::execution_state::read(context,&request.execution_id)?;
        context.host.store().set_execution_presentation(&request.execution_id,Some(request.label.trim()),None)?;
        context.host.signal_change();
        Ok(v1::RenameReply { label: request.label.trim().into() })
    }).await
}

/* === Private Functions === */

/* === Tests === */
