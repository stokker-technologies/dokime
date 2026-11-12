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

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.create_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionCreateCycleRequest, v1::CreateCycleReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionCreateCycleRequest) -> Result<v1::CreateCycleReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.origins.len() > context.settings.max_cases { return Err(Error::Invalid("Create Cycle: Reduce the Origin List to the Case Bound".into())); }
        if request.metadata.as_ref().is_some_and(|metadata| metadata.label.trim().is_empty()) { return Err(Error::Invalid("Create Cycle: Supply a Nonempty Cycle Label".into())); }
        let now = Some(super::projection::timestamp(context.host.now_ns()));
        let mut cycle = v1::CycleSummary { cycle_id: uuid::Uuid::now_v7().to_string(),metadata: request.metadata,continue_automatically: request.continue_automatically,created_at: now,updated_at: now,..Default::default() };
        let mut execution_ids = Vec::new();
        if !request.origins.is_empty() {
            if request.continue_automatically {
                let started = context.host.start(request.origins).await?;
                cycle.cycle_id = started.identifier;
                execution_ids = started.executions;
            } else {
                let cases = request.origins.into_iter().enumerate().map(|(index,origin)| { super::execution_state::case(&origin)?; Ok(v1::PlannedCase { origin: Some(origin),ordinal: index as u64,..Default::default() }) }).collect::<Result<Vec<_>>>()?;
                context.host.store().save_cycle(&cycle)?;
                let registered = context.host.store().register_cases(None,&[],&cycle.cycle_id,v1::PreviewSelectionReply { cases,..Default::default() })?;
                let bindings = registered.cases.into_iter().map(|case| v1::ExecutionBinding { execution_id: uuid::Uuid::now_v7().to_string(),case_id: case.case_id }).collect();
                let reply = super::execution_append_executions::handle(context,v1::ExecutionAppendExecutionsRequest { cycle_id: cycle.cycle_id.clone(),executions: bindings }).await?;
                execution_ids = reply.execution_ids;
            }
        }
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(v1::CreateCycleReply { cycle_id: cycle.cycle_id,execution_ids,observed_cursor: Some(super::execution_state::observed(context)) })
    }).await
}

/* === Private Functions === */

/* === Tests === */
