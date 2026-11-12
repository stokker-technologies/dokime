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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.record";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::PublishRecord, v1::RecordReceipt>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::PublishRecord) -> Result<v1::RecordReceipt> {
    resources.check_scope(&request.execution)?;
    let source = request.at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    let position = resources.evidence(&format!("record:{}", request.kind), request.encode_to_vec(), source)?;
    let snapshot = resources.store.read(&resources.execution)?;
    let ordinal = snapshot.records(&request.kind).into_iter().filter(|entry| entry.position <= position).count() as u64;
    Ok(v1::RecordReceipt { ordinal })
}

/* === Private Functions === */

/* === Tests === */
