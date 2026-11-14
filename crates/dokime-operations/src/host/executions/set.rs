/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::host::RunAction;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsSetRequest;
use dokime_protocol::host::ExecutionsSetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsSetRequest, ExecutionsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsSetRequest) -> Result<ExecutionsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let previous = crate::host::executions::get::handle(
            context,
            ExecutionsGetRequest {
                name: request.name.clone(),
                ..Default::default()
            },
        )
        .await?;
        if previous.revision != request.expected_revision {
            return Err(Error::Conflict("Set Execution: Get Its Current Revision".into()));
        }
        let value = request.value.ok_or_else(|| Error::Invalid("Set Execution: Supply the Whole Writable Value".into()))?;
        if value.label.trim().is_empty() {
            return Err(Error::Invalid("Set Execution: Supply a Nonempty Label".into()));
        }
        context.host.store().set_execution_presentation(&request.name, Some(&value.label), Some(value.archived))?;
        if previous.value.as_ref().is_some_and(|previous| previous.paused != value.paused) {
            context.host.control_execution(&request.name, if value.paused { RunAction::Pause } else { RunAction::Resume })?;
        }
        context.host.signal_change();
        Ok(ExecutionsSetResponse {
            revision: crate::host::paging::bytes(&value),
            value: Some(value),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsSetRequest, ExecutionsSetResponse, context, "dokime.host.v1", "Executions", "Set");
