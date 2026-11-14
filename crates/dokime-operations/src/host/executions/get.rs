/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsGetRequest, dokime_protocol::host::ExecutionsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsGetRequest,
) -> Result<dokime_protocol::host::ExecutionsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let progress = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?;
        let execution = crate::host::execution_state::detail(context, &request.name)?;
        let value = dokime_protocol::host::ExecutionWritable {
            label: context.host.store().execution_presentation(&request.name)?.1,
            paused: execution.controls.as_ref().is_some_and(|controls| controls.paused),
            archived: execution.summary.as_ref().is_some_and(|summary| summary.archived),
        };
        let revision = crate::host::paging::bytes(&value);
        Ok(dokime_protocol::host::ExecutionsGetResponse {
            value: Some(value),
            revision,
            progress: Some(dokime_protocol::model::ExecutionProgressValue::decode(progress.payload.as_slice())?),
            execution: Some(execution),
            resolved_at: Some(progress.resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
