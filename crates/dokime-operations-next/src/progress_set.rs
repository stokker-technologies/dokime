/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::{context::executions::ProgressSetRequest, context::executions::ProgressSetResponse};

/* === Constants === */

pub const ID: &str = "dokime.context.v1.executions.Progress/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<ProgressSetRequest, ProgressSetResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: ProgressSetRequest) -> Result<ProgressSetResponse> {
    resources.check_action_scope(&request.name, ID)?;
    let value = request.value.ok_or_else(|| crate::Error::Invalid("Set Progress: Supply the Whole Writable Value".into()))?;
    if value.completed > value.total {
        return Err(crate::Error::Invalid("Set Progress: Keep Completed Progress Within Its Total".into()));
    }
    let now = resources.clock.now_ns();
    let (saved, receipt) = resources.store.set_resource_value(
        &request.name,
        "progress",
        &request.expected_revision,
        &request.request_id,
        dokime_storage::Evidence {
            kind: "progress".into(),
            step: resources.step.clone(),
            payload: value.encode_to_vec(),
            source_ns: now,
            receipt_ns: now,
            operation: resources.operation.clone(),
            attempt: resources.attempt,
            ..Default::default()
        },
    )?;
    Ok(ProgressSetResponse {
        value: Some(value),
        revision: saved.revision,
        receipt: Some(receipt),
    })
}

/* === Private Functions === */

/* === Tests === */
