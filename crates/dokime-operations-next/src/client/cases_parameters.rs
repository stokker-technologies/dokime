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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Cases/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::CasesParametersRequest, dokime_protocol::client::CasesParametersResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::CasesParametersRequest,
) -> Result<dokime_protocol::client::CasesParametersResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            super::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let planned = super::execution_state::planned(context, &request.parent, &request.name)?;
                let parameters =
                    super::execution_state::parameters(&super::execution_state::case(planned.origin.as_ref().unwrap())?.parameters);
                Ok(dokime_protocol::client::CasesParametersResponse { parameters, ..Default::default() })
            })
            .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
