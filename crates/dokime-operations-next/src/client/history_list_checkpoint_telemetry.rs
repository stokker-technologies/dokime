/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History List Checkpoint Telemetry

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.list_checkpoint_telemetry";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryListCheckpointTelemetryRequest, v1::CheckpointTelemetryPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryListCheckpointTelemetryRequest) -> Result<v1::CheckpointTelemetryPage> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        let mut snapshot = super::execution_state::read(context, &cut.execution.identifier)?;
        let schemas = super::projection::schemas(&snapshot)?;
        if !schemas.streams.iter().any(|stream| stream.stream == request.table) {
            return Err(Error::NotFound("List Checkpoint Telemetry: Choose a Frozen Table from GetCheckpointEvidence".into()));
        }
        let after = super::history::after(&checkpoint, &format!("telemetry/{}", request.table), &request.after)?;
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        snapshot.samples = context.host.store().sample_page(
            &cut.execution.identifier,
            &request.table,
            after,
            cut.frontier,
            std::num::NonZeroUsize::new(limit + 1).unwrap(),
        )?;
        let more = snapshot.samples.len() > limit;
        snapshot.samples.truncate(limit);
        let last = snapshot.samples.last().map(|sample| sample.position);
        let rows = super::projection::samples(&snapshot, std::slice::from_ref(&request.table), None)?
            .into_iter()
            .map(|sample| v1::SavedTelemetrySample {
                epoch: context.host.identity().backend_id,
                ordinal: sample.sequence,
                sample: Some(sample),
            })
            .collect();
        let next_after = if more {
            Some(super::history::continuation(&checkpoint, &format!("telemetry/{}", request.table), last.unwrap()))
        } else {
            None
        };
        super::history::bounded(
            context,
            v1::CheckpointTelemetryPage {
                checkpoint: Some(checkpoint),
                table: request.table,
                rows,
                next_after,
                ..Default::default()
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
