/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Append Executions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.execution.append_executions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionAppendExecutionsRequest, v1::AppendCasesReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionAppendExecutionsRequest) -> Result<v1::AppendCasesReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let cycle = super::execution_state::cycle(context,&request.cycle_id)?;
        if cycle.sealed || request.executions.is_empty() || request.executions.len() > context.settings.max_cases { return Err(Error::Invalid("Append Executions: Supply Bounded Bindings to an Unsealed Cycle".into())); }
        let mut identifiers = std::collections::HashSet::new();
        let plugins = context.host.plugins()?;
        let mut prepared = Vec::new();
        for binding in &request.executions {
            if uuid::Uuid::parse_str(&binding.execution_id).is_err() || !identifiers.insert(&binding.execution_id) { return Err(Error::Invalid("Append Executions: Supply Unique Execution UUIDs".into())); }
            let case = context.host.store().planned_case(&binding.case_id)?;
            let origin = case.origin.ok_or_else(|| Error::Invalid("Append Executions: Materialize the Selected Case".into()))?;
            let run = match origin.kind.as_ref() { Some(v1::origin::Kind::PluginRun(run)) => run, _ => return Err(Error::Invalid("Append Executions: Use PluginRun Cases; Import Evidence Separately".into())) };
            let plugin = plugins.iter().find(|plugin| plugin.entry == run.plugin && plugin.state == v1::PluginState::Ready as i32).ok_or_else(|| Error::Conflict("Append Executions: Reconnect the Case's Plugin Before Admission".into()))?;
            let description = plugin.description.as_ref().unwrap();
            let catalog = description.catalog.as_ref().unwrap();
            let catalog_bytes = catalog.encode_to_vec();
            let origin = if catalog.digest != run.digest || description.started_at != run.started_at { context.host.materialize(&run.plugin,&run.template,run.test_case.as_ref().unwrap().parameters.clone()).await? } else { origin };
            prepared.push((dokime_storage::Execution { identifier: binding.execution_id.clone(),cycle: request.cycle_id.clone(),origin: Some(origin),catalog: catalog_bytes,..Default::default() },binding.case_id.clone()));
        }
        context.host.store().bind_executions(&request.cycle_id,&prepared)?;
        context.host.signal_change();
        if cycle.continue_automatically { for binding in &request.executions { context.host.start_existing(&binding.execution_id,v1::ExecutionMode::Automatic).await?; } }
        Ok(v1::AppendCasesReply { execution_ids: request.executions.into_iter().map(|binding| binding.execution_id).collect(),observed_cursor: Some(super::execution_state::observed(context)) })
    }).await
}

/* === Private Functions === */

/* === Tests === */
