/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read the Invocation's Revisioned Progress

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::{
    context::executions::{ProgressGetRequest, ProgressGetResponse},
    v1::ExecutionProgressValue,
};

/* === Constants === */

pub const ID: &str = "dokime.plugin.host.v1.executions.Progress/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<harmos_protocol::v1::OperationContract> {
    crate::declaration::<ProgressGetRequest, ProgressGetResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}

pub async fn handle(resources: &Resources, request: ProgressGetRequest) -> Result<ProgressGetResponse> {
    resources.check_action_scope(&request.name, ID)?;
    let saved = resources.store.resource_value(&request.name, "progress", None)?;
    Ok(ProgressGetResponse {
        value: Some(ExecutionProgressValue::decode(saved.payload.as_slice())?),
        revision: saved.revision,
        resolved_at: Some(saved.resolved_at),
    })
}

/* === Private Functions === */

/* === Tests === */
