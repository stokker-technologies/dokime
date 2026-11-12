/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Get Case

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.get_case";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionGetCaseRequest, v1::CaseDetail>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionGetCaseRequest) -> Result<v1::CaseDetail> {
    super::complete(ID, context.settings.read_timeout, async {
        let case = super::execution_state::planned(context, &request.cycle_id, &request.case_id)?;
        let summary = super::execution_state::cases(context, &request.cycle_id)?.into_iter().find(|case| case.case_id == request.case_id);
        let parameter_count = super::execution_state::case(case.origin.as_ref().unwrap())?.parameters.len() as u64;
        Ok(v1::CaseDetail {
            summary,
            parameter_count,
            generation_index: case.ordinal,
            observed_cursor: Some(super::execution_state::observed(context)),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
