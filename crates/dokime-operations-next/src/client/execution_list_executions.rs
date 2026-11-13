/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Executions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_executions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListExecutionsRequest, v1::ListExecutionsReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionListExecutionsRequest) -> Result<v1::ListExecutionsReply> {
    super::complete(ID, context.settings.read_timeout, async {
        super::execution_state::cycle(context, &request.cycle_id)?;
        let snapshots = super::execution_state::snapshots(context, &request.cycle_id)?;
        let executions = snapshots.iter().map(|snapshot| super::execution_state::summary(context, snapshot)).collect::<Result<Vec<_>>>()?;
        let cursor =
            super::paging::cursor(context, format!("executions/{}", request.cycle_id), executions.iter().map(super::paging::bytes));
        let (executions, next_after_execution_id) = super::paging::page(
            executions,
            request.limit,
            &request.after_execution_id,
            request.cursor.as_ref(),
            &cursor,
            context.settings.page_items,
            |row| &row.execution_id,
        )?;
        Ok(v1::ListExecutionsReply {
            executions,
            next_after_execution_id,
            cursor: Some(cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
