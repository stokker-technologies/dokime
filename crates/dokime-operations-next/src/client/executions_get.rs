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
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Executions/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsGetRequest, v1::ExecutionsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsGetRequest) -> Result<v1::ExecutionsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let progress = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?;
        let execution = super::execution_state::detail(context, &request.name)?;
        let value = v1::ExecutionWritable {
            label: context.host.store().execution_presentation(&request.name)?.1,
            paused: execution.controls.as_ref().is_some_and(|controls| controls.paused),
            archived: execution.summary.as_ref().is_some_and(|summary| summary.archived),
        };
        let revision = super::paging::bytes(&value);
        Ok(v1::ExecutionsGetResponse {
            value: Some(value),
            revision,
            progress: Some(v1::ExecutionProgressValue::decode(progress.payload.as_slice())?),
            execution: Some(execution),
            resolved_at: Some(progress.resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
