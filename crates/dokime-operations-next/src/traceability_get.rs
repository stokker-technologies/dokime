/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read the Invocation's Revisioned Traceability

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::{
    context::evidence::{TraceabilityGetRequest, TraceabilityGetResponse},
    v1::TraceabilityEntry,
};

/* === Constants === */

pub const ID: &str = "dokime.context.v1.evidence.Traceability/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<harmos_protocol::v1::OperationContract> {
    crate::declaration::<TraceabilityGetRequest, TraceabilityGetResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}

pub async fn handle(resources: &Resources, request: TraceabilityGetRequest) -> Result<TraceabilityGetResponse> {
    resources.check_action_scope(&request.name, ID)?;
    if request.entry.is_empty() {
        return Err(crate::Error::Invalid("Get Traceability: Select a Named Entry".into()));
    }
    let saved = resources.store.resource_value(&request.name, &format!("traceability/{}", request.entry), None)?;
    Ok(TraceabilityGetResponse {
        value: Some(TraceabilityEntry::decode(saved.payload.as_slice())?),
        revision: saved.revision,
        resolved_at: Some(saved.resolved_at),
    })
}

/* === Private Functions === */

/* === Tests === */
