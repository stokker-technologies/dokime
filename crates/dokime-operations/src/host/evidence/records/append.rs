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

use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Records/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<OperationContract> {
    crate::declaration::<RecordsAppendRequest, RecordsAppendResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: RecordsAppendRequest) -> Result<RecordsAppendResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let source = request.source_at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    let (position, receipt) = resources.evidence_requested(
        ID,
        &request.request_id,
        &request.encode_to_vec(),
        &format!("record:{}", request.kind),
        request.encode_to_vec(),
        source,
    )?;
    let mut after = 0;
    let mut ordinal = 0;
    let kind = format!("record:{}", request.kind);
    loop {
        let entries = resources.store.evidence_page(
            &resources.execution,
            after,
            position,
            std::num::NonZeroUsize::new(resources.context.settings.page_items).unwrap(),
        )?;
        if entries.is_empty() {
            break;
        }
        for entry in entries {
            after = entry.position;
            ordinal += u64::from(entry.kind == kind);
        }
    }
    Ok(RecordsAppendResponse { ordinal, receipt: Some(receipt) })
}

/* === Private Functions === */

/* === Tests === */
