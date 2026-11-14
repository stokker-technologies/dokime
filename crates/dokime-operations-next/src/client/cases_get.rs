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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Cases/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::CasesGetRequest, dokime_protocol::client::CasesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::CasesGetRequest,
) -> Result<dokime_protocol::client::CasesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            super::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let case = super::execution_state::planned(context, &request.parent, &request.name)?;
                let summary =
                    super::execution_state::cases(context, &request.parent)?.into_iter().find(|case| case.case_id == request.name);
                let parameter_count = super::execution_state::case(case.origin.as_ref().unwrap())?.parameters.len() as u64;
                Ok(dokime_protocol::client::CasesGetResponse {
                    value: Some(dokime_protocol::client::CaseDetail {
                        summary,
                        parameter_count,
                        generation_index: case.ordinal,
                        observed_cursor: Some(super::execution_state::observed(context)),
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
