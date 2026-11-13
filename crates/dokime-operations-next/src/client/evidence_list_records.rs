/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.evidence.list_records";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::EvidenceListRecordsRequest, v1::RecordPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::EvidenceListRecordsRequest) -> Result<v1::RecordPage> {
    super::complete(ID, context.settings.read_timeout, async {
        let selection = request.selection.unwrap_or_default();
        super::evidence_selection::validate(&selection)?;
        let store = context.host.store();
        if let Some(id) = selection.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) {
            store.metadata(id)?;
        }
        let maximum = if let Some(execution) = selection.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) {
            store.execution_evidence_frontier(execution)?
        } else if let Some(cycle) = selection.scope.as_ref().and_then(|scope| scope.cycle_id.as_deref()) {
            store.cycle_evidence_frontier(cycle)?
        } else {
            store.evidence_frontier()?
        };
        let key = super::evidence_selection::key(&selection);
        let mut after = super::evidence_selection::position(context, &key, request.after.as_ref(), maximum, 0)?;
        let through = super::evidence_selection::position(context, &key, request.through.as_ref(), maximum, maximum)?;
        if after > through || request.latest != 0 && request.after.is_some() {
            return Err(Error::Invalid("List Records: Choose Forward Cursors or a Latest Tail".into()));
        }
        let size = std::num::NonZeroUsize::new(context.settings.page_items).unwrap();
        let mut records = std::collections::VecDeque::new();
        let mut bytes = 0;
        loop {
            let entries = store.evidence_entries(after, through, size, false)?;
            if entries.is_empty() {
                after = through;
                break;
            }
            for (position, execution, evidence) in entries {
                let cut = store.metadata(&execution)?;
                let matched = super::evidence_selection::scope_matches(selection.scope.as_ref(), &cut, &evidence);
                let record = if matched { super::evidence_selection::project(&cut, &evidence)? } else { None };
                if let Some(record) = record {
                    // Structured Records Remain the Default; Logs Have an Explicit Type.
                    let kind_matches = if selection.record_types.is_empty() {
                        record.record_type != "dokime.log"
                    } else {
                        selection.record_types.iter().any(|pattern| super::evidence_selection::matches(pattern, &record.record_type))
                    };
                    let source_matches = selection.sources.is_empty()
                        || selection.sources.iter().any(|pattern| super::evidence_selection::matches(pattern, &record.source));
                    let severity_matches = selection.severities.is_empty() || selection.severities.contains(&record.severity);
                    if kind_matches && source_matches && severity_matches {
                        let length = prost::Message::encoded_len(&record) + 16;
                        if length + 512 > context.settings.message_bytes {
                            return Err(Error::Invalid("List Records: Increase the Message Budget for This Record".into()));
                        }
                        if request.latest == 0 && (records.len() == size.get() || bytes + length + 512 > context.settings.message_bytes) {
                            return Ok(v1::RecordPage {
                                records: records.into(),
                                scanned: Some(super::evidence_selection::cursor(context, &key, after)),
                                through: Some(super::evidence_selection::cursor(context, &key, through)),
                                replay_complete: false,
                                ..Default::default()
                            });
                        }
                        records.push_back(record);
                        bytes += length;
                        if request.latest != 0 {
                            let keep = (request.latest as usize).min(size.get());
                            while records.len() > keep || bytes + 512 > context.settings.message_bytes {
                                if let Some(record) = records.pop_front() {
                                    bytes -= prost::Message::encoded_len(&record) + 16;
                                }
                            }
                        }
                    }
                }
                after = position;
            }
        }
        Ok(v1::RecordPage {
            records: records.into(),
            scanned: Some(super::evidence_selection::cursor(context, &key, after)),
            through: Some(super::evidence_selection::cursor(context, &key, through)),
            replay_complete: true,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
