/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Case Parameters

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_case_parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListCaseParametersRequest, v1::ListCaseParametersReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionListCaseParametersRequest) -> Result<v1::ListCaseParametersReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let planned = super::execution_state::planned(context, &request.cycle_id, &request.case_id)?;
        let parameters = super::execution_state::parameters(&super::execution_state::case(planned.origin.as_ref().unwrap())?.parameters);
        Ok(v1::ListCaseParametersReply { parameters })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
