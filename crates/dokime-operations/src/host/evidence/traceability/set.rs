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
use crate::{Resources, Result};
use dokime_protocol::host::evidence::TraceabilitySetRequest;
use dokime_protocol::host::evidence::TraceabilitySetResponse;

use dokime_protocol::model::Value;
use dokime_protocol::model::value::Kind as ValueKind;

use crate::Error;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Traceability/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<OperationContract> {
    crate::declaration::<TraceabilitySetRequest, TraceabilitySetResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: TraceabilitySetRequest) -> Result<TraceabilitySetResponse> {
    let execution = resources.resolve_execution(request.name.as_deref(), ID)?;
    resources.check_action_scope(execution, ID)?;
    let value = request.value.ok_or_else(|| Error::Invalid("Set Traceability: Supply the Whole Writable Value".into()))?;
    if request.entry.is_empty() || value.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.entry) {
        return Err(Error::Invalid("Set Traceability: Match the Whole Value to Its Named Entry".into()));
    }
    if value.value.as_ref().is_none_or(|value| !valid_value(value))
        || value.metadata.as_ref().unwrap().attributes.values().any(|value| !valid_value(value))
    {
        return Err(Error::Invalid("Set Traceability: Supply Typed Values; Use Explicit Null to Clear a Scalar".into()));
    }
    let now = resources.clock.now_ns();
    let (saved, receipt) = resources.store.set_resource_value(
        execution,
        &format!("traceability/{}", request.entry),
        &request.expected_revision,
        &request.request_id,
        dokime_storage::Evidence {
            kind: "traceability".into(),
            step: resources.step.clone(),
            payload: value.encode_to_vec(),
            source_ns: now,
            receipt_ns: now,
            operation: resources.operation.clone(),
            attempt: resources.attempt,
            ..Default::default()
        },
    )?;
    Ok(TraceabilitySetResponse {
        value: Some(value),
        revision: saved.revision,
        receipt: Some(receipt),
    })
}

/* === Private Functions === */

fn valid_value(value: &Value) -> bool {
    match &value.kind {
        None => false,
        Some(ValueKind::List(values)) => values.values.iter().all(valid_value),
        Some(_) => true,
    }
}

/* === Tests === */
