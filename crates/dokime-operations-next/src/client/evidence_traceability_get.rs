/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Scoped Traceability at a Durable Cut

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1::{
    self,
    evidence::{TraceabilityGetRequest, TraceabilityGetResponse},
};

/* === Constants === */

pub const ID: &str = "dokime.v1.evidence.Traceability/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<TraceabilityGetRequest, TraceabilityGetResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: TraceabilityGetRequest) -> Result<TraceabilityGetResponse> {
    super::complete(ID, context.settings.read_timeout, async { read_page(context, request, None) }).await
}

pub(super) fn read_page(context: &Context, request: TraceabilityGetRequest, after: Option<&h::ReadCut>) -> Result<TraceabilityGetResponse> {
    let scope = request.scope.unwrap_or_default();
    if scope.session_id.as_ref().is_some_and(|session| session != &context.host.identity().epoch) {
        return Err(Error::Precondition("Get Traceability: Resynchronize with the Current Runtime Epoch".into()));
    }
    let page = request.page.unwrap_or_default();
    let limit = super::projection::limit(page.page_size, context.settings.page_items);
    let maximum = if page.max_bytes == 0 {
        context.settings.message_bytes
    } else {
        usize::try_from(page.max_bytes).unwrap_or(usize::MAX).min(context.settings.message_bytes)
    };
    let store = context.host.store();
    let (reader, _cancellation) = store.read_session()?;
    let supplied = request.at.is_some();
    let mut cut = if let Some(cut) = request.at {
        if !cut.revisions.is_empty() {
            return Err(Error::Invalid("Get Traceability: Use the Returned History Positions".into()));
        }
        cut
    } else {
        if !page.page_token.is_empty() {
            return Err(Error::Invalid("Get Traceability: Continue with the Returned Read Cut".into()));
        }
        let mut positions = Vec::new();
        let mut cut_bytes = 0;
        let mut after = String::new();
        loop {
            let identifiers = if let Some(execution) = &scope.execution_id {
                vec![execution.clone()]
            } else {
                reader.execution_identifier_page(
                    scope.cycle_id.as_deref(),
                    &after,
                    NonZeroUsize::new(context.settings.page_items).unwrap(),
                )?
            };
            if identifiers.is_empty() {
                break;
            }
            after = identifiers.last().unwrap().clone();
            for id in identifiers {
                let execution = reader.cut(&id)?;
                if scope.cycle_id.as_ref().is_some_and(|cycle| cycle != &execution.execution.cycle) {
                    continue;
                }
                let position = reader.resource_value(&id, "traceability", None)?.resolved_at;
                cut_bytes += h::ReadCut {
                    positions: vec![position.clone()],
                    revisions: Vec::new(),
                }
                .encoded_len();
                if cut_bytes >= maximum {
                    return Err(Error::Invalid("Get Traceability: Narrow the Scope to Fit Its Read Cut in the Reply Byte Bound".into()));
                }
                positions.push(position);
            }
            if scope.execution_id.is_some() {
                break;
            }
        }
        h::ReadCut { positions, revisions: Vec::new() }
    };
    cut.positions.sort_by(|left, right| {
        left.history.as_ref().map(|history| &history.epoch).cmp(&right.history.as_ref().map(|history| &history.epoch))
    });
    let mut previous = None;
    for position in &cut.positions {
        let history = position.history.as_ref().ok_or_else(|| Error::Invalid("Get Traceability: Supply a Returned History".into()))?;
        if previous == Some(&history.epoch) || scope.execution_id.as_ref().is_some_and(|execution| execution != &history.epoch) {
            return Err(Error::Invalid("Get Traceability: Use One Returned Position per Selected Execution".into()));
        }
        previous = Some(&history.epoch);
        if supplied {
            reader.resource_value(&history.epoch, "traceability", Some(position))?;
        }
        if scope.cycle_id.as_ref().is_some_and(|cycle| reader.cut(&history.epoch).is_ok_and(|cut| &cut.execution.cycle != cycle)) {
            return Err(Error::Invalid("Get Traceability: Keep the Read Cut Within Its Selected Cycle".into()));
        }
    }
    if scope.execution_id.is_some() && cut.positions.len() != 1 {
        return Err(Error::Invalid("Get Traceability: Retain the Selected Execution in Its Read Cut".into()));
    }
    if let Some(after) = after {
        for position in &after.positions {
            if !cut.positions.iter().any(|current| current.history == position.history && current.sequence >= position.sequence) {
                return Err(Error::Precondition("Watch Traceability: A Retained History Was Removed; Resynchronize".into()));
            }
        }
    }
    let mut digest = Sha256::new();
    digest.update(scope.encode_to_vec());
    digest.update(cut.encode_to_vec());
    if let Some(after) = after {
        digest.update(after.encode_to_vec());
    }
    let fingerprint = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let start = if page.page_token.is_empty() {
        None
    } else {
        let (binding, key) =
            page.page_token.split_once(':').ok_or_else(|| Error::Invalid("Get Traceability: Supply the Returned Page Token".into()))?;
        if binding != fingerprint {
            return Err(Error::Invalid("Get Traceability: Keep the Scope and Read Cut for Every Page".into()));
        }
        let (index, position) =
            key.split_once(':').ok_or_else(|| Error::Invalid("Get Traceability: Supply the Returned Page Token".into()))?;
        let index = index.parse::<usize>().map_err(|_| Error::Invalid("Get Traceability: Supply the Returned Page Token".into()))?;
        let position = position.parse::<u64>().map_err(|_| Error::Invalid("Get Traceability: Supply the Returned Page Token".into()))?;
        if cut.positions.get(index).is_none_or(|cut| position > cut.sequence) {
            return Err(Error::Invalid("Get Traceability: The Page Token Is Outside Its Read Cut".into()));
        }
        Some((index, position))
    };
    let mut response = TraceabilityGetResponse {
        resolved_at: Some(cut.clone()),
        ..Default::default()
    };
    let mut examined = 0;
    let mut continuation = start;
    let mut more = false;
    'histories: for (index, position) in cut.positions.iter().enumerate() {
        if start.is_some_and(|(start, _)| index < start) {
            continue;
        }
        let id = &position.history.as_ref().unwrap().epoch;
        let floor =
            after.and_then(|cut| cut.positions.iter().find(|saved| saved.history == position.history)).map_or(0, |saved| saved.sequence);
        let beginning = start.filter(|(start, _)| *start == index).map_or(floor, |(_, cursor)| cursor.max(floor));
        let events = reader.resource_events(
            id,
            beginning,
            position.sequence,
            NonZeroUsize::new(context.settings.page_items - examined + 1).unwrap(),
        )?;
        let frozen = reader.cut(id)?;
        let mut snapshot = dokime_storage::ExecutionSnapshot {
            execution: frozen.execution,
            state: frozen.state,
            seal: frozen.seal,
            evidence: Vec::new(),
            samples: Vec::new(),
        };
        let (case, _, _) = reader.execution_presentation(id)?;
        for event in events {
            if examined == context.settings.page_items || response.snapshots.len() == limit {
                more = true;
                break 'histories;
            }
            examined += 1;
            if event.kind != "traceability" {
                continuation = Some((index, event.position));
                continue;
            }
            snapshot.evidence = reader.attempt_start(id, &event)?.into_iter().collect();
            let mut actual = super::projection::entry_scope(&snapshot, &event)?;
            if !case.is_empty() {
                actual.case_id = Some(case.clone());
            }
            if !matches_scope(&scope, &actual) {
                continuation = Some((index, event.position));
                continue;
            }
            let row = v1::TraceabilitySnapshot {
                traceability_id: format!("{id}/{:020}", event.position),
                scope: Some(actual),
                created_at: Some(super::projection::timestamp(event.receipt_ns)),
                entries: vec![v1::TraceabilityEntry::decode(event.payload.as_slice())?],
                revision: event.position.to_be_bytes().to_vec(),
                resolved_at: Some(h::Position {
                    history: position.history.clone(),
                    sequence: event.position,
                }),
                ..Default::default()
            };
            response.snapshots.push(row);
            // Reserve the Actual Continuation Envelope Before Admitting Each Row.
            response.next_page_token = format!("{fingerprint}:{index}:{}", event.position);
            if response.encoded_len() > maximum {
                response.snapshots.pop();
                if response.snapshots.is_empty() {
                    return Err(Error::Invalid("Get Traceability: Increase the Reply Byte Bound or Narrow the Value".into()));
                }
                more = true;
                break 'histories;
            }
            continuation = Some((index, event.position));
        }
        continuation = Some((index, position.sequence));
    }
    response.next_page_token = if more {
        let (index, position) = continuation.ok_or_else(|| Error::Invalid("Get Traceability: Increase the Reply Byte Bound".into()))?;
        format!("{fingerprint}:{index}:{position}")
    } else {
        String::new()
    };
    if response.encoded_len() > maximum {
        return Err(Error::Invalid("Get Traceability: Narrow the Scope to Fit Its Read Cut".into()));
    }
    let mut positions = cut.positions;
    for row in &response.snapshots {
        positions.push(row.resolved_at.as_ref().unwrap().clone());
    }
    store.retain_read_positions(&positions)?;
    Ok(response)
}

/* === Private Functions === */

fn matches_scope(wanted: &v1::EvidenceScope, actual: &v1::EvidenceScope) -> bool {
    wanted.step_id.as_ref().is_none_or(|value| actual.step_id.as_ref() == Some(value))
        && wanted.case_id.as_ref().is_none_or(|value| actual.case_id.as_ref() == Some(value))
        && wanted.criterion_id.as_ref().is_none_or(|value| actual.criterion_id.as_ref() == Some(value))
        && wanted.template_id.as_ref().is_none_or(|value| actual.template_id.as_ref() == Some(value))
        && wanted.suite_id.as_ref().is_none_or(|value| actual.suite_id.as_ref() == Some(value))
        && wanted.origin.as_ref().is_none_or(|value| actual.origin.as_ref() == Some(value))
}

/* === Tests === */
