/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::{
    v1::ExecutionProgressValue,
    v1::executions::{ProgressGetRequest, ProgressGetResponse},
};

/* === Constants === */

pub const ID: &str = "dokime.v1.executions.Progress/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<ProgressGetRequest, ProgressGetResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: ProgressGetRequest) -> Result<ProgressGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let saved = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?;
        Ok(ProgressGetResponse {
            value: Some(ExecutionProgressValue::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            resolved_at: Some(saved.resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
