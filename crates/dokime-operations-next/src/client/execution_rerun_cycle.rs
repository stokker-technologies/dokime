/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Rerun Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.rerun_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionRerunCycleRequest, v1::RerunCycleReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionRerunCycleRequest) -> Result<v1::RerunCycleReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let source = super::execution_state::cycle(context,&request.source_cycle_id)?;
        if source.running_count > 0 || source.queued_count > 0 { return Err(Error::Conflict("Rerun Cycle: Finish the Source Cycle Before Rerunning".into())); }
        let mut origins = Vec::new();
        let mut skipped = Vec::new();
        for snapshot in super::execution_state::snapshots(context,&request.source_cycle_id)? {
            match snapshot.execution.origin { Some(origin) if matches!(origin.kind,Some(v1::origin::Kind::PluginRun(_))) => origins.push(origin), _ => skipped.push(v1::RerunCaseNotice { case_id: snapshot.execution.identifier,suite_id: String::new(),reason: "Imported Evidence Has No Runnable TestCase".into() }) }
        }
        if origins.is_empty() { return Err(Error::Conflict("Rerun Cycle: Select a Cycle with Runnable PluginRun Origins".into())); }
        let mut metadata = source.metadata.unwrap_or_default();
        if let Some(label) = request.label { if label.trim().is_empty() { return Err(Error::Invalid("Rerun Cycle: Supply a Nonempty Label".into())); } metadata.label = label; }
        if metadata.label.is_empty() { metadata.label = "Rerun".into(); }
        let reply = super::execution_create_cycle::handle(context,v1::ExecutionCreateCycleRequest { origins,metadata: Some(metadata),continue_automatically: true,..Default::default() }).await?;
        let mut cycle = super::execution_state::cycle(context,&reply.cycle_id)?;
        cycle.rerun_of = Some(request.source_cycle_id);
        context.host.store().save_cycle(&cycle)?;
        Ok(v1::RerunCycleReply { cycle_id: reply.cycle_id,observed_cursor: reply.observed_cursor,skipped,dropped_parameters: Vec::new() })
    }).await
}

/* === Private Functions === */

/* === Tests === */
