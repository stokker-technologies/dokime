/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Delete Case

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.delete_case";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionDeleteCaseRequest, v1::ControlReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionDeleteCaseRequest) -> Result<v1::ControlReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::execution_state::cycle(context,&request.cycle_id)?;
        context.host.store().delete_planned_case(&request.cycle_id,&request.case_id)?;
        context.host.signal_change();
        Ok(v1::ControlReply { observed_cursor: Some(super::execution_state::observed(context)) })
    }).await
}

/* === Private Functions === */

/* === Tests === */
