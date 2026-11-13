/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Records List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1::{
    self,
    evidence::{RecordsListRequest, RecordsListResponse},
};
use harmos_protocol::v1 as h;
use prost::Message;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;

// Local Imports

/* === Constants === */

pub const ID: &str = "dokime.v1.evidence.Records/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<RecordsListRequest, RecordsListResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: RecordsListRequest) -> Result<RecordsListResponse> {
    super::complete(ID, context.settings.read_timeout, async { read_page(context, request, None) }).await
}

pub(super) fn read_page(context: &Context, mut request: RecordsListRequest, after: Option<&h::ReadCut>) -> Result<RecordsListResponse> {
    let selection = request.selection.clone().unwrap_or_default();
    super::evidence_selection::validate(&selection)?;
    let mut scope = selection.scope.clone().unwrap_or_default();
    if let Some(selected) = request.scope.as_ref().and_then(|scope| scope.kind.as_ref()) {
        match selected {
            v1::record_scope::Kind::Execution(id) => {
                if scope.execution_id.as_ref().is_some_and(|value| value != id) {
                    return Err(Error::Invalid("List Records: Keep Both Scope Selections Consistent".into()));
                }
                scope.execution_id = Some(id.clone());
            }
            v1::record_scope::Kind::Cycle(cycle) => {
                if scope.cycle_id.as_ref().is_some_and(|value| value != &cycle.cycle_id) {
                    return Err(Error::Invalid("List Records: Keep Both Cycle Selections Consistent".into()));
                }
                scope.cycle_id = Some(cycle.cycle_id.clone());
            }
        }
    }
    if !request.step.is_empty() {
        scope.step_id = Some(request.step.clone());
    }
    if let Some(id) = &scope.execution_id {
        context.host.store().metadata(id)?;
    }
    let page = request.page.take().unwrap_or_default();
    let limit = super::projection::limit(page.page_size, context.settings.page_items);
    let maximum = if page.max_bytes == 0 {
        context.settings.message_bytes
    } else {
        usize::try_from(page.max_bytes).unwrap_or(usize::MAX).min(context.settings.message_bytes)
    };
    if !page.page_token.is_empty() && request.at.is_none() {
        return Err(Error::Invalid("List Records: Continue with the Returned Read Cut".into()));
    }
    let at = request.at.take();
    let position = if let Some(at) = &at {
        if at.positions.len() != 1 || !at.revisions.is_empty() {
            return Err(Error::Invalid("List Records: Use Its Returned Evidence Cut".into()));
        }
        context.host.store().evidence_position(at.positions.first())?
    } else {
        context.host.store().evidence_position(None)?
    };
    let cut = h::ReadCut {
        positions: vec![position.clone()],
        revisions: Vec::new(),
    };
    let floor = if let Some(after) = after {
        if after.positions.len() != 1
            || !after.revisions.is_empty()
            || after.positions[0].history != position.history
            || after.positions[0].sequence > position.sequence
        {
            return Err(Error::Precondition("Watch Records: A Retained History Was Removed; Resynchronize".into()));
        }
        after.positions[0].sequence
    } else {
        0
    };
    let mut digest = Sha256::new();
    digest.update(request.encode_to_vec());
    digest.update(cut.encode_to_vec());
    digest.update(floor.to_be_bytes());
    let binding = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let mut scanned = floor;
    if !page.page_token.is_empty() {
        let (key, offset) =
            page.page_token.split_once(':').ok_or_else(|| Error::Invalid("List Records: Use the Returned Page Token".into()))?;
        scanned = offset.parse().map_err(|_| Error::Invalid("List Records: Use the Returned Page Position".into()))?;
        if key != binding || scanned < floor || scanned > position.sequence {
            return Err(Error::Invalid("List Records: Keep the Selection and Cut for Every Page".into()));
        }
    }
    let store = context.host.store();
    let (reader, _cancellation) = store.read_session()?;
    reader.evidence_position(Some(&position))?;
    let entries =
        reader.evidence_entries(scanned, position.sequence, NonZeroUsize::new(context.settings.page_items + 1).unwrap(), false)?;
    let mut response = RecordsListResponse {
        resolved_at: Some(cut),
        ..Default::default()
    };
    let mut more = false;
    for (index, (sequence, execution, evidence)) in entries.into_iter().enumerate() {
        if index == context.settings.page_items {
            more = true;
            break;
        }
        let frozen = reader.metadata(&execution)?;
        if !super::evidence_selection::scope_matches(Some(&scope), &frozen, &evidence) {
            scanned = sequence;
            continue;
        }
        let Some(record) = super::evidence_selection::project(&frozen, &evidence)? else {
            scanned = sequence;
            continue;
        };
        let kind_matches = if request.record_type.is_empty() && selection.record_types.is_empty() {
            record.record_type != "dokime.log"
        } else {
            (request.record_type.is_empty() || request.record_type == record.record_type)
                && (selection.record_types.is_empty()
                    || selection.record_types.iter().any(|pattern| super::evidence_selection::matches(pattern, &record.record_type)))
        };
        let source_matches = selection.sources.is_empty()
            || selection.sources.iter().any(|pattern| super::evidence_selection::matches(pattern, &record.source));
        let severity_matches = selection.severities.is_empty() || selection.severities.contains(&record.severity);
        let version_matches = request.versions.is_empty() || request.versions.contains(&record.schema_version);
        let time = record.timestamp.as_ref().map(|time| (time.seconds, time.nanos));
        let in_window = request.between.as_ref().is_none_or(|window| {
            window.start.as_ref().is_none_or(|start| time.is_some_and(|time| time >= (start.seconds, start.nanos)))
                && window.end.as_ref().is_none_or(|end| time.is_some_and(|time| time <= (end.seconds, end.nanos)))
        });
        if kind_matches && source_matches && severity_matches && version_matches && in_window {
            if response.records.len() == limit {
                more = true;
                break;
            }
            response.records.push(record);
            response.next_page_token = format!("{binding}:{sequence}");
            if response.encoded_len() > maximum {
                response.records.pop();
                if response.records.is_empty() {
                    return Err(Error::Bound("List Records: Narrow the Record to Fit the Reply Byte Bound".into()));
                }
                more = true;
                break;
            }
        }
        scanned = sequence;
    }
    response.next_page_token = if more { format!("{binding}:{scanned}") } else { String::new() };
    super::bounded_reply(context, response)
}

/* === Private Functions === */

/* === Tests === */
