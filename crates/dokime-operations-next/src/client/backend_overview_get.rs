/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.backend.Overview/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::backend::OverviewGetRequest, v1::backend::OverviewGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::backend::OverviewGetRequest) -> Result<v1::backend::OverviewGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (overview,cut)=super::views::read(context,ID,request.at.as_ref(),async { 
        let cycles = super::execution_state::cycles(context)?;
let cycle_summaries=cycles;let cursor=super::execution_state::observed(context);let next_after_cycle_id=None;
        let mut counts = v1::EvidenceCounts::default();
        for id in context.host.store().identifiers()? {
            let cut = context.host.store().cut(&id)?;
            let mut after = 0;
            loop {
                let page = context.host.store().evidence_page(
                    &id,
                    after,
                    cut.frontier,
                    std::num::NonZeroUsize::new(context.settings.page_items).unwrap(),
                )?;
                if page.is_empty() {
                    break;
                }
                after = page.last().unwrap().position;
                for entry in page {
                    if entry.kind.starts_with("record:") {
                        counts.records += 1;
                    } else {
                        match entry.kind.as_str() {
                            "prompt" => counts.prompts += 1,
                            "traceability" => counts.traceability += 1,
                            "artifact" => counts.artifacts += 1,
                            "health" => counts.health += 1,
                            "note" => counts.notes += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        counts.telemetry = context.host.store().sample_count()?;
        let reply = v1::RuntimeShellSnapshot {
            session_id: context.host.identity().epoch,
            revision: *context.host.changes().borrow(),
            shutting_down: context.host.shutting_down(),
            test_suite_count: context.host.suites()?.len() as u64,
            cycle_summaries,
            evidence_counts: Some(counts),
            observed_cursor: Some(cursor),
            next_after_cycle_id,
            ..Default::default()
        };
        if reply.encoded_len() > context.settings.message_bytes {
            return Err(Error::Invalid("Get Runtime Shell: Reduce the Page Limit".into()));
        }
        Ok(reply) }).await?;
Ok(v1::backend::OverviewGetResponse {overview:Some(overview),resolved_at:Some(cut)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
