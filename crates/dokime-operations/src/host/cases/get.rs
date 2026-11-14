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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CasesGetRequest, dokime_protocol::host::CasesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: dokime_protocol::host::CasesGetRequest) -> Result<dokime_protocol::host::CasesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let case = crate::host::execution_state::planned(context, &request.parent, &request.name)?;
                let summary =
                    crate::host::execution_state::cases(context, &request.parent)?.into_iter().find(|case| case.case_id == request.name);
                let parameter_count = crate::host::execution_state::case(case.origin.as_ref().unwrap())?.parameters.len() as u64;
                Ok(dokime_protocol::host::CasesGetResponse {
                    value: Some(dokime_protocol::host::CaseDetail {
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
