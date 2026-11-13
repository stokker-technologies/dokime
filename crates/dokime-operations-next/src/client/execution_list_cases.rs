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
        Ok(v1::ListCasesReply { cases })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
