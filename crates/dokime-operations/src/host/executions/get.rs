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
use prost::Message;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionWritable;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsGetResponse;
use dokime_protocol::model::ExecutionProgressValue;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsGetRequest, ExecutionsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsGetRequest) -> Result<ExecutionsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let progress = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?;
        let execution = crate::host::execution_state::detail(context, &request.name)?;
        let value = ExecutionWritable {
            label: context.host.store().execution_presentation(&request.name)?.1,
            paused: execution.controls.as_ref().is_some_and(|controls| controls.paused),
            archived: execution.summary.as_ref().is_some_and(|summary| summary.archived),
        };
        let revision = crate::host::paging::bytes(&value);
        Ok(ExecutionsGetResponse {
            value: Some(value),
            revision,
            progress: Some(ExecutionProgressValue::decode(progress.payload.as_slice())?),
            execution: Some(execution),
            resolved_at: Some(progress.resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsGetRequest, ExecutionsGetResponse, context, "dokime.host.v1", "Executions", "Get");
