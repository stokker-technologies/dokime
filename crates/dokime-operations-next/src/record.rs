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

/* === Constants === */

pub const ID: &str = "dokime.context.v1.evidence.Records/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::evidence::RecordsAppendRequest, dokime_protocol::context::evidence::RecordsAppendResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::evidence::RecordsAppendRequest,
) -> Result<dokime_protocol::context::evidence::RecordsAppendResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let source = request.at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    let position = resources.evidence(&format!("record:{}", request.kind), request.encode_to_vec(), source)?;
    let snapshot = resources.store.read(&resources.execution)?;
    let ordinal = snapshot.records(&request.kind).into_iter().filter(|entry| entry.position <= position).count() as u64;
    Ok(dokime_protocol::context::evidence::RecordsAppendResponse { ordinal })
}

/* === Private Functions === */

/* === Tests === */
