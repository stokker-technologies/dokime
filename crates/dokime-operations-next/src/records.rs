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

pub const ID: &str = "dokime.context.v1.evidence.Records/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::evidence::RecordsListRequest, dokime_protocol::context::evidence::RecordsListResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::evidence::RecordsListRequest,
) -> Result<dokime_protocol::context::evidence::RecordsListResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let position = resources.store.resource_value(&resources.execution, "progress", request.at.as_ref())?.resolved_at;
    let page = request.page.clone().unwrap_or_default();
    let limit = if page.page_size == 0 { 256 } else { page.page_size as usize };
    let mut selection = request.clone();
    selection.page = None;
    selection.at = None;
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(selection.encode_to_vec());
    digest.update(position.encode_to_vec());
    let binding = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let start = if page.page_token.is_empty() {
        0
    } else {
        let (key, value) =
            page.page_token.split_once('/').ok_or_else(|| crate::Error::Invalid("List Records: Supply Its Returned Page Token".into()))?;
        if key != binding {
            return Err(crate::Error::Invalid("List Records: Keep the Selection and Durable Position".into()));
        }
        value.parse::<u64>().map_err(|_| crate::Error::Invalid("List Records: Supply the Returned Record Position".into()))?
    };
    let (reader, _cancellation) = resources.store.read_session()?;
    let mut snapshot = reader.read(&resources.execution)?;
    snapshot.evidence.retain(|entry| entry.position <= position.sequence);
    let mut records = Vec::new();
    let mut count = 0;
    let mut last = None;
    let mut through = start;
    let mut more = false;
    for evidence in snapshot.records(&request.kind) {
        if !snapshot.step_contains(&request.step, evidence.position) {
            continue;
        }
        count += 1;
        let record = dokime_protocol::context::evidence::RecordsAppendRequest::decode(evidence.payload.as_slice())?;
        let record = v1::Record {
            kind: record.kind,
            payload: record.payload,
            at: record.source_at,
            ordinal: count,
        };
        last = Some(record.clone());
        if evidence.position <= start {
            continue;
        }
        if records.len() == limit {
            more = true;
            continue;
        }
        records.push(record);
        through = evidence.position;
    }
    let response = dokime_protocol::context::evidence::RecordsListResponse {
        count,
        last,
        records,
        resolved_at: Some(position),
        next_page_token: if more { format!("{binding}/{through}") } else { String::new() },
    };
    if page.max_bytes != 0 && response.encoded_len() as u64 > page.max_bytes {
        return Err(crate::Error::Bound("List Records: Reduce the Page Size to Fit Its Reply Byte Bound".into()));
    }
    resources.evidence("read:records", request.encode_to_vec(), None)?;
    Ok(response)
}

/* === Private Functions === */

/* === Tests === */
