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
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Records/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(resources: &Resources, request: RecordsAppendRequest) -> Result<RecordsAppendResponse> {
    if resources.capturing_executions.is_empty() {
        resources.check_action_scope(&request.execution, ID)?;
        return append(resources, request);
    }
    if !request.execution.is_empty() {
        resources.check_scope(&request.execution)?;
    }
    for execution in &resources.capturing_executions {
        resources.store.require_run_open(execution)?;
    }
    let mut first = None;
    for execution in &resources.capturing_executions {
        let mut target = resources.clone();
        target.execution = execution.clone();
        let mut request = request.clone();
        request.execution = execution.clone();
        let response = append(&target, request)?;
        first.get_or_insert(response);
    }
    Ok(first.expect("Capturing Executions Are Nonempty"))
}

/* === Private Functions === */

fn append(resources: &Resources, request: RecordsAppendRequest) -> Result<RecordsAppendResponse> {
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
            NonZeroUsize::new(resources.context.settings.page_items).unwrap(),
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

/* === Tests === */

crate::native::unary!(RecordsAppendRequest, RecordsAppendResponse, resources, "dokime.host.v1.evidence", "Records", "Append");
