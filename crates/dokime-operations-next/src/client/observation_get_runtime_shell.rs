/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Observation Get Runtime Shell

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

pub const ID: &str = "dokime.observation.get_runtime_shell";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ObservationGetRuntimeShellRequest, v1::RuntimeShellSnapshot>(ID, settings.read_timeout) }
pub async fn handle(context: &Context, request: v1::ObservationGetRuntimeShellRequest) -> Result<v1::RuntimeShellSnapshot> { super::complete(ID, context.settings.read_timeout, async {
        let cycles = super::execution_state::cycles(context)?;
        let cursor = super::paging::cursor(context,"runtime.shell".into(),cycles.iter().map(Message::encode_to_vec));
        let (cycle_summaries,next_after_cycle_id) = super::paging::page(cycles,request.limit,&request.after_cycle_id,request.cursor.as_ref(),&cursor,context.settings.page_items,|cycle| &cycle.cycle_id)?;
        let mut counts = v1::EvidenceCounts::default();
        for id in context.host.store().identifiers()? {
            let cut = context.host.store().cut(&id)?;
            let mut after = 0;
            loop {
                let page = context.host.store().evidence_page(&id,after,cut.frontier,std::num::NonZeroUsize::new(context.settings.page_items).unwrap())?;
                if page.is_empty() { break; }
                after = page.last().unwrap().position;
                for entry in page { if entry.kind.starts_with("record:") { counts.records += 1; } else { match entry.kind.as_str() { "prompt" => counts.prompts += 1, "traceability" => counts.traceability += 1, "artifact" => counts.artifacts += 1, "health" => counts.health += 1, "note" => counts.notes += 1, _ => {} } } }
            }
        }
        counts.telemetry = context.host.store().sample_count()?;
        let reply = v1::RuntimeShellSnapshot { session_id: context.host.identity().epoch,revision: *context.host.changes().borrow(),shutting_down: context.host.shutting_down(),test_suite_count: context.host.suites()?.len() as u64,cycle_summaries,evidence_counts: Some(counts),observed_cursor: Some(cursor),next_after_cycle_id,..Default::default() };
        if reply.encoded_len() > context.settings.message_bytes { return Err(Error::Invalid("Get Runtime Shell: Reduce the Page Limit".into())); }
        Ok(reply)
}).await }

/* === Private Functions === */

/* === Tests === */
