/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Parameters

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CasesParametersRequest, dokime_protocol::host::CasesParametersResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CasesParametersRequest,
) -> Result<dokime_protocol::host::CasesParametersResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let planned = crate::host::execution_state::planned(context, &request.parent, &request.name)?;
                let parameters = crate::host::execution_state::parameters(
                    &crate::host::execution_state::case(planned.origin.as_ref().unwrap())?.parameters,
                );
                Ok(dokime_protocol::host::CasesParametersResponse { parameters, ..Default::default() })
            })
            .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
