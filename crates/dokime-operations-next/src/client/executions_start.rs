/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Start

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Executions/Start";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::ExecutionsStartRequest, dokime_protocol::client::ExecutionsStartResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::ExecutionsStartRequest,
) -> Result<dokime_protocol::client::ExecutionsStartResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let controls = context.host.controls(&request.name)?;
        let mode = match dokime_protocol::client::ExecutionMode::try_from(request.mode) {
            Ok(dokime_protocol::client::ExecutionMode::Interactive) => dokime_protocol::client::ExecutionMode::Interactive,
            Ok(dokime_protocol::client::ExecutionMode::Automatic | dokime_protocol::client::ExecutionMode::Unspecified) => {
                dokime_protocol::client::ExecutionMode::Automatic
            }
            _ => return Err(Error::Invalid("Start Execution: Choose an Execution Mode".into())),
        };
        if controls.can_cancel {
            if controls.mode != mode as i32 {
                return Err(Error::Conflict("Start Execution: Resolve the Existing Run Mode".into()));
            }
        } else {
            context.host.start_existing(&request.name, mode).await?;
        }
        Ok(dokime_protocol::client::ExecutionsStartResponse {
            execution: Some(super::execution_state::detail(context, &request.name)?),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
