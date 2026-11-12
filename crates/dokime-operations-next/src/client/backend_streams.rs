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

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn level(level: i32) -> Result<i32> {
    if v1::LogLevel::try_from(level).is_err() { return Err(Error::Invalid("Watch Diagnostics: Choose a Valid Minimum Log Level".into())); }
    Ok(if level == 0 { v1::LogLevel::Trace as i32 } else { level })
}
pub(super) fn trace(context: &Context, request: &v1::WatchBackendTrace) -> Result<v1::TraceBatch> {
    let identity = context.host.identity();
    if (request.after_sequence != 0 && request.epoch.is_none()) || request.epoch.as_ref().is_some_and(|epoch| epoch != &identity.epoch) { return Err(Error::Invalid("Backend Restarted: Read Its Identity and Restart the Trace".into())); }
    let scope = request.scope.as_ref().and_then(|scope| scope.kind.as_ref()).ok_or_else(|| Error::Invalid("Watch Trace: Select the Instance, a Cycle or an Execution".into()))?;
    let min_level = level(request.min_level)?;
    let (events, latest, dropped) = context.host.diagnostic_events(request.scope.as_ref())?;
    if request.after_sequence > latest { return Err(Error::Invalid("Watch Trace: Resume at a Sequence Returned by This Backend".into())); }
    let limit = super::projection::limit(request.max_events, context.settings.page_items);
    let byte_limit = if request.max_bytes == 0 { context.settings.message_bytes / 2 } else { (request.max_bytes as usize).min(context.settings.message_bytes / 2) };
    let mut batch = v1::TraceBatch { epoch: identity.epoch, dropped, next_sequence: request.after_sequence, ..Default::default() };
    for event in events.into_iter().filter(|event| event.sequence > request.after_sequence) {
        let matches = match scope { v1::trace_scope::Kind::Instance(_) => true, v1::trace_scope::Kind::Cycle(id) => event.cycle_id.as_ref() == Some(id), v1::trace_scope::Kind::Execution(id) => event.execution_id.as_ref() == Some(id) };
        if !matches || event.level < min_level { batch.next_sequence = event.sequence; continue; }
        let size = prost::Message::encoded_len(&event);
        if batch.events.len() >= limit || prost::Message::encoded_len(&batch).saturating_add(size) > byte_limit {
            if batch.events.is_empty() { return Err(Error::Invalid("Watch Trace: Increase the Byte Budget to Fit One Event".into())); }
            return Ok(batch);
        }
        batch.next_sequence = event.sequence;
        batch.events.push(event);
    }
    batch.next_sequence = latest;
    Ok(batch)
}

/* === Private Functions === */

/* === Tests === */
