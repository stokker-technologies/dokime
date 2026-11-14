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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsSetRequest, dokime_protocol::host::ExecutionsSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsSetRequest,
) -> Result<dokime_protocol::host::ExecutionsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let previous = crate::host::executions::get::handle(
            context,
            dokime_protocol::host::ExecutionsGetRequest {
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
            context.host.control_execution(
                &request.name,
                if value.paused { crate::host::RunAction::Pause } else { crate::host::RunAction::Resume },
            )?;
        }
        context.host.signal_change();
        Ok(dokime_protocol::host::ExecutionsSetResponse {
            revision: crate::host::paging::bytes(&value),
            value: Some(value),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
