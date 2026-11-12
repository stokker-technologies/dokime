/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Cases

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_cases";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListCasesRequest, v1::ListCasesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionListCasesRequest) -> Result<v1::ListCasesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let cases = super::execution_state::cases(context, &request.cycle_id)?;
        let cursor = super::paging::cursor(context, format!("cases/{}", request.cycle_id), cases.iter().map(super::paging::bytes));
        let (cases, next_after_case_id) = super::paging::page(
            cases,
            request.limit,
            &request.after_case_id,
            request.cursor.as_ref(),
            &cursor,
            context.settings.page_items,
            |row| &row.case_id,
        )?;
        Ok(v1::ListCasesReply {
            cases,
            next_after_case_id,
            cursor: Some(cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
