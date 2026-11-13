/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Cycles/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CyclesCreateRequest, v1::CyclesCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::CyclesCreateRequest) -> Result<v1::CyclesCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut request = request;
        let mut skipped = Vec::new();
        if !request.source_cycle.is_empty() {
            let source = super::execution_state::cycle(context, &request.source_cycle)?;
            if source.running_count > 0 || source.queued_count > 0 {
                return Err(Error::Precondition("Create Cycle: Finish the Source Before Rerunning".into()));
            }
            if !request.origins.is_empty() {
                return Err(Error::Invalid("Create Cycle: Select Origins or a Source Cycle".into()));
            }
            for snapshot in super::execution_state::snapshots(context, &request.source_cycle)? {
                match snapshot.execution.origin {
                    Some(origin) if matches!(origin.kind, Some(v1::origin::Kind::PluginRun(_))) => request.origins.push(origin),
                    _ => skipped.push(v1::RerunCaseNotice {
                        case_id: snapshot.execution.identifier,
                        reason: "Imported Evidence Has No Runnable TestCase".into(),
                        ..Default::default()
                    }),
                }
            }
            if request.metadata.is_none() {
                request.metadata = source.metadata;
            }
        }

        if request.origins.len() > context.settings.max_cases {
            return Err(Error::Invalid("Create Cycle: Reduce the Origin List to the Case Bound".into()));
        }
        if request.metadata.as_ref().is_some_and(|metadata| metadata.label.trim().is_empty()) {
            return Err(Error::Invalid("Create Cycle: Supply a Nonempty Cycle Label".into()));
        }
        let now = Some(super::projection::timestamp(context.host.now_ns()));
        let mut cycle = v1::CycleSummary {
            cycle_id: uuid::Uuid::now_v7().to_string(),
            metadata: request.metadata,
            continue_automatically: request.continue_automatically,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        let mut execution_ids = Vec::new();
        if !request.origins.is_empty() {
            if request.continue_automatically {
                let started = context.host.start(request.origins).await?;
                cycle.cycle_id = started.identifier;
                execution_ids = started.executions;
            } else {
                let cases = request
                    .origins
                    .into_iter()
                    .enumerate()
                    .map(|(index, origin)| {
                        super::execution_state::case(&origin)?;
                        Ok(v1::PlannedCase {
                            origin: Some(origin),
                            ordinal: index as u64,
                            ..Default::default()
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                context.host.store().save_cycle(&cycle)?;
                let registered = context.host.store().register_cases(
                    None,
                    &[],
                    &cycle.cycle_id,
                    v1::CasesCreateResponse { cases, ..Default::default() },
                    context.settings.message_bytes,
                )?;
                let bindings = registered
                    .cases
                    .into_iter()
                    .map(|case| v1::ExecutionBinding {
                        execution_id: uuid::Uuid::now_v7().to_string(),
                        case_id: case.case_id,
                    })
                    .collect();
                let reply = super::executions_create::handle(
                    context,
                    v1::ExecutionsCreateRequest {
                        parent: cycle.cycle_id.clone(),
                        executions: bindings,
                        request_id: request.request_id.clone(),
                    },
                )
                .await?;
                execution_ids =
                    reply.executions.into_iter().filter_map(|execution| execution.summary.map(|summary| summary.execution_id)).collect();
            }
        }
        if !request.source_cycle.is_empty() {
            cycle.rerun_of = Some(request.source_cycle);
        }
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        let _ = execution_ids;
        let admitted = super::executions_list::handle(
            context,
            v1::ExecutionsListRequest {
                parent: cycle.cycle_id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let mut receipt = super::views::receipt(context, &request.request_id)?;
        receipt.durable_at = admitted.resolved_at.as_ref().and_then(|cut| cut.positions.first()).cloned();
        receipt.accepted_at = receipt.durable_at.clone();
        Ok(v1::CyclesCreateResponse {
            cycle: Some(cycle),
            skipped,
            receipt: Some(receipt),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
