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
use dokime_protocol::{context::evidence::TraceabilitySetRequest, context::evidence::TraceabilitySetResponse};

/* === Constants === */

pub const ID: &str = "dokime.context.v1.evidence.Traceability/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<TraceabilitySetRequest, TraceabilitySetResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: TraceabilitySetRequest) -> Result<TraceabilitySetResponse> {
    resources.check_action_scope(&request.name, ID)?;
    let value = request.value.ok_or_else(|| crate::Error::Invalid("Set Traceability: Supply the Whole Writable Value".into()))?;
    if request.entry.is_empty() || value.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.entry) {
        return Err(crate::Error::Invalid("Set Traceability: Match the Whole Value to Its Named Entry".into()));
    }
    if value.value.as_ref().is_none_or(|value| !valid_value(value))
        || value.metadata.as_ref().unwrap().attributes.values().any(|value| !valid_value(value))
    {
        return Err(crate::Error::Invalid("Set Traceability: Supply Typed Values; Use Explicit Null to Clear a Scalar".into()));
    }
    let now = resources.clock.now_ns();
    let (saved, receipt) = resources.store.set_resource_value(
        &request.name,
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

fn valid_value(value: &dokime_protocol::v1::Value) -> bool {
    match &value.kind {
        None => false,
        Some(dokime_protocol::v1::value::Kind::List(values)) => values.values.iter().all(valid_value),
        Some(_) => true,
    }
}

/* === Tests === */
