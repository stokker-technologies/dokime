/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::CaseDetail;
use dokime_protocol::host::CasesGetRequest;
use dokime_protocol::host::CasesGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesGetRequest, CasesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CasesGetRequest) -> Result<CasesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let case = crate::host::execution_state::planned(context, &request.parent, &request.name)?;
                let summary =
                    crate::host::execution_state::cases(context, &request.parent)?.into_iter().find(|case| case.case_id == request.name);
                let parameter_count = crate::host::execution_state::case(case.origin.as_ref().unwrap())?.parameters.len() as u64;
                Ok(CasesGetResponse {
                    value: Some(CaseDetail {
                        summary,
                        parameter_count,
                        generation_index: case.ordinal,
                        observed_cursor: Some(crate::host::execution_state::observed(context)),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            })
            .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
