/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Get Checkpoint Context

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.get_checkpoint_context";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryGetCheckpointContextRequest, v1::CheckpointContext>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryGetCheckpointContextRequest) -> Result<v1::CheckpointContext> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        let (_, cycle) = context.host.store().sealed_cycle(&cut.execution.identifier)?;
        let snapshot = super::execution_state::evidence(context, &cut.execution.identifier)?;
        let mut prompts = Vec::new();
        let mut traceability = Vec::new();
        let mut health = Vec::new();
        let mut notes = Vec::new();
        for entry in &snapshot.evidence {
            match entry.kind.as_str() {
                "prompt" => {
                    let question = dokime_protocol::context::PromptsCreateRequest::decode(entry.payload.as_slice())?;
                    let identifier = super::projection::prompt_id(&cut.execution.identifier, &question.prompt, entry.position);
                    if let Some(prompt) = super::prompt_catalog::view(context, &identifier)?.prompt {
                        prompts.push(prompt);
                    }
                }
                "traceability" => {
                    let value = v1::TraceabilityEntry::decode(entry.payload.as_slice())?;
                    traceability.push(v1::TraceabilitySnapshot {
                        traceability_id: format!("{}/{}", cut.execution.identifier, entry.position),
                        scope: Some(super::projection::entry_scope(&snapshot, entry)?),
                        created_at: Some(super::projection::timestamp(entry.receipt_ns)),
                        entries: vec![value],
                        ..Default::default()
                    });
                }
                "health" => health.push(v1::HealthItem::decode(entry.payload.as_slice())?),
                "note" => notes.push(v1::OperatorNote::decode(entry.payload.as_slice())?),
                _ => {}
            }
        }
        let cycle = v1::HistoricalCycle {
            cycle_id: cycle.cycle_id,
            metadata: cycle.metadata,
            phase: cycle.phase,
            lifecycle: cycle.status,
            created_at: cycle.created_at,
            started_at: cycle.started_at,
            updated_at: cycle.updated_at,
            result: cycle.result,
            ..Default::default()
        };
        super::history::bounded(
            context,
            v1::CheckpointContext {
                checkpoint: Some(checkpoint),
                execution_id: Some(cut.execution.identifier.clone()),
                execution_scope: Some(super::projection::scope(&snapshot)),
                cycle: Some(v1::HistoricalCycleContext {
                    cycle: Some(cycle),
                    execution_order: vec![cut.execution.identifier],
                }),
                prompts,
                traceability,
                health,
                notes,
                ..Default::default()
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
