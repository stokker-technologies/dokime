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

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::CasesParametersRequest;
use dokime_protocol::host::CasesParametersResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesParametersRequest, CasesParametersResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CasesParametersRequest) -> Result<CasesParametersResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("{ID}/{}/{}", request.parent, request.name.clone()), request.at.as_ref(), async {
                let planned = crate::host::execution_state::planned(context, &request.parent, &request.name).await?;
                let parameters = crate::host::execution_state::parameters(
                    &crate::host::execution_state::case(planned.origin.as_ref().unwrap())?.parameters,
                );
                Ok(CasesParametersResponse { parameters, ..Default::default() })
            })
            .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CasesParametersRequest, CasesParametersResponse, context, "dokime.host.v1", "Cases", "Parameters");
