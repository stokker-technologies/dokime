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

pub const ID: &str = "dokime.execution.records";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::RecordsRequest, v1::Records>(ID, prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?)
}
pub async fn handle(resources: &Resources, request: v1::RecordsRequest) -> Result<v1::Records> {
    resources.check_scope(&request.execution)?;
    let snapshot = resources.store.read(&resources.execution)?;
    let mut records = Vec::new();
    for (ordinal, evidence) in snapshot.records(&request.kind).into_iter().enumerate() {
        if !snapshot.step_contains(&request.step, evidence.position) {
            continue;
        }
        let record = v1::PublishRecord::decode(evidence.payload.as_slice())?;
        records.push(v1::Record {
            kind: record.kind,
            payload: record.payload,
            at: record.at,
            ordinal: ordinal as u64 + 1,
        });
    }
    resources.evidence("read:records", request.encode_to_vec(), None)?;
    let count = records.len() as u64;
    let last = records.last().cloned();
    let start = records.len().saturating_sub(request.limit as usize);
    Ok(v1::Records {
        count,
        last,
        records: if request.limit == 0 { Vec::new() } else { records[start..].to_vec() },
    })
}

/* === Private Functions === */

/* === Tests === */
