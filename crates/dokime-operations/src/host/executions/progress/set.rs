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
use prost::Message;

// Local Imports
use crate::Error;
use crate::{Resources, Result};
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::executions::ProgressSetRequest;
use dokime_protocol::host::executions::ProgressSetResponse;
use dokime_storage::Evidence;
use harmos_protocol::model::OperationContract;
use prost_types::FileDescriptorSet;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Progress/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<OperationContract> {
    crate::declaration::<ProgressSetRequest, ProgressSetResponse>(ID, FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?)
}
pub async fn handle(resources: &Resources, request: ProgressSetRequest) -> Result<ProgressSetResponse> {
    let name = resources.resolve_execution(request.name.as_deref(), ID)?;
    resources.check_action_scope(name, ID)?;
    let value = request.value.ok_or_else(|| Error::Invalid("Set Progress: Supply the Whole Writable Value".into()))?;
    if value.completed > value.total {
        return Err(Error::Invalid("Set Progress: Keep Completed Progress Within Its Total".into()));
    }
    let now = resources.clock.now_ns();
    let (saved, receipt) = resources.store.set_resource_value(
        name,
        "progress",
        &request.expected_revision,
        &request.request_id,
        Evidence {
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
