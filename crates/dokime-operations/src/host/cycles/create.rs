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
use uuid::Uuid;

// Local Imports
use crate::host::{Context, cases, execution_state, executions, projection, views};
use crate::{Error, Result};
use dokime_protocol::host::CasesCreateResponse;
use dokime_protocol::host::CycleSummary;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesCreateResponse;
use dokime_protocol::host::ExecutionBinding;
use dokime_protocol::host::ExecutionsCreateRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::PlannedCase;
use dokime_protocol::host::RerunCaseNotice;
use dokime_protocol::model::origin::Kind;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: CyclesCreateRequest) -> Result<CyclesCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let mut request = request;
        let mut skipped = Vec::new();
        if !request.source_cycle.is_empty() {
            let source = execution_state::cycle(context, &request.source_cycle).await?;
            if source.running_count > 0 || source.queued_count > 0 {
                return Err(Error::Precondition("Create Cycle: Finish the Source Before Rerunning".into()));
            }
            if !request.origins.is_empty() {
                return Err(Error::Invalid("Create Cycle: Select Origins or a Source Cycle".into()));
            }
            for snapshot in execution_state::snapshots(context, &request.source_cycle)? {
                match snapshot.execution.origin {
                    Some(origin) if matches!(origin.kind, Some(Kind::PluginRun(_))) => request.origins.push(origin),
                    _ => skipped.push(RerunCaseNotice {
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
        let now = Some(projection::timestamp(context.host.now_ns()));
        let mut cycle = CycleSummary {
            cycle_id: Uuid::now_v7().to_string(),
            metadata: request.metadata,
            continue_automatically: request.continue_automatically,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        if !request.origins.is_empty() {
            let suites = context.host.suites()?;
            let cases = request
                .origins
                .into_iter()
                .enumerate()
                .map(|(index, origin)| {
                    execution_state::case(&origin)?;
                    let Some(Kind::PluginRun(run)) = origin.kind.as_ref() else { unreachable!() };
                    let (suite, template) = suites
                        .iter()
                        .filter(|(plugin, _)| plugin == &run.plugin)
                        .find_map(|(_, suite)| {
                            suite
                                .templates
                                .iter()
                                .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template))
                                .map(|template| (suite, template))
                        })
                        .ok_or_else(|| Error::Precondition("Create Cycle: Reconnect the Case's Template Before Admission".into()))?;
                    Ok(PlannedCase {
                        template_id: run.template.clone(),
                        suite_id: suite.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default(),
                        metadata: template.metadata.clone(),
                        origin: Some(origin),
                        ordinal: index as u64,
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            context.host.store().save_cycle(&cycle)?;
            let registration_id = format!("cycle/{}", cycle.cycle_id);
            let registered = cases::storage::register(
                context,
                &registration_id,
                &[],
                &cycle.cycle_id,
                CasesCreateResponse { cases, ..Default::default() },
            )
            .await?;
            let bindings = registered
                .cases
                .into_iter()
                .map(|case| ExecutionBinding {
                    execution_id: Uuid::now_v7().to_string(),
                    case_id: case.case_id,
                })
                .collect();
            executions::create::handle(
                context,
                ExecutionsCreateRequest {
                    parent: cycle.cycle_id.clone(),
                    executions: bindings,
                    request_id: request.request_id.clone(),
                },
            )
            .await?;
        }
        if !request.source_cycle.is_empty() {
            cycle.rerun_of = Some(request.source_cycle);
        }
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        let admitted = executions::list::handle(
            context,
            ExecutionsListRequest {
                parent: cycle.cycle_id.clone(),
                ..Default::default()
            },
        )
        .await?;
        let mut receipt = views::receipt(context, &request.request_id)?;
        receipt.durable_at = admitted.resolved_at.as_ref().and_then(|cut| cut.positions.first()).cloned();
        receipt.accepted_at = receipt.durable_at.clone();
        Ok(CyclesCreateResponse {
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

crate::native::unary!(CyclesCreateRequest, CyclesCreateResponse, context, "dokime.host.v1", "Cycles", "Create");
