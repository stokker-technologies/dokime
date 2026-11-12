/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Observation Get Traceability

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.observation.get_traceability";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ObservationGetTraceabilityRequest, v1::TraceabilityPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ObservationGetTraceabilityRequest) -> Result<v1::TraceabilityPage> {
    super::complete(ID, context.settings.read_timeout, async {
        let scope = request.scope.clone().unwrap_or_default();
        if scope.session_id.as_ref().is_some_and(|session| session != &context.host.identity().epoch) {
            return Err(Error::Conflict("Get Traceability: Resynchronize with the Current Runtime Epoch".into()));
        }
        let mut snapshots = Vec::new();
        let identifiers = if let Some(execution) = &scope.execution_id {
            super::execution_state::read(context, execution)?;
            vec![execution.clone()]
        } else {
            context.host.store().identifiers()?
        };
        for id in identifiers {
            let snapshot = super::execution_state::evidence(context, &id)?;
            if scope.cycle_id.as_ref().is_some_and(|cycle| cycle != &snapshot.execution.cycle) {
                continue;
            }
            for evidence in &snapshot.evidence {
                if evidence.kind != "traceability" || scope.step_id.as_ref().is_some_and(|step| step != &evidence.step) {
                    continue;
                }
                let mut actual = super::projection::entry_scope(&snapshot, evidence)?;
                let (case_id, _, _) = context.host.store().execution_presentation(&id)?;
                if !case_id.is_empty() {
                    actual.case_id = Some(case_id);
                }
                if scope.session_id.as_ref().is_some_and(|session| session != &context.host.identity().epoch)
                    || scope.case_id.as_ref().is_some_and(|case| actual.case_id.as_ref() != Some(case))
                    || scope.criterion_id.as_ref().is_some_and(|criterion| actual.criterion_id.as_ref() != Some(criterion))
                    || scope.template_id.as_ref().is_some_and(|template| actual.template_id.as_ref() != Some(template))
                    || scope.suite_id.as_ref().is_some_and(|suite| actual.suite_id.as_ref() != Some(suite))
                    || scope.origin.as_ref().is_some_and(|origin| actual.origin.as_ref() != Some(origin))
                {
                    continue;
                }
                let value = v1::SetTraceability::decode(evidence.payload.as_slice())?;
                snapshots.push(v1::TraceabilitySnapshot {
                    traceability_id: format!("{id}/{}", evidence.position),
                    scope: Some(super::projection::entry_scope(&snapshot, evidence)?),
                    created_at: Some(super::projection::timestamp(evidence.receipt_ns)),
                    entries: value.entries,
                    ..Default::default()
                });
            }
        }
        snapshots.sort_by(|left, right| left.traceability_id.cmp(&right.traceability_id));
        let cursor = super::paging::cursor(context, format!("traceability/{scope:?}"), snapshots.iter().map(super::paging::bytes));
        let fingerprint = format!("{:016x}", cursor.position);
        if !request.host_traceability_fingerprint.is_empty() && request.host_traceability_fingerprint != fingerprint {
            return Err(Error::Conflict("Get Traceability: Restart Paging with the Current Fingerprint".into()));
        }
        let (snapshots, next) = super::paging::page(
            snapshots,
            request.limit,
            &request.after_traceability_id,
            request.cursor.as_ref(),
            &cursor,
            context.settings.page_items,
            |snapshot| &snapshot.traceability_id,
        )?;
        super::bounded_reply(
            context,
            v1::TraceabilityPage {
                scope: Some(scope),
                snapshots,
                cursor: Some(cursor),
                next_after_traceability_id: next.clone().unwrap_or_default(),
                complete: next.is_none(),
                host_traceability_fingerprint: fingerprint,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
